//! 插件系统
//!
//! 定义 Plugin trait 和插件注册表。

use crate::Result;
use std::collections::{HashMap, HashSet};

/// Plugin trait - 所有插件必须实现此 trait
pub trait Plugin: Send + Sync {
    /// 构建插件，修改应用配置
    ///
    /// 在应用启动时调用，插件可以在此方法中注册路由、中间件等
    ///
    /// 注意：为避免循环依赖，此方法不直接接收 App 参数。
    /// 插件应通过其他方式注册路由和中间件。
    fn build(&self) {
        // 默认实现：什么都不做
    }

    /// 获取插件名称
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// 是否为必需插件
    fn is_required(&self) -> bool {
        false
    }

    /// 获取插件依赖的其他插件
    ///
    /// 返回依赖的插件名称列表
    fn dependencies(&self) -> &'static [&'static str] {
        &[]
    }
}

/// 插件注册表
pub struct PluginRegistry {
    pub(crate) plugins: Vec<Box<dyn Plugin>>,
    pub(crate) plugin_names: HashMap<String, usize>,
    dependency_graph: HashMap<String, Vec<String>>,
}

impl PluginRegistry {
    /// 创建新的插件注册表
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            plugin_names: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }

    /// 注册插件
    pub fn add(&mut self, plugin: Box<dyn Plugin>) -> Result<&mut Self> {
        let name = plugin.name().to_string();

        // 检查是否已存在
        if self.plugin_names.contains_key(&name) {
            return Err(crate::AeroXError::plugin(format!(
                "插件已存在: {}",
                name
            )));
        }

        let index = self.plugins.len();
        self.plugin_names.insert(name.clone(), index);

        // 记录依赖关系
        let deps: Vec<String> = plugin.dependencies().iter().map(|s| s.to_string()).collect();
        if !deps.is_empty() {
            self.dependency_graph.insert(name, deps);
        }

        self.plugins.push(plugin);
        Ok(self)
    }

    /// 获取所有插件
    pub fn plugins(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }

    /// 验证插件依赖
    ///
    /// 检查所有插件的依赖是否满足
    pub fn validate_dependencies(&self) -> Result<()> {
        let registered: HashSet<&str> = self.plugin_names.keys().map(|s| s.as_str()).collect();

        for (plugin, deps) in &self.dependency_graph {
            for dep in deps {
                if !registered.contains(dep.as_str()) {
                    return Err(crate::AeroXError::plugin(format!(
                        "插件 {} 依赖的插件 {} 未注册",
                        plugin, dep
                    )));
                }
            }
        }

        Ok(())
    }

    /// 获取插件初始化顺序
    ///
    /// 根据依赖关系返回插件初始化的顺序
    pub fn initialization_order(&self) -> Result<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();

        for plugin_name in self.plugin_names.keys() {
            self.visit(plugin_name, &mut order, &mut visited)?;
        }

        Ok(order)
    }

    /// 拓扑排序访问
    fn visit(
        &self,
        plugin_name: &str,
        order: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> Result<()> {
        if visited.contains(plugin_name) {
            return Ok(());
        }

        // 检查循环依赖
        if order.contains(&plugin_name.to_string()) {
            return Err(crate::AeroXError::plugin(format!(
                "检测到循环依赖: {}",
                plugin_name
            )));
        }

        // 访问依赖
        if let Some(deps) = self.dependency_graph.get(plugin_name) {
            for dep in deps {
                self.visit(dep, order, visited)?;
            }
        }

        order.push(plugin_name.to_string());
        visited.insert(plugin_name.to_string());

        Ok(())
    }

    /// 获取插件数量
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试插件
    struct PluginA;

    impl Plugin for PluginA {
        fn name(&self) -> &'static str {
            "plugin_a"
        }
    }

    // 测试插件 B 依赖 A
    struct PluginB;

    impl Plugin for PluginB {
        fn name(&self) -> &'static str {
            "plugin_b"
        }

        fn dependencies(&self) -> &'static [&'static str] {
            &["plugin_a"]
        }
    }

    // 测试插件 C 依赖 B
    struct PluginC;

    impl Plugin for PluginC {
        fn name(&self) -> &'static str {
            "plugin_c"
        }

        fn dependencies(&self) -> &'static [&'static str] {
            &["plugin_b"]
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_add_plugin() {
        let mut registry = PluginRegistry::new();
        registry.add(Box::new(PluginA)).unwrap();
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_duplicate_plugin() {
        let mut registry = PluginRegistry::new();
        registry.add(Box::new(PluginA)).unwrap();
        let result = registry.add(Box::new(PluginA));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_dependencies() {
        let mut registry = PluginRegistry::new();
        registry.add(Box::new(PluginA)).unwrap();
        registry.add(Box::new(PluginB)).unwrap();
        assert!(registry.validate_dependencies().is_ok());
    }

    #[test]
    fn test_validate_dependencies_missing() {
        let mut registry = PluginRegistry::new();
        registry.add(Box::new(PluginB)).unwrap();
        let result = registry.validate_dependencies();
        assert!(result.is_err());
    }

    #[test]
    fn test_initialization_order() {
        let mut registry = PluginRegistry::new();
        registry.add(Box::new(PluginC)).unwrap();
        registry.add(Box::new(PluginB)).unwrap();
        registry.add(Box::new(PluginA)).unwrap();

        let order = registry.initialization_order().unwrap();
        assert_eq!(order, vec!["plugin_a", "plugin_b", "plugin_c"]);
    }
}
