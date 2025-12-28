//! 连接 ID 和连接结构
//!
//! 定义连接 ID 和连接结构，供多个 crate 共享使用。

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

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

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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

/// 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 断开中
    Disconnecting,
    /// 已关闭
    Closed,
}

/// 连接结构
#[derive(Debug, Clone)]
pub struct Connection {
    /// 连接 ID
    pub id: ConnectionId,
    /// 远程地址
    pub remote_addr: std::net::SocketAddr,
    /// 连接状态
    pub state: ConnectionState,
    /// 创建时间
    pub created_at: Instant,
    /// 最后活跃时间
    pub last_active: Instant,
}

impl Connection {
    /// 创建新连接
    pub fn new(id: ConnectionId, remote_addr: std::net::SocketAddr) -> Self {
        let now = Instant::now();
        Self {
            id,
            remote_addr,
            state: ConnectionState::Connected,
            created_at: now,
            last_active: now,
        }
    }

    /// 更新活跃时间
    pub fn update_active(&mut self) {
        self.last_active = Instant::now();
    }

    /// 获取连接存活时间
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// 获取空闲时间
    pub fn idle_time(&self) -> std::time::Duration {
        self.last_active.elapsed()
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
        let generator = ConnectionIdGenerator::new();
        let id1 = generator.next();
        let id2 = generator.next();
        assert_eq!(id1.value(), 1);
        assert_eq!(id2.value(), 2);
    }

    #[test]
    fn test_connection_age() {
        let id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();
        let conn = Connection::new(id, addr);

        // 刚创建，年龄应该很短
        assert!(conn.age().as_millis() < 100);
    }

    #[test]
    fn test_connection_idle_time() {
        let id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();
        let mut conn = Connection::new(id, addr);

        // 刚创建，空闲时间应该很短
        assert!(conn.idle_time().as_millis() < 100);

        // 更新活跃时间后
        conn.update_active();
        assert!(conn.idle_time().as_millis() < 100);
    }
}
