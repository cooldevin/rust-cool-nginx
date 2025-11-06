//! 监控模块
//! 实现服务器监控和统计功能

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

pub struct StatusPage;

impl StatusPage {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate_status_page(&self, stats: &StatsSnapshot) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>服务器监控统计</title>
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
        <h1>服务器监控统计</h1>
        
        <div class="metric">
            <span class="metric-name">活跃连接数:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">总连接数:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">总请求数:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">错误请求数:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">每秒请求数:</span>
            <span class="metric-value">{:.2}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">错误率:</span>
            <span class="metric-value">{:.2}%</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">接收字节数:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">发送字节数:</span>
            <span class="metric-value">{}</span>
        </div>
        
        <div class="metric">
            <span class="metric-name">运行时间 (秒):</span>
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

    pub fn generate_json_status(&self, stats: &StatsSnapshot) -> String {
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

#[derive(Clone)]
pub struct ServerStats {
    // 请求统计
    pub total_requests: Arc<AtomicU64>,
    pub total_errors: Arc<AtomicU64>,
    
    // 流量统计
    pub bytes_in: Arc<AtomicU64>,
    pub bytes_out: Arc<AtomicU64>,
    
    // 连接统计
    pub active_connections: Arc<AtomicUsize>,
    pub total_connections: Arc<AtomicU64>,
    
    // 时间统计
    pub start_time: u64,
}

impl ServerStats {
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
            bytes_in: Arc::new(AtomicU64::new(0)),
            bytes_out: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicUsize::new(0)),
            total_connections: Arc::new(AtomicU64::new(0)),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    pub fn increment_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_errors(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_bytes_in(&self, bytes: u64) {
        self.bytes_in.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn add_bytes_out(&self, bytes: u64) {
        self.bytes_out.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn increment_connections(&self) -> usize {
        self.total_connections.fetch_add(1, Ordering::Relaxed);
        self.active_connections.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn decrement_connections(&self) -> usize {
        let current = self.active_connections.fetch_sub(1, Ordering::Relaxed);
        if current > 0 {
            current - 1
        } else {
            0
        }
    }

    pub fn get_stats(&self) -> StatsSnapshot {
        StatsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            bytes_in: self.bytes_in.load(Ordering::Relaxed),
            bytes_out: self.bytes_out.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            total_connections: self.total_connections.load(Ordering::Relaxed),
            uptime: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0) - self.start_time,
        }
    }
}

pub struct StatsSnapshot {
    pub total_requests: u64,
    pub total_errors: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub active_connections: usize,
    pub total_connections: u64,
    pub uptime: u64,
}

impl StatsSnapshot {
    pub fn requests_per_second(&self) -> f64 {
        if self.uptime > 0 {
            self.total_requests as f64 / self.uptime as f64
        } else {
            0.0
        }
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_requests > 0 {
            self.total_errors as f64 / self.total_requests as f64 * 100.0
        } else {
            0.0
        }
    }
}