use hyper::{Request, Response, StatusCode};
use http_body_util::Full;
use tokio::net::TcpStream;
use std::collections::HashMap;
use bytes::Bytes;

use crate::error::ProxyError;

#[derive(Clone)]
pub struct FastCgiClient {
    backend_addr: String,
}

impl FastCgiClient {
    pub fn new(backend_addr: String) -> Self {
        Self { backend_addr }
    }

    pub async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, ProxyError> {
        // 连接到 FastCGI 后端
        let _stream = TcpStream::connect(&self.backend_addr).await?;
        
        // 构建 FastCGI 参数
        let mut params = HashMap::new();
        params.insert("REQUEST_METHOD".to_string(), req.method().to_string());
        params.insert("REQUEST_URI".to_string(), req.uri().to_string());
        params.insert("SERVER_PROTOCOL".to_string(), format!("{:?}", req.version()));
        
        if let Some(host) = req.headers().get("host") {
            params.insert("HTTP_HOST".to_string(), host.to_str().unwrap_or("").to_string());
        }
        
        // 添加其他头部信息
        for (name, value) in req.headers() {
            let header_name = format!("HTTP_{}", name.as_str().to_uppercase().replace("-", "_"));
            params.insert(header_name, value.to_str().unwrap_or("").to_string());
        }
        
        // 收集请求体
        let body_bytes = http_body_util::BodyExt::collect(req.into_body()).await?.to_bytes();
        
        // 这里应该实现完整的 FastCGI 协议处理
        // 为简化起见，我们模拟一个 FastCGI 响应
        
        // 模拟处理结果
        let response_body = format!(
            "<html><body><h1>FastCGI Response</h1><p>Request processed for URI: {}</p><p>Body size: {} bytes</p></body></html>",
            params.get("REQUEST_URI").unwrap_or(&String::new()),
            body_bytes.len()
        );
        
        // 构建响应
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Full::new(Bytes::from(response_body)))?;
            
        Ok(response)
    }
}

// 简化的 FastCGI 负载均衡器
#[derive(Clone)]
pub struct FastCgiLoadBalancer {
    backends: Vec<String>,
    current: usize,
}

impl FastCgiLoadBalancer {
    pub fn new(backends: Vec<String>) -> Self {
        Self {
            backends,
            current: 0,
        }
    }

    pub fn get_next_backend(&mut self) -> Option<String> {
        if self.backends.is_empty() {
            return None;
        }
        
        let backend = self.backends[self.current].clone();
        self.current = (self.current + 1) % self.backends.len();
        Some(backend)
    }
    
    pub async fn handle_request(
        &mut self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, ProxyError> {
        match self.get_next_backend() {
            Some(backend_addr) => {
                let client = FastCgiClient::new(backend_addr);
                client.handle_request(req).await
            }
            None => {
                Ok(Response::builder()
                    .status(StatusCode::SERVICE_UNAVAILABLE)
                    .body(Full::new(Bytes::from("No FastCGI backends available")))?)
            }
        }
    }
}