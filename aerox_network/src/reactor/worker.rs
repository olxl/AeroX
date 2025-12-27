//! Worker 线程
//!
//! 每个 Worker 负责处理分配给它的连接。

use crate::reactor::acceptor::NewConnection;
use aerox_core::Result;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Worker 配置
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Worker ID
    pub id: usize,
    /// 消息通道大小
    pub channel_size: usize,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            id: 0,
            channel_size: 1024,
        }
    }
}

/// Worker 线程
///
/// 处理分配的连接和消息
pub struct Worker {
    /// Worker ID
    id: usize,
    /// 接收新连接的通道
    rx: mpsc::Receiver<NewConnection>,
    /// 活跃连接数
    active_connections: Arc<std::sync::atomic::AtomicUsize>,
}

impl Worker {
    /// 创建新的 Worker
    pub fn new(config: WorkerConfig) -> (Self, mpsc::Sender<NewConnection>) {
        let (tx, rx) = mpsc::channel(config.channel_size);

        let worker = Self {
            id: config.id,
            rx,
            active_connections: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        };

        (worker, tx)
    }

    /// 启动 Worker
    ///
    /// 返回 JoinHandle 用于等待 Worker 完成
    pub fn spawn(mut self) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            println!("Worker {} 启动", self.id);

            loop {
                // 接收新连接
                match self.rx.recv().await {
                    Some(NewConnection {
                        mut stream,
                        remote_addr,
                    }) => {
                        println!("Worker {} 接受新连接: {}", self.id, remote_addr);

                        // 增加活跃连接计数
                        self.active_connections
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        // TODO: 处理连接
                        // 当前阶段仅关闭连接
                        let _ = stream.shutdown().await;

                        // 减少活跃连接计数
                        self.active_connections
                            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    }
                    None => {
                        println!("Worker {} 通道关闭，退出", self.id);
                        break;
                    }
                }
            }

            Ok(())
        })
    }

    /// 获取活跃连接数
    pub fn active_connections(&self) -> usize {
        self.active_connections
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_creation() {
        let config = WorkerConfig {
            id: 0,
            ..Default::default()
        };

        let (worker, _tx) = Worker::new(config);
        assert_eq!(worker.id, 0);
        assert_eq!(worker.active_connections(), 0);
    }

    #[test]
    fn test_worker_config_default() {
        let config = WorkerConfig::default();
        assert_eq!(config.channel_size, 1024);
    }
}
