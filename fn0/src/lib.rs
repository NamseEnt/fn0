mod execute;
pub mod telemetry;

pub use bytes::Bytes;
use execute::Job;
use http_body_util::BodyExt;
pub use http_body_util::Full;
use hyper::Request;
use measure_cpu_time::SystemClock;
use std::path::PathBuf;
pub use wasmtime_wasi_http::{bindings::http::types::ErrorCode, body::HyperOutgoingBody};

#[derive(Clone)]
pub struct Config {
    pub port: Option<u16>,
    pub wasm_path: Option<PathBuf>,
    pub otlp_endpoint: Option<String>,
}

pub struct Fn0 {
    job_tx: tokio::sync::mpsc::Sender<Job<hyper::body::Incoming>>,
    telemetry_providers: Option<telemetry::TelemetryProviders>,
}

impl Fn0 {
    pub async fn new(config: Config) -> color_eyre::Result<Self> {
        let proxy_cache = match config.wasm_path {
            Some(wasm_path) => adapt_cache::fs::FsAdaptCache::new(
                wasm_path
                    .parent()
                    .ok_or_else(|| color_eyre::eyre::eyre!("cannot find wasm_path's parent"))?
                    .to_path_buf(),
                1024 * 1024,
            ),
            None => todo!("Remote URL support not implemented yet"),
        };

        let (job_tx, job_rx) = tokio::sync::mpsc::channel(10 * 1024);

        let telemetry_providers = telemetry::setup_telemetry(config.otlp_endpoint)?;

        tokio::spawn({
            async move {
                execute::Executor::new(proxy_cache, job_rx, SystemClock, false)
                    .run()
                    .await;
            }
        });

        Ok(Self {
            job_tx,
            telemetry_providers,
        })
    }

    pub async fn run(
        &self,
        code_id: String,
        req: Request<hyper::body::Incoming>,
    ) -> color_eyre::Result<execute::Response> {
        let (res_tx, res_rx) = tokio::sync::oneshot::channel();
        if self
            .job_tx
            .send(Job {
                req,
                res_tx,
                code_id: code_id.clone(),
            })
            .await
            .is_err()
        {
            telemetry::oneshot_drop_before_response(&code_id);
            return Ok(internal_error_response());
        };

        match res_rx.await {
            Ok(res) => Ok(res),
            Err(_err) => Ok(internal_error_response()),
        }
    }
}

impl Drop for Fn0 {
    fn drop(&mut self) {
        if let Some(providers) = self.telemetry_providers.take() {
            let _ = telemetry::shutdown_telemetry(Some(providers));
        }
    }
}
fn internal_error_response() -> execute::Response {
    let body = http_body_util::Full::new(Bytes::from("Internal Server Error"))
        .map_err(|_| ErrorCode::InternalError(None));
    let mut res = hyper::Response::new(HyperOutgoingBody::new(body));
    *res.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
    res
}
