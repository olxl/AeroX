//! 请求上下文
//!
//! 包含连接信息和请求数据。

use aerox_network::ConnectionId;

/// 请求上下文
#[derive(Debug, Clone)]
pub struct Context {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 请求时间戳
    pub timestamp: std::time::Instant,
}

impl Context {
    /// 创建新上下文
    pub fn new(connection_id: ConnectionId) -> Self {
        Self {
            connection_id,
            timestamp: std::time::Instant::now(),
        }
    }
}
