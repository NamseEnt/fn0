//! Writes the active worker's loopback address to the file fn0-worker-proxy
//! polls.
//!
//! The file — not an RPC — is the contract on purpose: it's durable, so
//! fn0-worker-proxy recovers its target after its own restart without the
//! agent being involved.

use crate::shutdown::Shutdown;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;
use tracing::*;

pub async fn run(shutdown: Shutdown, mut active_addr_rx: watch::Receiver<Option<SocketAddr>>) {
    let path = match std::env::var("FN0_WORKER_AGENT_PROXY_TARGET_FILE") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => {
            warn!("FN0_WORKER_AGENT_PROXY_TARGET_FILE not set; proxy target writer disabled");
            return;
        }
    };
    info!(path = %path.display(), "proxy target writer started");

    let initial = *active_addr_rx.borrow();
    write_target(&path, initial).await;

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            res = active_addr_rx.changed() => {
                if res.is_err() {
                    break;
                }
            }
        }
        let addr = *active_addr_rx.borrow_and_update();
        write_target(&path, addr).await;
    }
    info!("proxy target writer stopped");
}

async fn write_target(path: &Path, addr: Option<SocketAddr>) {
    let body = match addr {
        Some(a) => format!("{a}\n"),
        None => String::new(),
    };
    match write_atomic(path, &body).await {
        Ok(()) => info!(?addr, path = %path.display(), "wrote proxy target file"),
        Err(err) => error!(
            ?err,
            ?addr,
            path = %path.display(),
            "failed to write proxy target file"
        ),
    }
}

async fn write_atomic(path: &Path, body: &str) -> std::io::Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("target");
    let tmp = parent.join(format!(".{file_name}.tmp"));
    {
        let mut f = tokio::fs::File::create(&tmp).await?;
        f.write_all(body.as_bytes()).await?;
        f.sync_all().await?;
    }
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}
