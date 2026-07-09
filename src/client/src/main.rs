use clap::Parser;
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// bkg-beam - Expose local services via beam.eysho.info
/// 
/// Usage: bkg-beam <local_port>:me up:<remote_port>
/// Example: bkg-beam 22:me up:22
/// This exposes local port 22 as 22-me.up-22.beam.eysho.info
#[derive(Parser, Debug)]
#[command(name = "bkg-beam")]
#[command(author = "bkg")]
#[command(version = "0.1.0")]
struct Args {
    /// Local port with :me suffix (e.g., 22:me)
    #[arg(value_parser = parse_me_port)]
    me: u16,

    /// Remote port with up: prefix (e.g., up:22)
    #[arg(value_parser = parse_up_port)]
    up: u16,

    /// Server domain (default: beam.eysho.info)
    #[arg(short, long, default_value = "beam.eysho.info")]
    server: String,
}

fn parse_me_port(s: &str) -> Result<u16, String> {
    let s = s.trim();
    if s.ends_with(":me") {
        s[..s.len()-3].parse().map_err(|_| format!("Invalid port in '{}'", s))
    } else {
        s.parse().map_err(|_| format!("Invalid port in '{}'", s))
    }
}

fn parse_up_port(s: &str) -> Result<u16, String> {
    let s = s.trim();
    if s.starts_with("up:") {
        s[3..].parse().map_err(|_| format!("Invalid port in '{}'", s))
    } else {
        s.parse().map_err(|_| format!("Invalid port in '{}'", s))
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    let local_port = args.me;
    let remote_port = args.up;
    let server_addr = format!("{}:{}", args.server, remote_port);
    let local_addr = format!("0.0.0.0:{}", local_port);
    let tunnel_url = format!("{}-me.up-{}.{}", local_port, remote_port, args.server);

    info!("Beam Tunnel");
    info!("  Local port:   {}", local_port);
    info!("  Server:       {}:{}", args.server, remote_port);
    info!("  Tunnel URL:   {}", tunnel_url);
    info!("Press Ctrl+C to stop");

    // Start listening on local port
    let listener = TcpListener::bind(&local_addr)
        .await
        .expect("Failed to bind local port");

    info!("Listening on {} for incoming connections", local_addr);

    // Create tunnel identifier: me-port_up-port
    let tunnel_id = format!("{}-me_up-{}", local_port, remote_port);

    // Connect to server
    match TcpStream::connect(&server_addr).await {
        Ok(mut server_stream) => {
            info!("Connected to server");

            // Send tunnel identifier
            server_stream
                .write_all(tunnel_id.as_bytes())
                .await
                .expect("Failed to send tunnel id");
            server_stream
                .write_all(b"\n")
                .await
                .expect("Failed to send newline");

            // Read response
            let mut buf = [0u8; 1024];
            let n = server_stream
                .read(&mut buf)
                .await
                .expect("Failed to read response");
            let response = String::from_utf8_lossy(&buf[..n]);

            if !response.trim().eq("OK") {
                error!("Server rejected tunnel: {}", response);
                return;
            }

            info!("Tunnel active! Access your service at {}", tunnel_url);

            // Accept connections on local port
            loop {
                match listener.accept().await {
                    Ok((local_stream, peer_addr)) => {
                        info!("Connection from {}", peer_addr);

                        // For each new local connection, create a new server connection
                        match TcpStream::connect(&server_addr).await {
                            Ok(mut server_clone) => {
                                // Send tunnel identifier again
                                server_clone
                                    .write_all(tunnel_id.as_bytes())
                                    .await
                                    .unwrap_or_else(|e| error!("Write error: {}", e));
                                server_clone
                                    .write_all(b"\n")
                                    .await
                                    .unwrap_or_else(|e| error!("Write error: {}", e));

                                // Read OK response
                                let mut buf2 = [0u8; 1024];
                                let _ = server_clone.read(&mut buf2).await;

                                tokio::spawn(async move {
                                    if let Err(e) = forward_bidirectional(local_stream, server_clone).await {
                                        error!("Forward error: {}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                error!("Failed to connect to server: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Accept error: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to connect to server: {}", e);
        }
    }
}

async fn forward_bidirectional(
    mut from: TcpStream,
    mut to: TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut from_reader, mut from_writer) = tokio::io::split(from);
    let (mut to_reader, mut to_writer) = tokio::io::split(to);

    // Forward from -> to
    let from_to = tokio::io::copy(&mut from_reader, &mut to_writer);
    
    // Forward to -> from
    let to_from = tokio::io::copy(&mut to_reader, &mut from_writer);

    tokio::select! {
        res = from_to => {
            if let Err(e) = res {
                return Err(Box::new(e));
            }
        }
        res = to_from => {
            if let Err(e) = res {
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}
