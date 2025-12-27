//! ECS 基础组件定义
//!
//! 提供游戏开发常用的基础组件。

use bevy::prelude::*;
use aerox_network::ConnectionId;
use std::net::SocketAddr;
use std::time::Instant;

/// 玩家连接组件
///
/// 标识一个实体代表已连接的玩家。
#[derive(Component, Debug, Clone)]
pub struct PlayerConnection {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 客户端地址
    pub address: SocketAddr,
    /// 连接时间
    pub connected_at: Instant,
    /// 最后活动时间
    pub last_activity: Instant,
}

impl PlayerConnection {
    /// 创建新的玩家连接
    pub fn new(connection_id: ConnectionId, address: SocketAddr) -> Self {
        let now = Instant::now();
        Self {
            connection_id,
            address,
            connected_at: now,
            last_activity: now,
        }
    }

    /// 更新活动时间
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// 获取连接持续时间
    pub fn duration(&self) -> std::time::Duration {
        self.connected_at.elapsed()
    }

    /// 获取空闲时间
    pub fn idle_time(&self) -> std::time::Duration {
        self.last_activity.elapsed()
    }
}

/// 3D 位置组件
///
/// 实体在 3D 空间中的位置。
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// X 坐标
    pub x: f32,
    /// Y 坐标
    pub y: f32,
    /// Z 坐标
    pub z: f32,
}

impl Position {
    /// 创建新位置
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// 创建原点位置
    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// 计算到另一个位置的距离
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::origin()
    }
}

/// 3D 旋转组件
///
/// 实体的旋转角度（欧拉角）。
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Rotation {
    /// Pitch（俯仰角）
    pub pitch: f32,
    /// Yaw（偏航角）
    pub yaw: f32,
    /// Roll（翻滚角）
    pub roll: f32,
}

impl Rotation {
    /// 创建新旋转
    pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        Self { pitch, yaw, roll }
    }

    /// 创建零旋转
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self::zero()
    }
}

/// 速度组件
///
/// 实体的移动速度。
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    /// X 方向速度
    pub vx: f32,
    /// Y 方向速度
    pub vy: f32,
    /// Z 方向速度
    pub vz: f32,
}

impl Velocity {
    /// 创建新速度
    pub fn new(vx: f32, vy: f32, vz: f32) -> Self {
        Self { vx, vy, vz }
    }

    /// 创建零速度
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// 计算速度大小
    pub fn magnitude(&self) -> f32 {
        (self.vx * self.vx + self.vy * self.vy + self.vz * self.vz).sqrt()
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::zero()
    }
}

/// 生命值组件
///
/// 实体的生命值。
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Health {
    /// 当前生命值
    pub current: f32,
    /// 最大生命值
    pub max: f32,
}

impl Health {
    /// 创建新的生命值
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
        }
    }

    /// 创建满生命值
    pub fn full(max: f32) -> Self {
        Self::new(max)
    }

    /// 受到伤害
    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    /// 治疗
    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// 是否死亡
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    /// 是否满血
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// 获取生命值百分比
    pub fn percentage(&self) -> f32 {
        if self.max > 0.0 {
            self.current / self.max
        } else {
            0.0
        }
    }
}

/// 玩家名称组件
///
/// 玩家的显示名称。
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct PlayerName {
    /// 名称
    pub name: String,
}

impl PlayerName {
    /// 创建新的玩家名称
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }

    /// 获取名称长度
    pub fn len(&self) -> usize {
        self.name.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }
}

impl From<String> for PlayerName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl From<&str> for PlayerName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

/// 定时器组件
///
/// 用于定时触发事件。
#[derive(Component, Debug, Clone)]
pub struct GameTimer {
    /// 定时器时长
    pub duration: std::time::Duration,
    /// 已经过的时间
    pub elapsed: std::time::Duration,
    /// 是否重复
    pub repeating: bool,
}

impl GameTimer {
    /// 创建新的定时器
    pub fn new(duration: std::time::Duration, repeating: bool) -> Self {
        Self {
            duration,
            elapsed: std::time::Duration::ZERO,
            repeating,
        }
    }

    /// 创建一次性定时器
    pub fn once(duration: std::time::Duration) -> Self {
        Self::new(duration, false)
    }

    /// 创建重复定时器
    pub fn repeating(duration: std::time::Duration) -> Self {
        Self::new(duration, true)
    }

    /// 更新定时器
    ///
    /// 返回是否触发。
    pub fn tick(&mut self, delta: std::time::Duration) -> bool {
        self.elapsed += delta;

        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed = std::time::Duration::ZERO;
            }
            return true;
        }

        false
    }

    /// 重置定时器
    pub fn reset(&mut self) {
        self.elapsed = std::time::Duration::ZERO;
    }

    /// 获取进度（0.0 到 1.0）
    pub fn progress(&self) -> f32 {
        if self.duration.as_secs_f32() > 0.0 {
            (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
        } else {
            1.0
        }
    }

    /// 是否完成
    pub fn finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// 获取剩余时间
    pub fn remaining(&self) -> std::time::Duration {
        if self.elapsed >= self.duration {
            std::time::Duration::ZERO
        } else {
            self.duration - self.elapsed
        }
    }
}

// 导出 Timer 别名以保持 API 兼容性
pub use GameTimer as Timer;

/// 标签组件
///
/// 用于对实体进行分类和查询。
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Label {
    /// 标签值
    pub value: String,
}

impl Label {
    /// 创建新标签
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl From<String> for Label {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Label {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_connection() {
        let conn_id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();
        let mut player = PlayerConnection::new(conn_id, addr);

        assert_eq!(player.connection_id, conn_id);
        assert_eq!(player.address, addr);

        std::thread::sleep(std::time::Duration::from_millis(10));
        player.update_activity();

        assert!(player.idle_time() < std::time::Duration::from_millis(10));
    }

    #[test]
    fn test_position() {
        let pos1 = Position::new(1.0, 2.0, 3.0);
        let pos2 = Position::new(4.0, 6.0, 8.0);

        assert_eq!(pos1.x, 1.0);
        assert_eq!(pos1.y, 2.0);
        assert_eq!(pos1.z, 3.0);

        let distance = pos1.distance_to(&pos2);
        assert!((distance - 7.071).abs() < 0.01); // sqrt(27) ≈ 5.196
    }

    #[test]
    fn test_health() {
        let mut health = Health::full(100.0);
        assert_eq!(health.current, 100.0);
        assert_eq!(health.max, 100.0);
        assert!(health.is_full());

        health.damage(30.0);
        assert_eq!(health.current, 70.0);
        assert!(!health.is_full());
        assert!(!health.is_dead());

        health.heal(20.0);
        assert_eq!(health.current, 90.0);

        health.damage(100.0);
        assert!(health.is_dead());
    }

    #[test]
    fn test_timer() {
        let mut timer = GameTimer::once(std::time::Duration::from_millis(100));

        assert!(!timer.finished());
        assert_eq!(timer.progress(), 0.0);

        let triggered = timer.tick(std::time::Duration::from_millis(50));
        assert!(!triggered);
        assert!((timer.progress() - 0.5) < 0.01);

        let triggered = timer.tick(std::time::Duration::from_millis(60));
        assert!(triggered);
        assert!(timer.finished());
    }

    #[test]
    fn test_repeating_timer() {
        let mut timer = GameTimer::repeating(std::time::Duration::from_millis(100));

        // 第一次触发
        let triggered = timer.tick(std::time::Duration::from_millis(100));
        assert!(triggered);
        assert!(!timer.finished()); // 重复定时器重置
        assert_eq!(timer.elapsed, std::time::Duration::ZERO);
    }
}
