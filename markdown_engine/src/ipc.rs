use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast;
use warp::Filter;

/// Cap on a single HTTP `/update` body or UDS line. This is a live markdown
/// document, not a file store.
const MAX_IPC_BODY: u64 = 1024 * 1024;

fn token_eq(presented: &str, expected: &str) -> bool {
    let a = presented.as_bytes();
    let b = expected.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiEvent {
    pub event_type: String, // e.g., "input", "submit", "edited"
    pub element_id: String,
    pub payload: serde_json::Value,
}

pub struct IpcState {
    pub tx: broadcast::Sender<UiEvent>,
    pub auth_token: String,
}

pub async fn start_http_server(port: u16, state: Arc<IpcState>) {
    let state_filter = warp::any().map(move || state.clone());

    // Security Token Middleware
    let with_auth = warp::header::header::<String>("x-sec-token")
        .and(state_filter.clone())
        .and_then(|token: String, state: Arc<IpcState>| async move {
            if token_eq(&token, &state.auth_token) {
                Ok::<_, warp::Rejection>(())
            } else {
                Err(warp::reject::custom(Unauthorized))
            }
        });

    // POST /update - Updates the Markdown content dynamically
    let update_route = warp::post()
        .and(warp::path("update"))
        .and(with_auth.clone())
        .and(warp::body::content_length_limit(MAX_IPC_BODY))
        .and(warp::body::bytes())
        .map(|_, bytes: bytes::Bytes| {
            let md_content = String::from_utf8_lossy(&bytes).to_string();
            // Here you would trigger an AST re-parse and signal the UI
            println!("Received MD Update: {} bytes", md_content.len());
            warp::reply::json(&"Update received")
        });

    // GET /events - SSE stream of UI Events
    let events_route = warp::get()
        .and(warp::path("events"))
        .and(with_auth.clone())
        .and(state_filter.clone())
        .map(|_, state: Arc<IpcState>| {
            let rx = state.tx.subscribe();
            let stream = async_stream::stream! {
                let mut rx = rx;
                while let Ok(event) = rx.recv().await {
                    if let Ok(sse_event) = warp::sse::Event::default().json_data(event) {
                        yield Ok::<_, Infallible>(sse_event);
                    }
                }
            };
            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        });

    let routes = update_route.or(events_route);

    println!("Starting HTTP IPC server on 127.0.0.1:{}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

#[cfg(unix)]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::UnixListener;

#[cfg(unix)]
pub async fn start_uds_server(socket_path: &str, state: Arc<IpcState>) {
    // Remove existing socket file if it exists
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path).expect("Failed to bind to UDS");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Some(parent) = std::path::Path::new(socket_path).parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
            }
        }
        let _ = std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o600));
    }
    println!("Starting UNIX Domain Socket server at {}", socket_path);

    loop {
        match listener.accept().await {
            Ok((mut socket, _addr)) => {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    let (reader, mut writer) = socket.split();
                    let mut reader = BufReader::new(reader);
                    let mut line = String::new();
                    const MAX_LINE: usize = MAX_IPC_BODY as usize;

                    let mut rx = state_clone.tx.subscribe();

                    loop {
                        tokio::select! {
                            // Read from client
                            bytes_read = reader.read_line(&mut line) => {
                                match bytes_read {
                                    Ok(0) => break, // EOF, client disconnected
                                    Ok(_) => {
                                        if line.len() > MAX_LINE {
                                            break;
                                        }
                                        let trimmed = line.trim();
                                        if !trimmed.is_empty() {
                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                                // Handle simple authentication and updates
                                                let action = json["action"].as_str().unwrap_or("");
                                                let token = json["token"].as_str().unwrap_or("");

                                                if action == "update" {
                                                    if token_eq(token, &state_clone.auth_token) {
                                                        println!("UDS Received update!");
                                                        let _ = writer.write_all(b"{\"status\": \"ok\"}\n").await;
                                                    } else {
                                                        let _ = writer.write_all(b"{\"status\": \"unauthorized\"}\n").await;
                                                    }
                                                }
                                            }
                                        }
                                        line.clear();
                                    }
                                    Err(_) => break,
                                }
                            }
                            // Broadcast to client
                            event_res = rx.recv() => {
                                if let Ok(event) = event_res {
                                    if let Ok(json_str) = serde_json::to_string(&event) {
                                        let msg = format!("{}\n", json_str);
                                        if writer.write_all(msg.as_bytes()).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }
            Err(e) => println!("accept failed = {:?}", e),
        }
    }
}

#[derive(Debug)]
struct Unauthorized;
impl warp::reject::Reject for Unauthorized {}

#[cfg(test)]
mod tests {
    use super::token_eq;

    #[test]
    fn token_eq_matches_and_rejects() {
        assert!(token_eq("abc", "abc"));
        assert!(!token_eq("abc", "abd"));
        assert!(!token_eq("abc", "ab"));
    }
}
