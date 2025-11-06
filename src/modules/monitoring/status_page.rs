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
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>服务器监控统计 - Rust-Cool-Nginx</title>
    <style>
        :root {{
            --primary: #0ff;
            --secondary: #f0f;
            --dark: #0a0a1a;
            --darker: #050510;
            --light: #ffffff;
            --gray: #e0e0e0;
            --dark-gray: #2d2d4d;
            --gradient-start: #00dbde;
            --gradient-end: #fc00ff;
        }}
        
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background-color: var(--dark);
            color: var(--light);
            background-image: 
                radial-gradient(circle at 10% 20%, rgba(0, 219, 222, 0.05) 0%, transparent 20%),
                radial-gradient(circle at 90% 80%, rgba(252, 0, 255, 0.05) 0%, transparent 20%);
            min-height: 100vh;
            overflow-x: hidden;
            line-height: 1.6;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}
        
        header {{
            text-align: center;
            padding: 60px 20px 40px;
            position: relative;
        }}
        
        h1 {{
            font-size: 2.5rem;
            background: linear-gradient(90deg, var(--gradient-start), var(--gradient-end));
            -webkit-background-clip: text;
            background-clip: text;
            color: transparent;
            margin-bottom: 20px;
            text-shadow: 0 0 20px rgba(0, 219, 222, 0.3);
            letter-spacing: 2px;
        }}
        
        .subtitle {{
            font-size: 1.2rem;
            color: var(--gray);
            max-width: 700px;
            margin: 0 auto 40px;
            line-height: 1.7;
        }}
        
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
            gap: 30px;
            margin: 50px 0;
        }}
        
        .stat-card {{
            background: rgba(20, 20, 40, 0.7);
            border: 1px solid rgba(0, 255, 255, 0.1);
            border-radius: 15px;
            padding: 30px;
            transition: all 0.3s ease;
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
            position: relative;
            overflow: hidden;
            text-align: center;
        }}
        
        .stat-card::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            width: 100%;
            height: 4px;
            background: linear-gradient(90deg, var(--gradient-start), var(--gradient-end));
            transform: scaleX(0);
            transition: transform 0.3s ease;
            transform-origin: left;
        }}
        
        .stat-card:hover::before {{
            transform: scaleX(1);
        }}
        
        .stat-title {{
            font-size: 1.4rem;
            margin-bottom: 20px;
            color: var(--primary);
        }}
        
        .stat-value {{
            font-size: 2.5rem;
            font-weight: bold;
            color: #fff;
            margin: 15px 0;
            text-shadow: 0 0 15px rgba(0, 219, 222, 0.4);
        }}
        
        .stat-unit {{
            font-size: 1.1rem;
            color: var(--gray);
            margin-top: 10px;
        }}
        
        .actions {{
            text-align: center;
            margin: 60px 0;
            padding: 30px 0;
        }}
        
        .btn {{
            display: inline-block;
            padding: 14px 32px;
            background: linear-gradient(90deg, var(--gradient-start), var(--gradient-end));
            color: white;
            text-decoration: none;
            border-radius: 50px;
            font-weight: bold;
            transition: all 0.3s ease;
            border: none;
            cursor: pointer;
            box-shadow: 0 0 20px rgba(0, 219, 222, 0.4);
            font-size: 1.1rem;
            letter-spacing: 0.5px;
        }}
        
        .btn:hover {{
            transform: scale(1.05);
            box-shadow: 0 0 30px rgba(0, 219, 222, 0.7);
        }}
        
        .btn-secondary {{
            background: transparent;
            border: 2px solid var(--primary);
            color: var(--primary);
            margin-left: 15px;
        }}
        
        .btn-secondary:hover {{
            background: rgba(0, 255, 255, 0.1);
        }}
        
        footer {{
            text-align: center;
            padding: 40px 0;
            color: #aaa;
            border-top: 1px solid rgba(255, 255, 255, 0.1);
            margin-top: 70px;
        }}
        
        @media (max-width: 768px) {{
            .stats-grid {{
                grid-template-columns: 1fr;
                gap: 20px;
            }}
            
            h1 {{
                font-size: 2rem;
            }}
            
            .subtitle {{
                font-size: 1rem;
            }}
            
            .stat-value {{
                font-size: 2rem;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>服务器监控统计</h1>
            <p class="subtitle">实时监控 Rust-Cool-Nginx 服务器的运行状态和性能指标</p>
        </header>
        
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-title">活跃连接数</div>
                <div class="stat-value">{}</div>
                <div class="stat-unit">当前连接</div>
            </div>
            
            <div class="stat-card">
                <div class="stat-title">总连接数</div>
                <div class="stat-value">{}</div>
                <div class="stat-unit">历史累计</div>
            </div>
            
            <div class="stat-card">
                <div class="stat-title">总请求数</div>
                <div class="stat-value">{}</div>
                <div class="stat-unit">历史累计</div>
            </div>
            
            <div class="stat-card">
                <div class="stat-title">错误请求数</div>
                <div class="stat-value">{}</div>
                <div class="stat-unit">历史累计</div>
            </div>
            
            <div class="stat-card">
                <div class="stat-title">请求处理速度</div>
                <div class="stat-value">{:.2}</div>
                <div class="stat-unit">每秒请求数</div>
            </div>
            
            <div class="stat-card">
                <div class="stat-title">错误率</div>
                <div class="stat-value">{:.2}%</div>
                <div class="stat-unit">百分比</div>
            </div>
            
            <div class="stat-card">
                <div class="stat-title">接收流量</div>
                <div class="stat-value">{}</div>
                <div class="stat-unit">字节</div>
            </div>
            
            <div class="stat-card">
                <div class="stat-title">发送流量</div>
                <div class="stat-value">{}</div>
                <div class="stat-unit">字节</div>
            </div>
        </div>
        
        <div class="actions">
            <a href="/" class="btn">返回首页</a>
            <a href="/config.html" class="btn btn-secondary">查看配置</a>
        </div>
        
        <footer>
            <p>© 2025 Rust-Cool-Nginx - 高性能 Web 服务器 | 基于 Rust 和 Tokio 构建</p>
        </footer>
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
            stats.bytes_out
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