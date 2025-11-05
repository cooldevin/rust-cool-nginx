//! 反向代理实现

use hyper::{Method, Request, Response, StatusCode};
use http_body_util::Full;
use std::convert::Infallible;
use crate::error::ProxyError;

pub struct ReverseProxy {
    backend_addr: String,
}

impl ReverseProxy {
    pub fn new(backend_addr: String) -> Self {
        Self {
            backend_addr,
        }
    }

    pub async fn forward_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<bytes::Bytes>>, ProxyError> {
        // 实现请求转发逻辑
        // 这里应该包含完整的代理逻辑，包括修改请求头、转发请求、处理响应等
        
        // 暂时返回模拟响应
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain")
            .header("X-Proxy", "rust-cool-nginx")
            .body(Full::new(bytes::Bytes::from(format!(
                "Forwarded request to backend: {}\nOriginal request: {} {}",
                self.backend_addr,
                req.method(),
                req.uri()
            ))))
            .unwrap())
    }
}