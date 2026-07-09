use axum::{extract::Path, extract::State, routing::{get, post}, Json, Router};
use clap::Parser;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{copy_bidirectional, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

#[derive(Parser, Debug, Clone)]
#[command(name = "bkg-beam-server")]
#[command(version = "0.1.0")]
struct Args {
    #[arg(short, long, default_value = "beam.eysho.info")]
    domain: String,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(long, default_value_t = 8081)]
    admin_port: u16,
}

#[derive(Clone)]
struct TunnelState {
    tunnels: Arc<Mutex<HashMap<String, TunnelEntry>>>,
}

struct TunnelEntry {
    signal_tx: mpsc::Sender<()>,
    pending: VecDeque<TcpStream>,
    workers: VecDeque<TcpStream>,
}

#[derive(Clone)]
struct AdminState {
    domain: String,
    control_port: u16,
    state: TunnelState,
    accounts: AccountState,
}

#[derive(Clone)]
struct AccountState {
    users: Arc<Mutex<HashMap<String, UserAccount>>>,
    api_keys: Arc<Mutex<Vec<ApiKeyRecord>>>,
}

#[derive(Clone, Serialize)]
struct UserAccount {
    user_id: String,
    display_name: String,
    role: UserRole,
    license: LicenseRecord,
    created_at: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum UserRole {
    Admin,
    User,
}

#[derive(Clone, Serialize)]
struct LicenseRecord {
    license_id: String,
    plan: String,
    status: String,
}

#[derive(Clone, Serialize)]
struct ApiKeyRecord {
    key_id: String,
    owner_user_id: String,
    label: String,
    masked_key: String,
    status: String,
    created_at: u64,
}

#[derive(Deserialize)]
struct CreateApiKeyRequest {
    user_id: Option<String>,
    label: Option<String>,
    license_plan: Option<String>,
}

#[derive(Serialize)]
struct CreateApiKeyResponse {
    user: UserAccount,
    api_key: ApiKeyRecord,
    secret_once: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    domain: String,
    control_port: u16,
    tunnel_count: usize,
    user_count: usize,
    api_key_count: usize,
}

#[derive(Serialize)]
struct TunnelSnapshot {
    tunnel_id: String,
    owner_user_id: String,
    pending_connections: usize,
    worker_connections: usize,
}

#[derive(Serialize)]
struct AdminOverview {
    domain: String,
    users: Vec<UserAccount>,
    api_keys: Vec<ApiKeyRecord>,
    tunnels: Vec<TunnelSnapshot>,
    router_mappings: Vec<RouterMapping>,
}

#[derive(Serialize)]
struct RouterMapping {
    name: String,
    public_port: u16,
    tunnel_id: String,
    protocol: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let state = TunnelState {
        tunnels: Arc::new(Mutex::new(HashMap::new())),
    };
    let accounts = AccountState {
        users: Arc::new(Mutex::new(seed_users())),
        api_keys: Arc::new(Mutex::new(Vec::new())),
    };

    spawn_admin_api(args.clone(), state.clone(), accounts.clone());

    info!(
        "Beam server starting on 0.0.0.0:{} for {}",
        args.port, args.domain
    );

    let listener = TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let state = state.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, peer_addr, state).await {
                error!("Connection error from {peer_addr}: {err}");
            }
        });
    }
}

fn seed_users() -> HashMap<String, UserAccount> {
    let mut users = HashMap::new();
    users.insert(
        "admin".to_string(),
        UserAccount {
            user_id: "admin".to_string(),
            display_name: "Beam Admin".to_string(),
            role: UserRole::Admin,
            license: LicenseRecord {
                license_id: "lic-admin-root".to_string(),
                plan: "root".to_string(),
                status: "active".to_string(),
            },
            created_at: now_unix(),
        },
    );
    users.insert(
        "demo-user".to_string(),
        UserAccount {
            user_id: "demo-user".to_string(),
            display_name: "Demo User".to_string(),
            role: UserRole::User,
            license: LicenseRecord {
                license_id: "lic-demo-user".to_string(),
                plan: "free".to_string(),
                status: "active".to_string(),
            },
            created_at: now_unix(),
        },
    );
    users
}

fn spawn_admin_api(args: Args, state: TunnelState, accounts: AccountState) {
    tokio::spawn(async move {
        let admin_state = AdminState {
            domain: args.domain.clone(),
            control_port: args.port,
            state,
            accounts,
        };

        let app = Router::new()
            .route("/health", get(health))
            .route("/api/tunnels", get(list_tunnels))
            .route("/api/users/{user_id}/tunnels", get(list_user_tunnels))
            .route("/api/users/{user_id}/api-keys", get(list_user_api_keys))
            .route("/api/users/api-keys", post(create_user_api_key))
            .route("/api/admin/overview", get(admin_overview))
            .route("/api/admin/users", get(admin_users))
            .route("/api/admin/api-keys", get(admin_api_keys))
            .route("/api/admin/tunnels", get(admin_tunnels))
            .route("/api/admin/router-mappings", get(admin_router_mappings))
            .with_state(admin_state);

        let addr = format!("0.0.0.0:{}", args.admin_port);
        match TcpListener::bind(&addr).await {
            Ok(listener) => {
                info!("Beam admin API listening on {addr}");
                if let Err(err) = axum::serve(listener, app).await {
                    error!("Admin API failed: {err}");
                }
            }
            Err(err) => error!("Failed to bind admin API on {addr}: {err}"),
        }
    });
}

async fn health(State(admin): State<AdminState>) -> Json<HealthResponse> {
    let tunnels = admin.state.tunnels.lock().await;
    let users = admin.accounts.users.lock().await;
    let api_keys = admin.accounts.api_keys.lock().await;
    Json(HealthResponse {
        status: "ok",
        domain: admin.domain,
        control_port: admin.control_port,
        tunnel_count: tunnels.len(),
        user_count: users.len(),
        api_key_count: api_keys.len(),
    })
}

async fn list_tunnels(State(admin): State<AdminState>) -> Json<Vec<TunnelSnapshot>> {
    Json(snapshot_tunnels(&admin.state).await)
}

async fn list_user_tunnels(
    Path(user_id): Path<String>,
    State(admin): State<AdminState>,
) -> Json<Vec<TunnelSnapshot>> {
    let tunnels = snapshot_tunnels(&admin.state).await;
    let user_tunnels = tunnels
        .into_iter()
        .filter(|tunnel| tunnel.owner_user_id == user_id)
        .collect();

    Json(user_tunnels)
}

async fn list_user_api_keys(
    Path(user_id): Path<String>,
    State(admin): State<AdminState>,
) -> Json<Vec<ApiKeyRecord>> {
    let api_keys = admin.accounts.api_keys.lock().await;
    let keys = api_keys
        .iter()
        .filter(|key| key.owner_user_id == user_id)
        .cloned()
        .collect();

    Json(keys)
}

async fn create_user_api_key(
    State(admin): State<AdminState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Json<CreateApiKeyResponse> {
    let user_id = request.user_id.unwrap_or_else(|| "demo-user".to_string());
    let label = request.label.unwrap_or_else(|| "default".to_string());
    let license_plan = request.license_plan.unwrap_or_else(|| "free".to_string());
    let now = now_unix();

    let user = {
        let mut users = admin.accounts.users.lock().await;
        users
            .entry(user_id.clone())
            .or_insert_with(|| UserAccount {
                user_id: user_id.clone(),
                display_name: user_id.clone(),
                role: UserRole::User,
                license: LicenseRecord {
                    license_id: format!("lic-{user_id}-{now}"),
                    plan: license_plan.clone(),
                    status: "active".to_string(),
                },
                created_at: now,
            })
            .clone()
    };

    let secret_once = format!("beam_{}_{}", compact_token(&user_id), now);
    let key = ApiKeyRecord {
        key_id: format!("key-{user_id}-{now}"),
        owner_user_id: user_id,
        label,
        masked_key: mask_secret(&secret_once),
        status: "active".to_string(),
        created_at: now,
    };

    let mut api_keys = admin.accounts.api_keys.lock().await;
    api_keys.push(key.clone());

    Json(CreateApiKeyResponse {
        user,
        api_key: key,
        secret_once,
    })
}

async fn admin_overview(State(admin): State<AdminState>) -> Json<AdminOverview> {
    let users = admin_users_raw(&admin).await;
    let api_keys = admin_api_keys_raw(&admin).await;
    let tunnels = snapshot_tunnels(&admin.state).await;
    let router_mappings = router_mappings();

    Json(AdminOverview {
        domain: admin.domain,
        users,
        api_keys,
        tunnels,
        router_mappings,
    })
}

async fn admin_users(State(admin): State<AdminState>) -> Json<Vec<UserAccount>> {
    Json(admin_users_raw(&admin).await)
}

async fn admin_api_keys(State(admin): State<AdminState>) -> Json<Vec<ApiKeyRecord>> {
    Json(admin_api_keys_raw(&admin).await)
}

async fn admin_tunnels(State(admin): State<AdminState>) -> Json<Vec<TunnelSnapshot>> {
    Json(snapshot_tunnels(&admin.state).await)
}

async fn admin_router_mappings() -> Json<Vec<RouterMapping>> {
    Json(router_mappings())
}

async fn admin_users_raw(admin: &AdminState) -> Vec<UserAccount> {
    let users = admin.accounts.users.lock().await;
    users.values().cloned().collect()
}

async fn admin_api_keys_raw(admin: &AdminState) -> Vec<ApiKeyRecord> {
    let api_keys = admin.accounts.api_keys.lock().await;
    api_keys.clone()
}

async fn snapshot_tunnels(state: &TunnelState) -> Vec<TunnelSnapshot> {
    let tunnels = state.tunnels.lock().await;
    tunnels
        .iter()
        .map(|(tunnel_id, entry)| TunnelSnapshot {
            tunnel_id: tunnel_id.clone(),
            owner_user_id: owner_from_tunnel_id(tunnel_id),
            pending_connections: entry.pending.len(),
            worker_connections: entry.workers.len(),
        })
        .collect()
}

fn router_mappings() -> Vec<RouterMapping> {
    vec![RouterMapping {
        name: "default-ssh".to_string(),
        public_port: 2222,
        tunnel_id: "22-me_up-22".to_string(),
        protocol: "tcp/ssh".to_string(),
    }]
}

fn owner_from_tunnel_id(tunnel_id: &str) -> String {
    if tunnel_id.starts_with("admin-") {
        "admin".to_string()
    } else {
        "demo-user".to_string()
    }
}

fn compact_token(input: &str) -> String {
    input
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn mask_secret(secret: &str) -> String {
    let prefix: String = secret.chars().take(10).collect();
    format!("{prefix}…")
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

async fn handle_connection(
    stream: TcpStream,
    peer_addr: std::net::SocketAddr,
    state: TunnelState,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let read = reader.read_line(&mut line).await?;

    if read == 0 {
        warn!("Empty connection from {peer_addr}");
        return Ok(());
    }

    let mut stream = reader.into_inner();
    let mut parts = line.split_whitespace();
    let command = parts.next().unwrap_or_default();
    let tunnel_id = parts.next().unwrap_or_default().to_string();

    if tunnel_id.is_empty() {
        stream.write_all(b"ERROR missing tunnel id\n").await?;
        return Ok(());
    }

    match command {
        "REGISTER" => register_tunnel(tunnel_id, stream, state).await,
        "CONNECT" => enqueue_pending(tunnel_id, stream, state).await,
        "WORKER" => enqueue_worker(tunnel_id, stream, state).await,
        other => {
            stream
                .write_all(format!("ERROR unknown command {other}\n").as_bytes())
                .await?;
            Ok(())
        }
    }
}

async fn register_tunnel(
    tunnel_id: String,
    mut control_stream: TcpStream,
    state: TunnelState,
) -> Result<(), Box<dyn std::error::Error>> {
    let (signal_tx, mut signal_rx) = mpsc::channel::<()>(1024);

    {
        let mut tunnels = state.tunnels.lock().await;
        if tunnels.contains_key(&tunnel_id) {
            control_stream
                .write_all(b"ERROR tunnel already exists\n")
                .await?;
            return Ok(());
        }

        tunnels.insert(
            tunnel_id.clone(),
            TunnelEntry {
                signal_tx,
                pending: VecDeque::new(),
                workers: VecDeque::new(),
            },
        );
    }

    control_stream.write_all(b"OK\n").await?;
    info!("Tunnel registered: {tunnel_id}");

    while signal_rx.recv().await.is_some() {
        if let Err(err) = control_stream.write_all(b"CONNECT\n").await {
            error!("Control stream failed for {tunnel_id}: {err}");
            break;
        }
    }

    let mut tunnels = state.tunnels.lock().await;
    tunnels.remove(&tunnel_id);
    info!("Tunnel removed: {tunnel_id}");

    Ok(())
}

async fn enqueue_pending(
    tunnel_id: String,
    stream: TcpStream,
    state: TunnelState,
) -> Result<(), Box<dyn std::error::Error>> {
    let signal_tx = {
        let mut tunnels = state.tunnels.lock().await;
        let Some(entry) = tunnels.get_mut(&tunnel_id) else {
            let mut stream = stream;
            stream.write_all(b"ERROR unknown tunnel\n").await?;
            return Ok(());
        };

        entry.pending.push_back(stream);
        entry.signal_tx.clone()
    };

    let _ = signal_tx.send(()).await;
    pair_ready_streams(tunnel_id, state).await;
    Ok(())
}

async fn enqueue_worker(
    tunnel_id: String,
    mut stream: TcpStream,
    state: TunnelState,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        let mut tunnels = state.tunnels.lock().await;
        let Some(entry) = tunnels.get_mut(&tunnel_id) else {
            stream.write_all(b"ERROR unknown tunnel\n").await?;
            return Ok(());
        };

        stream.write_all(b"OK\n").await?;
        entry.workers.push_back(stream);
    }

    pair_ready_streams(tunnel_id, state).await;
    Ok(())
}

async fn pair_ready_streams(tunnel_id: String, state: TunnelState) {
    loop {
        let pair = {
            let mut tunnels = state.tunnels.lock().await;
            let Some(entry) = tunnels.get_mut(&tunnel_id) else {
                return;
            };

            match (entry.pending.pop_front(), entry.workers.pop_front()) {
                (Some(pending), Some(worker)) => Some((pending, worker)),
                (pending, worker) => {
                    if let Some(pending) = pending {
                        entry.pending.push_front(pending);
                    }
                    if let Some(worker) = worker {
                        entry.workers.push_front(worker);
                    }
                    None
                }
            }
        };

        let Some((mut pending, mut worker)) = pair else {
            return;
        };

        let tunnel_id = tunnel_id.clone();
        tokio::spawn(async move {
            info!("Forwarding stream for tunnel {tunnel_id}");
            if let Err(err) = copy_bidirectional(&mut pending, &mut worker).await {
                error!("Forwarding failed for tunnel {tunnel_id}: {err}");
            }
        });
    }
}
