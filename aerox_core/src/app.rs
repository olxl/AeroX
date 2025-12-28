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
    /// 状态数据
    state: State,
}

/// 应用状态
///
/// 存储应用级别的共享数据
#[derive(Default)]
pub struct State {
    inner: Vec<Box<dyn std::any::Any + Send + Sync>>,
}

impl State {
    /// 创建新状态
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// 插入数据
    pub fn insert<T: Send + Sync + 'static>(&mut self, data: T) {
        self.inner.push(Box::new(data));
    }

    /// 获取数据
    pub fn get<T: std::any::Any + Send + Sync + 'static>(&self) -> Option<&T> {
        for item in &self.inner {
            if let Some(typed) = item.downcast_ref::<T>() {
                return Some(typed);
            }
        }
        None
    }
}

impl App {
    /// 创建新的应用
    pub fn new() -> Self {
        Self {
            plugin_registry: PluginRegistry::new(),
            config: ServerConfig::default(),
            state: State::new(),
        }
    }

    /// 添加插件
    pub fn add_plugin(mut self, plugin: impl Plugin + 'static) -> Self {
        let _ = self.plugin_registry.add(Box::new(plugin));
        self
    }

    /// 添加已装箱的插件（内部使用）
    pub fn add_boxed_plugin(mut self, plugin: Box<dyn Plugin>) -> Self {
        let _ = self.plugin_registry.add(plugin);
        self
    }

    /// 设置服务器配置
    pub fn set_config(mut self, config: ServerConfig) -> Self {
        self.config = config;
        self
    }

    /// 插入状态数据
    pub fn insert_state<T: Send + Sync + 'static>(mut self, data: T) -> Self {
        self.state.insert(data);
        self
    }

    /// 获取状态引用
    pub fn state(&self) -> &State {
        &self.state
    }

    /// 获取配置引用
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// 获取插件注册表引用
    pub fn plugin_registry(&self) -> &PluginRegistry {
        &self.plugin_registry
    }

    /// 构建应用
    ///
    /// 验证插件依赖并运行所有插件的 build 方法
    pub fn build(self) -> Result<Self> {
        // 验证插件依赖
        self.plugin_registry.validate_dependencies()?;

        // 按照依赖顺序运行所有插件的 build 方法
        let order = self.plugin_registry.initialization_order()?;
        for plugin_name in order {
            if let Some(index) = self.plugin_registry.plugin_names.get(&plugin_name) {
                if let Some(plugin) = self.plugin_registry.plugins.get(*index) {
                    plugin.build();
                }
            }
        }

        println!("插件数量: {}", self.plugin_registry.count());

        Ok(self)
    }

    /// 运行应用
    pub async fn run(self) -> Result<()> {
        // 验证配置
        self.config
            .validate()
            .map_err(|e| AeroXError::config(e.to_string()))?;

        println!("AeroX 服务器启动中...");
        println!("监听地址: {}", self.config.bind_addr());
        println!("插件数量: {}", self.plugin_registry.count());

        // 实际的服务器启动逻辑应该在更高层的 crate 中实现
        // 这里只是配置验证和初始化

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

    // 测试插件
    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn name(&self) -> &'static str {
            "test_plugin"
        }

        fn build(&self) {
            println!("TestPlugin 构建");
        }
    }

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert_eq!(app.config.bind_address, "0.0.0.0");
    }

    #[test]
    fn test_app_add_plugin() {
        let app = App::new().add_plugin(TestPlugin);
        assert_eq!(app.plugin_registry().count(), 1);
    }

    #[test]
    fn test_app_state() {
        let app = App::new()
            .insert_state(42i32)
            .insert_state("test".to_string());

        assert_eq!(app.state().get::<i32>(), Some(&42));
        assert_eq!(app.state().get::<String>(), Some(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_app_build() {
        let app = App::new().add_plugin(TestPlugin);
        let app = app.build().unwrap();
        assert_eq!(app.plugin_registry().count(), 1);
    }
}
