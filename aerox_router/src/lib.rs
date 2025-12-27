//! AeroX 路由和中间件系统
//!
//! 提供消息路由和 Axum 风格的中间件系统。

pub mod router;
pub mod context;
pub mod middleware;

// 预导出
pub mod prelude {
    pub use crate::router::Router;
    pub use crate::context::Context;
}
