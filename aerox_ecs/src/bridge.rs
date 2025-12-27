//! 网络事件到 ECS 的桥接
//!
//! 将网络层的事件转换为 ECS 事件并分发到 World 中。

use bevy::prelude::*;
use crate::world::{EcsWorld, EcsMetrics};
use crate::events::*;
use aerox_network::ConnectionId;
use std::net::SocketAddr;
use std::time::Instant;

/// 网络事件桥接器
///
/// 负责将网络层的事件转换为 ECS 事件并发送到 World。
pub struct NetworkBridge {
    /// 是否启用桥接
    enabled: bool,
}

impl Default for NetworkBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkBridge {
    /// 创建新的桥接器
    pub fn new() -> Self {
        Self {
            enabled: true,
        }
    }

    /// 启用桥接
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用桥接
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// 桥接连接建立事件
    pub fn on_connected(
        &self,
        world: &mut EcsWorld,
        connection_id: ConnectionId,
        address: SocketAddr,
    ) {
        if !self.enabled {
            return;
        }

        let event = ConnectionEstablishedEvent {
            connection_id,
            address,
            timestamp: Instant::now(),
        };

        world.send_event(event);
        world.metrics_mut().events_processed += 1;
    }

    /// 桥接连接关闭事件
    pub fn on_closed(
        &self,
        world: &mut EcsWorld,
        connection_id: ConnectionId,
        address: SocketAddr,
        reason: String,
        duration: std::time::Duration,
    ) {
        if !self.enabled {
            return;
        }

        let event = ConnectionClosedEvent {
            connection_id,
            address,
            reason,
            duration,
        };

        world.send_event(event);
        world.metrics_mut().events_processed += 1;
    }

    /// 桥接消息接收事件
    pub fn on_message_received(
        &self,
        world: &mut EcsWorld,
        connection_id: ConnectionId,
        message_id: u32,
        sequence_id: u64,
        payload: bytes::Bytes,
    ) {
        if !self.enabled {
            return;
        }

        let event = MessageReceivedEvent {
            connection_id,
            message_id,
            sequence_id,
            payload,
            timestamp: Instant::now(),
        };

        world.send_event(event);
        world.metrics_mut().events_processed += 1;
    }

    /// 桥接消息发送事件
    pub fn on_message_sent(
        &self,
        world: &mut EcsWorld,
        connection_id: ConnectionId,
        message_id: u32,
        sequence_id: u64,
        payload_size: usize,
    ) {
        if !self.enabled {
            return;
        }

        let event = MessageSentEvent {
            connection_id,
            message_id,
            sequence_id,
            payload_size,
            timestamp: Instant::now(),
        };

        world.send_event(event);
        world.metrics_mut().events_processed += 1;
    }

    /// 桥接消息发送失败事件
    pub fn on_message_send_failed(
        &self,
        world: &mut EcsWorld,
        connection_id: ConnectionId,
        message_id: u32,
        error: String,
    ) {
        if !self.enabled {
            return;
        }

        let event = MessageSendFailedEvent {
            connection_id,
            message_id,
            error,
            timestamp: Instant::now(),
        };

        world.send_event(event);
        world.metrics_mut().events_processed += 1;
    }

    /// 桥接心跳超时事件
    pub fn on_heartbeat_timeout(
        &self,
        world: &mut EcsWorld,
        connection_id: ConnectionId,
        timeout_duration: std::time::Duration,
        last_activity: Instant,
    ) {
        if !self.enabled {
            return;
        }

        let event = HeartbeatTimeoutEvent {
            connection_id,
            timeout_duration,
            last_activity,
        };

        world.send_event(event);
        world.metrics_mut().events_processed += 1;
    }

    /// 桥接连接错误事件
    pub fn on_connection_error(
        &self,
        world: &mut EcsWorld,
        connection_id: ConnectionId,
        error_kind: ConnectionErrorKind,
        error_message: String,
    ) {
        if !self.enabled {
            return;
        }

        let event = ConnectionErrorEvent {
            connection_id,
            error_kind,
            error_message,
            timestamp: Instant::now(),
        };

        world.send_event(event);
        world.metrics_mut().events_processed += 1;
    }
}

/// 事件调度器
///
/// 负责在 ECS 系统中处理网络事件。
pub struct EventScheduler;

impl EventScheduler {
    /// 创建新的事件调度器
    pub fn new() -> Self {
        Self
    }

    /// 处理所有待处理的事件
    ///
    /// 在 ECS Schedule 中调用此函数来处理事件队列。
    pub fn process_events(world: &mut World) {
        // 更新指标
        if let Some(mut metrics) = world.get_resource_mut::<EcsMetrics>() {
            metrics.last_update = Instant::now();
        }
    }
}

impl Default for EventScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_enable_disable() {
        let bridge = NetworkBridge::new();
        assert!(bridge.enabled);

        let mut bridge = bridge;
        bridge.disable();
        assert!(!bridge.enabled);

        bridge.enable();
        assert!(bridge.enabled);
    }

    #[test]
    fn test_bridge_on_connected() {
        let mut world = EcsWorld::new();
        world.initialize().unwrap();
        let bridge = NetworkBridge::new();

        let conn_id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();

        bridge.on_connected(&mut world, conn_id, addr);

        // 验证事件已发送（通过指标变化）
        assert_eq!(world.metrics().events_processed, 1);
    }

    #[test]
    fn test_bridge_on_message_received() {
        let mut world = EcsWorld::new();
        world.initialize().unwrap();
        let bridge = NetworkBridge::new();

        let conn_id = ConnectionId::new(1);
        let payload = bytes::Bytes::from("test message");

        bridge.on_message_received(&mut world, conn_id, 1, 100, payload);

        assert_eq!(world.metrics().events_processed, 1);
    }

    #[test]
    fn test_bridge_disabled() {
        let mut world = EcsWorld::new();
        world.initialize().unwrap();

        let mut bridge = NetworkBridge::new();
        bridge.disable();

        let conn_id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();

        bridge.on_connected(&mut world, conn_id, addr);

        // 禁用状态下事件不应发送
        assert_eq!(world.metrics().events_processed, 0);
    }
}
