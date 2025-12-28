//! # AeroX 核心运行时
//!
//! 本 crate 提供 AeroX 框架的核心运行时和插件系统。
//!
//! ## 主要组件
//!
//! - [`App`]: 应用构建器，用于管理和启动应用程序
//! - [`Plugin`]: 插件 trait，定义可扩展的功能模块
//! - [`PluginRegistry`]: 插件注册表，管理插件生命周期
//! - [`AeroXError`]: 错误类型，提供统一的错误处理
//! - [`State`]: 应用状态容器，存储全局状态
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use aerox_core::{App, Plugin};
//! use aerox_core::Result;
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn build(&self) {
//!         println!("插件已加载！");
//!     }
//!
//!     fn name(&self) -> &'static str {
//!         "my_plugin"
//!     }
//! }
//!
//! fn main() -> Result<()> {
//!     let app = App::new()
//!         .add_plugin(MyPlugin);
//!
//!     app.build()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## 插件系统
//!
//! AeroX 使用插件系统来组织代码。每个插件都可以：
//!
//! - 定义自己的构建逻辑
//! - 声明对其他插件的依赖
//! - 设置为必需或可选
//!
//! 插件会按照依赖顺序自动初始化。
//!
//! ## 错误处理
//!
//! 所有函数返回 [`Result<T>`]，错误类型为 [`AeroXError`]。
//! 使用 `?` 运算符可以方便地传播错误。
//!
//! ```rust
//! use aerox_core::{Result, AeroXError};
//!
//! fn do_something() -> Result<()> {
//!     Err(AeroXError::config("配置错误"))
//! }
//! ```

pub mod app;
pub mod connection;
pub mod error;
pub mod plugin;

// 导出主要类型到 crate root
pub use crate::app::{App, State};
pub use crate::connection::{Connection, ConnectionId, ConnectionIdGenerator, ConnectionState};
pub use crate::error::{AeroXError, AeroXErrorKind, ErrorContext, Result};
pub use crate::plugin::{Plugin, PluginRegistry};

// 预导出
pub mod prelude {
    pub use crate::app::{App, State};
    pub use crate::error::{AeroXError, AeroXErrorKind, ErrorContext, Result};
    pub use crate::plugin::{Plugin, PluginRegistry};
}
