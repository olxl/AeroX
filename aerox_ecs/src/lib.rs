//! AeroX Bevy ECS 整合层
//!
//! 提供网络事件到 ECS 事件的转换和系统调度。

pub mod bridge;
pub mod components;
pub mod events;
pub mod systems;
pub mod world;

// 导出主要类型到 crate root
pub use crate::bridge::{EventScheduler, NetworkBridge};
pub use crate::components::*;
pub use crate::events::*;
pub use crate::systems::GameSystems;
pub use crate::world::{EcsMetrics, EcsWorld};

// 预导出
pub mod prelude {
    pub use crate::bridge::{EventScheduler, NetworkBridge};
    pub use crate::components::*;
    pub use crate::events::*;
    pub use crate::systems::GameSystems;
    pub use crate::world::{EcsMetrics, EcsWorld};
}
