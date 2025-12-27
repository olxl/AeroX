//! AeroX 网络层抽象和协议实现
//!
//! 提供 TCP、KCP、QUIC 等传输协议的抽象接口。

pub mod connection;
pub mod protocol;
pub mod reactor;
pub mod transport;

// 导出主要类型到 crate root
pub use crate::connection::{Connection, ConnectionId, ConnectionIdGenerator};
pub use crate::protocol::{Frame, FrameError, MessageCodec, MessageDecoder, MessageEncoder};
pub use crate::transport::Transport;
// 重新导出 aerox_core 的错误类型
pub use aerox_core::{AeroXError, Result};

// 预导出
pub mod prelude {
    pub use crate::connection::{Connection, ConnectionId};
    pub use crate::protocol::{Frame, MessageCodec};
    pub use crate::reactor::{Acceptor, ConnectionBalancer, TcpReactor, Worker};
    pub use crate::transport::Transport;
    pub use aerox_core::{AeroXError, Result};
}
