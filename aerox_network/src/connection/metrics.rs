//! 连接指标
//!
//! 收集和统计连接相关的指标。

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// 连接指标
#[derive(Debug)]
pub struct ConnectionMetrics {
    /// 当前连接数
    current_connections: AtomicUsize,
    /// 总连接数（累计）
    total_connections: AtomicU64,
    /// 总接收字节数
    total_bytes_received: AtomicU64,
    /// 总发送字节数
    total_bytes_sent: AtomicU64,
    /// 总接收消息数
    total_messages_received: AtomicU64,
    /// 总发送消息数
    total_messages_sent: AtomicU64,
}

impl ConnectionMetrics {
    /// 创建新的连接指标
    pub fn new() -> Self {
        Self {
            current_connections: AtomicUsize::new(0),
            total_connections: AtomicU64::new(0),
            total_bytes_received: AtomicU64::new(0),
            total_bytes_sent: AtomicU64::new(0),
            total_messages_received: AtomicU64::new(0),
            total_messages_sent: AtomicU64::new(0),
        }
    }

    /// 使用指定值创建连接指标（用于克隆）
    pub(crate) fn with_values(
        current_connections: usize,
        total_connections: u64,
        total_bytes_received: u64,
        total_bytes_sent: u64,
        total_messages_received: u64,
        total_messages_sent: u64,
    ) -> Self {
        Self {
            current_connections: AtomicUsize::new(current_connections),
            total_connections: AtomicU64::new(total_connections),
            total_bytes_received: AtomicU64::new(total_bytes_received),
            total_bytes_sent: AtomicU64::new(total_bytes_sent),
            total_messages_received: AtomicU64::new(total_messages_received),
            total_messages_sent: AtomicU64::new(total_messages_sent),
        }
    }

    /// 增加连接数
    pub fn inc_connections(&self) {
        self.current_connections.fetch_add(1, Ordering::Relaxed);
        self.total_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// 减少连接数
    pub fn dec_connections(&self) {
        self.current_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// 记录接收字节
    pub fn record_bytes_received(&self, bytes: u64) {
        self.total_bytes_received
            .fetch_add(bytes, Ordering::Relaxed);
    }

    /// 记录发送字节
    pub fn record_bytes_sent(&self, bytes: u64) {
        self.total_bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    /// 记录接收消息
    pub fn record_message_received(&self) {
        self.total_messages_received.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录发送消息
    pub fn record_message_sent(&self) {
        self.total_messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取当前连接数
    pub fn current_connections(&self) -> usize {
        self.current_connections.load(Ordering::Relaxed)
    }

    /// 获取总连接数
    pub fn total_connections(&self) -> u64 {
        self.total_connections.load(Ordering::Relaxed)
    }

    /// 获取总接收字节数
    pub fn total_bytes_received(&self) -> u64 {
        self.total_bytes_received.load(Ordering::Relaxed)
    }

    /// 获取总发送字节数
    pub fn total_bytes_sent(&self) -> u64 {
        self.total_bytes_sent.load(Ordering::Relaxed)
    }

    /// 获取总接收消息数
    pub fn total_messages_received(&self) -> u64 {
        self.total_messages_received.load(Ordering::Relaxed)
    }

    /// 获取总发送消息数
    pub fn total_messages_sent(&self) -> u64 {
        self.total_messages_sent.load(Ordering::Relaxed)
    }

    /// 生成摘要报告
    pub fn summary(&self) -> String {
        format!(
            "连接指标:\n\
             - 当前连接: {}\n\
             - 总连接数: {}\n\
             - 接收字节: {}\n\
             - 发送字节: {}\n\
             - 接收消息: {}\n\
             - 发送消息: {}",
            self.current_connections(),
            self.total_connections(),
            self.total_bytes_received(),
            self.total_bytes_sent(),
            self.total_messages_received(),
            self.total_messages_sent()
        )
    }
}

impl Default for ConnectionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = ConnectionMetrics::new();
        assert_eq!(metrics.current_connections(), 0);
        assert_eq!(metrics.total_connections(), 0);
    }

    #[test]
    fn test_metrics_inc_dec() {
        let metrics = ConnectionMetrics::new();

        metrics.inc_connections();
        assert_eq!(metrics.current_connections(), 1);
        assert_eq!(metrics.total_connections(), 1);

        metrics.dec_connections();
        assert_eq!(metrics.current_connections(), 0);
        assert_eq!(metrics.total_connections(), 1); // 总数不变
    }

    #[test]
    fn test_metrics_bytes() {
        let metrics = ConnectionMetrics::new();

        metrics.record_bytes_received(1024);
        assert_eq!(metrics.total_bytes_received(), 1024);

        metrics.record_bytes_sent(2048);
        assert_eq!(metrics.total_bytes_sent(), 2048);
    }

    #[test]
    fn test_metrics_messages() {
        let metrics = ConnectionMetrics::new();

        metrics.record_message_received();
        assert_eq!(metrics.total_messages_received(), 1);

        metrics.record_message_sent();
        assert_eq!(metrics.total_messages_sent(), 1);
    }

    #[test]
    fn test_metrics_summary() {
        let metrics = ConnectionMetrics::new();
        let summary = metrics.summary();
        assert!(summary.contains("连接指标"));
        assert!(summary.contains("当前连接: 0"));
    }
}
