//! 日志模块实现

use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use chrono::Local;

#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub struct Logger {
    log_file: Option<Mutex<File>>,
    level: LogLevel,
}

impl Logger {
    pub fn new(log_file_path: Option<&str>, level: LogLevel) -> Result<Self, Box<dyn std::error::Error>> {
        let log_file = if let Some(path) = log_file_path {
            Some(Mutex::new(File::create(path)?))
        } else {
            None
        };

        Ok(Self {
            log_file,
            level,
        })
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        // 检查日志级别
        if !self.should_log(&level) {
            return;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let level_str = match level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };

        let log_line = format!("[{}] {} - {}\n", timestamp, level_str, message);

        // 输出到控制台
        println!("{}", log_line.trim_end());

        // 写入日志文件
        if let Some(log_file) = &self.log_file {
            if let Ok(mut file) = log_file.lock() {
                let _ = file.write_all(log_line.as_bytes());
                let _ = file.flush();
            }
        }
    }

    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    fn should_log(&self, level: &LogLevel) -> bool {
        match (&self.level, level) {
            (LogLevel::Debug, _) => true,
            (LogLevel::Info, LogLevel::Debug) => false,
            (LogLevel::Info, _) => true,
            (LogLevel::Warn, LogLevel::Debug) | (LogLevel::Warn, LogLevel::Info) => false,
            (LogLevel::Warn, _) => true,
            (LogLevel::Error, LogLevel::Error) => true,
            (LogLevel::Error, _) => false,
        }
    }
}

// 访问日志记录器
pub struct AccessLogger {
    logger: Logger,
}

impl AccessLogger {
    pub fn new(log_file_path: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let logger = Logger::new(log_file_path, LogLevel::Info)?;
        Ok(Self { logger })
    }

    pub fn log_access(&self, client_ip: &str, method: &str, uri: &str, status: u16, response_size: usize) {
        let message = format!("{} - \"{} {}\" {} {}", client_ip, method, uri, status, response_size);
        self.logger.info(&message);
    }
}