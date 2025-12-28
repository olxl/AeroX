//! 连接 ID 和连接结构
//!
//! 重新导出 aerox_core 中的连接类型，保持向后兼容。

pub use aerox_core::{Connection, ConnectionId, ConnectionIdGenerator, ConnectionState};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_id() {
        let id1 = ConnectionId::new(1);
        let id2 = ConnectionId::new(2);
        assert_ne!(id1, id2);
        assert_eq!(id1.value(), 1);
    }

    #[test]
    fn test_id_generator() {
        let generator = ConnectionIdGenerator::new();
        let id1 = generator.next();
        let id2 = generator.next();
        assert_eq!(id1.value(), 1);
        assert_eq!(id2.value(), 2);
    }

    #[test]
    fn test_connection_age() {
        let id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();
        let conn = Connection::new(id, addr);

        // 刚创建，年龄应该很短
        assert!(conn.age().as_millis() < 100);
    }

    #[test]
    fn test_connection_idle_time() {
        let id = ConnectionId::new(1);
        let addr = "127.0.0.1:8080".parse().unwrap();
        let mut conn = Connection::new(id, addr);

        // 刚创建，空闲时间应该很短
        assert!(conn.idle_time().as_millis() < 100);

        // 更新活跃时间后
        conn.update_active();
        assert!(conn.idle_time().as_millis() < 100);
    }
}
