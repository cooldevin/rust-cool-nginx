//! HTTP 服务器核心实现

use hyper::{Method, Request, Response, StatusCode};
use http_body_util::Full;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::error::ProxyError;
use crate::config::Config;

pub struct HttpServer {
    config: Config,
}

impl HttpServer {
    pub fn new(config: Config) -> Self {
        Self {
            config,
        }
    }

    pub async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<bytes::Bytes>>, Infallible> {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/api/config") => {
                self.serve_config_api().await
            }
            (&Method::GET, "/health") => {
                self.serve_health_check().await
            }
            (&Method::GET, path) => {
                self.serve_static_file(path).await
            }
            _ => Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Full::new(bytes::Bytes::from("Method not allowed")))
                .unwrap()),
        }
    }

    async fn serve_config_api(&self) -> Result<Response<Full<bytes::Bytes>>, Infallible> {
        match serde_json::to_string_pretty(&self.config) {
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

    async fn serve_static_file(&self, path: &str) -> Result<Response<Full<bytes::Bytes>>, Infallible> {
        // 这里应该调用静态文件服务模块
        // 暂时返回简单的响应
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain")
            .body(Full::new(bytes::Bytes::from(format!("Serving file: {}", path))))
            .unwrap())
    }
}