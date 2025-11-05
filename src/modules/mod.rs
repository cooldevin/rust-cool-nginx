//! Nginx 模块系统

pub mod core;
pub mod http;
pub mod proxy;
pub mod static_files;
pub mod load_balancing;
pub mod caching;
pub mod compression;
pub mod access_control;
pub mod logging;
pub mod hot_reload;
pub mod process;
pub mod performance;
pub mod mail;
pub mod module;

// 重新导出核心类型
pub use core::NginxModuleSystem;
pub use http::HttpServer;
pub use proxy::ReverseProxy;
pub use static_files::StaticFileServer;
pub use load_balancing::{LoadBalancer, algorithms::{RoundRobin, WeightedRoundRobin, LeastConnections, IpHash}};
pub use caching::HttpCache;
pub use compression::CompressionModule;
pub use access_control::{IpFilter, BasicAuth};
pub use logging::{Logger, AccessLogger};
pub use hot_reload::HotReloadManager;
pub use process::{ProcessManager, WorkerProcess, WorkerStatus};
pub use performance::{PerformanceMonitor, MemoryPool};
pub use mail::{MailProxy, MailProtocol};