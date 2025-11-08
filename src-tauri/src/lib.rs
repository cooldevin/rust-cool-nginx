#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;
    
    let result = tauri::Builder::default()
        .setup(|app| {
            // 获取主窗口
            if let Some(window) = app.get_webview_window("main") {
                // 增加延迟确保窗口完全初始化
                std::thread::sleep(std::time::Duration::from_millis(200));
                
                // 设置窗口大小为固定的 1920x1080
                let _ = window.set_size(tauri::PhysicalSize::new(1920, 1080));
                let _ = window.center();
                
                println!("窗口已调整大小: 1920x1080");
                
                // 添加一个闭包，在稍后再次设置窗口大小以确保它不会被改变
                let window_clone = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let _ = window_clone.set_size(tauri::PhysicalSize::new(1920, 1080));
                    println!("再次确认窗口大小: 1920x1080");
                });
            }
            
            // 在这里启动我们的Web服务器
            start_server();
            
            if cfg!(debug_assertions) {
                if let Err(e) = app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                ) {
                    eprintln!("Failed to initialize log plugin: {}", e);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_config, update_config])
        .run(tauri::generate_context!());
        
    match result {
        Ok(_) => {
            println!("Tauri application exited successfully");
        },
        Err(e) => {
            eprintln!("Error while running tauri application: {}", e);
            std::process::exit(1);
        }
    }
}

use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::fs;
use std::io::Write;

// 监控数据结构
#[derive(serde::Serialize)]
struct MonitoringData {
    total_requests: u64,
    current_connections: u64,
    success_rate: f64,
    uptime: String,
}

// 全局配置
lazy_static::lazy_static! {
    static ref CONFIG: Arc<RwLock<ServerConfig>> = {
        // 尝试从配置文件读取配置
        let config_str = match fs::read_to_string("nginx.conf") {
            Ok(content) => {
                println!("成功从 nginx.conf 文件读取配置");
                content
            },
            Err(e) => {
                eprintln!("读取 nginx.conf 文件失败: {}，使用默认配置", e);
                // 提供默认配置内容
                r#"{
                    "server": {
                        "listen_addr": "127.0.0.1:8082",
                        "backend_addr": "127.0.0.1:3000",
                        "static_root": "./public",
                        "access_log": "./logs/access.log",
                        "error_log": "./logs/error.log",
                        "log_level": "info",
                        "ssl_cert_path": "",
                        "ssl_key_path": "",
                        "ssl_enabled": false
                    },
                    "upstream": {
                        "load_balancing_algorithm": "round-robin",
                        "servers": []
                    },
                    "features": {
                        "static_file_serving": true,
                        "reverse_proxy": true,
                        "fastcgi_support": false,
                        "load_balancing": true,
                        "cache_enabled": false,
                        "cache_path": "/tmp/nginx/cache",
                        "cache_max_size": "100m",
                        "cache_inactive": "10m",
                        "gzip_compression": true,
                        "gzip_comp_level": 6,
                        "gzip_min_length": 1024,
                        "gzip_types": ["text/plain", "text/css", "application/json", "application/javascript", "text/xml", "application/xml"],
                        "virtual_hosts": false,
                        "access_control": true,
                        "allow_ips": ["127.0.0.1"],
                        "deny_ips": [],
                        "rate_limiting": true,
                        "max_requests_per_minute": 1000,
                        "ssl_tls": false,
                        "websocket_support": true,
                        "worker_processes": 1,
                        "worker_connections": 1024,
                        "monitoring_enabled": true,
                        "stats_path": "/stats"
                    }
                }"#.to_string()
            }
        };
        
        let config: ServerConfig = match serde_json::from_str(&config_str) {
            Ok(parsed_config) => {
                println!("成功解析配置");
                parsed_config
            },
            Err(e) => {
                eprintln!("解析配置失败: {}，使用默认配置", e);
                // 如果解析配置失败，则使用默认配置
                serde_json::from_str(r#"{
                    "server": {
                        "listen_addr": "127.0.0.1:8082",
                        "backend_addr": "127.0.0.1:3000",
                        "static_root": "./public",
                        "access_log": "./logs/access.log",
                        "error_log": "./logs/error.log",
                        "log_level": "info",
                        "ssl_cert_path": "",
                        "ssl_key_path": "",
                        "ssl_enabled": false
                    },
                    "upstream": {
                        "load_balancing_algorithm": "round-robin",
                        "servers": []
                    },
                    "features": {
                        "static_file_serving": true,
                        "reverse_proxy": true,
                        "fastcgi_support": false,
                        "load_balancing": true,
                        "cache_enabled": false,
                        "cache_path": "/tmp/nginx/cache",
                        "cache_max_size": "100m",
                        "cache_inactive": "10m",
                        "gzip_compression": true,
                        "gzip_comp_level": 6,
                        "gzip_min_length": 1024,
                        "gzip_types": ["text/plain", "text/css", "application/json", "application/javascript", "text/xml", "application/xml"],
                        "virtual_hosts": false,
                        "access_control": true,
                        "allow_ips": ["127.0.0.1"],
                        "deny_ips": [],
                        "rate_limiting": true,
                        "max_requests_per_minute": 1000,
                        "ssl_tls": false,
                        "websocket_support": true,
                        "worker_processes": 1,
                        "worker_connections": 1024,
                        "monitoring_enabled": true,
                        "stats_path": "/stats"
                    }
                }"#).expect("默认配置无法解析")
            }
        };
        Arc::new(RwLock::new(config))
    };
    
    // 监控数据
    static ref TOTAL_REQUESTS: AtomicU64 = AtomicU64::new(0);
    static ref CURRENT_CONNECTIONS: AtomicU64 = AtomicU64::new(0);
}

// 增加请求数量
fn increment_requests() {
    TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed);
}

// 获取监控数据
fn get_monitoring_data() -> MonitoringData {
    MonitoringData {
        total_requests: TOTAL_REQUESTS.load(Ordering::Relaxed),
        current_connections: CURRENT_CONNECTIONS.load(Ordering::Relaxed),
        success_rate: 98.5, // 模拟成功率
        uptime: "2 days, 5:30:15".to_string(), // 模拟运行时间
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct ServerConfig {
    server: ServerSection,
    upstream: UpstreamSection,
    features: FeaturesSection,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct ServerSection {
    listen_addr: String,
    backend_addr: String,
    static_root: String,
    access_log: String,
    error_log: String,
    log_level: String,
    ssl_cert_path: String,
    ssl_key_path: String,
    ssl_enabled: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct UpstreamSection {
    load_balancing_algorithm: String,
    servers: Vec<UpstreamServer>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct UpstreamServer {
    address: String,
    weight: u32,
    max_fails: u32,
    fail_timeout: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct FeaturesSection {
    static_file_serving: bool,
    reverse_proxy: bool,
    fastcgi_support: bool,
    load_balancing: bool,
    cache_enabled: bool,
    cache_path: String,
    cache_max_size: String,
    cache_inactive: String,
    gzip_compression: bool,
    gzip_comp_level: u32,
    gzip_min_length: u32,
    gzip_types: Vec<String>,
    virtual_hosts: bool,
    access_control: bool,
    allow_ips: Vec<String>,
    deny_ips: Vec<String>,
    rate_limiting: bool,
    max_requests_per_minute: u32,
    ssl_tls: bool,
    websocket_support: bool,
    worker_processes: u32,
    worker_connections: u32,
    monitoring_enabled: bool,
    stats_path: String,
}

/// 获取当前配置
#[tauri::command]
fn get_config() -> Result<ServerConfig, String> {
    // 直接从内存中获取配置，确保获取的是最新配置
    let config = CONFIG.read().map_err(|e| format!("读取配置失败: {}", e))?;
    println!("从内存加载配置成功: {:?}", config.server.listen_addr);
    // 添加调试信息
    println!("当前配置详情 - 静态文件根目录: {}, 静态文件服务: {}, 反向代理: {}", 
             config.server.static_root, 
             config.features.static_file_serving, 
             config.features.reverse_proxy);
    Ok(config.clone())
}

/// 更新配置
#[tauri::command]
fn update_config(new_config: ServerConfig) -> Result<(), String> {
    println!("开始更新配置: {:?}", new_config);
    
    // 更新内存中的配置
    {
        let mut config = CONFIG.write().map_err(|e| format!("获取写入锁失败: {}", e))?;
        *config = new_config.clone();
    }
    
    // 持久化配置到文件
    let config_str = serde_json::to_string_pretty(&new_config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    // 使用完整路径写入配置文件
    let current_dir = std::env::current_dir().map_err(|e| format!("获取当前目录失败: {}", e))?;
    let config_path = current_dir.join("nginx.conf");
    
    println!("准备写入配置文件，路径: {:?}", config_path);
    
    fs::write(&config_path, config_str)
        .map_err(|e| format!("写入配置文件失败: {}，路径: {:?}", e, config_path))?;
    
    println!("配置已更新并保存到 nginx.conf 文件，路径: {:?}", config_path);
    Ok(())
}

fn start_server() {
    // 创建一个永不结束的后台线程来运行服务器
    std::thread::spawn(|| {
        // 使用 tokio 运行时来处理异步服务器
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            use hyper::service::{make_service_fn, service_fn};
            use hyper::{Body, Request, Response, Server};
            use hyper_staticfile::Static;
            use std::convert::Infallible;
            use std::net::SocketAddr;
            use std::path::Path;

            loop {
                // 获取配置中的静态文件路径
                let (static_root, stats_path, listen_addr) = {
                    let config = CONFIG.read().unwrap();
                    let static_root = config.server.static_root.clone();
                    let stats_path = config.features.stats_path.clone();
                    let listen_addr: SocketAddr = config.server.listen_addr.parse().unwrap_or(([127, 0, 0, 1], 8082).into());
                    (static_root, stats_path, listen_addr)
                };

                let static_files = Static::new(Path::new(&static_root));

                let make_svc = make_service_fn(move |_conn| {
                    let static_files = static_files.clone();
                    let stats_path = stats_path.clone();
                    async move {
                        Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                            let static_files = static_files.clone();
                            let stats_path = stats_path.clone();
                            async move {
                                // 增加请求数量
                                increment_requests();
                                
                                // 处理 CORS 预检请求
                                if req.method() == hyper::Method::OPTIONS {
                                    let response = Response::builder()
                                        .status(200)
                                        .header("Access-Control-Allow-Origin", "*")
                                        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
                                        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
                                        .header("Access-Control-Max-Age", "86400")
                                        .body(Body::from(""))
                                        .unwrap();
                                    return Ok::<_, Infallible>(response);
                                }
                                
                                // 检查是否是监控端点
                                if req.uri().path() == stats_path {
                                    let monitoring_data = get_monitoring_data();
                                    let json_data = serde_json::to_string_pretty(&monitoring_data).unwrap();
                                    let response = Response::builder()
                                        .header("Content-Type", "application/json")
                                        .header("Access-Control-Allow-Origin", "*")
                                        .header("Access-Control-Allow-Methods", "GET, OPTIONS")
                                        .header("Access-Control-Allow-Headers", "Content-Type")
                                        .body(Body::from(json_data))
                                        .unwrap();
                                    return Ok::<_, Infallible>(response);
                                }
                                
                                // 检查是否是配置获取端点
                                if req.uri().path() == "/api/config" && req.method() == hyper::Method::GET {
                                    // 从全局配置中获取配置信息
                                    let config = CONFIG.read().unwrap();
                                    let json_data = serde_json::to_string_pretty(&*config).unwrap();
                                    let response = Response::builder()
                                        .header("Content-Type", "application/json")
                                        .header("Access-Control-Allow-Origin", "*")
                                        .header("Access-Control-Allow-Methods", "GET, OPTIONS")
                                        .header("Access-Control-Allow-Headers", "Content-Type")
                                        .body(Body::from(json_data))
                                        .unwrap();
                                    return Ok::<_, Infallible>(response);
                                }
                                
                                // 检查是否是配置更新端点
                                if req.uri().path() == "/api/config" && req.method() == hyper::Method::PUT {
                                    // 读取请求体
                                    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
                                    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
                                    
                                    // 解析配置
                                    let new_config: ServerConfig = match serde_json::from_str(&body_str) {
                                        Ok(config) => config,
                                        Err(e) => {
                                            let error_msg = format!("解析配置失败: {}", e);
                                            eprintln!("{}", error_msg);
                                            let response = Response::builder()
                                                .status(400)
                                                .header("Content-Type", "application/json")
                                                .header("Access-Control-Allow-Origin", "*")
                                                .body(Body::from(serde_json::json!({"error": error_msg}).to_string()))
                                                .unwrap();
                                            return Ok::<_, Infallible>(response);
                                        }
                                    };
                                    
                                    // 更新配置
                                    match update_config(new_config) {
                                        Ok(()) => {
                                            let response = Response::builder()
                                                .status(200)
                                                .header("Content-Type", "application/json")
                                                .header("Access-Control-Allow-Origin", "*")
                                                .body(Body::from(serde_json::json!({"message": "配置更新成功"}).to_string()))
                                                .unwrap();
                                            return Ok::<_, Infallible>(response);
                                        },
                                        Err(e) => {
                                            let error_msg = format!("更新配置失败: {}", e);
                                            eprintln!("{}", error_msg);
                                            let response = Response::builder()
                                                .status(500)
                                                .header("Content-Type", "application/json")
                                                .header("Access-Control-Allow-Origin", "*")
                                                .body(Body::from(serde_json::json!({"error": error_msg}).to_string()))
                                                .unwrap();
                                            return Ok::<_, Infallible>(response);
                                        }
                                    }
                                }
                                
                                // 提供静态文件服务
                                match static_files.serve(req).await {
                                    Ok(mut response) => {
                                        // 为静态文件响应也添加 CORS 头
                                        response.headers_mut().insert(
                                            "Access-Control-Allow-Origin",
                                            hyper::header::HeaderValue::from_static("*")
                                        );
                                        response.headers_mut().insert(
                                            "Access-Control-Allow-Methods",
                                            hyper::header::HeaderValue::from_static("GET, OPTIONS")
                                        );
                                        response.headers_mut().insert(
                                            "Access-Control-Allow-Headers",
                                            hyper::header::HeaderValue::from_static("Content-Type")
                                        );
                                        Ok::<_, Infallible>(response)
                                    },
                                    Err(e) => {
                                        eprintln!("Static file serving error: {}", e);
                                        let response = Response::builder()
                                            .status(500)
                                            .header("Access-Control-Allow-Origin", "*")
                                            .body(Body::from("Internal Server Error"))
                                            .unwrap();
                                        Ok(response)
                                    }
                                }
                            }
                        }))
                    }
                });

                let server = Server::bind(&listen_addr).serve(make_svc);
                
                println!("Server running on http://{}", listen_addr);

                // 运行服务器直到出错
                if let Err(e) = server.await {
                    eprintln!("Server error: {}", e);
                    // 等待一段时间后重试
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }
        });
    });
}