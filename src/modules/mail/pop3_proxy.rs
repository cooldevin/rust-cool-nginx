//! POP3 代理实现

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Pop3Proxy {
    addr: String,
}

impl Pop3Proxy {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("POP3 proxy listening on {}", self.addr);

        loop {
            let (mut socket, _) = listener.accept().await?;
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(&mut socket).await {
                    eprintln!("Error handling POP3 client: {}", e);
                }
            });
        }
    }

    async fn handle_client(socket: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        // 发送欢迎信息
        socket.write_all(b"+OK POP3 Proxy Ready\r\n").await?;
        
        let mut buf = [0; 1024];
        loop {
            let n = socket.read(&mut buf).await?;
            if n == 0 {
                return Ok(());
            }
            
            let data = &buf[..n];
            let command = String::from_utf8_lossy(data);
            println!("POP3 command received: {}", command.trim());
            
            // 简单处理一些基本命令
            let command_upper = command.to_uppercase();
            if command_upper.starts_with("CAPA") {
                socket.write_all(b"+OK Capability list follows\r\n").await?;
                socket.write_all(b"TOP\r\n").await?;
                socket.write_all(b"USER\r\n").await?;
                socket.write_all(b"UIDL\r\n").await?;
                socket.write_all(b".\r\n").await?;
            } else if command_upper.starts_with("USER") {
                socket.write_all(b"+OK send PASS\r\n").await?;
            } else if command_upper.starts_with("PASS") {
                socket.write_all(b"+OK logged in\r\n").await?;
            } else if command_upper.starts_with("STAT") {
                socket.write_all(b"+OK 0 0\r\n").await?; // 0 messages, 0 bytes
            } else if command_upper.starts_with("LIST") {
                socket.write_all(b"+OK 0 messages\r\n").await?;
                socket.write_all(b".\r\n").await?;
            } else if command_upper.starts_with("QUIT") {
                socket.write_all(b"+OK Proxy closing connection\r\n").await?;
                break;
            } else {
                // 对于未知命令，返回错误
                socket.write_all(b"-ERR Command not understood\r\n").await?;
            }
        }
        
        Ok(())
    }
}