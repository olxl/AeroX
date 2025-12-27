//! 应用构建器
//!
//! 提供链式 API 构建应用。

use crate::plugin::{Plugin, PluginRegistry};
use crate::{AeroXError, Result};
use aerox_config::ServerConfig;

/// 应用构建器
pub struct App {
    /// 插件注册表
    plugin_registry: PluginRegistry,
    /// 服务器配置
    config: ServerConfig,
}

impl App {
    /// 创建新的应用
    pub fn new() -> Self {
        Self {
            plugin_registry: PluginRegistry::new(),
            config: ServerConfig::default(),
        }
    }

    /// 添加插件
    pub fn add_plugin(mut self, plugin: impl Plugin + 'static) -> Self {
        self.plugin_registry.add(Box::new(plugin));
        self
    }

    /// 设置服务器配置
    pub fn set_config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// 运行应用
    pub async fn run(self) -> Result<()> {
        // 验证配置
        self.config.validate()
            .map_err(|e| AeroXError::config(e.to_string()))?;

        // TODO: 启动 Reactor
        println!("AeroX 服务器启动中...");
        println!("配置: {:?}", self.config.bind_addr());

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        // 基础创建测试
        assert_eq!(app.config.bind_address, "0.0.0.0");
    }
}
