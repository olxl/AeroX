//! AeroX 核心运行时和插件系统
//!
//! 提供应用启动器、插件系统和系统调度器。

pub mod plugin;
pub mod app;

// 导出主要类型到 crate root
pub use crate::plugin::{Plugin, PluginRegistry};
pub use crate::app::App;

// 预导出
pub mod prelude {
    pub use crate::plugin::{Plugin, PluginRegistry};
    pub use crate::app::App;
}
