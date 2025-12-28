//! Worker 线程
//!
//! 每个 Worker 负责处理分配给它的连接。

use crate::connection::ConnectionId;
use crate::protocol::frame::Frame;
use crate::protocol::codec::MessageCodec;
use crate::reactor::acceptor::NewConnection;
use aerox_core::Result;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use futures_util::{stream::StreamExt, sink::SinkExt};
use tokio_util::codec::{FramedRead, FramedWrite};

#[cfg(feature = "aerox_router")]
use aerox_router::Router;
use std::sync::Arc as StdArc;

/// Worker 配置
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Worker ID
    pub id: usize,
    /// 消息通道大小
    pub channel_size: usize,
    /// 路由器（可选）
    #[cfg(feature = "aerox_router")]
    pub router: Option<StdArc<Router>>,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            id: 0,
            channel_size: 1024,
            #[cfg(feature = "aerox_router")]
            router: None,
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
    /// 路由器（可选）
    #[cfg(feature = "aerox_router")]
    router: Option<StdArc<Router>>,
}

impl Worker {
    /// 创建新的 Worker
    pub fn new(config: WorkerConfig) -> (Self, mpsc::Sender<NewConnection>) {
        let (tx, rx) = mpsc::channel(config.channel_size);

        let worker = Self {
            id: config.id,
            rx,
            active_connections: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            #[cfg(feature = "aerox_router")]
            router: config.router,
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
                        stream,
                        remote_addr,
                    }) => {
                        println!("Worker {} 接受新连接: {}", self.id, remote_addr);

                        // 增加活跃连接计数
                        self.active_connections
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        // 处理连接
                        let result = if cfg!(feature = "aerox_router") {
                            #[cfg(feature = "aerox_router")]
                            {
                                self.handle_connection_with_router(stream, remote_addr).await
                            }
                            #[cfg(not(feature = "aerox_router"))]
                            {
                                self.handle_connection_simple(stream, remote_addr).await
                            }
                        } else {
                            self.handle_connection_simple(stream, remote_addr).await
                        };

                        if let Err(e) = result {
                            eprintln!("Worker {} 连接处理错误: {}", self.id, e);
                        }

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

    /// 处理连接（简单版本 - 仅关闭）
    async fn handle_connection_simple(
        &self,
        mut stream: TcpStream,
        remote_addr: std::net::SocketAddr,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        println!("Worker {} 简单处理连接: {}", self.id, remote_addr);
        let _ = stream.shutdown().await;
        Ok(())
    }

    /// 处理连接（带路由器）
    #[cfg(feature = "aerox_router")]
    async fn handle_connection_with_router(
        &self,
        stream: TcpStream,
        remote_addr: std::net::SocketAddr,
    ) -> Result<()> {
        use bytes::Bytes;
        use std::sync::atomic::{AtomicU64, Ordering};

        println!("Worker {} 路由处理连接: {}", self.id, remote_addr);

        // 1. 分离读写
        let (read_half, write_half) = stream.into_split();
        let mut read_half = FramedRead::new(read_half, MessageCodec::new());
        let mut write_half = FramedWrite::new(write_half, MessageCodec::new());

        // 2. 创建响应通道（使用有界channel）
        let (response_tx, mut response_rx) = mpsc::channel::<(u16, Bytes)>(128);

        // 3. 启动后台写入任务
        let worker_id = self.id; // 捕获 worker_id 用于打印
        tokio::spawn(async move {
            while let Some((msg_id, data)) = response_rx.recv().await {
                let response_frame = Frame::new(msg_id, 0, data);
                // println!("Worker {} 发送响应: msg_id={}", worker_id, msg_id);
                if let Err(e) = write_half.send(response_frame).await {
                    eprintln!("Worker {} 发送响应失败: {}", worker_id, e);
                    break;
                }
            }
        });

        // 4. 使用简单的计数器生成连接 ID
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let conn_id = ConnectionId::new(COUNTER.fetch_add(1, Ordering::SeqCst));

        // 5. 主任务：只处理接收
        while let Some(result) = read_half.next().await {
            match result {
                Ok(frame) => {
                    // println!("Worker {} 收到消息: msg_id={}", self.id, frame.message_id);

                    // 创建 Context（使用普通mpsc::Sender）
                    let ctx = aerox_router::Context::with_responder(
                        conn_id,
                        remote_addr,
                        frame.message_id,
                        frame.sequence_id,
                        frame.body.clone(),
                        response_tx.clone(),
                    );

                    // 路由处理
                    if let Some(ref router) = self.router {
                        if let Err(e) = router.handle(ctx).await {
                            eprintln!("Worker {} 路由处理失败: {}", self.id, e);
                        }
                    } else {
                        eprintln!("Worker {} 警告: 没有配置路由器", self.id);
                    }
                }
                Err(e) => {
                    eprintln!("Worker {} 解码错误: {}", self.id, e);
                    break;
                }
            }
        }

        println!("Worker {} 连接关闭: {}", self.id, remote_addr);
        Ok(())
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
