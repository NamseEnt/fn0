use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal::unix::{SignalKind, signal};
use tokio_util::sync::CancellationToken;

/// Shutdown handle shared across all agent tasks.
///
/// `soft` distinguishes "agent is being replaced; keep the worker containers
/// alive for the next process to adopt" from "agent is going down for real;
/// tear everything down". Set before `trigger()` so tasks can read it on
/// their cancellation path.
#[derive(Clone)]
pub struct Shutdown {
    token: CancellationToken,
    soft: Arc<AtomicBool>,
}

impl Shutdown {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            soft: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn trigger(&self) {
        self.token.cancel();
    }

    pub fn trigger_soft(&self) {
        self.soft.store(true, Ordering::Release);
        self.token.cancel();
    }

    pub fn is_soft(&self) -> bool {
        self.soft.load(Ordering::Acquire)
    }

    pub async fn cancelled(&self) {
        self.token.cancelled().await;
    }

    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }
}

pub async fn wait_for_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
    tokio::select! {
        _ = sigterm.recv() => {}
        _ = sigint.recv() => {}
    }
}
