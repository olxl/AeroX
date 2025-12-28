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
//! ### 服务器
//!
//! ```rust,no_run,ignore
//! use aerox::Server;
//!
//! #[tokio::main]
//! async fn main() -> aerox::Result<()> {
//!     Server::bind("127.0.0.1:8080")
//!         .route(1001, |ctx| async move {
//!             println!("Received: {:?}", ctx.data());
//!             Ok(())
//!         })
//!         .run()
//!         .await
//! }
//! ```
//!
//! ### 客户端
//!
//! ```rust,no_run,ignore
//! use aerox::Client;
//!
//! #[tokio::main]
//! async fn main() -> aerox::Result<()> {
//!     let mut client = Client::connect("127.0.0.1:8080").await?;
//!     client.send(1001, &message).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## 模块组织
//!
//! ### 配置模块
//! - ServerConfig - 服务器基础配置
//! - ReactorConfig - Reactor 模式配置
//!
//! ### 核心模块
//! - App - 应用构建器
//! - Plugin - 插件 trait
//! - PluginRegistry - 插件注册表
//!
//! ### 网络模块
//! - Transport - 传输层抽象
//! - Connection - 连接管理
//! - ConnectionId - 连接唯一标识
//!
//! ### 路由模块
//! - Router - 消息路由器
//! - Context - 请求上下文
//!
//! ### 插件模块
//! - HeartbeatPlugin - 心跳检测插件
//! - RateLimitPlugin - 限流插件

// ============================================================================
// Conditional Compilation Based on Features
// ============================================================================

// Client API
#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "client")]
pub use crate::client::{Client, StreamClient};

// Server API
#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "server")]
pub use crate::server::{Server, ServerBuilder};

// ============================================================================
// Crate Re-exports (for advanced users)
// ============================================================================

#[cfg(feature = "server")]
pub use aerox_config;

#[cfg(feature = "server")]
pub use aerox_core;

#[cfg(feature = "server")]
pub use aerox_network;

#[cfg(feature = "server")]
pub use aerox_protobuf;

#[cfg(feature = "server")]
pub use aerox_ecs;

#[cfg(feature = "server")]
pub use aerox_router;

#[cfg(feature = "server")]
pub use aerox_plugins;

#[cfg(feature = "client")]
pub use aerox_client;

// ============================================================================
// Prelude Module
// ============================================================================

/// 预导出常用类型
///
/// 通过 `use aerox::prelude::*;` 导入所有常用类型
pub mod prelude {
    // Common types
    pub use std::result::Result as StdResult;

    #[cfg(feature = "server")]
    pub use aerox_config::{ServerConfig, ReactorConfig, ConfigError};

    #[cfg(feature = "server")]
    pub use aerox_core::{App, Plugin, State};

    #[cfg(feature = "server")]
    pub use aerox_router::prelude::*;

    #[cfg(feature = "server")]
    pub use aerox_plugins::prelude::*;

    #[cfg(feature = "client")]
    pub use crate::client::{Client, StreamClient};

    #[cfg(feature = "server")]
    pub use crate::server::{Server, ServerBuilder};
}

// ============================================================================
// Error Types
// ============================================================================

/// AeroX 统一错误类型
pub type Result<T> = std::result::Result<T, Error>;

/// AeroX 统一错误枚举
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// 核心错误
    #[cfg(feature = "server")]
    #[error(transparent)]
    Core(#[from] aerox_core::AeroXError),

    /// 客户端错误
    #[cfg(feature = "client")]
    #[error(transparent)]
    Client(#[from] aerox_client::ClientError),

    /// 配置错误
    #[cfg(feature = "server")]
    #[error(transparent)]
    Config(#[from] aerox_config::ConfigError),

    /// IO 错误
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// 自定义错误
    #[error("{0}")]
    Custom(String),
}

// ============================================================================
// Version Information
// ============================================================================

/// AeroX 版本号
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// AeroX 包名
pub const NAME: &str = env!("CARGO_PKG_NAME");
