// examples/minimal_client.rs
//
// This is a minimal Rust client demonstrating how to connect to the
// Extended Markdown IPC daemon. It uses reqwest to push Markdown updates
// and listen to Server-Sent Events (SSE) for UI interactions.
//
// Dependencies needed in Cargo.toml:
// reqwest = { version = "0.11", features = ["stream"] }
// tokio = { version = "1", features = ["full"] }
// futures-util = "0.3"
// serde_json = "1.0"

use futures_util::StreamExt;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

const IPC_URL: &str = "http://127.0.0.1:3030";
const SEC_TOKEN: &str = "super_secret_token_123";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    println!("Starting Rust Minimal Client...");

    // 1. Spawn a task to push a dynamic Markdown update
    let push_client = client.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(2)).await;
        println!("\n[IPC PUSH] Sending dynamic markdown update...");

        let markdown_content = "# Live Rust Update\n\nThis content was injected by the minimal Rust client!\n\n```plot\n{\"type\":\"scatter\"}\n```";

        let res = push_client
            .post(format!("{}/update", IPC_URL))
            .header("x-sec-token", SEC_TOKEN)
            .body(markdown_content)
            .send()
            .await;

        match res {
            Ok(r) => println!("[IPC PUSH SUCCESS] Response Status: {}", r.status()),
            Err(e) => eprintln!("[IPC PUSH ERROR] {}", e),
        }
    });

    // 2. Listen to the SSE stream for UI Events
    println!("Connecting to IPC Event Stream...");

    let response = client
        .get(format!("{}/events", IPC_URL))
        .header("x-sec-token", SEC_TOKEN)
        .header("Accept", "text/event-stream")
        .send()
        .await?;

    if !response.status().is_success() {
        eprintln!("Failed to connect to stream: {}", response.status());
        return Ok(());
    }

    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        if let Ok(bytes) = item {
            let chunk = String::from_utf8_lossy(&bytes);

            // Basic SSE chunk parsing
            for line in chunk.lines() {
                if line.starts_with("data: ") {
                    let json_str = line.trim_start_matches("data: ").trim();
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(json_str) {
                        println!("\n[IPC EVENT RECEIVED] 🔔");
                        println!("Type: {}", event["event_type"]);
                        println!("Element: {}", event["element_id"]);
                        println!("Payload: {}", event["payload"]);
                    }
                }
            }
        }
    }

    Ok(())
}
