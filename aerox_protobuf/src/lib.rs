//! AeroX Protobuf 编解码支持
//!
//! 提供 Protobuf 消息的自动注册和零拷贝编解码。

pub mod registry;

// 导出主要类型
pub use crate::registry::{
    decode_message, encode_message, MessageEncoder, MessageEncoderFn, MessageRegistry,
    RegistryError, unwrap_message, wrap_message,
};

// 预导出
pub mod prelude {
    pub use crate::registry::{
        decode_message, encode_message, MessageEncoder, MessageRegistry, RegistryError,
        unwrap_message, wrap_message,
    };
}
