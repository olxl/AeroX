//! AeroX Quick Start Example
//!
//! Demonstrates basic server-client communication using the high-level API.
//!
//! Run with:
//! ```bash
//! cargo run --example start
//! ```
//!
//! Note: This is a demonstration of the API structure. The actual server
//! implementation in App::run() is still under development.

use std::time::Duration;
use tokio::time::sleep;
use prost::Message;

// Simple protobuf messages for demonstration
#[derive(Clone, prost::Message)]
struct PingRequest {
    #[prost(uint64, tag = "1")]
    timestamp: u64,
    #[prost(string, tag = "2")]
    message: String,
}

#[derive(Clone, prost::Message)]
struct PongResponse {
    #[prost(uint64, tag = "1")]
    request_timestamp: u64,
    #[prost(uint64, tag = "2")]
    server_timestamp: u64,
    #[prost(string, tag = "3")]
    message: String,
}

// Message IDs
const MSG_ID_PING: u16 = 1001;
const MSG_ID_PONG: u16 = 1002;

#[tokio::main]
async fn main() -> aerox::Result<()> {
    println!("=== AeroX Quick Start Example ===\n");

    // Spawn server in background task
    let server_handle = tokio::spawn(run_server());

    // Give server time to start
    sleep(Duration::from_millis(100)).await;

    // Run client
    println!("🔌 Starting client...\n");
    if let Err(e) = run_client().await {
        eprintln!("❌ Client error: {}", e);
    }

    // Wait for server to finish
    match server_handle.await {
        Ok(result) => {
            if let Err(e) = result {
                eprintln!("❌ Server error: {}", e);
            }
        }
        Err(e) => eprintln!("❌ Server task error: {}", e),
    }

    println!("\n✓ Example complete!");
    Ok(())
}

/// Server implementation using the high-level Server API
async fn run_server() -> aerox::Result<()> {
    println!("🚀 Starting server on 127.0.0.1:8080...\n");

    // Note: This demonstrates the API structure.
    // The actual server implementation is still under development.
    let result = aerox::Server::bind("127.0.0.1:8080")
        .route(MSG_ID_PING, |ctx| {
            Box::pin(async move {
                // Decode the ping request
                match PingRequest::decode(ctx.data().clone()) {
                    Ok(ping) => {
                        println!("📥 Server received: {}", ping.message);
                        println!("   From: {}", ctx.peer_addr());
                        println!("   Msg ID: {}", ctx.message_id());

                        // Create pong response
                        let pong = PongResponse {
                            request_timestamp: ping.timestamp,
                            server_timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            message: format!("Echo: {}", ping.message),
                        };

                        // Encode and respond (encode_to_vec returns Vec<u8>, not Result)
                        let pong_bytes = prost::Message::encode_to_vec(&pong);
                        match ctx.respond(MSG_ID_PONG, pong_bytes.into()).await {
                            Ok(_) => {
                                println!("📤 Server sent pong response");
                            }
                            Err(e) => {
                                eprintln!("⚠️  Failed to send response: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to decode ping request: {}", e);
                    }
                }
                println!();
                Ok(())
            })
        })
        .run()
        .await;

    // The App::run() implementation is currently a stub
    // In production, this will start the actual TCP reactor
    match result {
        Ok(_) => println!("✅ Server started successfully"),
        Err(e) => println!("⚠️  Server start result: {}", e),
    }

    Ok(())
}

/// Client implementation using the high-level Client API
async fn run_client() -> aerox::Result<()> {
    println!("🔌 Connecting to server at 127.0.0.1:8080...");

    // Create client and connect
    let mut client = match aerox::Client::connect("127.0.0.1:8080").await {
        Ok(c) => {
            println!("✓ Connected!\n");
            c
        }
        Err(e) => {
            println!("ℹ️  Connection attempt result: {}", e);
            println!("   (This is expected since App::run() is a stub)\n");
            return Err(e);
        }
    };

    // Register response handler
    client.on_message(MSG_ID_PONG, |_msg_id, pong: PongResponse| {
        Box::pin(async move {
            println!("📥 Client received response:");
            println!("   Request timestamp: {}", pong.request_timestamp);
            println!("   Server timestamp: {}", pong.server_timestamp);
            println!("   Message: {}", pong.message);
            Ok(())
        })
    }).await?;

    // Create ping message
    let ping = PingRequest {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        message: "Hello, AeroX!".to_string(),
    };

    // Send ping
    println!("📤 Client sending: {}", ping.message);
    match client.send(MSG_ID_PING, &ping).await {
        Ok(_) => println!("✓ Message sent successfully\n"),
        Err(e) => {
            println!("ℹ️  Send result: {}\n", e);
            return Err(e.into());
        }
    }

    // Wait for response
    println!("⏳ Waiting for server response...");
    sleep(Duration::from_millis(500)).await;

    println!("✓ Client completed!");

    Ok(())
}
