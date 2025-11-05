//! HTTP 服务器核心实现

use hyper::{Method, Request, Response, StatusCode};
use http_body_util::Full;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::ProxyError;
use crate::config::Config;

pub struct HttpServer {
    config: Arc<RwLock<Config>>,
}

impl HttpServer {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<bytes::Bytes>>, Infallible> {
        // 获取当前配置
        let current_config = self.config.read().await.clone();
        
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/api/config") => {
                self.serve_config_api(&current_config).await
            }
            (&Method::GET, "/health") => {
                self.serve_health_check().await
            }
            (&Method::GET, path) => {
                self.serve_static_file(&current_config, path).await
            }
            _ => Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Full::new(bytes::Bytes::from("Method not allowed")))
                .unwrap()),
        }
    }

    async fn serve_config_api(&self, config: &Config) -> Result<Response<Full<bytes::Bytes>>, Infallible> {
        match serde_json::to_string_pretty(config) {
            Ok(config_json) => {
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Full::new(bytes::Bytes::from(config_json)))
                    .unwrap())
            }
            Err(e) => {
                eprintln!("Failed to serialize config: {}", e);
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Full::new(bytes::Bytes::from("Internal Server Error")))
                    .unwrap())
            }
        }
    }

    async fn serve_health_check(&self) -> Result<Response<Full<bytes::Bytes>>, Infallible> {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain")
            .body(Full::new(bytes::Bytes::from("OK")))
            .unwrap())
    }

    async fn serve_static_file(&self, config: &Config, path: &str) -> Result<Response<Full<bytes::Bytes>>, Infallible> {
        // 根据配置判断是否启用静态文件服务
        if config.features.static_file_serving {
            // 这里应该调用静态文件服务模块
            // 暂时返回简单的响应
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(Full::new(bytes::Bytes::from(format!("Serving file: {}", path))))
                .unwrap())
        } else {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(bytes::Bytes::from("Static file serving is disabled")))
                .unwrap())
        }
    }
    
    // 更新配置的方法
    pub async fn update_config(&self, new_config: Config) {
        let mut config = self.config.write().await;
        *config = new_config;
    }
}