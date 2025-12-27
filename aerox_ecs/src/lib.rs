//! AeroX Bevy ECS 整合层
//!
//! 提供网络事件到 ECS 事件的转换和 Tick 调度系统。

pub mod world;
pub mod events;

// 预导出
pub mod prelude {
    // pub use crate::world::EcsWorld;  // 后续实现
}
