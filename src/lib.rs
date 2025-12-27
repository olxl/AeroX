//! # AeroX - 高性能游戏服务器后端框架
//!
//! AeroX 是一个专注于游戏服务器后端和实时消息转发场景的高性能 Rust 框架。
//! 它基于 Reactor 模式实现高并发连接处理，并整合了 ECS 架构。
//!
//! ## 特性
//!
//! - 基于 Tokio 异步运行时的高性能 Reactor 模式
//! - 支持 TCP 协议（KCP 和 QUIC 规划中）
//! - Protobuf 消息协议支持
//! - Bevy ECS 整合
//! - 灵活的路由和中间件系统
//! - 插件化架构
//!
//! ## 快速开始
//!
//! ```rust,no_run,ignore
//! use AeroX::prelude::*;
//!
//! fn main() -> AeroX::Result<()> {
//!     // 创建配置
//!     let config = ServerConfig::default();
//!
//!     // 验证配置
//!     config.validate()?;
//!
//!     Ok(())
//! }
//! # Ok::<(), AeroX::AeroXError>(())
//! ```
//!
//! 注意：完整的 API 示例将在后续 Phase 中提供。
//!
//! ## 模块组织
//!
//! - [`config`] - 配置管理系统
//! - [`error`] - 错误类型定义
//! - [`reactor`] - Reactor 模式实现
//! - [`connection`] - 连接管理
//! - [`protocol`] - 协议和编解码
//! - [`router`] - 路由系统
//! - [`middleware`] - 中间件系统
//! - [`plugin`] - 插件系统

pub mod config;
pub mod error;

// 导出常用类型到 crate root
pub use crate::config::{ServerConfig, ReactorConfig};
pub use crate::error::{AeroXError, Result};

// 预导出常用类型
pub mod prelude {
    pub use crate::config::{ServerConfig, ReactorConfig};
    pub use crate::error::{AeroXError, Result};
}

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
