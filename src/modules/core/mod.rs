//! 核心模块系统
//! 提供模块注册、加载和管理功能

use crate::modules::http;
use crate::modules::proxy;
use crate::modules::static_files;
use crate::modules::load_balancing;
use crate::modules::caching;
// use crate::modules::compression;
use crate::modules::access_control;
use crate::modules::logging;
use crate::modules::hot_reload;
use crate::modules::process;
use crate::modules::performance;
use crate::modules::mail;

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
