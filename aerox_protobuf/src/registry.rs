//! 消息注册表
//!
//! 管理消息类型和消息 ID 的映射，支持序列化和反序列化。

use bytes::Bytes;
use std::collections::HashMap;
use thiserror::Error;

/// 消息注册错误
#[derive(Error, Debug)]
pub enum RegistryError {
    /// 消息未注册
    #[error("消息未注册: {0}")]
    MessageNotRegistered(u32),

    /// 消息已存在
    #[error("消息已存在: {0}")]
    MessageAlreadyExists(u32),

    /// 编码错误
    #[error("消息编码失败: {0}")]
    EncodeError(String),

    /// 解码错误
    #[error("消息解码失败: {0}")]
    DecodeError(#[from] prost::DecodeError),
}

/// 消息编码器 trait
///
/// 所有 Protobuf 消息都需要实现此 trait
pub trait MessageEncoder: prost::Message + Send + Sync + 'static {
    /// 获取消息类型 ID
    fn message_id(&self) -> u32;

    /// 获取消息类型名称
    fn type_name() -> &'static str
    where
        Self: Sized;
}

/// 消息编解码器
pub type MessageEncoderFn =
    fn(Bytes) -> Result<Box<dyn prost::Message + Send + Sync>, RegistryError>;

/// 消息注册表
pub struct MessageRegistry {
    /// 消息 ID 到消息名称的映射
    messages: HashMap<u32, String>,
}

impl MessageRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
        }
    }

    /// 注册消息类型
    pub fn register(&mut self, id: u32, name: String) -> Result<(), RegistryError> {
        if self.messages.contains_key(&id) {
            return Err(RegistryError::MessageAlreadyExists(id));
        }
        self.messages.insert(id, name);
        Ok(())
    }

    /// 检查消息是否已注册
    pub fn contains(&self, id: u32) -> bool {
        self.messages.contains_key(&id)
    }

    /// 获取消息名称
    pub fn get_name(&self, id: u32) -> Option<&String> {
        self.messages.get(&id)
    }

    /// 获取已注册消息数量
    pub fn count(&self) -> usize {
        self.messages.len()
    }

    /// 列出所有已注册的消息 ID
    pub fn list_ids(&self) -> Vec<u32> {
        self.messages.keys().copied().collect()
    }
}

impl Default for MessageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 编码 Protobuf 消息
///
/// 将任意实现了 prost::Message 的类型编码为 Bytes
pub fn encode_message<M: prost::Message>(msg: &M) -> Result<Bytes, RegistryError> {
    let mut buf = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf)
        .map_err(|e| RegistryError::EncodeError(e.to_string()))?;
    Ok(Bytes::from(buf))
}

/// 解码 Protobuf 消息
///
/// 从 Bytes 解码为指定的消息类型
pub fn decode_message<M: prost::Message + Default>(data: Bytes) -> Result<M, RegistryError> {
    M::decode(data).map_err(RegistryError::from)
}

/// 创建消息包装器
///
/// 将消息 ID 和消息负载组合成完整的消息帧
pub fn wrap_message(message_id: u32, sequence_id: u64, payload: Bytes) -> Result<Bytes, RegistryError> {
    // 简单的包装格式: [message_id: 4字节][sequence_id: 8字节][payload_length: 4字节][payload]
    let total_len = 4 + 8 + 4 + payload.len();
    let mut buf = Vec::with_capacity(total_len);

    // 写入 message_id
    buf.extend_from_slice(&message_id.to_be_bytes());
    // 写入 sequence_id
    buf.extend_from_slice(&sequence_id.to_be_bytes());
    // 写入 payload 长度
    buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    // 写入 payload
    buf.extend_from_slice(&payload);

    Ok(Bytes::from(buf))
}

/// 解包消息
///
/// 从消息帧中提取 message_id, sequence_id 和 payload
pub fn unwrap_message(data: Bytes) -> Result<(u32, u64, Bytes), RegistryError> {
    if data.len() < 16 {
        return Err(RegistryError::DecodeError(prost::DecodeError::new(
            "消息长度不足",
        )));
    }

    let message_id = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    let sequence_id = u64::from_be_bytes([
        data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11],
    ]);
    let payload_len = u32::from_be_bytes([data[12], data[13], data[14], data[15]]) as usize;

    if data.len() < 16 + payload_len {
        return Err(RegistryError::DecodeError(prost::DecodeError::new(
            "负载长度不匹配",
        )));
    }

    let payload = data.slice(16..16 + payload_len);

    Ok((message_id, sequence_id, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试消息结构
    #[derive(Clone, PartialEq, prost::Message)]
    struct TestMessage {
        #[prost(string, tag = "1")]
        content: String,
        #[prost(uint64, tag = "2")]
        timestamp: u64,
    }

    #[test]
    fn test_registry() {
        let mut registry = MessageRegistry::new();
        registry.register(1001, "TestMessage".to_string()).unwrap();
        assert!(registry.contains(1001));
        assert!(!registry.contains(1002));
        assert_eq!(registry.get_name(1001), Some(&"TestMessage".to_string()));
    }

    #[test]
    fn test_encode_decode() {
        let msg = TestMessage {
            content: "Hello, World!".to_string(),
            timestamp: 12345,
        };

        // 编码
        let encoded = encode_message(&msg).unwrap();
        assert!(!encoded.is_empty());

        // 解码
        let decoded: TestMessage = decode_message(encoded).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_wrap_unwrap_message() {
        let payload = Bytes::from("test payload data");
        let wrapped = wrap_message(1001, 12345, payload.clone()).unwrap();

        let (msg_id, seq_id, unpacked_payload) = unwrap_message(wrapped).unwrap();
        assert_eq!(msg_id, 1001);
        assert_eq!(seq_id, 12345);
        assert_eq!(unpacked_payload, payload);
    }

    #[test]
    fn test_registry_duplicate() {
        let mut registry = MessageRegistry::new();
        registry.register(1001, "FirstMessage".to_string()).unwrap();
        let result = registry.register(1001, "SecondMessage".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_list() {
        let mut registry = MessageRegistry::new();
        registry.register(1001, "Message1".to_string()).unwrap();
        registry.register(1002, "Message2".to_string()).unwrap();
        registry.register(1003, "Message3".to_string()).unwrap();

        let ids = registry.list_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&1001));
        assert!(ids.contains(&1002));
        assert!(ids.contains(&1003));
    }

    #[test]
    fn test_empty_message() {
        let payload = Bytes::new();
        let wrapped = wrap_message(1001, 1, payload).unwrap();
        let (_, _, unpacked) = unwrap_message(wrapped).unwrap();
        assert!(unpacked.is_empty());
    }
}

