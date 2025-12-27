//! 连接均衡器
//!
//! 负责将新连接分配给不同的 Worker。

use aerox_core::AeroXError;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 连接均衡器
///
/// 使用轮询算法将连接分配给 Worker
#[derive(Debug)]
pub struct ConnectionBalancer {
    /// Worker 数量
    worker_count: usize,
    /// 当前索引（原子操作）
    current: AtomicUsize,
}

impl ConnectionBalancer {
    /// 创建新的连接均衡器
    pub fn new(worker_count: usize) -> Self {
        assert!(worker_count > 0, "Worker count must be greater than 0");
        Self {
            worker_count,
            current: AtomicUsize::new(0),
        }
    }

    /// 获取下一个 Worker ID
    ///
    /// 使用轮询算法分配
    pub fn next_worker(&self) -> usize {
        let idx = self.current.fetch_add(1, Ordering::Relaxed);
        idx % self.worker_count
    }

    /// 获取 Worker 数量
    pub fn worker_count(&self) -> usize {
        self.worker_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balancer_creation() {
        let balancer = ConnectionBalancer::new(4);
        assert_eq!(balancer.worker_count(), 4);
    }

    #[test]
    fn test_balancer_distribution() {
        let balancer = ConnectionBalancer::new(4);

        // 测试轮询分配
        let mut counts = vec![0; 4];
        for _ in 0..16 {
            let worker_id = balancer.next_worker();
            counts[worker_id] += 1;
        }

        // 每个 Worker 应该分配到 4 个连接
        assert_eq!(counts, vec![4, 4, 4, 4]);
    }

    #[test]
    #[should_panic(expected = "Worker count must be greater than 0")]
    fn test_balancer_zero_workers() {
        ConnectionBalancer::new(0);
    }
}
