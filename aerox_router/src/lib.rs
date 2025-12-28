//! AeroX 路由和中间件系统
//!
//! 提供消息路由和 Axum 风格的中间件系统。

pub mod context;
pub mod middleware;
pub mod router;

// 重新导出主要类型
pub use crate::context::{Context, Extensions};
pub use crate::middleware::{Layer, LoggingMiddleware, Middleware, Next, Stack, TimeoutMiddleware};
pub use crate::router::{Handler, Router};

// 重新导出错误类型
pub use aerox_core::{AeroXError, Result};

// 预导出
pub mod prelude {
    pub use crate::context::{Context, Extensions};
    pub use crate::middleware::{LoggingMiddleware, Middleware, Next, Stack, TimeoutMiddleware};
    pub use crate::router::{Handler, Router};
    pub use aerox_core::{AeroXError, Result};
}
