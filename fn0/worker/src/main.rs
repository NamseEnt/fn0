mod cache;
mod deployments_watcher;
mod env_crypto;
mod telemetry;
mod worker_pool;

use base64::Engine;
use bytes::Bytes;
use cache::S3BundleCache;
use color_eyre::eyre::Result;
use fn0::{ExecutionContext, TursoHijack};
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_rustls::TlsAcceptor;
use worker_pool::{DispatchError, RequestEnvelope};

pub type WorkerContext = ExecutionContext<S3BundleCache>;

const DEPLOYMENTS_JSON_PATH: &str = "/etc/fn0-worker/deployments.json";
const DEFAULT_CACHE_SIZE_BYTES: usize = 512 * 1024 * 1024;

pub fn read_pem_env(name: &str) -> Option<String> {
    if let Ok(v) = std::env::var(name) {
        return Some(v);
    }
    let b64 = std::env::var(format!("{name}_BASE64")).ok()?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&b64)
        .ok()?;
    String::from_utf8(bytes).ok()
}

fn build_turso_hijack() -> Option<Arc<TursoHijack>> {
    let group_token = std::env::var("TURSO_GROUP_TOKEN")
        .ok()
        .filter(|s| !s.is_empty());
    let target_host_suffix = std::env::var("TURSO_DB_HOST_SUFFIX")
        .ok()
        .filter(|s| !s.is_empty());
    match (group_token, target_host_suffix) {
        (Some(group_token), Some(target_host_suffix)) => {
            let placeholder_host = std::env::var("TURSO_PLACEHOLDER_HOST")
                .unwrap_or_else(|_| "forte-db.fn0.dev".to_string());
            Some(Arc::new(TursoHijack {
                placeholder_host,
                target_host_suffix,
                group_token,
            }))
        }
        _ => {
            tracing::warn!(
                "TURSO_GROUP_TOKEN or TURSO_DB_HOST_SUFFIX not set; Turso hijack disabled"
            );
            None
        }
    }
}

fn main() -> Result<()> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    color_eyre::install()?;

    let otlp_endpoint = std::env::var("OTLP_ENDPOINT").expect("OTLP_ENDPOINT is required");
    let otlp_basic_auth = std::env::var("OTLP_BASIC_AUTH").ok();

    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();
    let telemetry_providers = telemetry::setup(&otlp_endpoint, otlp_basic_auth.as_deref())?;

    let result = rt.block_on(run());

    telemetry::shutdown(telemetry_providers)?;
    result
}

async fn run() -> Result<()> {
    let cwasm_bucket = std::env::var("CWASM_BUCKET").expect("CWASM_BUCKET is required");
    let s3_endpoint = std::env::var("S3_ENDPOINT").expect("S3_ENDPOINT is required");
    let s3_region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let s3_access_key_id =
        std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID is required");
    let s3_secret_access_key =
        std::env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY is required");
    let http_port: u16 = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "443".to_string())
        .parse()
        .expect("HTTP_PORT must be a valid port");

    let env_key_base64 =
        std::env::var("FN0_ENV_KEY_BASE64").expect("FN0_ENV_KEY_BASE64 is required");
    let env_key = env_crypto::decode_key_base64(&env_key_base64)?;

    let cache_size_bytes = std::env::var("FN0_BUNDLE_CACHE_SIZE_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_CACHE_SIZE_BYTES);

    let operator = opendal::Operator::new(
        opendal::services::S3::default()
            .bucket(&cwasm_bucket)
            .region(&s3_region)
            .endpoint(&s3_endpoint)
            .access_key_id(&s3_access_key_id)
            .secret_access_key(&s3_secret_access_key)
            .disable_config_load()
            .disable_ec2_metadata(),
    )?
    .finish();

    let engine = fn0::build_engine().map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
    fn0::spawn_epoch_ticker(engine.clone());
    let linker = fn0::build_linker(&engine);

    let cache = S3BundleCache::new(
        engine.clone(),
        linker.clone(),
        operator,
        env_key,
        cache_size_bytes,
    );

    let execution_context = {
        let mut ctx = ExecutionContext::new(engine, linker, cache.clone());
        if let Some(hijack) = build_turso_hijack() {
            ctx = ctx.with_turso_hijack(hijack);
        }
        Arc::new(ctx)
    };

    let generation = Arc::new(AtomicU64::new(0));
    let instance_count = Arc::new(AtomicU64::new(0));

    let num_workers = worker_pool::default_num_threads();
    let worker_senders = Arc::new(worker_pool::spawn_workers(
        execution_context.clone(),
        num_workers,
    ));
    tracing::info!(threads = num_workers, "worker threads started");

    let watcher_handle = tokio::spawn({
        let generation = generation.clone();
        let cache = cache.clone();
        async move {
            deployments_watcher::run(&PathBuf::from(DEPLOYMENTS_JSON_PATH), generation, cache)
                .await;
        }
    });

    let http_handle = tokio::spawn({
        let worker_senders = worker_senders.clone();
        let generation = generation.clone();
        let instance_count = instance_count.clone();
        async move {
            if let Err(err) =
                run_http_server(http_port, worker_senders, generation, instance_count).await
            {
                tracing::error!(%err, "HTTP server error");
            }
        }
    });

    tokio::select! {
        _ = watcher_handle => {},
        _ = http_handle => {},
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received ctrl-c, shutting down");
        },
    }

    Ok(())
}

async fn run_http_server(
    port: u16,
    worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
    generation: Arc<AtomicU64>,
    instance_count: Arc<AtomicU64>,
) -> Result<()> {
    let tls_acceptor = match (
        read_pem_env("ORIGIN_CERT_PEM"),
        read_pem_env("ORIGIN_KEY_PEM"),
    ) {
        (Some(cert_pem), Some(key_pem)) => {
            let certs: Vec<_> = rustls_pemfile::certs(&mut cert_pem.as_bytes())
                .collect::<std::result::Result<_, _>>()?;
            let key = rustls_pemfile::private_key(&mut key_pem.as_bytes())?
                .ok_or_else(|| color_eyre::eyre::eyre!("no private key found in ORIGIN_KEY_PEM"))?;
            let config = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, key)?;
            Some(TlsAcceptor::from(Arc::new(config)))
        }
        _ => {
            tracing::warn!("ORIGIN_CERT_PEM/ORIGIN_KEY_PEM not set, running plain HTTP");
            None
        }
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, tls = tls_acceptor.is_some(), "HTTP(S) server listening");

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        let is_loopback = peer_addr.ip().is_loopback();

        let worker_senders = worker_senders.clone();
        let generation = generation.clone();
        let instance_count = instance_count.clone();
        let tls_acceptor = tls_acceptor.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let worker_senders = worker_senders.clone();
                let generation = generation.clone();
                let instance_count = instance_count.clone();
                async move {
                    handle_request(req, worker_senders, generation, instance_count, is_loopback)
                        .await
                }
            });

            let result = if let Some(acceptor) = tls_acceptor {
                match acceptor.accept(socket).await {
                    Ok(tls_stream) => {
                        http1::Builder::new()
                            .serve_connection(TokioIo::new(tls_stream), service)
                            .await
                    }
                    Err(err) => {
                        tracing::error!(%err, "TLS handshake failed");
                        return;
                    }
                }
            } else {
                http1::Builder::new()
                    .serve_connection(TokioIo::new(socket), service)
                    .await
            };

            if let Err(err) = result {
                tracing::error!(%err, "Failed to serve connection");
            }
        });
    }
}

struct InFlightGuard {
    counter: Arc<AtomicU64>,
}
impl InFlightGuard {
    fn new(counter: Arc<AtomicU64>) -> Self {
        counter.fetch_add(1, Ordering::Relaxed);
        Self { counter }
    }
}
impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}

type HyperResponse = hyper::Response<Full<Bytes>>;

async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
    generation: Arc<AtomicU64>,
    instance_count: Arc<AtomicU64>,
    is_loopback: bool,
) -> std::result::Result<HyperResponse, anyhow::Error> {
    match req.uri().path() {
        "/status" if is_loopback => {
            let body = serde_json::json!({
                "generation": generation.load(Ordering::Relaxed),
                "instances": instance_count.load(Ordering::Relaxed),
            });
            let s = serde_json::to_string(&body).unwrap();
            Ok(hyper::Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(s)))
                .unwrap())
        }
        "/status" => Ok(hyper::Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("not found")))
            .unwrap()),
        "/health" => Ok(hyper::Response::new(Full::new(Bytes::from("good")))),
        "/role" => Ok(hyper::Response::new(Full::new(Bytes::from("worker")))),
        path if path.starts_with("/__forte_queue_task/") => Ok(hyper::Response::builder()
            .status(403)
            .body(Full::new(Bytes::from("Forbidden")))
            .unwrap()),
        _ => {
            let _guard = InFlightGuard::new(instance_count);

            let host = req
                .headers()
                .get("host")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            let code_id = host.split('.').next().unwrap_or("unknown").to_string();

            let mapped_req = req.map(|body| {
                UnsyncBoxBody::new(body)
                    .map_err(|e: hyper::Error| anyhow::anyhow!(e))
                    .boxed_unsync()
            });

            let (resp_tx, resp_rx) = oneshot::channel();
            let envelope = RequestEnvelope {
                code_id: code_id.clone(),
                req: mapped_req,
                resp_tx,
            };

            if let Err(err) = worker_pool::dispatch(&worker_senders, envelope) {
                match err {
                    DispatchError::Full => {
                        tracing::warn!(%code_id, "worker queue full");
                        return Ok(hyper::Response::builder()
                            .status(503)
                            .body(Full::new(Bytes::from("Service Unavailable")))
                            .unwrap());
                    }
                    DispatchError::Closed => {
                        tracing::error!(%code_id, "worker queue closed");
                        return Ok(hyper::Response::builder()
                            .status(500)
                            .body(Full::new(Bytes::from("Internal Server Error")))
                            .unwrap());
                    }
                }
            }

            let run_result = match resp_rx.await {
                Ok(r) => r,
                Err(_) => {
                    tracing::error!(%code_id, "worker dropped response channel");
                    return Ok(hyper::Response::builder()
                        .status(500)
                        .body(Full::new(Bytes::from("Internal Server Error")))
                        .unwrap());
                }
            };

            match run_result {
                Ok(resp) => {
                    let (parts, body) = resp.into_parts();
                    let collected: std::result::Result<
                        http_body_util::Collected<Bytes>,
                        anyhow::Error,
                    > = body.collect().await;
                    let body_bytes = match collected {
                        Ok(c) => c.to_bytes(),
                        Err(_) => Bytes::new(),
                    };
                    Ok(hyper::Response::from_parts(parts, Full::new(body_bytes)))
                }
                Err(err) => {
                    tracing::error!(%err, %code_id, "Failed to run fn0");
                    Ok(hyper::Response::builder()
                        .status(502)
                        .body(Full::new(Bytes::from("Bad Gateway")))
                        .unwrap())
                }
            }
        }
    }
}
