//! WebSocket 支持模块

use hyper::{header, Method, Request, Response, StatusCode};
use http_body_util::Full;
use tokio_tungstenite::{accept_async, tungstenite::Error as WsError};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;

pub struct WebSocketProxy {
    backend_addr: String,
}

impl WebSocketProxy {
    pub fn new(backend_addr: String) -> Self {
        Self { backend_addr }
    }

    /// 检查请求是否为 WebSocket 升级请求
    pub fn is_websocket_upgrade<B>(&self, req: &Request<B>) -> bool {
        if req.method() != Method::GET {
            return false;
        }

        let has_upgrade_header = req
            .headers()
            .get(header::UPGRADE)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.to_lowercase() == "websocket")
            .unwrap_or(false);

        let has_connection_header = req
            .headers()
            .get(header::CONNECTION)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.to_lowercase().contains("upgrade"))
            .unwrap_or(false);

        let has_sec_websocket_key = req.headers().contains_key("sec-websocket-key");

        has_upgrade_header && has_connection_header && has_sec_websocket_key
    }

    /// 处理 WebSocket 连接
    pub async fn handle_websocket(
        &self,
        mut req: Request<hyper::body::Incoming>,
        client_addr: SocketAddr,
    ) -> Result<Response<Full<bytes::Bytes>>, Box<dyn std::error::Error>> {
        println!("Handling WebSocket connection from {}", client_addr);

        // 建立与客户端的 WebSocket 连接
        let (client_ws_stream, client_response) = accept_async(req).await?;

        // 在实际实现中，这里会连接到后端 WebSocket 服务器
        // 为简化起见，我们创建一个回显服务器
        tokio::spawn(async move {
            let (mut client_ws_sender, mut client_ws_receiver) = client_ws_stream.split();

            // 简单的回显逻辑：接收消息并发送回去
            while let Some(msg) = client_ws_receiver.next().await {
                match msg {
                    Ok(message) => {
                        if let Err(e) = client_ws_sender.send(message).await {
                            match e {
                                WsError::ConnectionClosed | WsError::AlreadyClosed => break,
                                _ => {
                                    eprintln!("WebSocket error: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("WebSocket receive error: {}", e);
                        break;
                    }
                }
            }

            println!("WebSocket connection closed for {}", client_addr);
        });

        Ok(client_response.map(|_| Full::new(bytes::Bytes::new())))
    }

    /// 代理 WebSocket 连接到后端服务器
    pub async fn proxy_to_backend(
        &self,
        mut req: Request<hyper::body::Incoming>,
        client_addr: SocketAddr,
    ) -> Result<Response<Full<bytes::Bytes>>, Box<dyn std::error::Error>> {
        println!("Proxying WebSocket connection from {} to backend {}", client_addr, self.backend_addr);

        // 建立与客户端的 WebSocket 连接
        let (client_ws_stream, client_response) = accept_async(req).await?;

        // 在实际实现中，这里会连接到后端 WebSocket 服务器
        // 为简化起见，我们只记录日志
        tokio::spawn(async move {
            let (mut client_ws_sender, mut client_ws_receiver) = client_ws_stream.split();

            // 这里应该连接到后端服务器并转发消息
            // 为简化起见，我们只实现基本的回显功能
            while let Some(msg) = client_ws_receiver.next().await {
                match msg {
                    Ok(message) => {
                        if let Err(e) = client_ws_sender.send(message).await {
                            match e {
                                WsError::ConnectionClosed | WsError::AlreadyClosed => break,
                                _ => {
                                    eprintln!("WebSocket error: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("WebSocket receive error: {}", e);
                        break;
                    }
                }
            }

            println!("WebSocket proxy connection closed for {}", client_addr);
        });

        Ok(client_response.map(|_| Full::new(bytes::Bytes::new())))
    }
}