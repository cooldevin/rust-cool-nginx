use std::net::SocketAddr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub listen_addr: String,
    pub backend_addr: String,
    pub static_root: String,
    pub access_log: Option<String>,
    pub error_log: Option<String>,
    pub log_level: Option<String>,
    pub ssl_cert_path: Option<String>,
    pub ssl_key_path: Option<String>,
    pub ssl_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamServer {
    pub address: String,
    pub weight: Option<u32>,
    pub max_fails: Option<u32>,
    pub fail_timeout: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    pub load_balancing_algorithm: Option<String>,
    pub servers: Option<Vec<UpstreamServer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub cache_enabled: Option<bool>,
    pub cache_path: Option<String>,
    pub cache_max_size: Option<String>,
    pub cache_inactive: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GzipConfig {
    pub gzip_compression: Option<bool>,
    pub gzip_comp_level: Option<u32>,
    pub gzip_min_length: Option<u32>,
    pub gzip_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlConfig {
    pub access_control: Option<bool>,
    pub allow_ips: Option<Vec<String>>,
    pub deny_ips: Option<Vec<String>>,
    pub rate_limiting: Option<bool>,
    pub max_requests_per_minute: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub worker_processes: Option<u32>,
    pub worker_connections: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub monitoring_enabled: Option<bool>,
    pub stats_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    // 基本 HTTP 服务
    pub static_file_serving: bool,
    pub reverse_proxy: bool,
    pub fastcgi_support: bool,
    
    // 高级 HTTP 服务
    pub load_balancing: bool,
    #[serde(flatten)]
    pub cache: CacheConfig,
    #[serde(flatten)]
    pub gzip: GzipConfig,
    pub virtual_hosts: bool,
    
    // 安全特性
    #[serde(flatten)]
    pub access_control: AccessControlConfig,
    pub ssl_tls: bool,
    
    // 其他功能
    pub websocket_support: bool,
    
    // 性能配置
    #[serde(flatten)]
    pub performance: PerformanceConfig,
    
    // 监控配置
    #[serde(flatten)]
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub upstream: Option<UpstreamConfig>,
    pub features: FeaturesConfig,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string("nginx.conf")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
    
    pub fn default() -> Self {
        Self {
            server: ServerConfig {
                listen_addr: "127.0.0.1:80".to_string(),
                backend_addr: String::new(),
                static_root: "./public".to_string(),
                access_log: Some("./logs/access.log".to_string()),
                error_log: Some("./logs/error.log".to_string()),
                log_level: Some("info".to_string()),
                ssl_cert_path: Some("./certs/server.crt".to_string()),
                ssl_key_path: Some("./certs/server.key".to_string()),
                ssl_enabled: Some(false),
            },
            upstream: Some(UpstreamConfig {
                load_balancing_algorithm: Some("round_robin".to_string()),
                servers: Some(vec![
                    UpstreamServer {
                        address: "127.0.0.1:8080".to_string(),
                        weight: Some(1),
                        max_fails: Some(3),
                        fail_timeout: Some("10s".to_string()),
                    },
                    UpstreamServer {
                        address: "127.0.0.1:8081".to_string(),
                        weight: Some(1),
                        max_fails: Some(3),
                        fail_timeout: Some("10s".to_string()),
                    }
                ])
            }),
            features: FeaturesConfig {
                // 基本 HTTP 服务
                static_file_serving: true,
                reverse_proxy: true,
                fastcgi_support: false,
                
                // 高级 HTTP 服务
                load_balancing: false,
                cache: CacheConfig {
                    cache_enabled: Some(false),
                    cache_path: Some("./cache".to_string()),
                    cache_max_size: Some("100M".to_string()),
                    cache_inactive: Some("60m".to_string()),
                },
                gzip: GzipConfig {
                    gzip_compression: Some(true),
                    gzip_comp_level: Some(6),
                    gzip_min_length: Some(1024),
                    gzip_types: Some(vec![
                        "text/plain".to_string(),
                        "text/css".to_string(),
                        "application/json".to_string(),
                        "application/javascript".to_string(),
                        "text/xml".to_string(),
                        "application/xml".to_string()
                    ]),
                },
                virtual_hosts: false,
                
                // 安全特性
                access_control: AccessControlConfig {
                    access_control: Some(true),
                    allow_ips: Some(vec!["127.0.0.1".to_string(), "192.168.0.0/16".to_string()]),
                    deny_ips: Some(vec![]),
                    rate_limiting: Some(true),
                    max_requests_per_minute: Some(1000),
                },
                ssl_tls: false,
                
                // 其他功能
                websocket_support: true,
                
                // 性能配置
                performance: PerformanceConfig {
                    worker_processes: Some(4),
                    worker_connections: Some(1024),
                },
                
                // 监控配置
                monitoring: MonitoringConfig {
                    monitoring_enabled: Some(true),
                    stats_path: Some("/nginx_status".to_string()),
                },
            }
        }
    }
    
    // 获取日志配置
    pub fn get_access_log_path(&self) -> Option<&String> {
        self.server.access_log.as_ref()
    }
    
    pub fn get_error_log_path(&self) -> Option<&String> {
        self.server.error_log.as_ref()
    }
    
    pub fn get_log_level(&self) -> Option<&String> {
        self.server.log_level.as_ref()
    }
    
    // 获取缓存配置
    pub fn is_cache_enabled(&self) -> bool {
        self.features.cache.cache_enabled.unwrap_or(false)
    }
    
    pub fn get_cache_path(&self) -> Option<&String> {
        self.features.cache.cache_path.as_ref()
    }
    
    // 获取 Gzip 配置
    pub fn is_gzip_enabled(&self) -> bool {
        self.features.gzip.gzip_compression.unwrap_or(false)
    }
    
    pub fn get_gzip_comp_level(&self) -> u32 {
        self.features.gzip.gzip_comp_level.unwrap_or(6)
    }
    
    // 获取访问控制配置
    pub fn is_access_control_enabled(&self) -> bool {
        self.features.access_control.access_control.unwrap_or(false)
    }
    
    pub fn is_rate_limiting_enabled(&self) -> bool {
        self.features.access_control.rate_limiting.unwrap_or(false)
    }
    
    pub fn get_max_requests_per_minute(&self) -> u32 {
        self.features.access_control.max_requests_per_minute.unwrap_or(1000)
    }
    
    // 获取监控配置
    pub fn is_monitoring_enabled(&self) -> bool {
        self.features.monitoring.monitoring_enabled.unwrap_or(false)
    }
    
    pub fn get_stats_path(&self) -> Option<&String> {
        self.features.monitoring.stats_path.as_ref()
    }
    
    // 获取 upstream 配置
    pub fn get_upstream_servers(&self) -> Option<&Vec<UpstreamServer>> {
        self.upstream.as_ref().and_then(|u| u.servers.as_ref())
    }
    
    pub fn get_load_balancing_algorithm(&self) -> Option<&String> {
        self.upstream.as_ref().and_then(|u| u.load_balancing_algorithm.as_ref())
    }
}