//! Tiny TCP forwarder fronting fn0-worker containers on :443.
//!
//! It sits in front of fn0-worker-agent so the agent can restart without
//! dropping inbound connections: the agent only writes the active worker
//! address into a target file, which this process polls. The file is the
//! contract — durable, so this process also recovers its target after its
//! own restart. Data path is a raw TCP `copy`; TLS is terminated by the
//! worker, so the bytes here are opaque.

mod forward;
mod target_file;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tracing::*;

const DEFAULT_PUBLIC_LISTEN_ADDR: &str = "0.0.0.0:443";
const DEFAULT_TARGET_FILE: &str = "/etc/fn0-worker-proxy/target";
const DEFAULT_POLL_INTERVAL_SECS: u64 = 1;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fn0_worker_proxy=info,info".parse().unwrap()),
        )
        .init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "fn0-worker-proxy starting"
    );

    let public_addr: SocketAddr = std::env::var("FN0_WORKER_PROXY_LISTEN_ADDR")
        .unwrap_or_else(|_| DEFAULT_PUBLIC_LISTEN_ADDR.to_string())
        .parse()
        .expect("FN0_WORKER_PROXY_LISTEN_ADDR must be host:port");
    let target_file: PathBuf = std::env::var("FN0_WORKER_PROXY_TARGET_FILE")
        .unwrap_or_else(|_| DEFAULT_TARGET_FILE.to_string())
        .into();
    let poll_interval = Duration::from_secs(
        std::env::var("FN0_WORKER_PROXY_POLL_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_POLL_INTERVAL_SECS),
    );

    let public_listener = TcpListener::bind(public_addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind public listener {public_addr}: {err}"));
    info!(
        %public_addr,
        target_file = %target_file.display(),
        poll_secs = poll_interval.as_secs(),
        "listener bound"
    );

    let (target_tx, target_rx) = watch::channel::<Option<SocketAddr>>(None);

    tokio::spawn(target_file::serve(target_file, poll_interval, target_tx));
    forward::serve(public_listener, target_rx).await;
}
