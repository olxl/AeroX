//! 连接管理器
//!
//! 高层连接管理和生命周期控制。

use crate::connection::metrics::ConnectionMetrics;
use crate::connection::{Connection, ConnectionId, ConnectionPool};
use aerox_core::Result;

/// 连接管理器
///
/// 负责连接的生命周期管理和指标收集
pub struct ConnectionManager {
    /// 连接池
    pool: ConnectionPool,
    /// 连接指标
    metrics: ConnectionMetrics,
    /// 连接 ID 生成器
    id_generator: crate::connection::ConnectionIdGenerator,
}

/// 连接管理器配置
#[derive(Debug, Clone)]
pub struct ConnectionManagerConfig {
    /// 空闲连接超时时间（秒）
    pub idle_timeout_secs: u64,
    /// 是否启用自动清理
    pub enable_auto_cleanup: bool,
    /// 清理间隔（秒）
    pub cleanup_interval_secs: u64,
}

impl Default for ConnectionManagerConfig {
    fn default() -> Self {
        Self {
            idle_timeout_secs: 300, // 5 分钟
            enable_auto_cleanup: true,
            cleanup_interval_secs: 60, // 1 分钟
        }
    }
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new(_config: ConnectionManagerConfig) -> Self {
        Self {
            pool: ConnectionPool::new(),
            metrics: ConnectionMetrics::new(),
            id_generator: crate::connection::ConnectionIdGenerator::new(),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(ConnectionManagerConfig::default())
    }

    /// 创建新连接并加入池中
    pub fn create_connection(&self, remote_addr: std::net::SocketAddr) -> Result<ConnectionId> {
        let id = self.id_generator.next();
        let conn = Connection::new(id, remote_addr);

        self.pool.add(conn.clone())?;
        self.metrics.inc_connections();

        println!("连接创建: {} (远程: {})", id, remote_addr);
        Ok(id)
    }

    /// 移除连接
    pub fn remove_connection(&self, id: ConnectionId) -> Result<bool> {
        if let Some(_conn) = self.pool.remove(id)? {
            self.metrics.dec_connections();
            println!("连接移除: {}", id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 获取连接
    pub fn get_connection(&self, id: ConnectionId) -> Result<Option<Connection>> {
        self.pool.get(id)
    }

    /// 获取连接数量
    pub fn connection_count(&self) -> Result<usize> {
        self.pool.len()
    }

    /// 获取连接指标
    pub fn metrics(&self) -> &ConnectionMetrics {
        &self.metrics
    }

    /// 启动清理任务
    ///
    /// 定期清理空闲连接
    pub fn spawn_cleanup_task(
        &self,
        config: ConnectionManagerConfig,
    ) -> tokio::task::JoinHandle<Result<()>> {
        let pool = self.pool.clone();
        let timeout = std::time::Duration::from_secs(config.idle_timeout_secs);
        let interval = std::time::Duration::from_secs(config.cleanup_interval_secs);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                match pool.cleanup_idle(timeout) {
                    Ok(count) => {
                        if count > 0 {
                            println!("清理了 {} 个空闲连接", count);
                        }
                    }
                    Err(e) => {
                        eprintln!("清理连接失败: {}", e);
                    }
                }
            }
        })
    }

    /// 生成报告
    pub fn report(&self) -> String {
        format!(
            "连接管理器报告:\n\
             - 连接数: {}\n\
             - {}",
            self.connection_count().unwrap_or(0),
            self.metrics.summary()
        )
    }
}

impl Clone for ConnectionManager {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            metrics: self.metrics.clone_inner(),
            id_generator: crate::connection::ConnectionIdGenerator::new(),
        }
    }
}

// 为 ConnectionMetrics 实现 Clone
impl ConnectionMetrics {
    fn clone_inner(&self) -> Self {
        Self::with_values(
            self.current_connections(),
            self.total_connections(),
            self.total_bytes_received(),
            self.total_bytes_sent(),
            self.total_messages_received(),
            self.total_messages_sent(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = ConnectionManager::with_defaults();
        assert_eq!(manager.connection_count().unwrap(), 0);
    }

    #[test]
    fn test_manager_create_connection() {
        let manager = ConnectionManager::with_defaults();
        let addr = "127.0.0.1:8080".parse().unwrap();

        let id = manager.create_connection(addr).unwrap();
        assert_eq!(manager.connection_count().unwrap(), 1);

        let conn = manager.get_connection(id).unwrap();
        assert!(conn.is_some());
    }

    #[test]
    fn test_manager_remove_connection() {
        let manager = ConnectionManager::with_defaults();
        let addr = "127.0.0.1:8080".parse().unwrap();

        let id = manager.create_connection(addr).unwrap();
        let removed = manager.remove_connection(id).unwrap();
        assert!(removed);
        assert_eq!(manager.connection_count().unwrap(), 0);
    }

    #[test]
    fn test_manager_metrics() {
        let manager = ConnectionManager::with_defaults();
        let addr = "127.0.0.1:8080".parse().unwrap();

        manager.create_connection(addr).unwrap();
        manager.create_connection(addr).unwrap();

        assert_eq!(manager.metrics().current_connections(), 2);
        assert_eq!(manager.metrics().total_connections(), 2);
    }

    #[test]
    fn test_manager_report() {
        let manager = ConnectionManager::with_defaults();
        let report = manager.report();
        assert!(report.contains("连接管理器报告"));
        assert!(report.contains("连接数: 0"));
    }
}
