//! 基本认证实现

use hyper::{Request, header};
use http_body_util::Full;
use std::collections::HashMap;

pub struct BasicAuth {
    users: HashMap<String, String>, // username -> password hash
}

impl BasicAuth {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, username: String, password: String) {
        // 在实际实现中，我们应该存储密码的哈希值而不是明文密码
        // 这里为了简化，我们直接存储密码
        self.users.insert(username, password);
    }

    pub fn authenticate(&self, req: &Request<impl hyper::body::Body>) -> Result<bool, Box<dyn std::error::Error>> {
        // 检查 Authorization 头
        if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Basic ") {
                    let encoded = &auth_str[6..]; // 移除 "Basic " 前缀
                    
                    // 解码 base64
                    if let Ok(decoded) = base64::decode(encoded) {
                        if let Ok(credentials) = String::from_utf8(decoded) {
                            let parts: Vec<&str> = credentials.split(':').collect();
                            if parts.len() == 2 {
                                let username = parts[0];
                                let password = parts[1];
                                
                                // 验证用户名和密码
                                if let Some(stored_password) = self.users.get(username) {
                                    return Ok(stored_password == password);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }

    pub fn create_challenge_response(&self) -> hyper::Response<Full<bytes::Bytes>> {
        hyper::Response::builder()
            .status(hyper::StatusCode::UNAUTHORIZED)
            .header(header::WWW_AUTHENTICATE, "Basic realm=\"Restricted Area\"")
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Full::new(bytes::Bytes::from("Unauthorized")))
            .unwrap()
    }
}