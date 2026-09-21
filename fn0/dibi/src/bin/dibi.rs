use std::path::PathBuf;

use clap::Parser;
use dibi::{DibiServer, DibiServerConfig};

#[derive(Debug, Parser)]
struct Arguments {
    #[arg(long)]
    data_dir: PathBuf,
    #[arg(long)]
    listen: std::net::SocketAddr,
    #[arg(long)]
    cert: PathBuf,
    #[arg(long)]
    key: PathBuf,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let arguments = Arguments::parse();
    let config = DibiServerConfig::new(
        arguments.data_dir,
        arguments.listen,
        arguments.cert,
        arguments.key,
    );
    let server = DibiServer::bind(config).map_err(|error| error.to_string())?;
    eprintln!(
        "dibi listening on {}",
        server.local_addr().map_err(|error| error.to_string())?
    );
    server
        .run(shutdown_signal())
        .await
        .map_err(|error| error.to_string())
}

#[cfg(unix)]
async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("failed to install SIGTERM handler");
    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate.recv() => {}
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
