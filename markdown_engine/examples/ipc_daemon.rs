use markdown_engine::ipc::{start_http_server, IpcState};
use std::sync::Arc;
use tokio::sync::broadcast;

#[cfg(unix)]
use markdown_engine::ipc::start_uds_server;

#[tokio::main]
async fn main() {
    let port = 3030;
    let auth_token = std::env::var("SIMPLE_UI_IPC_TOKEN").unwrap_or_else(|_| {
        eprintln!("simple-ui: set SIMPLE_UI_IPC_TOKEN (no default token)");
        std::process::exit(2);
    });
    if auth_token.is_empty() {
        eprintln!("simple-ui: SIMPLE_UI_IPC_TOKEN must be non-empty");
        std::process::exit(2);
    }
    let socket_path = "/tmp/extended_markdown.sock";

    let (tx, _) = broadcast::channel(100);
    let state = Arc::new(IpcState { tx, auth_token });

    println!("Booting IPC Daemon...");

    // Spawn HTTP Server
    let http_state = state.clone();
    let http_handle = tokio::spawn(async move {
        start_http_server(port, http_state).await;
    });

    // Spawn UDS Server (Unix Only)
    #[cfg(unix)]
    let uds_handle = {
        let uds_state = state.clone();
        tokio::spawn(async move {
            start_uds_server(socket_path, uds_state).await;
        })
    };

    #[cfg(unix)]
    let _ = tokio::try_join!(http_handle, uds_handle);

    #[cfg(not(unix))]
    let _ = http_handle.await;
}
