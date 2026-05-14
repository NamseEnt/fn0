//! Polls the target file and publishes the current forward target.
//!
//! File contract:
//! - absent or empty            → no target (new connections are dropped)
//! - a single `host:port` line  → forward new connections there
//!
//! The worker-agent writes this file atomically (tmp + rename), so a poll
//! never observes a partial write.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::watch;
use tracing::*;

pub async fn serve(
    path: PathBuf,
    poll_interval: Duration,
    target_tx: watch::Sender<Option<SocketAddr>>,
) {
    info!(path = %path.display(), "target file poller started");
    let mut current: Option<SocketAddr> = None;
    loop {
        let next = read_target(&path).await;
        if next != current {
            info!(?current, ?next, "forward target changed");
            current = next;
            let _ = target_tx.send(next);
        }
        tokio::time::sleep(poll_interval).await;
    }
}

async fn read_target(path: &Path) -> Option<SocketAddr> {
    let content = match tokio::fs::read_to_string(path).await {
        Ok(c) => c,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                warn!(?err, path = %path.display(), "failed to read target file");
            }
            return None;
        }
    };
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.parse::<SocketAddr>() {
        Ok(addr) => Some(addr),
        Err(err) => {
            warn!(?err, content = %trimmed, "target file content is not a valid host:port");
            None
        }
    }
}
