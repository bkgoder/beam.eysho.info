use clap::Parser;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

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
    tunnels: Arc<RwLock<HashMap<String, Arc<Mutex<TcpStream>>>>>,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let state = TunnelState {
        tunnels: Arc::new(RwLock::new(HashMap::new())),
    };
    
    info!("Beam server starting on port {} for {}", args.port, args.domain);
    
    let listener = TcpListener::bind(format!("0.0.0.0:{}", args.port))
        .await
        .expect("Failed to bind");
    
    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    handle_connection(stream, peer_addr, state_clone).await;
                });
            }
            Err(e) => error!("Accept error: {}", e),
        }
    }
}

async fn handle_connection(mut stream: TcpStream, peer_addr: std::net::SocketAddr, state: TunnelState) {
    let mut buf = [0u8; 1024];
    match stream.read(&mut buf).await {
        Ok(n) if n > 0 => {
            let tunnel_id = String::from_utf8_lossy(&buf[..n]).trim().to_string();
            info!("Connection from {}: tunnel_id={}", peer_addr, tunnel_id);

            if tunnel_id.contains("-me_up-") {
                // Client registration - store the stream
                let has_tunnel = {
                    let tunnels = state.tunnels.read().unwrap();
                    tunnels.contains_key(&tunnel_id)
                };
                
                if has_tunnel {
                    let _ = stream.write_all(b"ERROR: Already exists\n").await;
                    error!("Tunnel already exists: {}", tunnel_id);
                    return;
                }
                
                {
                    let mut tunnels = state.tunnels.write().unwrap();
                    tunnels.insert(tunnel_id.clone(), Arc::new(Mutex::new(stream)));
                }
                info!("Tunnel registered: {}", tunnel_id);
            } else {
                // Incoming connection - check if tunnel exists
                let has_tunnel = {
                    let tunnels = state.tunnels.read().unwrap();
                    tunnels.contains_key(&tunnel_id)
                };
                
                if has_tunnel {
                    let _ = stream.write_all(b"OK\n").await;
                    info!("Forwarding for tunnel: {}", tunnel_id);
                } else {
                    let _ = stream.write_all(b"ERROR: Unknown tunnel\n").await;
                    error!("Unknown tunnel: {}", tunnel_id);
                }
            }
        }
        _ => warn!("Empty connection from {}", peer_addr),
    }
}
