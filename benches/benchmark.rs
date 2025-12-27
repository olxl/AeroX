//! AeroX 性能基准测试
//!
//! 测试各个模块的性能指标。

#![cfg(feature = "benchmark")]

use std::time::Duration;
use aerox_network::{ConnectionId, Frame};
use aerox_protobuf::MessageRegistry;
use aerox_router::*;
use aerox_ecs::*;

/// 基准测试辅助宏
macro_rules! bench {
    ($name:expr, $code:block) => {
        let start = std::time::Instant::now();
        let iterations = 10000;
        for _ in 0..iterations {
            $code
        }
        let duration = start.elapsed();
        let avg_ns = duration.as_nanos() / iterations as u128;
        println!("  {:30}: {:>8} ns/op ({} ops in {:?})",
            $name, avg_ns, iterations, duration);
    };
}

fn main() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   AeroX 性能基准测试");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    bench_connection_id();
    bench_frame_operations();
    bench_message_encoding();
    bench_router_dispatch();
    bench_ecs_operations();

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   基准测试完成");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

/// 测试 ConnectionId 生成性能
fn bench_connection_id() {
    println!("\n📊 ConnectionId 基准测试:");

    bench!("new()", {
        let _id = ConnectionId::new(1);
    });

    bench!("clone()", {
        let id = ConnectionId::new(1);
        let _id2 = id.clone();
    });

    bench!("eq()", {
        let id1 = ConnectionId::new(1);
        let id2 = ConnectionId::new(1);
        let _eq = id1 == id2;
    });
}

/// 测试 Frame 操作性能
fn bench_frame_operations() {
    println!("\n📊 Frame 操作基准测试:");

    bench!("Frame::new()", {
        let frame = Frame::new(1, 100, bytes::Bytes::from("hello world"));
        let _ = frame;
    });

    bench!("Frame::serialize()", {
        let frame = Frame::new(1, 100, bytes::Bytes::from("hello world"));
        let _data = frame.serialize();
    });

    bench!("Frame::deserialize()", {
        let frame = Frame::new(1, 100, bytes::Bytes::from("hello world"));
        let data = frame.serialize();
        let _frame2 = Frame::deserialize(&mut data.as_ref());
    });
}

/// 测试消息编解码性能
fn bench_message_encoding() {
    println!("\n📊 消息编解码基准测试:");

    let registry = MessageRegistry::new();
    let payload = bytes::Bytes::from("test message payload");

    bench!("wrap_message", {
        let _wrapped = registry.wrap_message(1, 100, payload.clone());
    });

    let wrapped = registry.wrap_message(1, 100, payload);

    bench!("unwrap_message", {
        let _result = registry.unwrap_message(&wrapped);
    });

    bench!("encode_message", {
        let _encoded = registry.encode_message(1, 100, &wrapped.payload);
    });
}

/// 测试路由分发性能
fn bench_router_dispatch() {
    println!("\n📊 路由分发基准测试:");

    let mut router = Router::new();

    // 注册一些处理器
    for i in 1..=10 {
        let msg_id = i;
        router.register(msg_id, move |ctx: Context| {
            Box::pin(async move {
                // 模拟处理
                Ok(ctx)
            })
        });
    }

    let conn_id = ConnectionId::new(1);
    let payload = bytes::Bytes::from("test");
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    bench!("route_message", {
        let ctx = Context::new(conn_id, payload.clone(), std::collections::HashMap::new());
        let _ = rt.block_on(router.route_message(ctx, 1));
    });
}

/// 测试 ECS 操作性能
fn bench_ecs_operations() {
    println!("\n📊 ECS 操作基准测试:");

    let mut world = EcsWorld::new();
    world.initialize().unwrap();

    bench!("EcsWorld::spawn()", {
        let entity = world.spawn();
        let _ = entity.id();
    });

    bench!("EcsWorld::spawn_bundle", {
        use aerox_ecs::components::*;
        let entity = world.spawn_bundle((
            Position::origin(),
            Health::full(100.0),
        ));
        let _ = entity.id();
    });

    bench!("send_event", {
        use aerox_ecs::events::*;
        let event = ConnectionEstablishedEvent {
            connection_id: ConnectionId::new(1),
            address: "127.0.0.1:8080".parse().unwrap(),
            timestamp: std::time::Instant::now(),
        };
        world.send_event(event);
    });

    bench!("NetworkBridge::on_connected", {
        use aerox_ecs::bridge::*;
        let bridge = NetworkBridge::new();
        let conn_id = ConnectionId::new(1);
        let addr: std::net::SocketAddr = "127.0.0.1:8080".parse().unwrap();
        bridge.on_connected(&mut world, conn_id, addr);
    });
}

/// 内存使用基准
fn bench_memory_usage() {
    println!("\n📊 内存使用基准:");

    // ConnectionId
    let ids: Vec<ConnectionId> = (0..10000)
        .map(|i| ConnectionId::new(i))
        .collect();
    let size = std::mem::size_of_val(&ids[..]);
    println!("  {:30}: {:>8} bytes (10,000 IDs)",
        "ConnectionId vec", size);

    // Frame
    let frames: Vec<Frame> = (0..1000)
        .map(|i| Frame::new(i, i, bytes::Bytes::from("test")))
        .collect();
    let size = std::mem::size_of_val(&frames[..]);
    println!("  {:30}: {:>8} bytes (1,000 Frames)",
        "Frame vec", size);
}

/// 并发性能测试
#[tokio::main]
async fn bench_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 并发操作基准:");

    let start = std::time::Instant::now();
    let mut handles = vec![];

    // 生成 1000 个连接
    for i in 0..1000 {
        let handle = tokio::spawn(async move {
            let _id = ConnectionId::new(i);
            // 模拟一些工作
            tokio::time::sleep(Duration::from_micros(100)).await;
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await?;
    }

    let duration = start.elapsed();
    println!("  {:30}: {:>8} ops/sec (1000 concurrent)",
        "concurrent_connection_id",
        1000 * 1_000_000_000 / duration.as_nanos() as u64
    );

    Ok(())
}

/// 网络吞吐量测试（模拟）
fn bench_network_throughput() {
    println!("\n📊 网络吞吐量基准:");

    let data_sizes = vec![64, 256, 1024, 4096, 16384];

    for size in data_sizes {
        let payload = bytes::Bytes::from(vec![0u8; size]);
        let frame = Frame::new(1, 100, payload.clone());

        let start = std::time::Instant::now();
        let iterations = 10000;

        for _ in 0..iterations {
            let serialized = frame.serialize();
            let _deserialized = Frame::deserialize(&mut serialized.as_ref());
        }

        let duration = start.elapsed();
        let total_bytes = size * iterations;
        let throughput = (total_bytes as f64 / duration.as_secs_f64()) / 1024.0 / 1024.0;

        println!("  {:30}: {:>8.2} MB/s ({} byte messages)",
            "serialize+deserialize",
            throughput,
            size
        );
    }
}

/// ECS 组件基准
fn bench_ecs_components() {
    println!("\n📊 ECS 组件基准:");

    use aerox_ecs::components::*;

    bench!("Position::new", {
        let _pos = Position::new(1.0, 2.0, 3.0);
    });

    bench!("Position::distance_to", {
        let pos1 = Position::new(1.0, 2.0, 3.0);
        let pos2 = Position::new(4.0, 6.0, 8.0);
        let _dist = pos1.distance_to(&pos2);
    });

    bench!("Health::damage", {
        let mut health = Health::full(100.0);
        health.damage(10.0);
    });

    bench!("Health::heal", {
        let mut health = Health::new(100.0);
        health.heal(10.0);
    });

    bench!("GameTimer::tick", {
        let mut timer = GameTimer::once(Duration::from_millis(100));
        timer.tick(Duration::from_millis(50));
    });
}

/// 运行完整的基准测试套件
#[tokio::main]
async fn run_full_suite() -> Result<(), Box<dyn std::error::Error>> {
    main();
    bench_memory_usage();
    bench_network_throughput();
    bench_ecs_components();
    bench_concurrent_operations().await?;
    Ok(())
}
