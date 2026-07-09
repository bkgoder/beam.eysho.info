use clap::Parser;
use log::{error, info};
use tokio::io::{copy_bidirectional, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug, Clone)]
#[command(name = "bkg-beam-router")]
#[command(version = "0.1.0")]
struct Args {
    #[arg(long, default_value = "0.0.0.0:2222")]
    listen: String,

    #[arg(long, default_value = "127.0.0.1:8080")]
    server: String,

    #[arg(long)]
    tunnel_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let listener = TcpListener::bind(&args.listen).await?;

    info!(
        "Beam router listening on {} and forwarding to tunnel {} through {}",
        args.listen, args.tunnel_id, args.server
    );

    loop {
        let (incoming, peer_addr) = listener.accept().await?;
        let args = args.clone();

        tokio::spawn(async move {
            info!("Router accepted connection from {peer_addr}");
            if let Err(err) = route_connection(incoming, args).await {
                error!("Router connection failed from {peer_addr}: {err}");
            }
        });
    }
}

async fn route_connection(
    mut incoming: TcpStream,
    args: Args,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut server_stream = TcpStream::connect(&args.server).await?;
    server_stream
        .write_all(format!("CONNECT {}\n", args.tunnel_id).as_bytes())
        .await?;

    copy_bidirectional(&mut incoming, &mut server_stream).await?;
    Ok(())
}
