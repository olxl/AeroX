//! AeroX Protobuf 编解码支持
//!
//! 提供 Protobuf 消息的自动注册和零拷贝编解码。

pub mod registry;

// 预导出
pub mod prelude {
    pub use crate::registry::MessageRegistry;
}
