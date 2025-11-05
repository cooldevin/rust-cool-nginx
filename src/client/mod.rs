use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::client::conn::http1::Builder;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

use crate::error::ProxyError;

#[derive(Clone)]
pub struct HttpClient;

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn forward_request(
        &self,
        req: Request<Incoming>,
        backend_addr: String,
    ) -> Result<Response<Full<Bytes>>, ProxyError> {
        println!("Connecting to backend: {}", backend_addr);
        // Connect to the backend server
        let stream = TcpStream::connect(&backend_addr).await?;
        let io = TokioIo::new(stream);

        // Establish HTTP connection to backend
        let (mut sender, conn) = Builder::new()
            .handshake(io)
            .await?;

        // Spawn a task to poll the connection
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Server connection failed: {}", err);
            }
        });

        // Extract request information before consuming the body
        let method = req.method().clone();
        let uri = req.uri().clone();
        let version = req.version();
        let headers = req.headers().clone();
        
        // Log request details
        println!("Original request: {} {} {:?}", method, uri, version);
        println!("Request headers:");
        for (name, value) in &headers {
            println!("  {}: {}", name, value.to_str().unwrap_or("<non-UTF8>"));
        }
        
        // Collect the body
        let body_bytes = http_body_util::BodyExt::collect(req.into_body()).await?.to_bytes();
        println!("Request body length: {}", body_bytes.len());
        if body_bytes.len() > 0 {
            println!("Request body (first 500 chars): {}", 
                String::from_utf8_lossy(&body_bytes[..std::cmp::min(500, body_bytes.len())])
            );
        }
        
        // Reconstruct the request with the collected body
        let mut req = Request::builder()
            .method(method)
            .uri(uri)
            .version(version)
            .body(Full::new(body_bytes))
            .map_err(|e| ProxyError::HttpHttpError(e))?;
        
        // Apply headers to the new request
        *req.headers_mut() = headers.clone();
        
        // Update the Host header to match the backend address
        if let Ok(host) = hyper::header::HeaderValue::from_str(&backend_addr) {
            req.headers_mut().insert(hyper::header::HOST, host);
        }
        
        // Modify the request to make it suitable for forwarding
        let mut parts = req.uri().clone().into_parts();
        
        // Construct a new URI with the backend address
        let scheme = hyper::http::uri::Scheme::HTTP;
        let authority = backend_addr.parse::<hyper::http::uri::Authority>()?;
        
        parts.scheme = Some(scheme);
        parts.authority = Some(authority);
        
        let uri = hyper::Uri::from_parts(parts)?;
        println!("Forwarding request to: {}", uri);

        *req.uri_mut() = uri;
        
        // Re-apply the original headers
        *req.headers_mut() = headers;

        // Send the request to the backend and get the response
        let resp = sender.send_request(req).await?;
        println!("Received response from backend, status: {}", resp.status());
        
        // Log response details
        println!("Response headers:");
        for (name, value) in resp.headers() {
            println!("  {}: {}", name, value.to_str().unwrap_or("<non-UTF8>"));
        }

        // Get status before moving the response
        let status = resp.status();

        // Collect the response body
        let collected = http_body_util::BodyExt::collect(resp.into_body()).await?;
        let response_body = collected.to_bytes();
        
        // Log response body (first 500 chars)
        if response_body.len() > 0 {
            let body_str = String::from_utf8_lossy(&response_body);
            println!("Response body (first 500 chars): {}", 
                if body_str.len() > 500 { 
                    &body_str[..500] 
                } else { 
                    &body_str 
                }
            );
        }

        // Build and return the response
        Ok(Response::builder()
            .status(status)
            .body(Full::new(response_body))?)
    }
}