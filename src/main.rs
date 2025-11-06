mod config;
mod server;
mod client;
mod error;
mod static_server;
mod modules;

use config::Config;
use server::ProxyServer;
use static_server::StaticServer;
use std::env;
use std::net::SocketAddr;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "static" {
        // 启动静态文件服务器
        let static_root = args.get(2).cloned().unwrap_or_else(|| ".".to_string());
        // 尝试从配置文件读取监听地址，如果没有则使用默认值
        let config = Config::load().unwrap_or_else(|_| Config::default());
        let default_addr: SocketAddr = ([127, 0, 0, 1], 80).into(); // 更改为默认使用80端口以匹配配置
        let addr = args.get(3).and_then(|s| s.parse::<SocketAddr>().ok())
                       .unwrap_or_else(|| {
                           config.server.listen_addr
                               .parse::<SocketAddr>()
                               .unwrap_or(default_addr)
                       });
        
        println!("Starting static file server on http://{} serving {}", addr, static_root);
        
        let static_server = StaticServer::new(static_root);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        
        // 创建一个用于优雅关闭的信号监听
        let ctrl_c = signal::ctrl_c();
        
        println!("Server running. Press Ctrl+C to stop...");
        
        tokio::pin!(ctrl_c);
        
        loop {
            tokio::select! {
                // 处理传入的连接
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            let static_server = static_server.clone();
                            
                            tokio::task::spawn(async move {
                                use hyper::service::service_fn;
                                use hyper_util::rt::TokioIo;
                                use hyper::server::conn::http1;
                                
                                let io = TokioIo::new(stream);
                                
                                let service = service_fn(move |req| {
                                    let static_server = static_server.clone();
                                    async move {
                                        static_server.handle_request(req).await.map_err(|e| {
                                            eprintln!("Error handling request: {}", e);
                                            panic!("Error handling request: {}", e)
                                        })
                                    }
                                });
                                
                                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                                    eprintln!("Error serving connection: {:?}", err);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Error accepting connection: {:?}", e);
                        }
                    }
                }
                // 处理关闭信号
                _ = &mut ctrl_c => {
                    println!("Received Ctrl+C, shutting down server...");
                    break;
                }
            }
        }
    } else {
        // 启动反向代理服务器
        let config: Config = Config::load().unwrap_or_else(|_| Config::default());
        let server: ProxyServer = ProxyServer::new(config);
        
        println!("Starting reverse proxy server. Press Ctrl+C to stop...");
        
        // 使用 tokio::select! 宏来同时处理服务器和关闭信号
        tokio::select! {
            result = server.start() => {
                if let Err(e) = result {
                    eprintln!("Server error: {}", e);
                }
            }
            _ = signal::ctrl_c() => {
                println!("Received Ctrl+C, shutting down server...");
            }
        }
    }
    
    Ok(())
}