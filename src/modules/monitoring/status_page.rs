//! 状态页面模块

use hyper::{Response, StatusCode, header};
use http_body_util::Full;

use super::StatsSnapshot;

pub struct StatusPage;

impl StatusPage {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_status_page(&self, stats: &StatsSnapshot) -> Response<Full<bytes::Bytes>> {
        let html = self.build_status_html(stats);
        
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Full::new(bytes::Bytes::from(html)))
            .unwrap()
    }

    pub fn generate_json_status(&self, stats: &StatsSnapshot) -> Response<Full<bytes::Bytes>> {
        let json = self.build_status_json(stats);
        
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::new(bytes::Bytes::from(json)))
            .unwrap()
    }

    fn build_status_html(&self, stats: &StatsSnapshot) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Nginx Status</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 40px;
            background-color: #f4f4f4;
        }}
        .container {{
            max-width: 800px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 5px;
            box-shadow: 0 0 10px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            border-bottom: 2px solid #007acc;
            padding-bottom: 10px;
        }}
        .metric {{
            display: flex;
            justify-content: space-between;
            padding: 10px 0;
            border-bottom: 1px solid #eee;
        }}
        .metric-name {{
            font-weight: bold;
        }}
        .metric-value {{
            color: #007acc;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Nginx Server Status</h1>
        
        <div class="metric">
            <span class="metric-name">Active connections:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Total connections:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Total requests:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Total errors:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Requests per second:</span>
            <span class="metric-value">{:.2}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Error rate:</span>
            <span class="metric-value">{:.2}%</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Bytes in:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Bytes out:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">Uptime (seconds):</span>
            <span class="metric-value">{}</span>
        </div>
    </div>
</body>
</html>"#,
            stats.active_connections,
            stats.total_connections,
            stats.total_requests,
            stats.total_errors,
            stats.requests_per_second(),
            stats.error_rate(),
            stats.bytes_in,
            stats.bytes_out,
            stats.uptime
        )
    }

    fn build_status_json(&self, stats: &StatsSnapshot) -> String {
        format!(
            r#"{{
  "active_connections": {},
  "total_connections": {},
  "total_requests": {},
  "total_errors": {},
  "requests_per_second": {:.2},
  "error_rate": {:.2},
  "bytes_in": {},
  "bytes_out": {},
  "uptime": {}
}}"#,
            stats.active_connections,
            stats.total_connections,
            stats.total_requests,
            stats.total_errors,
            stats.requests_per_second(),
            stats.error_rate(),
            stats.bytes_in,
            stats.bytes_out,
            stats.uptime
        )
    }
}