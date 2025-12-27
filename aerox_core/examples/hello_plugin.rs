//! 插件系统示例
//!
//! 演示如何创建和使用插件。

use aerox_core::{App, Plugin};

/// 一个简单的日志插件
struct LoggingPlugin {
    log_level: String,
}

impl LoggingPlugin {
    fn new(log_level: &str) -> Self {
        Self {
            log_level: log_level.to_string(),
        }
    }
}

impl Plugin for LoggingPlugin {
    fn name(&self) -> &'static str {
        "logging_plugin"
    }

    fn build(&self) {
        println!("🔧 LoggingPlugin 初始化 (日志级别: {})", self.log_level);
    }

    fn is_required(&self) -> bool {
        true
    }
}

/// 一个数据存储插件
struct StoragePlugin {
    capacity: usize,
}

impl Plugin for StoragePlugin {
    fn name(&self) -> &'static str {
        "storage_plugin"
    }

    fn dependencies(&self) -> &'static [&'static str] {
        &["logging_plugin"]
    }

    fn build(&self) {
        println!("📦 StoragePlugin 初始化 (容量: {} MB)", self.capacity);
    }
}

/// 认证插件（可选）
struct AuthPlugin;

impl Plugin for AuthPlugin {
    fn name(&self) -> &'static str {
        "auth_plugin"
    }

    fn build(&self) {
        println!("🔐 AuthPlugin 初始化");
    }

    fn is_required(&self) -> bool {
        false
    }
}

#[tokio::main]
async fn main() -> aerox_core::Result<()> {
    println!("=== AeroX 插件系统示例 ===\n");

    // 创建应用并添加插件
    let app = App::new()
        // 添加必需插件
        .add_plugin(LoggingPlugin::new("INFO"))
        .add_plugin(StoragePlugin { capacity: 1024 })
        // 添加可选插件
        .add_plugin(AuthPlugin)
        // 插入状态数据
        .insert_state("应用状态数据")
        .insert_state(42i32);

    println!("\n开始构建应用...\n");

    // 构建应用（会按依赖顺序初始化插件）
    let app = app.build()?;

    println!("\n应用构建完成！");
    println!("已加载插件数量: {}", app.plugin_registry().count());

    // 检查状态数据
    if let Some(data) = app.state().get::<&str>() {
        println!("\n状态数据: {}", data);
    }

    if let Some(number) = app.state().get::<i32>() {
        println!("状态数字: {}", number);
    }

    println!("\n✅ 所有插件加载成功！");

    Ok(())
}
