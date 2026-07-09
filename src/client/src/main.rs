use clap::Parser;
use log::{error, info};
use tokio::io::{copy_bidirectional, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// bkg-beam - expose a local TCP service through a beam server.
///
/// Usage:
///   bkg-beam <local_port>:me up:<remote_port> --server beam.eysho.info --server-port 8080
///
/// Example:
///   bkg-beam 22:me up:22
#[derive(Parser, Debug)]
#[command(name = "bkg-beam")]
#[command(author = "bkg")]
#[command(version = "0.1.0")]
struct Args {
    /// Local port with optional :me suffix, for example 22:me.
    #[arg(value_parser = parse_me_port)]
    me: u16,

    /// Public/remote port with optional up: prefix, for example up:22.
    #[arg(value_parser = parse_up_port)]
    up: u16,

    /// Beam server domain.
    #[arg(short, long, default_value = "beam.eysho.info")]
    server: String,

    /// Beam control/data port.
    #[arg(long, default_value_t = 8080)]
    server_port: u16,

    /// Local host to connect to when the server opens a tunnel connection.
    #[arg(long, default_value = "127.0.0.1")]
    local_host: String,
}

fn parse_me_port(s: &str) -> Result<u16, String> {
    let s = s.trim();
    if let Some(port) = s.strip_suffix(":me") {
        port.parse().map_err(|_| format!("Invalid port in '{s}'"))
    } else {
        s.parse().map_err(|_| format!("Invalid port in '{s}'"))
    }
}

fn parse_up_port(s: &str) -> Result<u16, String> {
    let s = s.trim();
    if let Some(port) = s.strip_prefix("up:") {
        port.parse().map_err(|_| format!("Invalid port in '{s}'"))
    } else {
        s.parse().map_err(|_| format!("Invalid port in '{s}'"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let local_port = args.me;
    let remote_port = args.up;
    let server_addr = format!("{}:{}", args.server, args.server_port);
    let local_addr = format!("{}:{}", args.local_host, local_port);
    let tunnel_id = format!("{}-me_up-{}", local_port, remote_port);
    let tunnel_url = format!("{}-me.up-{}.{}", local_port, remote_port, args.server);

    info!("Beam tunnel");
    info!("  Local target: {}", local_addr);
    info!("  Beam server:  {}", server_addr);
    info!("  Tunnel id:    {}", tunnel_id);
    info!("  Tunnel URL:   {}", tunnel_url);

    let mut control = TcpStream::connect(&server_addr).await?;
    control
        .write_all(format!("REGISTER {tunnel_id}\n").as_bytes())
        .await?;

    let mut control = BufReader::new(control);
    let mut response = String::new();
    control.read_line(&mut response).await?;

    if response.trim() != "OK" {
        error!("Server rejected tunnel registration: {}", response.trim());
        return Ok(());
    }

    info!("Tunnel active. Waiting for server-side connections.");

    loop {
        let mut line = String::new();
        let read = control.read_line(&mut line).await?;
        if read == 0 {
            error!("Control connection closed by server");
            return Ok(());
        }

        match line.trim() {
            "CONNECT" => {
                let server_addr = server_addr.clone();
                let local_addr = local_addr.clone();
                let tunnel_id = tunnel_id.clone();

                tokio::spawn(async move {
                    if let Err(err) = open_worker(server_addr, local_addr, tunnel_id).await {
                        error!("Worker connection failed: {err}");
                    }
                });
            }
            other if other.is_empty() => {}
            other => error!("Unknown control message from server: {other}"),
        }
    }
}

async fn open_worker(
    server_addr: String,
    local_addr: String,
    tunnel_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut local_stream = TcpStream::connect(&local_addr).await?;
    let mut worker_stream = TcpStream::connect(&server_addr).await?;

    worker_stream
        .write_all(format!("WORKER {tunnel_id}\n").as_bytes())
        .await?;

    let mut reader = BufReader::new(worker_stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    if response.trim() != "OK" {
        return Err(format!("server rejected worker: {}", response.trim()).into());
    }

    let mut worker_stream = reader.into_inner();
    copy_bidirectional(&mut worker_stream, &mut local_stream).await?;
    Ok(())
}
