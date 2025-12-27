//! 连接池
//!
//! 管理活跃连接的集合。

use crate::connection::{Connection, ConnectionId};
use aerox_core::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// 连接池
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    /// 内部连接存储（使用 Arc<RwLock> 实现并发访问）
    inner: Arc<RwLock<ConnectionPoolInner>>,
}

/// 连接池内部存储
#[derive(Debug)]
struct ConnectionPoolInner {
    /// 连接映射: ID -> Connection
    connections: HashMap<ConnectionId, Connection>,
}

impl ConnectionPool {
    /// 创建新的连接池
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ConnectionPoolInner {
                connections: HashMap::new(),
            })),
        }
    }

    /// 添加连接
    pub fn add(&self, conn: Connection) -> Result<()> {
        let mut inner = self
            .inner
            .write()
            .map_err(|e| aerox_core::AeroXError::network(format!("获取写锁失败: {}", e)))?;

        inner.connections.insert(conn.id, conn);
        Ok(())
    }

    /// 移除连接
    pub fn remove(&self, id: ConnectionId) -> Result<Option<Connection>> {
        let mut inner = self
            .inner
            .write()
            .map_err(|e| aerox_core::AeroXError::network(format!("获取写锁失败: {}", e)))?;

        Ok(inner.connections.remove(&id))
    }

    /// 获取连接
    pub fn get(&self, id: ConnectionId) -> Result<Option<Connection>> {
        let inner = self
            .inner
            .read()
            .map_err(|e| aerox_core::AeroXError::network(format!("获取读锁失败: {}", e)))?;

        Ok(inner.connections.get(&id).cloned())
    }

    /// 检查连接是否存在
    pub fn contains(&self, id: ConnectionId) -> Result<bool> {
        let inner = self
            .inner
            .read()
            .map_err(|e| aerox_core::AeroXError::network(format!("获取读锁失败: {}", e)))?;

        Ok(inner.connections.contains_key(&id))
    }

    /// 获取连接数量
    pub fn len(&self) -> Result<usize> {
        let inner = self
            .inner
            .read()
            .map_err(|e| aerox_core::AeroXError::network(format!("获取读锁失败: {}", e)))?;

        Ok(inner.connections.len())
    }

    /// 是否为空
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// 清理空闲连接
    ///
    /// 移除超过指定空闲时间的连接
    pub fn cleanup_idle(&self, timeout: std::time::Duration) -> Result<usize> {
        let mut inner = self
            .inner
            .write()
            .map_err(|e| aerox_core::AeroXError::network(format!("获取写锁失败: {}", e)))?;

        let _now = Instant::now();
        let mut to_remove = Vec::new();

        for (&id, conn) in inner.connections.iter() {
            if conn.idle_time() > timeout {
                to_remove.push(id);
            }
        }

        for id in to_remove.iter() {
            inner.connections.remove(id);
        }

        Ok(to_remove.len())
    }

    /// 获取所有连接 ID
    pub fn all_ids(&self) -> Result<Vec<ConnectionId>> {
        let inner = self
            .inner
            .read()
            .map_err(|e| aerox_core::AeroXError::network(format!("获取读锁失败: {}", e)))?;

        Ok(inner.connections.keys().copied().collect())
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool = ConnectionPool::new();
        assert_eq!(pool.len().unwrap(), 0);
        assert!(pool.is_empty().unwrap());
    }

    #[test]
    fn test_pool_add_remove() {
        let pool = ConnectionPool::new();
        let id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();
        let conn = Connection::new(id, addr);

        // 添加连接
        pool.add(conn).unwrap();
        assert_eq!(pool.len().unwrap(), 1);
        assert!(pool.contains(id).unwrap());

        // 移除连接
        let removed = pool.remove(id).unwrap();
        assert!(removed.is_some());
        assert_eq!(pool.len().unwrap(), 0);
        assert!(!pool.contains(id).unwrap());
    }

    #[test]
    fn test_pool_get() {
        let pool = ConnectionPool::new();
        let id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();
        let conn = Connection::new(id, addr);

        pool.add(conn).unwrap();

        let retrieved = pool.get(id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, id);
    }

    #[test]
    fn test_pool_cleanup() {
        let pool = ConnectionPool::new();

        // 添加多个连接
        for i in 1..=3 {
            let id = ConnectionId::new(i);
            let addr = "127.0.0.1:8080".parse().unwrap();
            let conn = Connection::new(id, addr);
            pool.add(conn).unwrap();
        }

        // 等待一段时间
        std::thread::sleep(std::time::Duration::from_millis(10));

        // 清理空闲时间超过 1ms 的连接
        let cleaned = pool
            .cleanup_idle(std::time::Duration::from_millis(1))
            .unwrap();
        assert_eq!(cleaned, 3);
        assert_eq!(pool.len().unwrap(), 0);
    }
}
