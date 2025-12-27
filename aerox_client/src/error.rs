//! Client-specific error types

use aerox_core::AeroXError;
use thiserror::Error;

/// Client-specific errors
#[derive(Error, Debug)]
pub enum ClientError {
    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Not connected to server
    #[error("Not connected")]
    NotConnected,

    /// Send operation failed
    #[error("Send failed: {0}")]
    SendFailed(String),

    /// Receive operation failed
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),

    /// Handler error
    #[error("Handler error for message {0}: {1}")]
    HandlerError(u16, String),

    /// Reconnect exhausted
    #[error("Reconnect exhausted after {0} attempts")]
    ReconnectExhausted(usize),

    /// Timeout
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

impl From<ClientError> for AeroXError {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::ConnectionFailed(msg) => AeroXError::connection(msg),
            ClientError::NotConnected => AeroXError::connection("Not connected"),
            ClientError::SendFailed(msg) => AeroXError::network(msg),
            ClientError::ReceiveFailed(msg) => AeroXError::network(msg),
            ClientError::HandlerError(id, msg) => {
                AeroXError::plugin(format!("Handler {} error: {}", id, msg))
            }
            ClientError::ReconnectExhausted(n) => {
                AeroXError::connection(format!("Reconnect failed after {} attempts", n))
            }
            ClientError::Timeout(_msg) => AeroXError::timeout(),
            ClientError::InvalidConfig(msg) => AeroXError::config(msg),
        }
    }
}

/// Client result type
pub type Result<T> = std::result::Result<T, ClientError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let err = ClientError::NotConnected;
        let aerox_err: AeroXError = err.into();
        assert!(matches!(
            aerox_err.kind(),
            aerox_core::AeroXErrorKind::Connection
        ));
    }

    #[test]
    fn test_error_display() {
        let err = ClientError::ConnectionFailed("test".to_string());
        assert_eq!(err.to_string(), "Connection failed: test");
    }
}
