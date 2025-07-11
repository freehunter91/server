use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
// use sysinfo::{Pid, System, SystemExt}; // Temporarily disabled
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::accept_async;

use crate::http::{Request, Response, StatusCode};

// 서버의 공유 상태를 정의합니다.
struct AppState {
    // sys: System, // Temporarily disabled
    // pid: Pid, // Temporarily disabled
    connected_ips: HashSet<String>,
    network_usage: u64,
}

// 여러 스레드에서 안전하게 공유하기 위한 타입 별칭입니다.
type ServerState = Arc<Mutex<AppState>>;

// 클라이언트에게 보낼 상태 데이터 구조체입니다.
#[derive(Serialize)]
struct StatusPayload {
    cpu_usage: f32,
    memory_usage: u64,
    network_usage: u64,
    connected_ips: Vec<String>,
}

// 서버 실행의 메인 함수입니다.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);

    let state = Arc::new(Mutex::new(AppState {
        // sys: System::new(), // Temporarily disabled
        // pid: sysinfo::get_current_pid().unwrap(), // Temporarily disabled
        connected_ips: HashSet::new(),
        network_usage: 0,
    }));

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr, state.clone()));
    }

    Ok(())
}

async fn handle_connection(mut stream: TcpStream, addr: SocketAddr, state: ServerState) {
    let mut buffer = [0; 1024];
    if stream.peek(&mut buffer).await.unwrap_or(0) == 0 {
        return; 
    }

    if let Ok(request) = Request::try_from(&buffer[..]) {
        {
            let mut state_lock = state.lock().unwrap();
            state_lock.connected_ips.insert(addr.ip().to_string());
        }

        match request.path() {
            "/ws" => {
                if let Ok(ws_stream) = accept_async(stream).await {
                    tokio::spawn(handle_websocket_stream(ws_stream, state));
                }
            }
            path => {
                let (response, bytes_sent) = serve_static_file(path).await;
                if response.write_to(&mut stream).await.is_ok() {
                    let mut state_lock = state.lock().unwrap();
                    state_lock.network_usage += bytes_sent;
                }
            }
        }
    } else {
        let response = Response::new(StatusCode::BadRequest, None);
        response.write_to(&mut stream).await.ok();
    }
}

async fn serve_static_file(path: &str) -> (Response, u64) {
    let filepath = match path {
        "/" => PathBuf::from("dashboard_assets/dashboard.html"),
        p if p.starts_with("/static/") => {
            PathBuf::from("static").join(p.strip_prefix("/static/").unwrap())
        }
        "/main.js" => PathBuf::from("dashboard_assets/main.js"),
        "/style.css" => PathBuf::from("dashboard_assets/style.css"),
        _ => return (Response::new(StatusCode::NotFound, Some(b"Not Found".to_vec())), 0),
    };

    match tokio::fs::read(filepath).await {
        Ok(content) => {
            let len = content.len() as u64;
            (Response::new(StatusCode::Ok, Some(content)), len)
        }
        Err(_) => (Response::new(StatusCode::NotFound, Some(b"Not Found".to_vec())), 0),
    }
}

async fn handle_websocket_stream(
    mut ws_stream: tokio_tungstenite::WebSocketStream<TcpStream>,
    state: ServerState,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let payload_json = {
                    let state_lock = state.lock().unwrap();
                    
                    // Send dummy data for now
                    let payload = StatusPayload {
                        cpu_usage: 0.0,
                        memory_usage: 0,
                        network_usage: state_lock.network_usage,
                        connected_ips: state_lock.connected_ips.iter().cloned().collect(),
                    };
                    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
                };

                if ws_stream.send(Message::Text(payload_json.into())).await.is_err() {
                    break;
                }
            }
            maybe_msg = ws_stream.next() => {
                if let Some(Ok(Message::Close(_))) = maybe_msg {
                    break;
                }
            }
        }
    }
}