//! ECS World 包装
//!
//! 提供 Bevy ECS World 的包装和扩展功能。

use bevy::prelude::*;

/// AeroX ECS World 包装器
///
/// 提供对 Bevy World 的扩展功能，包括资源管理、系统调度等。
#[derive(Debug)]
pub struct EcsWorld {
    /// Bevy ECS World
    world: World,
    /// 是否已初始化
    initialized: bool,
}

impl Default for EcsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl EcsWorld {
    /// 创建新的 ECS World
    pub fn new() -> Self {
        let mut world = World::new();
        // 注册基础资源
        world.insert_resource(EcsMetrics::default());

        Self {
            world,
            initialized: false,
        }
    }

    /// 初始化 World
    ///
    /// 注册所有必要的组件和资源。
    pub fn initialize(&mut self) -> aerox_core::Result<()> {
        if self.initialized {
            return Ok(());
        }

        // 注册基础组件
        self.register_components();

        self.initialized = true;
        Ok(())
    }

    /// 注册组件类型
    fn register_components(&mut self) {
        // Bevy 0.14 会自动注册组件，这里主要是预留扩展点
        // 未来可能需要手动注册某些反射类型
    }

    /// 获取底层 World 的引用
    pub fn world(&self) -> &World {
        &self.world
    }

    /// 获取底层 World 的可变引用
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// 发送事件到 ECS
    ///
    /// 将事件发送到 World 的事件队列中。
    pub fn send_event<E: Event>(&mut self, event: E) {
        self.world.send_event(event);
    }

    /// 批量发送事件
    pub fn send_events<E: Event>(&mut self, events: Vec<E>) {
        for event in events {
            self.world.send_event(event);
        }
    }

    /// 添加资源
    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.world.insert_resource(resource);
    }

    /// 获取资源
    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.world.get_resource()
    }

    /// 获取可变资源
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<Mut<R>> {
        self.world.get_resource_mut()
    }

    /// 生成实体
    pub fn spawn(&mut self) -> EntityWorldMut {
        self.world.spawn_empty()
    }

    /// 生成实体并带组件
    pub fn spawn_bundle(&mut self, bundle: impl Bundle) -> EntityWorldMut {
        self.world.spawn(bundle)
    }

    /// 获取 ECS 指标
    pub fn metrics(&self) -> &EcsMetrics {
        self.world.get_resource::<EcsMetrics>()
            .expect("EcsMetrics should always exist")
    }

    /// 获取可变的 ECS 指标
    pub fn metrics_mut(&mut self) -> Mut<EcsMetrics> {
        self.world.get_resource_mut::<EcsMetrics>()
            .expect("EcsMetrics should always exist")
    }
}

/// ECS 指标
///
/// 跟踪 ECS 系统的各项指标。
#[derive(Debug, Clone, Resource)]
pub struct EcsMetrics {
    /// 实体总数
    pub entity_count: usize,
    /// 系统运行次数
    pub system_runs: u64,
    /// 事件处理数量
    pub events_processed: u64,
    /// 最后更新时间
    pub last_update: std::time::Instant,
}

impl Default for EcsMetrics {
    fn default() -> Self {
        Self {
            entity_count: 0,
            system_runs: 0,
            events_processed: 0,
            last_update: std::time::Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = EcsWorld::new();
        assert!(!world.initialized);
    }

    #[test]
    fn test_world_initialize() {
        let mut world = EcsWorld::new();
        assert!(world.initialize().is_ok());
        assert!(world.initialized);
    }

    #[test]
    fn test_resource_management() {
        let mut world = EcsWorld::new();
        world.initialize().unwrap();

        // 添加自定义资源
        #[derive(Debug, Clone, Resource)]
        struct TestResource {
            value: i32,
        }

        world.insert_resource(TestResource { value: 42 });

        let resource = world.get_resource::<TestResource>();
        assert!(resource.is_some());
        assert_eq!(resource.unwrap().value, 42);
    }

    #[test]
    fn test_entity_spawn() {
        let mut world = EcsWorld::new();
        world.initialize().unwrap();

        // 定义一个简单组件
        #[derive(Component)]
        struct Position {
            x: f32,
            y: f32,
        }

        let entity = world.spawn_bundle(Position { x: 1.0, y: 2.0 }).id();

        // 验证实体存在
        assert!(world.world().get_entity(entity).is_some());
    }

    #[test]
    fn test_metrics() {
        let world = EcsWorld::new();
        let metrics = world.metrics();
        assert_eq!(metrics.entity_count, 0);
        assert_eq!(metrics.system_runs, 0);
        assert_eq!(metrics.events_processed, 0);
    }
}
