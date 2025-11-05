use hyper::{Method, Request, Response, StatusCode, header};
use http_body_util::Full;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::ProxyError;
use crate::config::Config;

#[derive(Clone)]
pub struct StaticServer {
    root_dir: PathBuf,
    enable_auto_index: bool,
}

impl StaticServer {
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
            enable_auto_index: true, // 默认启用自动索引
        }
    }

    pub fn with_auto_index<P: AsRef<Path>>(root_dir: P, enable_auto_index: bool) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
            enable_auto_index,
        }
    }

    pub async fn serve_file(&self, req: &Request<hyper::body::Incoming>, path: &str) -> Result<Response<Full<bytes::Bytes>>, ProxyError> {
        println!("Requested path: {}", path);
        
        // 移除前导斜杠（如果有的话）
        let clean_path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        
        println!("Cleaned path: {}", clean_path);

        // 防止目录遍历攻击
        let mut full_path = self.root_dir.clone();
        if !clean_path.is_empty() {
            for component in Path::new(clean_path).components() {
                match component {
                    std::path::Component::Normal(part) => full_path.push(part),
                    _ => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Full::new(bytes::Bytes::from("Invalid path")))
                            .unwrap());
                    }
                }
            }
        }

        // 检查路径是否存在
        if !full_path.exists() {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(bytes::Bytes::from("File not found")))
                .unwrap());
        }

        // 获取文件元数据
        let metadata = tokio::fs::metadata(&full_path).await?;
        let last_modified = metadata.modified()?;
        
        // 转换为 HTTP 时间格式
        let last_modified_str = format_http_date(last_modified);
        
        // 检查 If-Modified-Since 头
        if let Some(if_modified_since) = req.headers().get(header::IF_MODIFIED_SINCE) {
            if let Ok(if_modified_since_str) = if_modified_since.to_str() {
                if let Ok(if_modified_since_time) = parse_http_date(if_modified_since_str) {
                    // 如果文件未修改，返回 304
                    if last_modified <= if_modified_since_time {
                        return Ok(Response::builder()
                            .status(StatusCode::NOT_MODIFIED)
                            .header(header::LAST_MODIFIED, &last_modified_str)
                            .body(Full::new(bytes::Bytes::new()))
                            .unwrap());
                    }
                }
            }
        }

        // 如果是目录，则尝试寻找 index.html 或生成目录索引
        if full_path.is_dir() {
            let index_path = full_path.join("index.html");
            if index_path.exists() && index_path.is_file() {
                full_path = index_path;
            } else if self.enable_auto_index {
                // 生成目录索引
                return self.generate_directory_listing(&full_path, path).await;
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Full::new(bytes::Bytes::from("Directory listing denied")))
                    .unwrap());
            }
        }

        println!("Full path: {:?}", full_path);

        // 检查文件是否存在
        if !full_path.exists() || !full_path.is_file() {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(bytes::Bytes::from("File not found")))
                .unwrap());
        }

        // 安全检查：确保路径仍在根目录内
        if !full_path.starts_with(&self.root_dir) {
            return Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Full::new(bytes::Bytes::from("Access forbidden")))
                .unwrap());
        }

        // 读取文件内容
        let mut file = File::open(&full_path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;

        // 获取 MIME 类型
        let mime_type = mime_guess::from_path(&full_path).first_or_octet_stream();

        // 构造响应
        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type.to_string())
            .header(header::CONTENT_LENGTH, contents.len())
            .header(header::LAST_MODIFIED, last_modified_str)
            .body(Full::new(bytes::Bytes::from(contents)))?;

        Ok(response)
    }

    // 生成目录索引
    async fn generate_directory_listing(&self, dir_path: &Path, request_path: &str) -> Result<Response<Full<bytes::Bytes>>, ProxyError> {
        let mut listing = String::new();
        
        // HTML 页面头部
        listing.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        listing.push_str("<meta charset=\"utf-8\">\n");
        listing.push_str("<title>Directory listing for ");
        listing.push_str(request_path);
        listing.push_str("</title>\n</head>\n<body>\n");
        listing.push_str("<h1>Directory listing for ");
        listing.push_str(request_path);
        listing.push_str("</h1>\n<hr>\n<ul>\n");
        
        // 添加返回上级目录的链接（如果不是根目录）
        if request_path != "/" {
            listing.push_str("<li><a href=\"../\">../</a></li>\n");
        }
        
        // 读取目录内容
        let mut entries = tokio::fs::read_dir(dir_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            // 跳过隐藏文件
            if !file_name_str.starts_with('.') {
                let metadata = entry.metadata().await?;
                let mut link_name = file_name_str.to_string();
                
                if metadata.is_dir() {
                    link_name.push('/');
                }
                
                listing.push_str("<li><a href=\"");
                listing.push_str(&link_name);
                listing.push_str("\">");
                listing.push_str(&link_name);
                listing.push_str("</a></li>\n");
            }
        }
        
        listing.push_str("</ul>\n<hr>\n</body>\n</html>");
        
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Full::new(bytes::Bytes::from(listing)))?)
    }

    // 提供配置信息的API端点
    async fn serve_config_api(&self) -> Result<Response<Full<bytes::Bytes>>, ProxyError> {
        // 尝试加载配置
        let config = Config::load().unwrap_or_else(|_| Config::default());
        
        // 将配置序列化为JSON
        let config_json = serde_json::to_string_pretty(&config)?;
        
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::new(bytes::Bytes::from(config_json)))?)
    }

    pub async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<bytes::Bytes>>, Infallible> {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/api/config") => {
                // 提供配置信息的API端点
                match self.serve_config_api().await {
                    Ok(response) => Ok(response),
                    Err(_) => Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(bytes::Bytes::from("Internal Server Error")))
                        .unwrap()),
                }
            }
            (&Method::GET, path) => {
                match self.serve_file(&req, path).await {
                    Ok(response) => Ok(response),
                    Err(_) => Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(bytes::Bytes::from("Internal Server Error")))
                        .unwrap()),
                }
            }
            _ => Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Full::new(bytes::Bytes::from("Method not allowed")))
                .unwrap()),
        }
    }
}

// 格式化时间为 HTTP 日期格式
fn format_http_date(time: SystemTime) -> String {
    use chrono::{DateTime, Utc};
    
    let datetime: DateTime<Utc> = time.into();
    datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

// 解析 HTTP 日期格式
fn parse_http_date(date_str: &str) -> Result<SystemTime, Box<dyn std::error::Error>> {
    use chrono::{DateTime, Utc, TimeZone};
    
    let datetime: DateTime<Utc> = DateTime::parse_from_str(date_str, "%a, %d %b %Y %H:%M:%S GMT")?
        .with_timezone(&Utc);
    let system_time: SystemTime = datetime.into();
    Ok(system_time)
}