//! 配置管理模块
//! 实现配置的加载、热重载等功能

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::Config;
use std::collections::HashMap;

pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    variables: Arc<RwLock<HashMap<String, String>>>,
}

impl ConfigManager {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            variables: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }

    pub async fn reload_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let new_config = Config::load()?;
        *self.config.write().await = new_config;
        Ok(())
    }

    pub async fn update_config(&self, new_config: Config) {
        *self.config.write().await = new_config;
    }

    // 热重载配置文件
    pub async fn hot_reload(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Reloading configuration...");
        self.reload_config().await?;
        println!("Configuration reloaded successfully");
        Ok(())
    }

    // 设置变量
    pub async fn set_variable(&self, name: String, value: String) {
        self.variables.write().await.insert(name, value);
    }

    // 获取变量
    pub async fn get_variable(&self, name: &str) -> Option<String> {
        self.variables.read().await.get(name).cloned()
    }

    // 解析包含变量的字符串
    pub async fn parse_with_variables(&self, input: &str) -> String {
        let variables = self.variables.read().await;
        let mut result = input.to_string();
        
        for (name, value) in variables.iter() {
            let placeholder = format!("${{{}}}", name);
            result = result.replace(&placeholder, value);
        }
        
        result
    }

    // 包含其他配置文件
    pub async fn include_config(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 在实际实现中，这里会加载并合并其他配置文件
        println!("Including config from: {}", path);
        Ok(())
    }
}