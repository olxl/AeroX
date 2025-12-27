//! ECS 事件定义
//!
//! 将网络层事件转换为 ECS 事件，便于游戏逻辑处理。

use bevy::prelude::*;
use bytes::Bytes;
use aerox_network::ConnectionId;
use std::net::SocketAddr;

/// 连接已建立事件
///
/// 当新客户端连接到服务器时触发。
#[derive(Event, Debug, Clone)]
pub struct ConnectionEstablishedEvent {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 客户端地址
    pub address: SocketAddr,
    /// 连接时间戳
    pub timestamp: std::time::Instant,
}

/// 连接已关闭事件
///
/// 当客户端断开连接时触发。
#[derive(Event, Debug, Clone)]
pub struct ConnectionClosedEvent {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 客户端地址
    pub address: SocketAddr,
    /// 关闭原因
    pub reason: String,
    /// 连接持续时间
    pub duration: std::time::Duration,
}

/// 消息接收事件
///
/// 当收到客户端消息时触发。
#[derive(Event, Debug, Clone)]
pub struct MessageReceivedEvent {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 消息 ID
    pub message_id: u32,
    /// 序列号
    pub sequence_id: u64,
    /// 消息内容
    pub payload: Bytes,
    /// 接收时间戳
    pub timestamp: std::time::Instant,
}

/// 消息发送事件
///
/// 当消息成功发送给客户端时触发。
#[derive(Event, Debug, Clone)]
pub struct MessageSentEvent {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 消息 ID
    pub message_id: u32,
    /// 序列号
    pub sequence_id: u64,
    /// 消息大小（字节）
    pub payload_size: usize,
    /// 发送时间戳
    pub timestamp: std::time::Instant,
}

/// 消息发送失败事件
///
/// 当消息发送失败时触发。
#[derive(Event, Debug, Clone)]
pub struct MessageSendFailedEvent {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 消息 ID
    pub message_id: u32,
    /// 失败原因
    pub error: String,
    /// 时间戳
    pub timestamp: std::time::Instant,
}

/// 心跳超时事件
///
/// 当客户端心跳超时时触发。
#[derive(Event, Debug, Clone)]
pub struct HeartbeatTimeoutEvent {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 超时时长
    pub timeout_duration: std::time::Duration,
    /// 最后活动时间
    pub last_activity: std::time::Instant,
}

/// 连接错误事件
///
/// 当连接发生错误时触发。
#[derive(Event, Debug, Clone)]
pub struct ConnectionErrorEvent {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 错误类型
    pub error_kind: ConnectionErrorKind,
    /// 错误信息
    pub error_message: String,
    /// 时间戳
    pub timestamp: std::time::Instant,
}

/// 连接错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionErrorKind {
    /// IO 错误
    IoError,
    /// 协议错误
    ProtocolError,
    /// 编解码错误
    CodecError,
    /// 超时错误
    TimeoutError,
    /// 其他错误
    Other,
}

/// 自定义事件
///
/// 用户自定义的游戏逻辑事件。
///
/// # 示例
///
/// ```ignore
/// #[derive(Event, Debug, Clone)]
/// struct PlayerMoveEvent {
///     player_id: u64,
///     new_position: Vec3,
/// }
/// ```
pub type CustomEvent = ();

/// 网络事件包装器
///
/// 用于统一处理所有网络相关的事件。
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// 连接建立
    Connected(ConnectionEstablishedEvent),
    /// 连接关闭
    Closed(ConnectionClosedEvent),
    /// 消息接收
    MessageReceived(MessageReceivedEvent),
    /// 消息发送
    MessageSent(MessageSentEvent),
    /// 消息发送失败
    MessageSendFailed(MessageSendFailedEvent),
    /// 心跳超时
    HeartbeatTimeout(HeartbeatTimeoutEvent),
    /// 连接错误
    Error(ConnectionErrorEvent),
}

impl NetworkEvent {
    /// 获取事件连接 ID
    pub fn connection_id(&self) -> ConnectionId {
        match self {
            NetworkEvent::Connected(e) => e.connection_id,
            NetworkEvent::Closed(e) => e.connection_id,
            NetworkEvent::MessageReceived(e) => e.connection_id,
            NetworkEvent::MessageSent(e) => e.connection_id,
            NetworkEvent::MessageSendFailed(e) => e.connection_id,
            NetworkEvent::HeartbeatTimeout(e) => e.connection_id,
            NetworkEvent::Error(e) => e.connection_id,
        }
    }

    /// 转换为 MessageReceivedEvent
    pub fn as_message_received(&self) -> Option<&MessageReceivedEvent> {
        match self {
            NetworkEvent::MessageReceived(e) => Some(e),
            _ => None,
        }
    }

    /// 转换为 ConnectionEstablishedEvent
    pub fn as_connection_established(&self) -> Option<&ConnectionEstablishedEvent> {
        match self {
            NetworkEvent::Connected(e) => Some(e),
            _ => None,
        }
    }

    /// 转换为 ConnectionClosedEvent
    pub fn as_connection_closed(&self) -> Option<&ConnectionClosedEvent> {
        match self {
            NetworkEvent::Closed(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_established_event() {
        let event = ConnectionEstablishedEvent {
            connection_id: ConnectionId::new(1),
            address: "127.0.0.1:8080".parse().unwrap(),
            timestamp: std::time::Instant::now(),
        };

        assert_eq!(event.address, "127.0.0.1:8080".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn test_message_received_event() {
        let event = MessageReceivedEvent {
            connection_id: ConnectionId::new(1),
            message_id: 1,
            sequence_id: 100,
            payload: Bytes::from("hello"),
            timestamp: std::time::Instant::now(),
        };

        assert_eq!(event.message_id, 1);
        assert_eq!(event.sequence_id, 100);
        assert_eq!(event.payload, Bytes::from("hello"));
    }

    #[test]
    fn test_network_event_wrapper() {
        let msg_event = MessageReceivedEvent {
            connection_id: ConnectionId::new(1),
            message_id: 1,
            sequence_id: 100,
            payload: Bytes::from("test"),
            timestamp: std::time::Instant::now(),
        };

        let network_event = NetworkEvent::MessageReceived(msg_event);
        assert!(network_event.as_message_received().is_some());
        assert!(network_event.as_connection_established().is_none());
    }
}
