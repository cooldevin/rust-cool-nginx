use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Method, Request, Response, Server};
use hyper_staticfile::Static;

use crate::config::{self, ServerConfig};
use crate::cors;
use crate::monitoring;

/// Global round-robin counter for load balancing.
static ROUND_ROBIN_INDEX: AtomicUsize = AtomicUsize::new(0);

/// Start the HTTP server in a background thread.
pub fn start(config_lock: Arc<RwLock<ServerConfig>>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            loop {
                let (static_root, stats_path, listen_addr) = {
                    let cfg = config_lock
                        .read()
                        .expect("CONFIG lock poisoned during server start");
                    let addr: SocketAddr = cfg
                        .server
                        .listen_addr
                        .parse()
                        .unwrap_or_else(|_| ([127, 0, 0, 1], 8082).into());
                    (
                        cfg.server.static_root.clone(),
                        cfg.features.stats_path.clone(),
                        addr,
                    )
                };

                let static_files = Static::new(Path::new(&static_root));
                let config_for_svc = Arc::clone(&config_lock);

                let make_svc = make_service_fn(move |_conn| {
                    monitoring::connection_opened();
                    let static_files = static_files.clone();
                    let stats_path = stats_path.clone();
                    let config_handle = Arc::clone(&config_for_svc);

                    async move {
                        Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                            let static_files = static_files.clone();
                            let stats_path = stats_path.clone();
                            let config_handle = Arc::clone(&config_handle);

                            async move {
                                monitoring::increment_requests();
                                let result = handle_request(
                                    req,
                                    static_files,
                                    &stats_path,
                                    &config_handle,
                                )
                                .await;

                                match &result {
                                    Ok(resp) if resp.status().is_success() => {
                                        monitoring::record_success();
                                    }
                                    _ => {
                                        monitoring::record_error();
                                    }
                                }

                                result
                            }
                        }))
                    }
                });

                monitoring::init();
                let server = Server::bind(&listen_addr).serve(make_svc);
                println!("Server running on http://{}", listen_addr);

                if let Err(e) = server.await {
                    eprintln!("Server error: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        });
    });
}

/// Central request router.
async fn handle_request(
    req: Request<Body>,
    static_files: Static,
    stats_path: &str,
    config_lock: &Arc<RwLock<ServerConfig>>,
) -> Result<Response<Body>, Infallible> {
    // CORS pre-flight
    if req.method() == Method::OPTIONS {
        return Ok(cors::preflight_response());
    }

    let path = req.uri().path().to_owned();

    // Monitoring endpoint
    if path == stats_path {
        return Ok(handle_stats());
    }

    // Config API
    if path == "/api/config" {
        return match *req.method() {
            Method::GET => Ok(handle_get_config(config_lock)),
            Method::PUT => Ok(handle_put_config(req, config_lock).await),
            _ => Ok(method_not_allowed()),
        };
    }

    // Reverse-proxy for /admin-api/
    if path.starts_with("/admin-api/") {
        return Ok(handle_proxy(req, config_lock).await);
    }

    // Static file serving (fallback)
    handle_static(req, static_files).await
}

// ── Handlers ────────────────────────────────────────────────────────────────

fn handle_stats() -> Response<Body> {
    let data = monitoring::get_monitoring_data();
    let json = serde_json::to_string_pretty(&data).unwrap_or_default();

    cors::add_cors_headers(Response::builder())
        .header("Content-Type", "application/json")
        .body(Body::from(json))
        .expect("failed to build stats response")
}

fn handle_get_config(config_lock: &Arc<RwLock<ServerConfig>>) -> Response<Body> {
    let cfg = match config_lock.read() {
        Ok(c) => c,
        Err(e) => {
            return error_response(500, &format!("读取配置失败: {}", e));
        }
    };

    let json = match serde_json::to_string_pretty(&*cfg) {
        Ok(j) => j,
        Err(e) => {
            return error_response(500, &format!("序列化配置失败: {}", e));
        }
    };

    cors::add_cors_headers(Response::builder())
        .header("Content-Type", "application/json")
        .body(Body::from(json))
        .expect("failed to build config response")
}

async fn handle_put_config(
    req: Request<Body>,
    config_lock: &Arc<RwLock<ServerConfig>>,
) -> Response<Body> {
    let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
        Ok(b) => b,
        Err(e) => {
            return error_response(400, &format!("读取请求体失败: {}", e));
        }
    };

    let body_str = match String::from_utf8(body_bytes.to_vec()) {
        Ok(s) => s,
        Err(e) => {
            return error_response(400, &format!("请求体不是有效 UTF-8: {}", e));
        }
    };

    let new_config: ServerConfig = match serde_json::from_str(&body_str) {
        Ok(c) => c,
        Err(e) => {
            return error_response(400, &format!("解析配置失败: {}", e));
        }
    };

    // Update in-memory config
    {
        let mut cfg = match config_lock.write() {
            Ok(c) => c,
            Err(e) => {
                return error_response(500, &format!("获取写入锁失败: {}", e));
            }
        };
        *cfg = new_config.clone();
    }

    // Persist to disk
    if let Err(e) = config::save_config(&new_config) {
        return error_response(500, &format!("保存配置失败: {}", e));
    }

    cors::add_cors_headers(Response::builder())
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::json!({"message": "配置更新成功"}).to_string(),
        ))
        .expect("failed to build put-config response")
}

async fn handle_proxy(
    req: Request<Body>,
    config_lock: &Arc<RwLock<ServerConfig>>,
) -> Response<Body> {
    let servers = {
        let cfg = match config_lock.read() {
            Ok(c) => c,
            Err(e) => {
                return error_response(500, &format!("读取配置失败: {}", e));
            }
        };
        cfg.upstream.servers.clone()
    };

    if servers.is_empty() {
        return error_response(502, "没有可用的上游服务器");
    }

    // Round-robin load balancing
    let index = ROUND_ROBIN_INDEX.fetch_add(1, Ordering::Relaxed) % servers.len();
    let upstream = &servers[index];

    let method = req.method().clone();
    let headers = req.headers().clone();
    let uri = req.uri().clone();
    let body = req.into_body();

    let forward_url = format!(
        "http://{}{}",
        upstream.address,
        uri.path_and_query().map(|p| p.as_str()).unwrap_or("")
    );

    let mut forward_req = match Request::builder()
        .method(method)
        .uri(&forward_url)
        .body(body)
    {
        Ok(r) => r,
        Err(e) => {
            return error_response(500, &format!("构建转发请求失败: {}", e));
        }
    };

    // Copy headers, replacing Host
    for (name, value) in headers.iter() {
        if name != hyper::header::HOST {
            forward_req.headers_mut().insert(name, value.clone());
        }
    }
    if let Ok(host) = hyper::header::HeaderValue::from_str(&upstream.address) {
        forward_req
            .headers_mut()
            .insert(hyper::header::HOST, host);
    }

    let client = Client::new();
    match client.request(forward_req).await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("转发请求到上游服务器失败: {}", e);
            error_response(502, "Bad Gateway")
        }
    }
}

async fn handle_static(
    req: Request<Body>,
    static_files: Static,
) -> Result<Response<Body>, Infallible> {
    match static_files.serve(req).await {
        Ok(mut response) => {
            cors::insert_cors_headers(&mut response);
            Ok(response)
        }
        Err(e) => {
            eprintln!("Static file serving error: {}", e);
            Ok(error_response(500, "Internal Server Error"))
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn error_response(status: u16, message: &str) -> Response<Body> {
    let body = serde_json::json!({"error": message}).to_string();

    cors::add_cors_headers(Response::builder())
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(500)
                .body(Body::from("Internal Server Error"))
                .expect("fallback response must not fail")
        })
}

fn method_not_allowed() -> Response<Body> {
    error_response(405, "Method Not Allowed")
}
