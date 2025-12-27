//! 消息编解码器
//!
//! 提供流式消息的编解码功能。

use crate::protocol::frame::{Frame, FrameError};
use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

/// 消息编码器
///
/// 将 Frame 编码为字节流
#[derive(Debug, Clone, Default)]
pub struct MessageEncoder;

impl MessageEncoder {
    /// 创建新的编码器
    pub fn new() -> Self {
        Self
    }
}

impl Encoder<Frame> for MessageEncoder {
    type Error = FrameError;

    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // 验证帧
        item.validate()?;

        // 编码帧
        let encoded = item.encode();
        dst.extend_from_slice(&encoded);

        Ok(())
    }
}

/// 消息解码器
///
/// 从字节流解码 Frame
#[derive(Debug, Clone, Default)]
pub struct MessageDecoder {
    /// 是否读取帧头
    _phantom: std::marker::PhantomData<()>,
}

impl MessageDecoder {
    /// 创建新的解码器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Decoder for MessageDecoder {
    type Item = Frame;
    type Error = FrameError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Frame::decode(src)
    }
}

/// 编解码器组合
///
/// 同时提供编码和解码功能
#[derive(Debug, Clone, Default)]
pub struct MessageCodec {
    encoder: MessageEncoder,
    decoder: MessageDecoder,
}

impl MessageCodec {
    /// 创建新的编解码器
    pub fn new() -> Self {
        Self {
            encoder: MessageEncoder::new(),
            decoder: MessageDecoder::new(),
        }
    }

    /// 获取编码器引用
    pub fn encoder(&mut self) -> &mut MessageEncoder {
        &mut self.encoder
    }

    /// 获取解码器引用
    pub fn decoder(&mut self) -> &mut MessageDecoder {
        &mut self.decoder
    }
}

impl Encoder<Frame> for MessageCodec {
    type Error = FrameError;

    fn encode(&mut self, item: Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encoder.encode(item, dst)
    }
}

impl Decoder for MessageCodec {
    type Item = Frame;
    type Error = FrameError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decoder.decode(src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_encoder() {
        let mut encoder = MessageEncoder::new();
        let mut dst = BytesMut::new();

        let frame = Frame::new(1, 100, Bytes::from("hello"));
        encoder.encode(frame.clone(), &mut dst).unwrap();

        // 验证编码结果
        assert!(!dst.is_empty());
        assert_eq!(dst.len(), frame.frame_size());
    }

    #[test]
    fn test_decoder_complete_frame() {
        let mut decoder = MessageDecoder::new();

        // 编码一个帧
        let frame = Frame::new(1, 100, Bytes::from("hello"));
        let mut src = frame.encode();

        // 解码
        let decoded = decoder.decode(&mut src).unwrap().unwrap();

        assert_eq!(decoded.message_id, frame.message_id);
        assert_eq!(decoded.sequence_id, frame.sequence_id);
        assert_eq!(decoded.body, frame.body);
    }

    #[test]
    fn test_decoder_incomplete_frame() {
        let mut decoder = MessageDecoder::new();
        let mut src = BytesMut::from(&[0x01, 0x02, 0x03][..]); // 不完整数据

        let result = decoder.decode(&mut src).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_codec_round_trip() {
        let mut codec = MessageCodec::new();
        let mut dst = BytesMut::new();

        // 编码
        let original = Frame::new(42, 12345, Bytes::from("test data"));
        codec.encode(original.clone(), &mut dst).unwrap();

        // 解码
        let decoded = codec.decode(&mut dst).unwrap().unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_codec_multiple_frames() {
        let mut codec = MessageCodec::new();
        let mut dst = BytesMut::new();

        // 编码多个帧
        let frame1 = Frame::new(1, 100, Bytes::from("first"));
        let frame2 = Frame::new(2, 200, Bytes::from("second"));
        let frame3 = Frame::new(3, 300, Bytes::from("third"));

        codec.encode(frame1.clone(), &mut dst).unwrap();
        codec.encode(frame2.clone(), &mut dst).unwrap();
        codec.encode(frame3.clone(), &mut dst).unwrap();

        // 解码所有帧
        let decoded1 = codec.decode(&mut dst).unwrap().unwrap();
        let decoded2 = codec.decode(&mut dst).unwrap().unwrap();
        let decoded3 = codec.decode(&mut dst).unwrap().unwrap();

        assert_eq!(decoded1, frame1);
        assert_eq!(decoded2, frame2);
        assert_eq!(decoded3, frame3);
    }

    #[test]
    fn test_encoder_too_large_frame() {
        let mut encoder = MessageEncoder::new();
        let mut dst = BytesMut::new();

        let large_body = vec![0u8; Frame::MAX_BODY_SIZE + 1];
        let frame = Frame::new(1, 100, Bytes::from(large_body));

        let result = encoder.encode(frame, &mut dst);
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_partial_frame() {
        let mut decoder = MessageDecoder::new();

        // 编码一个帧
        let frame = Frame::new(1, 100, Bytes::from("hello world"));
        let encoded = frame.encode();

        // 只发送部分数据
        let partial_len = encoded.len() / 2;
        let mut src = BytesMut::from(&encoded[..partial_len]);

        // 应该返回 None（数据不完整）
        let result = decoder.decode(&mut src).unwrap();
        assert!(result.is_none());

        // 添加剩余数据
        src.extend_from_slice(&encoded[partial_len..]);

        // 现在应该能解码了
        let decoded = decoder.decode(&mut src).unwrap().unwrap();
        assert_eq!(decoded.message_id, frame.message_id);
    }

    #[test]
    fn test_codec_new() {
        let codec = MessageCodec::new();
        // 编码器和解码器都是无状态的，这里只测试创建
        let _ = &codec.encoder;
        let _ = &codec.decoder;
    }
}
