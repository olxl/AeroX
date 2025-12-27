//! 限流插件
//!
//! 提供请求频率限制功能。

use aerox_core::Plugin;
use aerox_core::App;
use aerox_config::ServerConfig;

/// 限流插件
pub struct RateLimitPlugin {
    /// 配置
    pub config: ServerConfig,
}

impl RateLimitPlugin {
    /// 从配置创建插件
    pub fn from_config(config: ServerConfig) -> Self {
        Self { config }
    }
}

impl Plugin for RateLimitPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: 注册限流中间件
        println!(
            "注册限流插件: 每连接每秒最大请求={:?}, 全局每秒最大请求={:?}",
            self.config.max_requests_per_second_per_connection,
            self.config.max_requests_per_second_total
        );
    }

    fn name(&self) -> &'static str {
        "RateLimitPlugin"
    }
}
