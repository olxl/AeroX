//! AeroX 集成测试
//!
//! 测试各个模块之间的集成功能。

// 配置系统集成测试
#[cfg(test)]
mod config_tests {
    use aerox_config::ServerConfig;
    use std::env;

    #[test]
    fn test_config_default_and_validation() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_config_env_override() {
        env::set_var("AEROX_PORT", "9999");
        let config = ServerConfig::default()
            .load_with_env_override()
            .unwrap();
        assert_eq!(config.port, 9999);
        env::remove_var("AEROX_PORT");
    }

    #[test]
    fn test_config_summary() {
        let config = ServerConfig::default();
        let summary = config.summary();
        assert!(summary.contains("8080"));
    }
}

// 网络层集成测试
#[cfg(test)]
mod network_tests {
    use aerox_network::{ConnectionId, Frame};

    #[test]
    fn test_connection_id_generation() {
        let id1 = ConnectionId::new(1);
        let id2 = ConnectionId::new(2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_frame_creation() {
        let frame = Frame::new(1, 100, bytes::Bytes::from("hello"));
        assert_eq!(frame.message_id, 1);
        assert_eq!(frame.sequence_id, 100);
        assert_eq!(frame.body, bytes::Bytes::from("hello"));
    }
}

// ECS 集成测试
#[cfg(test)]
mod ecs_tests {
    use aerox_ecs::*;
    use aerox_network::ConnectionId;
    use std::net::SocketAddr;

    #[test]
    fn test_ecs_world_creation() {
        let mut world = EcsWorld::new();
        assert!(world.initialize().is_ok());
    }

    #[test]
    fn test_network_bridge() {
        let mut world = EcsWorld::new();
        world.initialize().unwrap();

        let bridge = NetworkBridge::new();
        let conn_id = ConnectionId::new(1);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        bridge.on_connected(&mut world, conn_id, addr);

        assert_eq!(world.metrics().events_processed, 1);
    }

    #[test]
    fn test_ecs_components() {
        let mut world = EcsWorld::new();
        world.initialize().unwrap();

        // 创建玩家实体
        let entity = world.spawn_bundle((
            components::PlayerConnection::new(
                ConnectionId::new(1),
                "127.0.0.1:8080".parse().unwrap()
            ),
            components::Position::origin(),
            components::Health::full(100.0),
        )).id();

        // 验证实体存在
        assert!(world.world().get_entity(entity).is_some());
    }

    #[test]
    fn test_health_component() {
        let mut health = components::Health::full(100.0);
        assert_eq!(health.current, 100.0);
        assert!(!health.is_dead());

        health.damage(30.0);
        assert_eq!(health.current, 70.0);

        health.heal(20.0);
        assert_eq!(health.current, 90.0);

        health.damage(100.0);
        assert!(health.is_dead());
    }

    #[test]
    fn test_timer_component() {
        let mut timer = components::GameTimer::once(std::time::Duration::from_millis(100));
        assert!(!timer.finished());

        let triggered = timer.tick(std::time::Duration::from_millis(50));
        assert!(!triggered);
        assert!(!timer.finished());

        let triggered = timer.tick(std::time::Duration::from_millis(60));
        assert!(triggered);
        assert!(timer.finished());
    }

    #[test]
    fn test_position_component() {
        let pos1 = components::Position::new(1.0, 2.0, 3.0);
        let pos2 = components::Position::new(4.0, 6.0, 8.0);

        let distance = pos1.distance_to(&pos2);
        assert!(distance > 0.0);
        assert!((distance - 7.071).abs() < 0.01);
    }
}

// 插件系统集成测试
#[cfg(test)]
mod plugin_tests {
    use aerox_core::app::App;
    use aerox_core::plugin::Plugin;

    // 简单测试插件
    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn build(&self) {}
        fn name(&self) -> &'static str {
            "test_plugin"
        }
    }

    #[test]
    fn test_plugin_registration() {
        let app = App::new()
            .add_plugin(TestPlugin)
            .insert_state(42i32);

        let built_app = app.build().unwrap();
        assert_eq!(built_app.state().get::<i32>(), Some(&42));
    }

    #[test]
    fn test_plugin_dependencies() {
        struct PluginA;
        struct PluginB;

        impl Plugin for PluginA {
            fn build(&self) {}
            fn name(&self) -> &'static str {
                "plugin_a"
            }
        }

        impl Plugin for PluginB {
            fn build(&self) {}
            fn name(&self) -> &'static str {
                "plugin_b"
            }

            fn dependencies(&self) -> &'static [&'static str] {
                &["plugin_a"]
            }
        }

        let app = App::new()
            .add_plugin(PluginB)
            .add_plugin(PluginA);

        assert!(app.build().is_ok());
    }
}

// 错误处理集成测试
#[cfg(test)]
mod error_tests {
    use aerox_core::AeroXError;

    #[test]
    fn test_error_display() {
        let err = AeroXError::config("test error");
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let aerox_err: AeroXError = io_err.into();
        // 验证错误转换成功
        assert!(aerox_err.to_string().contains("file not found"));
    }
}

// 路由系统集成测试
#[cfg(test)]
mod router_tests {
    #[test]
    fn test_router_creation() {
        // 基本创建测试
        let _router = aerox_router::router::Router::new();
        // Router 创建成功即可
    }
}
