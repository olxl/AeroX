//! Client events

use std::net::SocketAddr;

/// Client event
#[derive(Debug, Clone)]
pub enum ClientEvent {
    /// Connected to server
    Connected { addr: SocketAddr },

    /// Disconnected from server
    Disconnected { reason: String },

    /// Message received
    MessageReceived { msg_id: u16 },

    /// Message sent
    MessageSent { msg_id: u16 },

    /// Error occurred
    Error { error: String },

    /// Reconnecting to server
    Reconnecting { attempt: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_event_connected() {
        let addr = "127.0.0.1:8080".parse().unwrap();
        let event = ClientEvent::Connected { addr };
        match event {
            ClientEvent::Connected { addr: a } => {
                assert_eq!(a, addr);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_client_event_disconnected() {
        let event = ClientEvent::Disconnected {
            reason: "Connection lost".to_string(),
        };
        match event {
            ClientEvent::Disconnected { reason } => {
                assert_eq!(reason, "Connection lost");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
