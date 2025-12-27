//! 消息帧
//!
//! 定义网络消息的帧格式。

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt;

/// 消息帧
///
/// 采用 Length-Prefix-Message 格式
///
/// ```text
/// +--------+--------+--------+----------+
/// | Length | Msg ID | Seq ID |   Body   |
/// | 4 bytes| 2 bytes| 4 bytes| variable |
/// +--------+--------+--------+----------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// 消息 ID
    pub message_id: u16,
    /// 序列号（用于请求匹配）
    pub sequence_id: u32,
    /// 消息体
    pub body: Bytes,
}

impl Frame {
    /// 帧头大小（不包含长度前缀，只包含 消息ID + 序列ID）
    pub const HEADER_SIZE: usize = 2 + 4;

    /// 长度前缀大小
    pub const LENGTH_SIZE: usize = 4;

    /// 最大消息体大小（16MB）
    pub const MAX_BODY_SIZE: usize = 16 * 1024 * 1024;

    /// 创建新的消息帧
    pub fn new(message_id: u16, sequence_id: u32, body: Bytes) -> Self {
        Self {
            message_id,
            sequence_id,
            body,
        }
    }

    /// 创建无消息体的帧
    pub fn empty(message_id: u16, sequence_id: u32) -> Self {
        Self {
            message_id,
            sequence_id,
            body: Bytes::new(),
        }
    }

    /// 计算完整帧大小（包含长度前缀）
    pub fn frame_size(&self) -> usize {
        Self::LENGTH_SIZE + Self::HEADER_SIZE + self.body.len()
    }

    /// 计算帧内容大小（不包含长度前缀）
    fn payload_size(&self) -> usize {
        Self::HEADER_SIZE + self.body.len()
    }

    /// 编码帧为字节流
    pub fn encode(&self) -> BytesMut {
        let payload_size = self.payload_size();
        let total_size = Self::LENGTH_SIZE + payload_size;
        let mut buf = BytesMut::with_capacity(total_size);

        // 写入长度（不包含长度字段本身）- 使用小端序
        buf.put_u32_le(payload_size as u32);

        // 写入消息 ID - 使用小端序
        buf.put_u16_le(self.message_id);

        // 写入序列 ID - 使用小端序
        buf.put_u32_le(self.sequence_id);

        // 写入消息体
        buf.put(self.body.clone());

        buf
    }

    /// 从字节流解码帧
    ///
    /// 返回 (frame, consumed_bytes)
    pub fn decode(buf: &mut BytesMut) -> Result<Option<Self>, FrameError> {
        // 检查是否有足够的数据读取长度字段
        if buf.len() < 4 {
            return Ok(None);
        }

        // 读取帧长度（不包含长度字段本身）- 使用小端序
        let frame_len = buf.get_u32_le() as usize;

        // 检查最大长度限制
        if frame_len > Self::HEADER_SIZE + Self::MAX_BODY_SIZE {
            return Err(FrameError::FrameTooLarge(frame_len));
        }

        // 检查是否有完整的帧
        if buf.len() < frame_len {
            // 需要将数据放回去，因为还没读取完整
            let mut restored = BytesMut::with_capacity(Self::LENGTH_SIZE + buf.len());
            restored.put_u32_le(frame_len as u32);
            restored.extend_from_slice(&buf[..]);
            *buf = restored;
            return Ok(None);
        }

        // 读取消息 ID - 使用小端序
        let message_id = buf.get_u16_le();

        // 读取序列 ID - 使用小端序
        let sequence_id = buf.get_u32_le();

        // 读取消息体
        let body_len = frame_len - Self::HEADER_SIZE;
        let body = buf.split_to(body_len).freeze();

        Ok(Some(Self {
            message_id,
            sequence_id,
            body,
        }))
    }

    /// 检查帧是否有效
    pub fn validate(&self) -> Result<(), FrameError> {
        if self.body.len() > Self::MAX_BODY_SIZE {
            return Err(FrameError::BodyTooLarge(self.body.len()));
        }
        Ok(())
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Frame[msg_id={}, seq={}, body_len={}]",
            self.message_id,
            self.sequence_id,
            self.body.len()
        )
    }
}

/// 帧错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    /// 帧过大
    FrameTooLarge(usize),
    /// 消息体过大
    BodyTooLarge(usize),
    /// 无效的帧格式
    InvalidFormat(String),
    /// 数据不完整
    Incomplete,
    /// IO 错误
    Io(String),
}

impl From<std::io::Error> for FrameError {
    fn from(err: std::io::Error) -> Self {
        FrameError::Io(err.to_string())
    }
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameTooLarge(size) => write!(f, "帧过大: {} 字节", size),
            Self::BodyTooLarge(size) => write!(f, "消息体过大: {} 字节", size),
            Self::InvalidFormat(msg) => write!(f, "无效的帧格式: {}", msg),
            Self::Incomplete => write!(f, "数据不完整"),
            Self::Io(msg) => write!(f, "IO 错误: {}", msg),
        }
    }
}

impl std::error::Error for FrameError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        let frame = Frame::new(1, 100, Bytes::from("hello"));
        assert_eq!(frame.message_id, 1);
        assert_eq!(frame.sequence_id, 100);
        assert_eq!(frame.body, Bytes::from("hello"));
    }

    #[test]
    fn test_empty_frame() {
        let frame = Frame::empty(1, 100);
        assert_eq!(frame.body.len(), 0);
        // 4 (长度) + 2 (msg_id) + 4 (seq_id) + 0 (body) = 10
        assert_eq!(frame.frame_size(), 10);
    }

    #[test]
    fn test_frame_encode_decode() {
        let original = Frame::new(42, 12345, Bytes::from("test data"));
        let encoded = original.encode();

        let mut buf = encoded;
        let decoded = Frame::decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded.message_id, original.message_id);
        assert_eq!(decoded.sequence_id, original.sequence_id);
        assert_eq!(decoded.body, original.body);
    }

    #[test]
    fn test_frame_incomplete() {
        let mut buf = BytesMut::from(&[0x01, 0x02, 0x03][..]); // 不足 4 字节
        assert!(Frame::decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn test_frame_size_calculation() {
        let frame = Frame::new(1, 100, Bytes::from("hello"));
        // 4 (长度) + 2 (msg_id) + 4 (seq_id) + 5 (body) = 15
        assert_eq!(frame.frame_size(), 15);
    }

    #[test]
    fn test_frame_validate() {
        let valid_frame = Frame::new(1, 100, Bytes::from("test"));
        assert!(valid_frame.validate().is_ok());

        let invalid_body = vec![0u8; Frame::MAX_BODY_SIZE + 1];
        let invalid_frame = Frame::new(1, 100, Bytes::from(invalid_body));
        assert!(invalid_frame.validate().is_err());
    }

    #[test]
    fn test_frame_display() {
        let frame = Frame::new(1, 100, Bytes::from("hello"));
        let display = format!("{}", frame);
        assert!(display.contains("msg_id=1"));
        assert!(display.contains("seq=100"));
        assert!(display.contains("body_len=5"));
    }

    #[test]
    fn test_multiple_frames_in_buffer() {
        let frame1 = Frame::new(1, 100, Bytes::from("first"));
        let frame2 = Frame::new(2, 200, Bytes::from("second"));

        let mut buf = BytesMut::new();
        buf.extend_from_slice(&frame1.encode());
        buf.extend_from_slice(&frame2.encode());

        // 解码第一个帧
        let decoded1 = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(decoded1.message_id, 1);

        // 解码第二个帧
        let decoded2 = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(decoded2.message_id, 2);

        // 缓冲区应该为空
        assert!(buf.is_empty());
    }
}
