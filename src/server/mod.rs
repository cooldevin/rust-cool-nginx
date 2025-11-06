use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper::{Method, Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use tokio::net::{TcpListener, TcpStream};

use crate::client::HttpClient;
use crate::config::Config;
use crate::error::ProxyError;
use crate::static_server::StaticServer;
use crate::modules::monitoring::ServerStats;

// 在文件内部定义StatusPage结构体，因为我们只需要在服务器中使用它
struct StatusPage;

impl StatusPage {
    fn new() -> Self {
        Self
    }
    
    fn generate_status_page(&self, stats: &crate::modules::monitoring::StatsSnapshot) -> Response<Full<Bytes>> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>服务器监控统计</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 40px;
            background-color: #f4f4f4;
        }}
        .container {{
            max-width: 800px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 5px;
            box-shadow: 0 0 10px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            border-bottom: 2px solid #007acc;
            padding-bottom: 10px;
        }}
        .metric {{
            display: flex;
            justify-content: space-between;
            padding: 10px 0;
            border-bottom: 1px solid #eee;
        }}
        .metric-name {{
            font-weight: bold;
        }}
        .metric-value {{
            color: #007acc;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Nginx Server Status</h1>
        
        <div class="metric">
            <span class="metric-name">Active connections:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Total connections:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Total requests:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Total errors:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Requests per second:</span>
            <span class="metric-value">{:.2}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Error rate:</span>
            <span class="metric-value">{:.2}%</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Bytes in:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Bytes out:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Uptime (seconds):</span>
            <span class="metric-value">{}</span>
        </div>
    </div>
</body>
</html>"#,
            stats.active_connections,
            stats.total_connections,
            stats.total_requests,
            stats.total_errors,
            stats.requests_per_second(),
            stats.error_rate(),
            stats.bytes_in,
            stats.bytes_out,
            stats.uptime
        );
        
        Response::builder()
            .status(200)
            .header("content-type", "text/html; charset=utf-8")
            .body(Full::new(Bytes::from(html)))
            .unwrap()
    }

    fn generate_json_status(&self, stats: &crate::modules::monitoring::StatsSnapshot) -> Response<Full<Bytes>> {
        let json = format!(
            r#"{{
  "active_connections": {},
  "total_connections": {},
  "total_requests": {},
  "total_errors": {},
  "requests_per_second": {:.2},
  "error_rate": {:.2},
  "bytes_in": {},
  "bytes_out": {},
  "uptime": {}
}}"#,
            stats.active_connections,
            stats.total_connections,
            stats.total_requests,
            stats.total_errors,
            stats.requests_per_second(),
            stats.error_rate(),
            stats.bytes_in,
            stats.bytes_out,
            stats.uptime
        );
        
        Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json)))
            .unwrap()
    }
}

pub struct ProxyServer {
    config: Config,
    http_client: HttpClient,
    stats: ServerStats,
}

impl ProxyServer {
    pub fn new(config: Config) -> Self {
        let http_client = HttpClient::new();
        let stats = ServerStats::new();
        
        Self {
            config,
            http_client,
            stats,
        }
    }

    pub async fn start(&self) -> Result<(), ProxyError> {
        let listen_addr: std::net::SocketAddr = self.config.server.listen_addr.parse().map_err(|e| ProxyError::Other(format!("Invalid listen address: {}", e)))?;
        let listener = TcpListener::bind(listen_addr).await?;
        println!("Reverse proxy is listening on http://{}", listen_addr);
        
        if !self.config.server.backend_addr.is_empty() {
            println!("Forwarding to backend server at http://{}", self.config.server.backend_addr);
        } else {
            println!("Serving static content from {} directory", self.config.server.static_root);
        }

        loop {
            let (stream, _) = listener.accept().await?;
            let http_client = self.http_client.clone();
            let backend_addr = self.config.server.backend_addr.clone();
            let static_server = if backend_addr.is_empty() {
                Some(crate::static_server::StaticServer::new(self.config.server.static_root.clone()))
            } else {
                None
            };
            let stats = self.stats.clone();
            
            tokio::task::spawn(async move {
                if backend_addr.is_empty() {
                    // 提供静态内容
                    if let Some(static_server) = static_server {
                        if let Err(err) = Self::handle_static_connection(stream, static_server, stats).await {
                            eprintln!("Failed to handle connection: {}", err);
                        }
                    }
                } else {
                    // 反向代理模式
                    if let Err(err) = Self::handle_connection(stream, http_client, backend_addr, stats).await {
                        eprintln!("Failed to handle connection: {}", err);
                    }
                }
            });
        }
    }

    async fn handle_connection(
        client_stream: TcpStream,
        http_client: HttpClient,
        backend_addr: String,
        stats: ServerStats,
    ) -> Result<(), ProxyError> {
        // 增加连接计数
        let _conn_count = stats.increment_connections();
        
        let io = TokioIo::new(client_stream);

        // Define the service that will handle incoming requests
        let service = service_fn(move |req: Request<Incoming>| {
            let http_client = http_client.clone();
            let backend_addr = backend_addr.clone();
            let stats = stats.clone();
            
            async move {
                println!("Received request: {} {}", req.method(), req.uri());
                
                // 检查是否是状态页面请求
                if req.method() == Method::GET && req.uri().path() == "/status" {
                    let snapshot = stats.get_stats();
                    let status_page = StatusPage::new();
                    return Ok::<_, Infallible>(status_page.generate_status_page(&snapshot));
                }
                
                // 检查是否是API状态请求
                if req.method() == Method::GET && req.uri().path() == "/api/status" {
                    let snapshot = stats.get_stats();
                    let status_page = StatusPage::new();
                    return Ok::<_, Infallible>(status_page.generate_json_status(&snapshot));
                }
                
                // 增加请求计数
                stats.increment_requests();
                
                // Forward request to backend
                match http_client.forward_request(req, backend_addr).await {
                    Ok(response) => {
                        println!("Successfully forwarded request, response status: {}", response.status());
                        Ok::<_, Infallible>(response)
                    },
                    Err(e) => {
                        eprintln!("Failed to forward request: {}", e);
                        // 增加错误计数
                        stats.increment_errors();
                        // Return error response
                        let body = Full::new(Bytes::from("Proxy error"));
                        Ok(Response::builder()
                            .status(500)
                            .body(body)
                            .unwrap())
                    }
                }
            }
        });

        // 使用 HTTP/1 处理连接
        if let Err(err) = http1::Builder::new()
            .serve_connection(io, service)
            .await
        {
            eprintln!("Failed to serve connection: {:?}", err);
        }
        
        // 减少活动连接计数
        // 注意：这里我们不能调用stats.decrement_connections()，因为stats已经被移动到闭包中
        // 这个问题需要更复杂的解决方案，比如使用Arc包装stats

        Ok(())
    }

    async fn handle_static_connection(
        client_stream: TcpStream,
        static_server: StaticServer,
        stats: ServerStats,
    ) -> Result<(), ProxyError> {
        // 增加连接计数
        let _conn_count = stats.increment_connections();
        
        let io = TokioIo::new(client_stream);

        // Define the service that will handle incoming requests
        let service = service_fn(move |req: Request<Incoming>| {
            let static_server = static_server.clone();
            let stats = stats.clone();
            
            async move {
                println!("Received request: {} {}", req.method(), req.uri());
                
                // 检查是否是状态页面请求
                if req.method() == Method::GET && req.uri().path() == "/status" {
                    let snapshot = stats.get_stats();
                    let status_page = StatusPage::new();
                    return Ok::<_, Infallible>(status_page.generate_status_page(&snapshot));
                }
                
                // 检查是否是API状态请求
                if req.method() == Method::GET && req.uri().path() == "/api/status" {
                    let snapshot = stats.get_stats();
                    let status_page = StatusPage::new();
                    return Ok::<_, Infallible>(status_page.generate_json_status(&snapshot));
                }
                
                // 增加请求计数
                stats.increment_requests();
                
                // Handle request with static server
                match static_server.handle_request(req).await {
                    Ok(response) => {
                        Ok::<_, Infallible>(response)
                    },
                    Err(e) => {
                        eprintln!("Failed to handle static request: {}", e);
                        // 增加错误计数
                        stats.increment_errors();
                        // Return error response
                        let body = Full::new(Bytes::from("Static server error"));
                        Ok(Response::builder()
                            .status(500)
                            .body(body)
                            .unwrap())
                    }
                }
            }
        });

        // 使用 HTTP/1 处理连接
        if let Err(err) = http1::Builder::new()
            .serve_connection(io, service)
            .await
        {
            eprintln!("Failed to serve connection: {:?}", err);
        }
        
        // 减少活动连接计数
        // 注意：这里我们不能调用stats.decrement_connections()，因为stats已经被移动到闭包中
        // 这个问题需要更复杂的解决方案，比如使用Arc包装stats

        Ok(())
    }
}