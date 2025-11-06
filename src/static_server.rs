use hyper::{Method, Request, Response, StatusCode, header};
use http_body_util::Full;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tokio::sync::RwLock;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

use crate::error::ProxyError;
use crate::config::Config;

#[derive(Clone)]
pub struct StaticServer {
    root_dir: PathBuf,
    enable_auto_index: bool,
    config: Arc<RwLock<Config>>,
}

impl StaticServer {
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        // 加载配置
        let config = Config::load().unwrap_or_else(|_| Config::default());
        
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
            enable_auto_index: true, // 默认启用自动索引
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub fn with_auto_index<P: AsRef<Path>>(root_dir: P, enable_auto_index: bool) -> Self {
        // 加载配置
        let config = Config::load().unwrap_or_else(|_| Config::default());
        
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
            enable_auto_index,
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn serve_file(&self, req: &Request<hyper::body::Incoming>, path: &str) -> Result<Response<Full<bytes::Bytes>>, ProxyError> {
        // 获取当前最新的配置
        let current_config = self.config.read().await.clone();
        
        println!("Requested path: {}", path);
        
        // 根据配置判断是否启用静态文件服务
        if !current_config.features.static_file_serving {
            // 如果静态文件服务未启用，返回404
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(bytes::Bytes::from("Static file serving is disabled")))
                .unwrap());
        }
        
        // 访问控制检查
        if current_config.features.access_control.access_control.unwrap_or(false) {
            // 尝试从请求头中获取客户端IP
            let client_ip_opt = req.headers().get("X-Forwarded-For")
                .or_else(|| req.headers().get("X-Real-IP"))
                .and_then(|ip| ip.to_str().ok())
                .map(|s| s.to_string());
                
            if let Some(client_ip) = client_ip_opt {
                let allow_ips = &current_config.features.access_control.allow_ips;
                let deny_ips = &current_config.features.access_control.deny_ips;
                
                // 检查是否在拒绝列表中
                if let Some(deny_list) = deny_ips {
                    if deny_list.contains(&client_ip) {
                        return Ok(Response::builder()
                            .status(StatusCode::FORBIDDEN)
                            .body(Full::new(bytes::Bytes::from("Access denied")))
                            .unwrap());
                    }
                }
                
                // 检查是否在允许列表中
                if let Some(allow_list) = allow_ips {
                    if !allow_list.is_empty() && !allow_list.contains(&client_ip) {
                        return Ok(Response::builder()
                            .status(StatusCode::FORBIDDEN)
                            .body(Full::new(bytes::Bytes::from("Access denied")))
                            .unwrap());
                    }
                }
            }
        }
        
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
        
        // 检查是否应该启用 GZIP 压缩
        let should_gzip = current_config.features.gzip.gzip_compression.unwrap_or(false) 
            && contents.len() > current_config.features.gzip.gzip_min_length.unwrap_or(1024) as usize
            && req.headers().get("accept-encoding")
                .and_then(|encodings| encodings.to_str().ok())
                .map(|encodings| encodings.contains("gzip"))
                .unwrap_or(false);

        // 构造响应
        let response_builder = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type.to_string())
            .header(header::LAST_MODIFIED, last_modified_str);

        if should_gzip {
            // 执行 GZIP 压缩
            let compression_level = current_config.features.gzip.gzip_comp_level.unwrap_or(6);
            let mut encoder = GzEncoder::new(Vec::new(), Compression::new(compression_level));
            encoder.write_all(&contents)?;
            let compressed_contents = encoder.finish()?;
            
            let response = response_builder
                .header(header::CONTENT_ENCODING, "gzip")
                .header(header::CONTENT_LENGTH, compressed_contents.len())
                .body(Full::new(bytes::Bytes::from(compressed_contents)))?;
            
            Ok(response)
        } else {
            let response = response_builder
                .header(header::CONTENT_LENGTH, contents.len())
                .body(Full::new(bytes::Bytes::from(contents)))?;
            
            Ok(response)
        }
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
        // 获取当前配置
        let config = self.config.read().await;
        
        // 将配置序列化为JSON
        let config_json = serde_json::to_string_pretty(&*config)?;
        
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::new(bytes::Bytes::from(config_json)))?)
    }

    // 更新配置的API端点
    async fn update_config_api(&self, req: Request<hyper::body::Incoming>) -> Result<Response<Full<bytes::Bytes>>, ProxyError> {
        use http_body_util::BodyExt;
        
        // 读取请求体
        let body_bytes = req.into_body().collect().await?.to_bytes();
        let body_str = String::from_utf8(body_bytes.to_vec())
            .map_err(|e| ProxyError::Other(format!("UTF-8 decode error: {}", e)))?;
        
        // 解析新的配置
        let new_config: Config = match serde_json::from_str(&body_str) {
            Ok(config) => config,
            Err(e) => {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Full::new(bytes::Bytes::from(format!("{{\"error\": \"Invalid JSON: {}\"}}", e))))
                    .unwrap());
            }
        };
        
        // 更新配置
        {
            let mut config = self.config.write().await;
            *config = new_config;
        }
        
        // 尝试保存到文件
        match serde_json::to_string_pretty(&*self.config.read().await) {
            Ok(config_str) => {
                if let Err(e) = tokio::fs::write("nginx.conf", config_str).await {
                    eprintln!("Failed to save config to file: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize config: {}", e);
            }
        }
        
        // 返回成功响应
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::new(bytes::Bytes::from("{\"status\": \"success\"}")))
            .unwrap())
    }

    pub async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<bytes::Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/api/config") => {
                // 提供配置信息的API端点
                match self.serve_config_api().await {
                    Ok(response) => Ok(response),
                    Err(e) => {
                        eprintln!("Error serving config API: {}", e);
                        Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Full::new(bytes::Bytes::from("Internal Server Error")))
                            .unwrap())
                    },
                }
            }
            (&Method::POST, "/api/config") => {
                // 更新配置的API端点
                match self.update_config_api(req).await {
                    Ok(response) => Ok(response),
                    Err(e) => {
                        eprintln!("Error updating config: {}", e);
                        Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Full::new(bytes::Bytes::from("Internal Server Error")))
                            .unwrap())
                    },
                }
            }
            (&Method::GET, "/status") => {
                // 从监控模块导入所需组件
                use crate::modules::monitoring::{ServerStats, StatusPage, StatsSnapshot};
                
                // 创建临时统计信息（在实际应用中，这应该来自共享的ServerStats实例）
                let stats = ServerStats::new();
                let snapshot = stats.get_stats();
                let status_page = StatusPage::new();
                
                let html_content = status_page.generate_status_page(&snapshot);
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Full::new(bytes::Bytes::from(html_content)))
                    .unwrap())
            }
            (&Method::GET, "/api/status") => {
                // 从监控模块导入所需组件
                use crate::modules::monitoring::{ServerStats, StatusPage, StatsSnapshot};
                
                // 创建临时统计信息（在实际应用中，这应该来自共享的ServerStats实例）
                let stats = ServerStats::new();
                let snapshot = stats.get_stats();
                let status_page = StatusPage::new();
                
                let json_content = status_page.generate_json_status(&snapshot);
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Full::new(bytes::Bytes::from(json_content)))
                    .unwrap())
            }
            (&Method::GET, path) => {
                match self.serve_file(&req, path).await {
                    Ok(response) => Ok(response),
                    Err(e) => {
                        eprintln!("Error serving file: {}", e);
                        Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Full::new(bytes::Bytes::from("Internal Server Error")))
                            .unwrap())
                    },
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