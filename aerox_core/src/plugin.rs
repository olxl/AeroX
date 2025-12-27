//! 插件系统
//!
//! 定义 Plugin trait 和插件注册表。

use std::collections::HashMap;
use crate::app::App;

/// Plugin trait - 所有插件必须实现此 trait
pub trait Plugin: Send + Sync {
    /// 构建插件，修改应用配置
    fn build(&self, app: &mut App) {
        // 默认实现：什么都不做
        let _ = app;
    }

    /// 获取插件名称
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// 是否为必需插件
    fn is_required(&self) -> bool {
        false
    }
}

/// 插件注册表
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
    plugin_names: HashMap<String, usize>,
}

impl PluginRegistry {
    /// 创建新的插件注册表
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            plugin_names: HashMap::new(),
        }
    }

    /// 注册插件
    pub fn add(&mut self, plugin: Box<dyn Plugin>) -> &mut Self {
        let name = plugin.name().to_string();
        self.plugin_names.insert(name, self.plugins.len());
        self.plugins.push(plugin);
        self
    }

    /// 获取所有插件
    pub fn plugins(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
