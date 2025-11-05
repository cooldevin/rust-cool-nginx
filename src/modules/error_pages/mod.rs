//! 自定义错误页面模块

use hyper::{Response, StatusCode, header};
use http_body_util::Full;
use std::collections::HashMap;

pub struct ErrorPages {
    pages: HashMap<StatusCode, String>,
}

impl ErrorPages {
    pub fn new() -> Self {
        let mut error_pages = Self {
            pages: HashMap::new(),
        };
        
        // 添加默认错误页面
        error_pages.add_default_pages();
        error_pages
    }

    fn add_default_pages(&mut self) {
        self.pages.insert(
            StatusCode::NOT_FOUND,
            Self::create_default_error_page(404, "Not Found", "The requested resource was not found on this server.")
        );
        
        self.pages.insert(
            StatusCode::INTERNAL_SERVER_ERROR,
            Self::create_default_error_page(500, "Internal Server Error", "The server encountered an internal error and was unable to complete your request.")
        );
        
        self.pages.insert(
            StatusCode::FORBIDDEN,
            Self::create_default_error_page(403, "Forbidden", "You don't have permission to access this resource.")
        );
        
        self.pages.insert(
            StatusCode::BAD_REQUEST,
            Self::create_default_error_page(400, "Bad Request", "Your browser sent a request that this server could not understand.")
        );
        
        self.pages.insert(
            StatusCode::UNAUTHORIZED,
            Self::create_default_error_page(401, "Unauthorized", "This server could not verify that you are authorized to access the document requested.")
        );
    }

    pub fn add_page(&mut self, status_code: StatusCode, content: String) {
        self.pages.insert(status_code, content);
    }

    pub fn get_page(&self, status_code: StatusCode) -> Option<&String> {
        self.pages.get(&status_code)
    }

    pub fn create_error_response(&self, status_code: StatusCode) -> Response<Full<bytes::Bytes>> {
        let content = self.pages.get(&status_code)
            .cloned()
            .unwrap_or_else(|| Self::create_default_error_page(
                status_code.as_u16(), 
                status_code.canonical_reason().unwrap_or("Unknown Error"), 
                "An error occurred while processing your request."
            ));
            
        Response::builder()
            .status(status_code)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Full::new(bytes::Bytes::from(content)))
            .unwrap()
    }

    fn create_default_error_page(status_code: u16, title: &str, message: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>{} - {}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 40px;
            background-color: #f4f4f4;
        }}
        .container {{
            max-width: 600px;
            margin: 0 auto;
            background-color: white;
            padding: 30px;
            border-radius: 5px;
            box-shadow: 0 0 10px rgba(0,0,0,0.1);
            text-align: center;
        }}
        h1 {{
            color: #d9534f;
            font-size: 36px;
            margin-bottom: 20px;
        }}
        .error-code {{
            font-size: 72px;
            font-weight: bold;
            color: #d9534f;
            margin: 20px 0;
        }}
        .error-message {{
            font-size: 18px;
            color: #666;
            margin-bottom: 30px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error-code">{} {}</div>
        <h1>{}</h1>
        <div class="error-message">{}</div>
    </div>
</body>
</html>"#,
            status_code, title, status_code, title, title, message
        )
    }
}