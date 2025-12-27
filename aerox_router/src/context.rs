//! 请求上下文
//!
//! 包含请求的所有相关信息。

use aerox_network::ConnectionId;
use bytes::Bytes;
use std::net::SocketAddr;

/// 请求上下文
///
/// 包含单个请求的所有信息
#[derive(Debug, Clone)]
pub struct Context {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 远程地址
    pub peer_addr: SocketAddr,
    /// 消息 ID
    pub message_id: u16,
    /// 序列 ID
    pub sequence_id: u32,
    /// 请求数据
    pub data: Bytes,
    /// 用户数据扩展（简化实现）
    pub extensions: Extensions,
    /// 请求时间戳
    pub timestamp: std::time::Instant,
}

impl Context {
    /// 创建新的上下文
    pub fn new(
        connection_id: ConnectionId,
        peer_addr: SocketAddr,
        message_id: u16,
        sequence_id: u32,
        data: Bytes,
    ) -> Self {
        Self {
            connection_id,
            peer_addr,
            message_id,
            sequence_id,
            data,
            extensions: Extensions::default(),
            timestamp: std::time::Instant::now(),
        }
    }

    /// 获取连接 ID
    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }

    /// 获取远程地址
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    /// 获取消息 ID
    pub fn message_id(&self) -> u16 {
        self.message_id
    }

    /// 获取序列 ID
    pub fn sequence_id(&self) -> u32 {
        self.sequence_id
    }

    /// 获取请求数据
    pub fn data(&self) -> &Bytes {
        &self.data
    }

    /// 获取请求数据的克隆
    pub fn data_clone(&self) -> Bytes {
        self.data.clone()
    }
}

/// 扩展数据
///
/// 用于在中间件和处理器之间传递数据（简化实现）
#[derive(Debug, Default, Clone)]
pub struct Extensions {
    // 简化版本：为MVP阶段先使用占位实现
    // 完整实现可以使用 HashMap<TypeId, Box<dyn Any>>
    _private: (),
}

impl Extensions {
    /// 创建新的扩展
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: 在后续阶段实现完整的类型安全扩展存储
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let conn_id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();
        let data = Bytes::from("test data");

        let ctx = Context::new(conn_id, addr, 100, 1000, data);

        assert_eq!(ctx.connection_id(), conn_id);
        assert_eq!(ctx.peer_addr(), addr);
        assert_eq!(ctx.message_id(), 100);
        assert_eq!(ctx.sequence_id(), 1000);
        assert_eq!(ctx.data(), &Bytes::from("test data"));
    }

    #[test]
    fn test_context_data_clone() {
        let conn_id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();
        let data = Bytes::from("test");

        let ctx = Context::new(conn_id, addr, 100, 1000, data.clone());
        let cloned = ctx.data_clone();

        assert_eq!(cloned, data);
    }
}
