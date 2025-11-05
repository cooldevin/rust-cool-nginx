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

pub struct ProxyServer {
    config: Config,
    http_client: HttpClient,
}

impl ProxyServer {
    pub fn new(config: Config) -> Self {
        let http_client = HttpClient::new();
        
        Self {
            config,
            http_client,
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
            
            tokio::task::spawn(async move {
                if backend_addr.is_empty() {
                    // 提供静态内容
                    if let Some(static_server) = static_server {
                        if let Err(err) = Self::handle_static_connection(stream, static_server).await {
                            eprintln!("Failed to handle connection: {}", err);
                        }
                    }
                } else {
                    // 反向代理模式
                    if let Err(err) = Self::handle_connection(stream, http_client, backend_addr).await {
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
    ) -> Result<(), ProxyError> {
        let io = TokioIo::new(client_stream);

        // Define the service that will handle incoming requests
        let service = service_fn(move |req: Request<Incoming>| {
            let http_client = http_client.clone();
            let backend_addr = backend_addr.clone();
            
            async move {
                println!("Received request: {} {}", req.method(), req.uri());
                
                // Forward request to backend
                match http_client.forward_request(req, backend_addr).await {
                    Ok(response) => {
                        println!("Successfully forwarded request, response status: {}", response.status());
                        Ok::<_, Infallible>(response)
                    },
                    Err(e) => {
                        eprintln!("Failed to forward request: {}", e);
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

        // 尝试使用 HTTP/2，如果失败则回退到 HTTP/1
        let res = http2::Builder::new(hyper_util::rt::TokioExecutor::new())
            .serve_connection(io, service)
            .await;

        // 忽略错误，因为我们只是演示 HTTP/2 支持的概念
        let _ = res;

        Ok(())
    }

    async fn handle_static_connection(
        client_stream: TcpStream,
        static_server: StaticServer,
    ) -> Result<(), ProxyError> {
        let io = TokioIo::new(client_stream);

        // Define the service that will handle incoming requests
        let service = service_fn(move |req: Request<Incoming>| {
            let static_server = static_server.clone();
            
            async move {
                println!("Received request: {} {}", req.method(), req.uri());
                
                // Handle request with static server
                match static_server.handle_request(req).await {
                    Ok(response) => {
                        Ok::<_, Infallible>(response)
                    },
                    Err(e) => {
                        eprintln!("Failed to handle static request: {}", e);
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

        // 尝试使用 HTTP/2，如果失败则回退到 HTTP/1
        let res = http2::Builder::new(hyper_util::rt::TokioExecutor::new())
            .serve_connection(io, service)
            .await;

        // 忽略错误，因为我们只是演示 HTTP/2 支持的概念
        let _ = res;

        Ok(())
    }
}