use clap::Parser;
use log::{error, info, warn};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::io::{copy_bidirectional, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

#[derive(Parser, Debug)]
#[command(name = "bkg-beam-server")]
#[command(version = "0.1.0")]
struct Args {
    #[arg(short, long, default_value = "beam.eysho.info")]
    domain: String,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let state = TunnelState {
        tunnels: Arc::new(Mutex::new(HashMap::new())),
    };

    info!("Beam server starting on 0.0.0.0:{} for {}", args.port, args.domain);

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
            control_stream.write_all(b"ERROR tunnel already exists\n").await?;
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
