//! IMAP 代理实现

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct ImapProxy {
    addr: String,
}

impl ImapProxy {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("IMAP proxy listening on {}", self.addr);

        loop {
            let (mut socket, _) = listener.accept().await?;
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(&mut socket).await {
                    eprintln!("Error handling IMAP client: {}", e);
                }
            });
        }
    }

    async fn handle_client(socket: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        // 发送欢迎信息
        socket.write_all(b"* OK [CAPABILITY IMAP4rev1] IMAP Proxy Ready\r\n").await?;
        
        let mut buf = [0; 1024];
        loop {
            let n = socket.read(&mut buf).await?;
            if n == 0 {
                return Ok(());
            }
            
            let data = &buf[..n];
            let command = String::from_utf8_lossy(data);
            println!("IMAP command received: {}", command.trim());
            
            // 简单处理一些基本命令
            if command.to_uppercase().starts_with("CAPABILITY") {
                socket.write_all(b"* CAPABILITY IMAP4rev1\r\n").await?;
                // 提取标签并发送完成响应
                if let Some(tag) = command.split_whitespace().next() {
                    let response = format!("{} OK CAPABILITY completed\r\n", tag);
                    socket.write_all(response.as_bytes()).await?;
                }
            } else if command.to_uppercase().starts_with("NOOP") {
                if let Some(tag) = command.split_whitespace().next() {
                    let response = format!("{} OK NOOP completed\r\n", tag);
                    socket.write_all(response.as_bytes()).await?;
                }
            } else if command.to_uppercase().starts_with("LOGOUT") {
                socket.write_all(b"* BYE Proxy closing connection\r\n").await?;
                if let Some(tag) = command.split_whitespace().next() {
                    let response = format!("{} OK LOGOUT completed\r\n", tag);
                    socket.write_all(response.as_bytes()).await?;
                }
                break;
            } else {
                // 对于未知命令，返回错误
                if let Some(tag) = command.split_whitespace().next() {
                    let response = format!("{} BAD Command not understood\r\n", tag);
                    socket.write_all(response.as_bytes()).await?;
                }
            }
        }
        
        Ok(())
    }
}