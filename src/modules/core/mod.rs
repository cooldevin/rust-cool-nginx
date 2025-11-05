//! 核心模块接口

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

/// Nginx 模块系统核心
pub struct NginxModuleSystem {
    config: Arc<RwLock<Config>>,
}

impl NginxModuleSystem {
    pub fn new(config: Config) -> Self {
        let config_arc = Arc::new(RwLock::new(config));
        
        Self {
            config: config_arc,
        }
    }

    /// 获取配置
    pub fn config(&self) -> Arc<RwLock<Config>> {
        self.config.clone()
    }
}
// 重新导出各个模块
pub use crate::modules::http;
pub use crate::modules::proxy;
pub use crate::modules::static_files;
pub use crate::modules::load_balancing;
pub use crate::modules::caching;
pub use crate::modules::compression;
pub use crate::modules::access_control;
pub use crate::modules::logging;
pub use crate::modules::hot_reload;
pub use crate::modules::process;
pub use crate::modules::performance;
pub use crate::modules::mail;
