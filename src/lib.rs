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
//! ### 使用 Prelude（推荐）
//!
//! ```rust,no_run,ignore
//! use aerox::prelude::*;  // 导入所有常用类型
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
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
//! ### 配置模块 (aerox_config)
//! - ServerConfig - 服务器基础配置
//! - ReactorConfig - Reactor 模式配置
//!
//! ### 核心模块 (aerox_core)
//! - App - 应用构建器
//! - Plugin - 插件 trait
//! - State - 状态管理
//! - Connection - 连接抽象
//! - ConnectionId - 连接唯一标识
//!
//! ### ECS 模块 (aerox_ecs)
//! - EcsWorld - ECS 世界管理
//! - NetworkBridge - 网络事件桥接
//! - EventScheduler - 事件调度器
//! - GameSystems - 游戏系统集合
//! - 组件: Player, Position, Velocity, Health 等
//! - 事件: ConnectionEstablishedEvent, MessageReceivedEvent 等
//!
//! ### 网络模块 (aerox_network)
//! - Transport - 传输层抽象
//! - TcpReactor - TCP 反应器
//! - Connection - 连接管理
//! - ConnectionId - 连接唯一标识
//! - MessageCodec - 消息编解码器
//!
//! ### Protobuf 模块 (aerox_protobuf)
//! - MessageRegistry - 消息注册表
//! - MessageEncoder - 消息编码器
//! - encode_message/decode_message - 编解码函数
//!
//! ### 路由模块 (aerox_router)
//! - Router - 消息路由器
//! - Handler - 消息处理器 trait
//! - Context - 请求上下文
//! - Middleware - 中间件 trait
//! - Extensions - 类型安全的扩展数据
//!
//! ### 插件模块 (aerox_plugins)
//! - HeartbeatPlugin - 心跳检测插件
//! - RateLimitPlugin - 限流插件
//!
//! ## 示例
//!
//! - `ecs_basics` - ECS 基础和位置更新系统
//! - `router_middleware` - 路由和中间件系统
//! - `complete_game_server` - 完整的游戏服务器示例
//!
//! 运行示例:
//!
//! ```bash
//! cargo run --example ecs_basics
//! cargo run --example router_middleware
//! cargo run --example complete_game_server
//! ```

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
///
/// ## 包含的内容
///
/// ### 服务器端 API
/// - 配置管理（aerox_config）
/// - 核心运行时和插件系统（aerox_core）
/// - ECS 游戏逻辑（aerox_ecs）
/// - 网络传输层（aerox_network）
/// - Protobuf 编解码（aerox_protobuf）
/// - 路由和中间件（aerox_router）
/// - 官方插件（aerox_plugins）
///
/// ### 客户端 API
/// - 高级客户端（Client）
/// - 流式客户端（StreamClient）
///
/// ### 高级 API
/// - 服务器构建器（Server, ServerBuilder）
///
/// ### 统一错误处理
/// - Error（统一错误类型）
/// - Result（统一结果类型）
pub mod prelude {
    // 标准库重导出
    pub use std::result::Result as StdResult;

    // === 服务器端 API ===

    // 配置模块
    #[cfg(feature = "server")]
    pub use aerox_config::{ServerConfig, ReactorConfig, ConfigError};

    // 核心模块 - 应用、插件、连接管理
    #[cfg(feature = "server")]
    pub use aerox_core::{
        App,           // 应用构建器
        Plugin,        // 插件 trait
        State,         // 状态管理
        Connection,    // 连接抽象
        ConnectionId,  // 连接 ID
        AeroXError,    // 统一错误类型
        Result as CoreResult,  // 核心结果类型
    };

    // ECS 模块 - 游戏逻辑核心
    #[cfg(feature = "server")]
    pub use aerox_ecs::prelude::*;

    // 网络模块 - 传输层
    #[cfg(feature = "server")]
    pub use aerox_network::prelude::*;

    // Protobuf 支持 - 消息编解码
    #[cfg(feature = "server")]
    pub use aerox_protobuf::prelude::*;

    // 路由系统 - 消息分发
    #[cfg(feature = "server")]
    pub use aerox_router::prelude::*;

    // 官方插件
    #[cfg(feature = "server")]
    pub use aerox_plugins::prelude::*;

    // === 客户端 API ===
    #[cfg(feature = "client")]
    pub use crate::client::{Client, StreamClient};

    // === 高级 API ===
    #[cfg(feature = "server")]
    pub use crate::server::{Server, ServerBuilder};

    // === 统一错误处理 ===
    pub use crate::{Error, Result};
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
