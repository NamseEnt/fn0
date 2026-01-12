use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;

/// HMR message types sent to the client
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum HmrPayload {
    /// Full page reload required
    #[serde(rename = "reload")]
    FullReload,

    /// Module update (HMR)
    #[serde(rename = "update")]
    Update {
        /// Modules that need to be updated
        updates: Vec<HmrModuleUpdate>,
        /// Timestamp for cache busting
        timestamp: u64,
    },

    /// Error occurred during compilation
    #[serde(rename = "error")]
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        file: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        loc: Option<ErrorLocation>,
    },

    /// Connected to HMR server
    #[serde(rename = "connected")]
    Connected,
}

#[derive(Debug, Clone, Serialize)]
pub struct HmrModuleUpdate {
    /// Module ID (URL path, e.g., "/src/components/Button.tsx")
    pub id: String,
    /// The HMR boundary module that accepts this update
    pub accepted_by: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorLocation {
    pub line: u32,
    pub column: u32,
}

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

    /// Send a full page reload message
    pub fn send_reload(&self) {
        let payload = HmrPayload::FullReload;
        if let Ok(json) = serde_json::to_string(&payload) {
            let _ = self.tx.send(json);
        }
    }

    /// Send module update for HMR
    pub fn send_update(&self, updates: Vec<HmrModuleUpdate>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let payload = HmrPayload::Update { updates, timestamp };
        if let Ok(json) = serde_json::to_string(&payload) {
            let _ = self.tx.send(json);
        }
    }

    /// Send error message
    pub fn send_error(&self, message: String, file: Option<String>, loc: Option<ErrorLocation>) {
        let payload = HmrPayload::Error { message, file, loc };
        if let Ok(json) = serde_json::to_string(&payload) {
            let _ = self.tx.send(json);
        }
    }

    /// Send connected message
    pub fn send_connected(&self) {
        let payload = HmrPayload::Connected;
        if let Ok(json) = serde_json::to_string(&payload) {
            let _ = self.tx.send(json);
        }
    }
}

impl Default for HmrBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}
