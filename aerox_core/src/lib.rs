//! AeroX 核心运行时和插件系统
//!
//! 提供应用启动器、插件系统和系统调度器。

pub mod app;
pub mod error;
pub mod plugin;

// 导出主要类型到 crate root
pub use crate::app::App;
pub use crate::error::{AeroXError, AeroXErrorKind, ErrorContext, Result};
pub use crate::plugin::{Plugin, PluginRegistry};

// 预导出
pub mod prelude {
    pub use crate::app::App;
    pub use crate::error::{AeroXError, AeroXErrorKind, ErrorContext, Result};
    pub use crate::plugin::{Plugin, PluginRegistry};
}
