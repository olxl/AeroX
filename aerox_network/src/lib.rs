//! AeroX 网络层抽象和协议实现
//!
//! 提供 TCP、KCP、QUIC 等传输协议的抽象接口。

pub mod transport;
pub mod connection;

// 导出主要类型到 crate root
pub use crate::connection::{Connection, ConnectionId, ConnectionIdGenerator};
pub use crate::transport::Transport;
// 重新导出 aerox_core 的错误类型
pub use aerox_core::{AeroXError, Result};

// 预导出
pub mod prelude {
    pub use crate::transport::Transport;
    pub use crate::connection::{Connection, ConnectionId};
    pub use aerox_core::{AeroXError, Result};
}
