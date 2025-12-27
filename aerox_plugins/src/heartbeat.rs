//! 心跳插件
//!
//! 提供连接心跳检测功能。

use aerox_core::Plugin;

/// 心跳插件
pub struct HeartbeatPlugin {
    /// 心跳间隔
    pub interval_secs: u64,
    /// 超时时间
    pub timeout_secs: u64,
}

impl Default for HeartbeatPlugin {
    fn default() -> Self {
        Self {
            interval_secs: 30,
            timeout_secs: 60,
        }
    }
}

impl Plugin for HeartbeatPlugin {
    fn build(&self) {
        // TODO: 注册心跳检测系统
        println!(
            "注册心跳插件: 间隔={}s, 超时={}s",
            self.interval_secs, self.timeout_secs
        );
    }

    fn name(&self) -> &'static str {
        "HeartbeatPlugin"
    }
}
