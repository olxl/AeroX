//! 连接管理
//!
//! 定义连接 ID 和连接结构。

use std::sync::atomic::{AtomicU64, Ordering};

/// 连接唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(u64);

impl ConnectionId {
    /// 创建新的连接 ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 获取内部值
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// 连接 ID 生成器
#[derive(Debug)]
pub struct ConnectionIdGenerator {
    next_id: AtomicU64,
}

impl ConnectionIdGenerator {
    /// 创建新的生成器
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
        }
    }

    /// 生成下一个 ID
    pub fn next(&self) -> ConnectionId {
        ConnectionId(self.next_id.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for ConnectionIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 连接结构
#[derive(Debug)]
pub struct Connection {
    /// 连接 ID
    pub id: ConnectionId,
    /// 远程地址
    pub remote_addr: std::net::SocketAddr,
}

impl Connection {
    /// 创建新连接
    pub fn new(id: ConnectionId, remote_addr: std::net::SocketAddr) -> Self {
        Self { id, remote_addr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_id() {
        let id1 = ConnectionId::new(1);
        let id2 = ConnectionId::new(2);
        assert_ne!(id1, id2);
        assert_eq!(id1.value(), 1);
    }

    #[test]
    fn test_id_generator() {
        let gen = ConnectionIdGenerator::new();
        let id1 = gen.next();
        let id2 = gen.next();
        assert_eq!(id1.value(), 1);
        assert_eq!(id2.value(), 2);
    }
}
