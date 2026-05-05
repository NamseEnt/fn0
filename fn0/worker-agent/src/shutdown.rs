use tokio::signal::unix::{SignalKind, signal};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct Shutdown(CancellationToken);

impl Shutdown {
    pub fn new() -> Self {
        Self(CancellationToken::new())
    }

    pub fn trigger(&self) {
        self.0.cancel();
    }

    pub async fn cancelled(&self) {
        self.0.cancelled().await;
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.is_cancelled()
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
