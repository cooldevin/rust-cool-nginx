//! SMTP 代理实现

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct SmtpProxy {
    addr: String,
}

impl SmtpProxy {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("SMTP proxy listening on {}", self.addr);

        loop {
            let (mut socket, _) = listener.accept().await?;
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(&mut socket).await {
                    eprintln!("Error handling SMTP client: {}", e);
                }
            });
        }
    }

    async fn handle_client(socket: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        // 发送欢迎信息
        socket.write_all(b"220 SMTP Proxy Ready\r\n").await?;
        
        let mut buf = [0; 1024];
        loop {
            let n = socket.read(&mut buf).await?;
            if n == 0 {
                return Ok(());
            }
            
            let data = &buf[..n];
            let command = String::from_utf8_lossy(data);
            println!("SMTP command received: {}", command.trim());
            
            // 简单处理一些基本命令
            let command_upper = command.to_uppercase();
            if command_upper.starts_with("EHLO") {
                socket.write_all(b"250-smtp.proxy.local\r\n").await?;
                socket.write_all(b"250-PIPELINING\r\n").await?;
                socket.write_all(b"250-SIZE 10240000\r\n").await?;
                socket.write_all(b"250-VRFY\r\n").await?;
                socket.write_all(b"250-ETRN\r\n").await?;
                socket.write_all(b"250-STARTTLS\r\n").await?;
                socket.write_all(b"250-AUTH PLAIN LOGIN\r\n").await?;
                socket.write_all(b"250-AUTH=PLAIN LOGIN\r\n").await?;
                socket.write_all(b"250-ENHANCEDSTATUSCODES\r\n").await?;
                socket.write_all(b"250-8BITMIME\r\n").await?;
                socket.write_all(b"250 DSN\r\n").await?;
            } else if command_upper.starts_with("HELO") {
                socket.write_all(b"250 smtp.proxy.local\r\n").await?;
            } else if command_upper.starts_with("MAIL FROM") {
                socket.write_all(b"250 Ok\r\n").await?;
            } else if command_upper.starts_with("RCPT TO") {
                socket.write_all(b"250 Ok\r\n").await?;
            } else if command_upper.starts_with("DATA") {
                socket.write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n").await?;
                // 等待数据结束标记
                loop {
                    let n = socket.read(&mut buf).await?;
                    if n == 0 {
                        return Ok(());
                    }
                    if n >= 3 && buf[n-3] == b'\r' && buf[n-2] == b'\n' && buf[n-1] == b'.' {
                        break;
                    }
                }
                socket.write_all(b"250 Ok: queued\r\n").await?;
            } else if command_upper.starts_with("QUIT") {
                socket.write_all(b"221 Bye\r\n").await?;
                break;
            } else if command_upper.starts_with("RSET") {
                socket.write_all(b"250 Ok\r\n").await?;
            } else if command_upper.starts_with("NOOP") {
                socket.write_all(b"250 Ok\r\n").await?;
            } else if command_upper.starts_with("VRFY") {
                socket.write_all(b"252 Cannot VRFY user\r\n").await?;
            } else if command_upper.starts_with("EXPN") {
                socket.write_all(b"252 Cannot EXPN\r\n").await?;
            } else {
                // 对于未知命令，返回错误
                socket.write_all(b"500 Command not understood\r\n").await?;
            }
        }
        
        Ok(())
    }
}