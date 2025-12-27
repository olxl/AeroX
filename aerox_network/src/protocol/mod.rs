//! 协议模块
//!
//! 消息编解码和帧格式定义。

pub mod codec;
pub mod frame;

// 重新导出主要类型
pub use codec::{MessageCodec, MessageDecoder, MessageEncoder};
pub use frame::{Frame, FrameError};
