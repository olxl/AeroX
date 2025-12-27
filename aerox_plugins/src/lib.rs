//! AeroX 官方插件集合
//!
//! 提供常用的官方插件，如心跳、限流等。

pub mod heartbeat;
pub mod ratelimit;

// 预导出
pub mod prelude {
    pub use crate::heartbeat::HeartbeatPlugin;
    pub use crate::ratelimit::RateLimitPlugin;
}
