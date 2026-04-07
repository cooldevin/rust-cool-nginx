use std::fs;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

/// Top-level server configuration.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerConfig {
    pub server: ServerSection,
    pub upstream: UpstreamSection,
    pub features: FeaturesSection,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ServerSection {
    pub listen_addr: String,
    pub backend_addr: String,
    pub static_root: String,
    pub access_log: String,
    pub error_log: String,
    pub log_level: String,
    pub ssl_cert_path: String,
    pub ssl_key_path: String,
    pub ssl_enabled: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpstreamSection {
    pub load_balancing_algorithm: String,
    pub servers: Vec<UpstreamServer>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpstreamServer {
    pub address: String,
    pub weight: u32,
    pub max_fails: u32,
    pub fail_timeout: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FeaturesSection {
    pub static_file_serving: bool,
    pub reverse_proxy: bool,
    pub fastcgi_support: bool,
    pub load_balancing: bool,
    pub cache_enabled: bool,
    pub cache_path: String,
    pub cache_max_size: String,
    pub cache_inactive: String,
    pub gzip_compression: bool,
    pub gzip_comp_level: u32,
    pub gzip_min_length: u32,
    pub gzip_types: Vec<String>,
    pub virtual_hosts: bool,
    pub access_control: bool,
    pub allow_ips: Vec<String>,
    pub deny_ips: Vec<String>,
    pub rate_limiting: bool,
    pub max_requests_per_minute: u32,
    pub ssl_tls: bool,
    pub websocket_support: bool,
    pub worker_processes: u32,
    pub worker_connections: u32,
    pub monitoring_enabled: bool,
    pub stats_path: String,
}

const DEFAULT_CONFIG_JSON: &str = r#"{
    "server": {
        "listen_addr": "127.0.0.1:8082",
        "backend_addr": "127.0.0.1:3000",
        "static_root": "./public",
        "access_log": "./logs/access.log",
        "error_log": "./logs/error.log",
        "log_level": "info",
        "ssl_cert_path": "",
        "ssl_key_path": "",
        "ssl_enabled": false
    },
    "upstream": {
        "load_balancing_algorithm": "round-robin",
        "servers": []
    },
    "features": {
        "static_file_serving": true,
        "reverse_proxy": true,
        "fastcgi_support": false,
        "load_balancing": true,
        "cache_enabled": false,
        "cache_path": "/tmp/nginx/cache",
        "cache_max_size": "100m",
        "cache_inactive": "10m",
        "gzip_compression": true,
        "gzip_comp_level": 6,
        "gzip_min_length": 1024,
        "gzip_types": ["text/plain", "text/css", "application/json", "application/javascript", "text/xml", "application/xml"],
        "virtual_hosts": false,
        "access_control": true,
        "allow_ips": ["127.0.0.1"],
        "deny_ips": [],
        "rate_limiting": true,
        "max_requests_per_minute": 1000,
        "ssl_tls": false,
        "websocket_support": true,
        "worker_processes": 1,
        "worker_connections": 1024,
        "monitoring_enabled": true,
        "stats_path": "/stats"
    }
}"#;

/// Parse a config string, falling back to defaults on failure.
fn parse_config(json_str: &str) -> ServerConfig {
    match serde_json::from_str(json_str) {
        Ok(config) => {
            println!("成功解析配置");
            config
        }
        Err(e) => {
            eprintln!("解析配置失败: {}，使用默认配置", e);
            serde_json::from_str(DEFAULT_CONFIG_JSON).expect("默认配置无法解析")
        }
    }
}

/// Load configuration: read from `nginx.conf` or fall back to defaults.
pub fn load_config() -> Arc<RwLock<ServerConfig>> {
    let config_str = match fs::read_to_string("nginx.conf") {
        Ok(content) => {
            println!("成功从 nginx.conf 文件读取配置");
            content
        }
        Err(e) => {
            eprintln!("读取 nginx.conf 文件失败: {}，使用默认配置", e);
            DEFAULT_CONFIG_JSON.to_string()
        }
    };

    Arc::new(RwLock::new(parse_config(&config_str)))
}

/// Persist configuration to `nginx.conf`.
pub fn save_config(config: &ServerConfig) -> Result<(), String> {
    let config_str = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    let current_dir =
        std::env::current_dir().map_err(|e| format!("获取当前目录失败: {}", e))?;
    let config_path = current_dir.join("nginx.conf");

    println!("准备写入配置文件，路径: {:?}", config_path);

    fs::write(&config_path, config_str)
        .map_err(|e| format!("写入配置文件失败: {}，路径: {:?}", e, config_path))?;

    println!("配置已更新并保存到 nginx.conf 文件，路径: {:?}", config_path);
    Ok(())
}
