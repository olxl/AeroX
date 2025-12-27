//! # AeroX Client
//!
//! Client library for connecting to AeroX servers.
//!
//! ## Features
//!
//! - Async message send/receive
//! - Protobuf message support
//! - Stream API (low-level)
//! - High-level API (automatic message handling)
//! - Optional auto-reconnect
//!
//! ## Quick Start
//!
//! ### Stream API
//!
//! ```rust,no_run,ignore
//! use aerox_client::StreamClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = StreamClient::connect("127.0.0.1:8080".parse()?).await?;
//!
//!     // Send message (requires a prost::Message type)
//!     // client.send_message(1001, &my_message).await?;
//!
//!     // Receive message
//!     // let (msg_id, response) = client.recv_message::<MyMessage>().await?;
//!
//!     client.close().await?;
//!     Ok(())
//! }
//! ```
//!
//! ### High-level API
//!
//! ```rust,no_run,ignore
//! use aerox_client::HighLevelClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = HighLevelClient::connect("127.0.0.1:8080".parse()?).await?;
//!
//!     // Register message handler (requires prost::Message types)
//!     // client.on_message(1001, |id, msg: MyMessage| async move {
//!     //     println!("Received: {:?}", msg);
//!     //     Ok(())
//!     // }).await?;
//!
//!     // Send message
//!     // client.send(1001, &my_message).await?;
//!
//!     client.shutdown().await?;
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod connection;
pub mod error;

// Stream API
pub mod stream;

// High-level API
pub mod high_level;

// Re-export main types
pub use crate::config::ClientConfig;
pub use crate::connection::{ClientConnection, ClientState};
pub use crate::error::{ClientError, Result};

// Re-export Stream API
pub use crate::stream::StreamClient;

// Re-export High-level API
pub use crate::high_level::{HighLevelClient, ClientEvent};

// Prelude module for common imports
pub mod prelude {
    pub use crate::config::ClientConfig;
    pub use crate::connection::{ClientConnection, ClientState};
    pub use crate::error::{ClientError, Result};
    pub use crate::high_level::{HighLevelClient, ClientEvent};
    pub use crate::stream::StreamClient;
}
