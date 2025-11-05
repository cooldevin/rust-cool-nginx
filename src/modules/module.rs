//! 模块 trait 定义

use std::any::Any;

/// Nginx 模块 trait
pub trait NginxModule: Send + Sync {
    /// 获取模块名称
    fn name(&self) -> &str;
    
    /// 初始化模块
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// 重新加载模块配置
    fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// 关闭模块
    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// 获取模块版本
    fn version(&self) -> &str;
    
    /// 检查模块是否启用
    fn is_enabled(&self) -> bool;
    
    /// 获取模块配置
    fn get_config(&self) -> Option<&dyn Any>;
}

/// 核心模块 trait
pub trait CoreModule: NginxModule {
    /// 处理 HTTP 请求
    fn handle_http_request(&self, request: &hyper::Request<hyper::body::Incoming>) 
        -> Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, Box<dyn std::error::Error>>;
}

/// 事件模块 trait
pub trait EventModule: NginxModule {
    /// 处理事件
    fn handle_event(&self, event: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// 邮件模块 trait
pub trait MailModule: NginxModule {
    /// 处理邮件请求
    fn handle_mail_request(&self, protocol: &str, request: &str) 
        -> Result<String, Box<dyn std::error::Error>>;
}