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
//! ## Crate 结构
//!
//! - [`aerox_config`] - 配置管理系统
//! - [`aerox_core`] - 核心运行时和插件系统
//! - [`aerox_network`] - 网络层抽象和协议实现
//! - [`aerox_protobuf`] - Protobuf 编解码支持
//! - [`aerox_ecs`] - Bevy ECS 整合层
//! - [`aerox_router`] - 路由和中间件系统
//! - [`aerox_plugins`] - 官方插件集合
//!
//! ## 快速开始
//!
//! ```rust,no_run,ignore
//! use aerox::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> aerox::Result<()> {
//!     // 创建应用
//!     let app = App::new()
//!         .add_plugin(HeartbeatPlugin::default())
//!         .set_config(ServerConfig::default());
//!
//!     // 运行服务器
//!     app.run().await?;
//!
//!     Ok(())
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
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

// 重新导出所有子 crate
pub use aerox_config;
pub use aerox_core;
pub use aerox_network;
pub use aerox_protobuf;
pub use aerox_ecs;
pub use aerox_router;
pub use aerox_plugins;

// 预导出常用类型
pub mod prelude {
    // 配置
    pub use aerox_config::{ServerConfig, ReactorConfig, ConfigError};

    // 核心
    pub use aerox_core::{App, Plugin};
    pub use aerox_core::plugin::PluginRegistry;

    // 网络
    pub use aerox_network::{Connection, ConnectionId, TransportError};

    // 路由
    pub use aerox_router::{Router, Context, RouterError};

    // 插件
    pub use aerox_plugins::{HeartbeatPlugin, RateLimitPlugin};

    // 常用 Result 类型
    pub use std::result::Result as StdResult;
}

/// AeroX 统一错误类型
pub type Result<T> = StdResult<T, Box<dyn std::error::Error + Send + Sync>>;

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
