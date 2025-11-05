//! 跨平台支持模块

use std::env;

#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub family: String,
}

impl PlatformInfo {
    pub fn new() -> Self {
        Self {
            os: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
            family: env::consts::FAMILY.to_string(),
        }
    }

    pub fn is_windows(&self) -> bool {
        self.os == "windows"
    }

    pub fn is_unix(&self) -> bool {
        self.family == "unix"
    }

    pub fn is_linux(&self) -> bool {
        self.os == "linux"
    }

    pub fn is_macos(&self) -> bool {
        self.os == "macos"
    }

    pub fn supports_async_io(&self) -> bool {
        // 所有主要平台都支持异步 IO
        true
    }

    pub fn get_default_config_path(&self) -> String {
        if self.is_windows() {
            r"C:\ProgramData\nginx\nginx.conf".to_string()
        } else {
            "/etc/nginx/nginx.conf".to_string()
        }
    }

    pub fn get_default_log_path(&self) -> String {
        if self.is_windows() {
            r"C:\ProgramData\nginx\logs".to_string()
        } else {
            "/var/log/nginx".to_string()
        }
    }

    pub fn get_default_pid_path(&self) -> String {
        if self.is_windows() {
            r"C:\ProgramData\nginx\nginx.pid".to_string()
        } else {
            "/var/run/nginx.pid".to_string()
        }
    }
}

/// 跨平台文件路径处理
pub struct PathUtils;

impl PathUtils {
    /// 标准化路径分隔符
    pub fn normalize_path(path: &str) -> String {
        if cfg!(windows) {
            path.replace('/', "\\")
        } else {
            path.replace('\\', "/")
        }
    }

    /// 连接路径
    pub fn join_paths(paths: &[&str]) -> String {
        if cfg!(windows) {
            paths.join("\\")
        } else {
            paths.join("/")
        }
    }

    /// 检查路径是否为绝对路径
    pub fn is_absolute_path(path: &str) -> bool {
        if cfg!(windows) {
            // Windows 绝对路径示例: C:\path\to\file 或 \\server\share\file
            path.len() >= 2 && path.chars().nth(1) == Some(':') || path.starts_with("\\\\")
        } else {
            // Unix 绝对路径以 / 开头
            path.starts_with('/')
        }
    }
}

/// 跨平台信号处理
pub struct SignalHandler;

impl SignalHandler {
    /// 注册信号处理函数
    pub async fn register_signal_handlers() -> Result<(), Box<dyn std::error::Error>> {
        // 在 Unix 系统上注册信号处理
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            
            // 注册 SIGTERM 信号
            let mut sigterm = signal(SignalKind::terminate())?;
            tokio::spawn(async move {
                sigterm.recv().await;
                println!("Received SIGTERM, shutting down gracefully...");
                // 在实际实现中，这里会执行优雅关闭逻辑
            });
            
            // 注册 SIGINT 信号 (Ctrl+C)
            let mut sigint = signal(SignalKind::interrupt())?;
            tokio::spawn(async move {
                sigint.recv().await;
                println!("Received SIGINT, shutting down gracefully...");
                // 在实际实现中，这里会执行优雅关闭逻辑
            });
            
            // 注册 SIGHUP 信号 (配置重载)
            let mut sighup = signal(SignalKind::hangup())?;
            tokio::spawn(async move {
                sighup.recv().await;
                println!("Received SIGHUP, reloading configuration...");
                // 在实际实现中，这里会执行配置重载逻辑
            });
        }
        
        // 在 Windows 系统上注册 Ctrl+C 处理
        #[cfg(windows)]
        {
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.unwrap();
                println!("Received Ctrl+C, shutting down gracefully...");
                // 在实际实现中，这里会执行优雅关闭逻辑
            });
        }
        
        Ok(())
    }
}