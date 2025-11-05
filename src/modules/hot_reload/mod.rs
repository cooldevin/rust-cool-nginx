//! 热重载模块实现

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::Config;
use tokio::signal;
use std::path::Path;

pub struct HotReloadManager {
    config: Arc<RwLock<Config>>,
    config_path: String,
}

impl HotReloadManager {
    pub fn new(config: Arc<RwLock<Config>>, config_path: String) -> Self {
        Self {
            config,
            config_path,
        }
    }

    /// 启动热重载监听器
    pub async fn start_listening(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 监听 SIGUSR1 信号（在 Unix 系统上）或文件变化
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigusr1 = signal(SignalKind::user_defined1())?;
            
            loop {
                sigusr1.recv().await;
                println!("Received SIGUSR1, reloading configuration...");
                self.reload_config().await?;
            }
        }
        
        // 在非 Unix 系统上，我们可以监听文件变化
        #[cfg(not(unix))]
        {
            self.watch_config_file().await?;
        }
        
        Ok(())
    }

    /// 重新加载配置文件
    async fn reload_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Path::new(&self.config_path);
        if config_path.exists() {
            let config_str = std::fs::read_to_string(config_path)?;
            let new_config: Config = toml::from_str(&config_str)?;
            
            {
                let mut config = self.config.write().await;
                *config = new_config;
            }
            
            println!("Configuration reloaded successfully");
        } else {
            eprintln!("Configuration file not found: {}", self.config_path);
        }
        
        Ok(())
    }

    /// 监听配置文件变化（非 Unix 系统）
    #[cfg(not(unix))]
    async fn watch_config_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::time::{sleep, Duration};
        use std::time::SystemTime;
        
        let mut last_modified = std::fs::metadata(&self.config_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::now());
        
        loop {
            sleep(Duration::from_secs(5)).await;
            
            if let Ok(metadata) = std::fs::metadata(&self.config_path) {
                if let Ok(modified) = metadata.modified() {
                    if modified > last_modified {
                        println!("Configuration file changed, reloading...");
                        self.reload_config().await?;
                        last_modified = modified;
                    }
                }
            }
        }
    }
}