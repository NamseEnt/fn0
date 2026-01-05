use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct HmrBroadcaster {
    tx: Arc<broadcast::Sender<String>>,
}

impl HmrBroadcaster {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(16);
        Self { tx: Arc::new(tx) }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    pub fn send_reload(&self) {
        let _ = self.tx.send(r#"{"type":"reload"}"#.to_string());
    }
}

impl Default for HmrBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}
