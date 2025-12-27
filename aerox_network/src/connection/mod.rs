//! 连接管理
//!
//! 连接生命周期管理和连接池实现。

pub mod id;
pub mod manager;
pub mod metrics;
pub mod pool;

// 重新导出主要类型
pub use id::{Connection, ConnectionId, ConnectionIdGenerator, ConnectionState};
pub use manager::{ConnectionManager, ConnectionManagerConfig};
pub use metrics::ConnectionMetrics;
pub use pool::ConnectionPool;
