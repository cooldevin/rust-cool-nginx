//! 压缩模块实现

use hyper::{Response, header, StatusCode};
use http_body_util::Full;
use flate2::{Compression, write::GzEncoder};
use std::io::Write;

pub struct CompressionModule;

impl CompressionModule {
    pub fn new() -> Self {
        Self
    }

    pub fn compress_response(&self, response: Response<Full<bytes::Bytes>>, accept_encoding: &str) -> Result<Response<Full<bytes::Bytes>>, Box<dyn std::error::Error>> {
        // 检查客户端是否支持 gzip
        if accept_encoding.contains("gzip") {
            // 检查内容类型是否适合压缩
            if let Some(content_type) = response.headers().get(header::CONTENT_TYPE) {
                let content_type_str = content_type.to_str().unwrap_or("");
                if Self::is_compressible_content_type(content_type_str) {
                    // 检查内容长度是否足够大以值得压缩
                    let body = response.body();
                    if body.size_hint().lower() > 200 {
                        return self.gzip_compress(response);
                    }
                }
            }
        }

        // 如果不满足压缩条件，返回原始响应
        Ok(response)
    }

    fn gzip_compress(&self, response: Response<Full<bytes::Bytes>>) -> Result<Response<Full<bytes::Bytes>>, Box<dyn std::error::Error>> {
        let (parts, body) = response.into_parts();
        let body_bytes = body.collect().await?.to_bytes();
        
        // 执行 gzip 压缩
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&body_bytes)?;
        let compressed_bytes = encoder.finish()?;

        // 构建新的响应
        let mut new_response = Response::builder()
            .status(parts.status)
            .body(Full::new(bytes::Bytes::from(compressed_bytes)))?;

        // 复制原始响应头
        *new_response.headers_mut() = parts.headers.clone();
        
        // 添加压缩相关头部
        new_response.headers_mut().insert(header::CONTENT_ENCODING, header::HeaderValue::from_static("gzip"));
        new_response.headers_mut().insert(header::VARY, header::HeaderValue::from_static("Accept-Encoding"));

        Ok(new_response)
    }

    fn is_compressible_content_type(content_type: &str) -> bool {
        // 定义可压缩的内容类型
        const COMPRESSIBLE_TYPES: &[&str] = &[
            "text/html",
            "text/css",
            "text/plain",
            "text/xml",
            "text/javascript",
            "application/javascript",
            "application/json",
            "application/xml",
        ];

        COMPRESSIBLE_TYPES.iter().any(|&t| content_type.starts_with(t))
    }
}