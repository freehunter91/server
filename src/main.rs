use actix_files as fs;
use actix_web::{get, middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::Message;
use chrono::{NaiveDate, Utc};
use config::Config;
use env_logger::Env;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Duration,
};
use sysinfo::{CpuExt, NetworkExt, Pid, System, SystemExt};
use tokio::time::interval;
use tokio::fs as tokio_fs;

#[derive(Deserialize)]
struct AppConfig {
    host: String,
    port: u16,
    log_level: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct NetworkHistoryEntry {
    date: NaiveDate,
    usage: u64,
}

struct AppState {
    sys: System,
    pid: Pid,
    connected_ips: HashSet<String>,
    network_current_rate: u64, // 실시간 전송률 (Bytes/sec)
    network_history: Vec<NetworkHistoryEntry>, // 일별 평균 전송률
    last_history_update: NaiveDate,
    last_network_totals: HashMap<String, (u64, u64)>, // 인터페이스별 이전 누적 전송량 (received, transmitted)
    current_day_total_bytes: u64, // 현재 날짜의 총 전송 바이트
    current_day_sample_count: u64, // 현재 날짜의 샘플 수 (초)
}

#[derive(Serialize)]
struct StatusPayload {
    cpu_usage: f32,
    memory_usage: u64,
    network_current_rate: u64,
    connected_ips: Vec<String>,
}

type ServerState = Arc<Mutex<AppState>>;

const HISTORY_FILE: &str = "/app/data/network_history.json";
const HISTORY_RETENTION_DAYS: i64 = 30;

async fn load_network_history() -> Vec<NetworkHistoryEntry> {
    match tokio_fs::read_to_string(HISTORY_FILE).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| {
            eprintln!("Failed to parse network history file, starting fresh.");
            Vec::new()
        }),
        Err(_) => {
            eprintln!("Network history file not found, starting fresh.");
            Vec::new()
        }
    }
}

async fn save_network_history(history: &Vec<NetworkHistoryEntry>) {
    let json = serde_json::to_string_pretty(history).unwrap();
    if let Err(e) = tokio_fs::write(HISTORY_FILE, json).await {
        eprintln!("Failed to save network history: {}", e);
    }
}

#[get("/ws")]
async fn ws(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<ServerState>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    let ip = req.peer_addr().map_or_else(
        || "unknown".to_string(),
        |addr| addr.ip().to_string(),
    );

    println!("New WebSocket connection from: {}", ip); // Added debug print

    {
        let mut state_lock = state.lock().unwrap();
        state_lock.connected_ips.insert(ip.clone());
    }

    actix_web::rt::spawn(async move {
        ws_handler(session, msg_stream, state, ip).await;
    });

    Ok(res)
}

async fn ws_handler(
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    state: web::Data<ServerState>,
    ip: String,
) {
    println!("ws_handler started for IP: {}", ip); // Added debug print
    let mut interval = interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                println!("ws_handler: interval tick"); // Added debug print
                let (payload_json, usage_to_add_to_history, history_date_for_save) = {
                    let mut state_lock = state.lock().unwrap();
                    state_lock.sys.refresh_cpu();
                    state_lock.sys.refresh_memory();
                    state_lock.sys.refresh_networks();

                    let cpu_usage = state_lock.sys.global_cpu_info().cpu_usage();
                    let memory_usage = state_lock.sys.used_memory();

                    // 실시간 네트워크 전송률 계산
                    let mut current_rate = 0;
                    let mut current_network_data = HashMap::new();
                    for (interface_name, network) in state_lock.sys.networks() {
                        let received = network.total_received();
                        let transmitted = network.total_transmitted();

                        if let Some((last_received, last_transmitted)) = state_lock.last_network_totals.get(interface_name) {
                            current_rate += (received - last_received) + (transmitted - last_transmitted);
                        }
                        current_network_data.insert(interface_name.to_string(), (received, transmitted));
                    }
                    state_lock.last_network_totals = current_network_data;
                    state_lock.network_current_rate = current_rate;

                    // 일별 총 전송량 업데이트
                    let today = Utc::now().date_naive();
                    let mut usage_to_add = 0;
                    let mut date_to_save = today;

                    if state_lock.last_history_update < today {
                        // 이전 날짜의 평균 사용량을 이력에 추가
                        if state_lock.current_day_sample_count > 0 {
                            usage_to_add = state_lock.current_day_total_bytes / state_lock.current_day_sample_count;
                            date_to_save = state_lock.last_history_update;
                        }
                        state_lock.current_day_total_bytes = 0; // 현재 날짜의 총 전송 바이트 초기화
                        state_lock.current_day_sample_count = 0; // 현재 날짜의 샘플 수 초기화
                        state_lock.last_history_update = today;
                    }
                    state_lock.current_day_total_bytes += current_rate; // 현재 날짜의 총 전송 바이트에 실시간 전송률 추가
                    state_lock.current_day_sample_count += 1; // 샘플 수 증가

                    let payload = StatusPayload {
                        cpu_usage,
                        memory_usage,
                        network_current_rate: state_lock.network_current_rate,
                        connected_ips: state_lock.connected_ips.iter().cloned().collect(),
                    };
                    (serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string()), usage_to_add, date_to_save)
                };

                // 이력 저장 (락 해제 후)
                if usage_to_add_to_history > 0 {
                    let mut state_lock = state.lock().unwrap(); // 다시 잠금
                    state_lock.network_history.push(NetworkHistoryEntry {
                        date: history_date_for_save,
                        usage: usage_to_add_to_history,
                    });
                    state_lock.network_history.retain(|entry| {
                        (Utc::now().date_naive() - entry.date).num_days() < HISTORY_RETENTION_DAYS
                    });
                    save_network_history(&state_lock.network_history).await;
                }

                println!("Sending payload: {}", payload_json);

                if session.text(payload_json).await.is_err() {
                    break;
                }
            }
            Some(Ok(msg)) = msg_stream.next() => {
                if let Message::Close(_) = msg {
                    println!("ws_handler: received close message"); // Added debug print
                    break;
                }
            }
            else => {
                println!("ws_handler: select! else branch"); // Added debug print
                break;
            }
        }
    }

    let mut state_lock = state.lock().unwrap();
    state_lock.connected_ips.remove(&ip);
    println!("ws_handler finished for IP: {}", ip); // Added debug print
}

#[get("/api/network_history")]
async fn get_network_history(state: web::Data<ServerState>) -> HttpResponse {
    let state_lock = state.lock().unwrap();
    HttpResponse::Ok().json(&state_lock.network_history)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .expect("Failed to read config.toml");

    let app_config: AppConfig = config
        .try_deserialize()
        .expect("Failed to deserialize config");

    env_logger::init_from_env(Env::default().default_filter_or(&app_config.log_level));

    let initial_history = load_network_history().await;
    let last_update = initial_history.last().map_or_else(|| Utc::now().date_naive(), |entry| entry.date);

    let state = web::Data::new(Arc::new(Mutex::new(AppState {
        sys: System::new_all(),
        pid: sysinfo::get_current_pid().unwrap(),
        connected_ips: HashSet::new(),
        network_current_rate: 0,
        network_history: initial_history,
        last_history_update: last_update,
        last_network_totals: HashMap::new(),
        current_day_total_bytes: 0,
        current_day_sample_count: 0,
    })));

    println!("Starting server at http://{}:{}/", app_config.host, app_config.port);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Logger::default())
            .service(ws)
            .service(get_network_history)
            .service(fs::Files::new("/dash", "./dashboard_assets").index_file("dashboard.html"))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind((app_config.host, app_config.port))?
    .run()
    .await
}