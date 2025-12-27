//! High-level API
//!
//! Provides automatic message handling with background receiver task.

mod client;
mod event;
mod handler;

pub use client::HighLevelClient;
pub use event::ClientEvent;
pub use handler::{FnHandler, HandlerRegistry, MessageHandler};
