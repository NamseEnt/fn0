use wasmtime_wasi_http::bindings::http::types::ErrorCode;

#[derive(Clone)]
pub struct MetricsTx {
    inner: tokio::sync::mpsc::UnboundedSender<Metrics>,
}

impl MetricsTx {
    pub fn new() -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(metrics) = rx.recv().await {
                println!("{:?}", metrics);
            }
        });
        Self { inner: tx }
    }

    #[cfg(test)]
    pub fn from_sender(sender: tokio::sync::mpsc::UnboundedSender<Metrics>) -> Self {
        Self { inner: sender }
    }

    pub fn send(&self, metrics: Metrics) {
        // Gracefully handle metrics channel closure to prevent panics
        // This can happen during shutdown or if the metrics receiver is dropped
        let _ = self.inner.send(metrics);
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Metrics {
    Wasmtime {
        func: &'static str,
        code_id: String,
        error: anyhow::Error,
    },
    OneshotDropBeforeResponse {
        code_id: String,
    },
    ProxyReturnsErrorCode {
        code_id: String,
        error_code: ErrorCode,
    },
    RequestTaskJoinError {
        code_id: String,
        error: tokio::task::JoinError,
    },
    CpuTime {
        code_id: String,
        cpu_time: std::time::Duration,
    },
    CpuTimeout {
        code_id: String,
        cpu_time: std::time::Duration,
    },
    Trapped {
        code_id: String,
        trap: wasmtime::Trap,
    },
    CanceledUnexpectedly {
        code_id: String,
        error: anyhow::Error,
    },
    CreateInstance {
        code_id: String,
    },
    ProxyCacheError {
        code_id: String,
        error: adapt_cache::Error<anyhow::Error>,
    },
    CodeIdParseError,
}
