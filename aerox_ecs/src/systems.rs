//! ECS 系统示例
//!
//! 演示如何使用 Bevy ECS 系统处理游戏逻辑。

use bevy::prelude::*;
use crate::components::*;
use crate::events::*;
use crate::world::EcsMetrics;

/// 连接管理系统
///
/// 当玩家连接时创建对应的实体。
pub fn connection_management_system(
    mut commands: Commands,
    mut events: EventReader<ConnectionEstablishedEvent>,
    mut metrics: ResMut<EcsMetrics>,
) {
    for event in events.read() {
        // 创建玩家实体
        commands.spawn((
            PlayerConnection::new(event.connection_id, event.address),
            Position::origin(),
            Health::full(100.0),
        ));

        metrics.entity_count += 1;
        info!("Player connected: {}", event.connection_id);
    }
}

/// 连接断开系统
///
/// 当玩家断开连接时清理实体。
pub fn disconnection_system(
    mut commands: Commands,
    mut events: EventReader<ConnectionClosedEvent>,
    query: Query<(Entity, &PlayerConnection)>,
    mut metrics: ResMut<EcsMetrics>,
) {
    for event in events.read() {
        // 查找并移除对应的实体
        for (entity, conn) in query.iter() {
            if conn.connection_id == event.connection_id {
                commands.entity(entity).despawn();
                metrics.entity_count -= 1;
                info!(
                    "Player disconnected: {} (reason: {})",
                    event.connection_id, event.reason
                );
                break;
            }
        }
    }
}

/// 消息处理系统
///
/// 处理接收到的网络消息。
pub fn message_handling_system(
    mut events: EventReader<MessageReceivedEvent>,
    query: Query<&PlayerConnection>,
    mut metrics: ResMut<EcsMetrics>,
) {
    for event in events.read() {
        // 查找发送者
        for conn in query.iter() {
            if conn.connection_id == event.connection_id {
                debug!(
                    "Message from {}: msg_id={}, seq={}, size={}",
                    event.connection_id,
                    event.message_id,
                    event.sequence_id,
                    event.payload.len()
                );
                break;
            }
        }

        metrics.events_processed += 1;
    }
}

/// 位置更新系统
///
/// 根据速度更新实体位置。
pub fn position_update_system(
    mut query: Query<(&mut Position, &Velocity)>,
    time: Res<Time>,
) {
    for (mut pos, vel) in query.iter_mut() {
        let delta = time.delta_seconds();
        pos.x += vel.vx * delta;
        pos.y += vel.vy * delta;
        pos.z += vel.vz * delta;
    }
}

/// 定时器更新系统
///
/// 更新所有定时器组件。
pub fn timer_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut GameTimer)>,
) {
    for (entity, mut timer) in query.iter_mut() {
        if timer.tick(time.delta()) {
            // 定时器触发
            info!("Timer triggered for entity {:?}", entity);

            // 如果不是重复定时器，完成后移除组件
            if !timer.repeating && timer.finished() {
                commands.entity(entity).remove::<GameTimer>();
            }
        }
    }
}

/// 生命值恢复系统
///
/// 定期为玩家恢复生命值。
#[derive(Component)]
pub struct HealthRegeneration {
    /// 每秒恢复量
    pub rate: f32,
}

impl HealthRegeneration {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

pub fn health_regen_system(
    mut query: Query<&mut Health, With<HealthRegeneration>>,
    regen_query: Query<&HealthRegeneration>,
    time: Res<Time>,
) {
    // 获取恢复速率（假设所有实体使用相同的速率）
    let rate = regen_query.iter().next().map(|r| r.rate).unwrap_or(0.0);

    for mut health in query.iter_mut() {
        if !health.is_dead() && !health.is_full() {
            health.heal(rate * time.delta_seconds());
        }
    }
}

/// 心跳超时阈值资源
#[derive(Resource, Clone, Copy)]
pub struct HeartbeatTimeoutThreshold {
    pub duration: std::time::Duration,
}

impl Default for HeartbeatTimeoutThreshold {
    fn default() -> Self {
        Self {
            duration: std::time::Duration::from_secs(30),
        }
    }
}

/// 心跳检测系统
///
/// 检测长时间未活动的连接。
pub fn heartbeat_detection_system(
    mut commands: Commands,
    mut events: EventWriter<HeartbeatTimeoutEvent>,
    query: Query<(Entity, &PlayerConnection)>,
    threshold: Res<HeartbeatTimeoutThreshold>,
) {
    for (entity, conn) in query.iter() {
        let idle_time = conn.idle_time();

        if idle_time > threshold.duration {
            // 触发心跳超时事件
            events.send(HeartbeatTimeoutEvent {
                connection_id: conn.connection_id,
                timeout_duration: threshold.duration,
                last_activity: conn.last_activity,
            });

            // 移除实体
            commands.entity(entity).despawn();
            warn!(
                "Heartbeat timeout for {}: idle={:?}",
                conn.connection_id, idle_time
            );
        }
    }
}

/// 清理断开连接系统
///
/// 清理断开连接后的资源。
pub fn cleanup_disconnected_system(
    query: Query<(Entity, &PlayerConnection)>,
    metrics: Res<EcsMetrics>,
) {
    // 这个系统可以用来清理其他资源，如：
    // - 保存玩家数据
    // - 通知其他玩家
    // - 释放分配的资源
    // 等

    // 这里只是一个示例框架
    let _ = (query, metrics);
}

/// 系统集合
///
/// 将相关系统分组以便调度。
///
/// # 示例
///
/// ```ignore
/// let mut app = App::new();
///
/// // 添加网络系统
/// app.add_systems(
///     Update,
///     (
///         connection_management_system,
///         disconnection_system,
///         message_handling_system,
///     ).chain()
/// );
///
/// // 添加游戏逻辑系统
/// app.add_systems(
///     Update,
///     (
///         position_update_system,
///         health_regen_system,
///         timer_update_system,
///     ).chain()
/// );
///
/// // 添加维护系统
/// app.add_systems(
///     Update,
///     (
///         heartbeat_detection_system,
///         cleanup_disconnected_system,
///     ).chain()
/// );
/// ```
pub struct GameSystems;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::EcsWorld;

    #[test]
    fn test_position_update_system() {
        let mut world = World::new();
        world.insert_resource::<Time>(Time::default());

        // 创建实体
        world.spawn((
            Position::origin(),
            Velocity::new(1.0, 2.0, 3.0),
        ));

        // 模拟时间流逝
        let mut time = world.resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs_f32(0.016)); // ~60 FPS
        drop(time);

        // 运行系统
        let mut schedule = Schedule::default();
        schedule.add_systems(position_update_system);
        schedule.run(&mut world);

        // 验证位置更新
        let pos = world.query::<&Position>().single(&world);
        assert!(pos.x > 0.0);
        assert!(pos.y > 0.0);
        assert!(pos.z > 0.0);
    }

    #[test]
    fn test_health_regen_system() {
        let mut world = World::new();

        // 创建带生命值恢复的实体
        world.spawn((
            Health::new(100.0),
            HealthRegeneration::new(10.0), // 每秒恢复 10 点
        ));

        world.insert_resource::<Time>(Time::default());

        // 初始生命值为 50
        let mut health = world.query::<&mut Health>().single_mut(&mut world);
        health.current = 50.0;
        drop(health);

        // 模拟 1 秒过去
        let mut time = world.resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(1));
        drop(time);

        // 运行系统
        let mut schedule = Schedule::default();
        schedule.add_systems(health_regen_system);
        schedule.run(&mut world);

        // 验证生命值恢复
        let health = world.query::<&Health>().single(&world);
        assert!((health.current - 60.0).abs() < 0.1); // 50 + 10 = 60
    }

    #[test]
    fn test_timer_system() {
        let mut world = World::new();

        // 创建带定时器的实体
        world.spawn(GameTimer::once(std::time::Duration::from_millis(100)));

        world.insert_resource::<Time>(Time::default());

        // 第一次触发
        let mut time = world.resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(50));
        drop(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(timer_update_system);

        // 运行系统（不应触发）
        schedule.run(&mut world);
        let timer = world.query::<&GameTimer>().single(&world);
        assert!(!timer.finished());

        // 第二次触发（应该触发）
        let mut time = world.resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(60));
        drop(time);

        schedule.run(&mut world);

        // 验证定时器已触发（一次性定时器会被移除）
        let timer_count = world.query::<&GameTimer>().iter(&world).count();
        assert_eq!(timer_count, 0);
    }
}
