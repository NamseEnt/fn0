mod cache;
mod env_crypto;
mod env_yaml;
mod manifest_poller;
mod presign_sync;
mod queue_consumer;
mod static_pages;
mod telemetry;
mod vault_client;
mod worker_pool;

use base64::Engine;
use bytes::Bytes;
use cache::S3BundleCache;
use color_eyre::eyre::Result;
use fn0::{
    CrossProjectEnqueueHijack, CrossProjectInvokeDispatcher, CrossProjectInvokeHijack,
    ExecutionContext, MetricCardinalityGate, ObjectStorageHijack, OtlpHijack, PresignGate,
    PublicStorageHijack, PurgeGate, QueueHijack, TursoHijack, VaultHijack,
};
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_rustls::TlsAcceptor;
use vault_client::VaultClient;
use worker_pool::{DispatchError, RequestEnvelope};

pub type WorkerContext = ExecutionContext<S3BundleCache>;

const DEFAULT_CACHE_SIZE_BYTES: usize = 512 * 1024 * 1024;
const DEFAULT_OPS_PORT: u16 = 9090;
const REQUEST_DEADLINE: std::time::Duration = std::time::Duration::from_secs(30);

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

fn build_otlp_hijack(metric_gate: Arc<MetricCardinalityGate>) -> Arc<OtlpHijack> {
    let target_host =
        std::env::var("FN0_OTLP_TARGET_HOST").expect("FN0_OTLP_TARGET_HOST must be set");
    let target_scheme: hyper::http::uri::Scheme = std::env::var("FN0_OTLP_TARGET_SCHEME")
        .expect("FN0_OTLP_TARGET_SCHEME must be set")
        .parse()
        .expect("FN0_OTLP_TARGET_SCHEME must be 'http' or 'https'");
    let auth_raw = std::env::var("FN0_OTLP_AUTH").expect("FN0_OTLP_AUTH must be set");
    let auth = if auth_raw.is_empty() {
        None
    } else {
        Some(auth_raw)
    };
    let target_path_prefix =
        std::env::var("FN0_OTLP_TARGET_PATH_PREFIX").unwrap_or_else(|_| "".to_string());
    let placeholder_host = std::env::var("FN0_OTLP_PLACEHOLDER_HOST")
        .unwrap_or_else(|_| "fn0-otel.fn0.dev".to_string());
    Arc::new(OtlpHijack {
        placeholder_host,
        target_scheme,
        target_host,
        target_path_prefix,
        auth,
        metric_gate: Some(metric_gate),
    })
}

fn build_queue_hijack() -> Arc<QueueHijack> {
    Arc::new(QueueHijack::from_env().expect("queue hijack init failed"))
}

fn build_cross_project_enqueue_hijack() -> Arc<CrossProjectEnqueueHijack> {
    Arc::new(
        CrossProjectEnqueueHijack::from_env().expect("control invoke queue hijack init failed"),
    )
}

fn build_cross_project_invoke_hijack() -> Arc<CrossProjectInvokeHijack> {
    Arc::new(
        CrossProjectInvokeHijack::from_env().expect("control invoke direct hijack init failed"),
    )
}

struct WorkerCrossProjectInvokeDispatcher {
    senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
}

impl CrossProjectInvokeDispatcher for WorkerCrossProjectInvokeDispatcher {
    fn dispatch(
        &self,
        target_project_id: String,
        req: fn0::Request,
    ) -> anyhow::Result<oneshot::Receiver<anyhow::Result<fn0::Response>>> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let envelope = RequestEnvelope {
            project_id: target_project_id,
            req,
            resp_tx,
            enqueued_at: std::time::Instant::now(),
        };
        worker_pool::dispatch(&self.senders, envelope).map_err(|e| match e {
            DispatchError::Full => anyhow::anyhow!("worker pool full"),
            DispatchError::Closed => anyhow::anyhow!("worker pool closed"),
        })?;
        Ok(resp_rx)
    }
}

fn build_vault_hijack() -> Arc<VaultHijack> {
    Arc::new(VaultHijack::from_env().expect("vault hijack init failed"))
}

fn build_turso_hijack() -> Arc<TursoHijack> {
    let group_token = std::env::var("TURSO_GROUP_TOKEN").expect("TURSO_GROUP_TOKEN must be set");
    let target_host_suffix =
        std::env::var("TURSO_DB_HOST_SUFFIX").expect("TURSO_DB_HOST_SUFFIX must be set");
    let placeholder_host =
        std::env::var("TURSO_PLACEHOLDER_HOST").unwrap_or_else(|_| "fn0-db.fn0.dev".to_string());
    Arc::new(TursoHijack {
        placeholder_host,
        target_host_suffix,
        group_token,
    })
}

fn build_object_storage_hijack(presign_gate: Arc<PresignGate>) -> Arc<ObjectStorageHijack> {
    Arc::new(
        ObjectStorageHijack::from_env()
            .expect("object storage hijack init failed")
            .with_presign_gate(presign_gate),
    )
}

fn build_public_storage_hijack(purge_gate: Arc<PurgeGate>) -> Arc<PublicStorageHijack> {
    Arc::new(
        PublicStorageHijack::from_env()
            .expect("public storage hijack init failed")
            .with_purge_gate(purge_gate),
    )
}

fn main() -> Result<()> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    color_eyre::install()?;

    let otlp_endpoint = std::env::var("OTLP_ENDPOINT").expect("OTLP_ENDPOINT must be set");
    let otlp_basic_auth = std::env::var("OTLP_BASIC_AUTH").expect("OTLP_BASIC_AUTH must be set");

    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();
    let telemetry_providers = telemetry::setup(&otlp_endpoint, Some(&otlp_basic_auth))?;
    install_panic_hook();

    let result = rt.block_on(run());

    telemetry::shutdown(telemetry_providers)?;
    result
}

fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "<unknown>".to_string());
        let payload = info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&'static str>() {
            (*s).to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "<non-string panic payload>".to_string()
        };
        let backtrace = std::backtrace::Backtrace::force_capture();
        tracing::error!(
            location = %location,
            panic = %message,
            backtrace = %backtrace,
            "panic captured"
        );
        prev(info);
    }));
}

async fn run() -> Result<()> {
    let cwasm_bucket = std::env::var("CWASM_BUCKET").expect("CWASM_BUCKET is required");
    let s3_endpoint = std::env::var("S3_ENDPOINT").expect("S3_ENDPOINT is required");
    let s3_region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let s3_access_key_id =
        std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID is required");
    let s3_secret_access_key =
        std::env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY is required");
    let user_port: u16 = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "443".to_string())
        .parse()
        .expect("HTTP_PORT must be a valid port");
    let ops_port: u16 = std::env::var("FN0_WORKER_OPS_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_OPS_PORT);

    let vault_client = Arc::new(
        VaultClient::from_env()
            .map_err(|err| color_eyre::eyre::eyre!("vault client init: {err}"))?,
    );

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
        vault_client.clone(),
        cache_size_bytes,
    );
    let static_page_store = static_pages::StaticPageStore::from_env()
        .map_err(|error| color_eyre::eyre::eyre!("static page store init: {error}"))?;

    let direct_hijack = build_cross_project_invoke_hijack();
    let presign_gate = Arc::new(PresignGate::new());
    let purge_gate = Arc::new(PurgeGate::new());
    let metric_gate = Arc::new(MetricCardinalityGate::new());

    let execution_context = Arc::new(
        ExecutionContext::new(engine, linker, cache.clone())
            .with_turso_hijack(build_turso_hijack())
            .with_queue_hijack(build_queue_hijack())
            .with_cross_project_enqueue_hijack(build_cross_project_enqueue_hijack())
            .with_cross_project_invoke_hijack(direct_hijack.clone())
            .with_vault_hijack(build_vault_hijack())
            .with_otlp_hijack(build_otlp_hijack(metric_gate.clone()))
            .with_object_storage_hijack(build_object_storage_hijack(presign_gate.clone()))
            .with_public_storage_hijack(build_public_storage_hijack(purge_gate))
            .with_static_page_storage(Arc::new(static_page_store.clone())),
    );

    // Recorded on the worker's own meter rather than stamped into guest
    // payloads: this is operator-facing capacity data against the shared
    // Grafana pool, not a project's own telemetry.
    let active_series_gate = metric_gate.clone();
    let _active_series_gauge = opentelemetry::global::meter("fn0-worker")
        .u64_observable_gauge("fn0.metric.active_series")
        .with_unit("1")
        .with_description("Active metric series the cardinality gate tracks, per project")
        .with_callback(move |observer| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_secs() as i64)
                .unwrap_or(0);
            for (project_id, count) in active_series_gate.snapshot(now) {
                observer.observe(
                    count as u64,
                    &[opentelemetry::KeyValue::new("project_id", project_id)],
                );
            }
        })
        .build();

    let manifest_loaded = Arc::new(AtomicBool::new(false));
    let instance_count = Arc::new(AtomicU64::new(0));
    let drain_flag = Arc::new(AtomicBool::new(false));

    let num_workers = worker_pool::default_num_threads();
    let worker_senders = Arc::new(worker_pool::spawn_workers(
        execution_context.clone(),
        num_workers,
    ));
    tracing::info!(threads = num_workers, "worker threads started");

    direct_hijack.set_dispatcher(Arc::new(WorkerCrossProjectInvokeDispatcher {
        senders: worker_senders.clone(),
    }));

    let manifest_db =
        manifest_poller::build_database_from_env().map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
    let manifest_handle = tokio::spawn({
        let cache = cache.clone();
        let manifest_loaded = manifest_loaded.clone();
        async move {
            manifest_poller::run(manifest_db, cache, manifest_loaded).await;
        }
    });

    let presign_db =
        manifest_poller::build_database_from_env().map_err(|e| color_eyre::eyre::eyre!("{e}"))?;
    let presign_handle = tokio::spawn(presign_sync::run(presign_db, presign_gate.clone()));

    let queue_consumer_handle = {
        let config = queue_consumer::QueueConsumerConfig::from_env()
            .map_err(|err| color_eyre::eyre::eyre!("queue consumer config: {err}"))?;
        let worker_senders = worker_senders.clone();
        tokio::spawn(async move {
            queue_consumer::run(config, worker_senders).await;
        })
    };

    let apex_route = apex_route_from_env();

    let user_handle = tokio::spawn({
        let worker_senders = worker_senders.clone();
        let instance_count = instance_count.clone();
        let cache = cache.clone();
        let drain_flag = drain_flag.clone();
        async move {
            if let Err(err) = run_user_server(
                user_port,
                worker_senders,
                instance_count,
                drain_flag,
                cache,
                apex_route,
            )
            .await
            {
                tracing::error!(%err, "user server error");
            }
        }
    });

    let ops_handle = tokio::spawn({
        let manifest_loaded = manifest_loaded.clone();
        let instance_count = instance_count.clone();
        let drain_flag = drain_flag.clone();
        async move {
            if let Err(err) =
                run_ops_server(ops_port, manifest_loaded, instance_count, drain_flag).await
            {
                tracing::error!(%err, "ops server error");
            }
        }
    });

    tokio::select! {
        _ = manifest_handle => {},
        _ = presign_handle => {},
        _ = user_handle => {},
        _ = ops_handle => {},
        _ = queue_consumer_handle => {},
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received ctrl-c, shutting down");
        },
    }

    Ok(())
}

struct ApexRoute {
    domain: String,
    project_id: String,
}

fn apex_route_from_env() -> Option<Arc<ApexRoute>> {
    match (
        std::env::var("FN0_APEX_DOMAIN"),
        std::env::var("FN0_APEX_PROJECT_ID"),
    ) {
        (Ok(domain), Ok(project_id)) => Some(Arc::new(ApexRoute { domain, project_id })),
        _ => None,
    }
}

async fn run_user_server(
    port: u16,
    worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
    instance_count: Arc<AtomicU64>,
    drain_flag: Arc<AtomicBool>,
    cache: S3BundleCache,
    apex_route: Option<Arc<ApexRoute>>,
) -> Result<()> {
    let tls_acceptor = {
        let cert_pem =
            read_pem_env("ORIGIN_CERT_PEM").expect("ORIGIN_CERT_PEM (or _BASE64) must be set");
        let key_pem =
            read_pem_env("ORIGIN_KEY_PEM").expect("ORIGIN_KEY_PEM (or _BASE64) must be set");
        let certs: Vec<_> = rustls_pemfile::certs(&mut cert_pem.as_bytes())
            .collect::<std::result::Result<_, _>>()?;
        let key = rustls_pemfile::private_key(&mut key_pem.as_bytes())?
            .ok_or_else(|| color_eyre::eyre::eyre!("no private key found in ORIGIN_KEY_PEM"))?;
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;
        TlsAcceptor::from(Arc::new(config))
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "user server listening (TLS)");

    loop {
        let (socket, _peer_addr) = listener.accept().await?;

        let worker_senders = worker_senders.clone();
        let instance_count = instance_count.clone();
        let drain_flag = drain_flag.clone();
        let tls_acceptor = tls_acceptor.clone();
        let cache = cache.clone();
        let apex_route = apex_route.clone();

        tokio::spawn(async move {
            // Sniff first byte to multiplex TLS user traffic (Cloudflare → NLB
            // → here, TLS ClientHello starts with 0x16) vs plain HTTP /health
            // (OCI NLB health check on the same 443 port). peek() leaves the
            // byte in the socket so the chosen handler can read it again.
            let mut sniff_buf = [0u8; 1];
            let first_byte = match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                socket.peek(&mut sniff_buf),
            )
            .await
            {
                Ok(Ok(0)) => return,
                Ok(Ok(_)) => sniff_buf[0],
                Ok(Err(err)) => {
                    tracing::warn!(%err, "peek failed");
                    return;
                }
                Err(_) => {
                    tracing::warn!("peek timeout");
                    return;
                }
            };

            if first_byte == 0x16 {
                let service = service_fn(move |req| {
                    let worker_senders = worker_senders.clone();
                    let instance_count = instance_count.clone();
                    let drain_flag = drain_flag.clone();
                    let cache = cache.clone();
                    let apex_route = apex_route.clone();
                    async move {
                        handle_user_request(
                            req,
                            worker_senders,
                            instance_count,
                            drain_flag,
                            cache,
                            apex_route,
                        )
                        .await
                    }
                });

                let result = match tls_acceptor.accept(socket).await {
                    Ok(tls_stream) => {
                        http1::Builder::new()
                            .serve_connection(TokioIo::new(tls_stream), service)
                            .await
                    }
                    Err(err) => {
                        tracing::error!(%err, "TLS handshake failed");
                        return;
                    }
                };

                if let Err(err) = result {
                    tracing::error!(%err, "Failed to serve connection");
                }
            } else {
                let service = service_fn(handle_plain_health_request);
                if let Err(err) = http1::Builder::new()
                    .serve_connection(TokioIo::new(socket), service)
                    .await
                {
                    tracing::warn!(%err, "plain HTTP serve failed");
                }
            }
        });
    }
}

async fn run_ops_server(
    port: u16,
    manifest_loaded: Arc<AtomicBool>,
    instance_count: Arc<AtomicU64>,
    drain_flag: Arc<AtomicBool>,
) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "ops server listening");

    loop {
        let (socket, _peer_addr) = listener.accept().await?;
        let manifest_loaded = manifest_loaded.clone();
        let instance_count = instance_count.clone();
        let drain_flag = drain_flag.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let manifest_loaded = manifest_loaded.clone();
                let instance_count = instance_count.clone();
                let drain_flag = drain_flag.clone();
                async move { handle_ops_request(req, manifest_loaded, instance_count, drain_flag).await }
            });

            if let Err(err) = http1::Builder::new()
                .serve_connection(TokioIo::new(socket), service)
                .await
            {
                tracing::error!(%err, "Failed to serve ops connection");
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

async fn handle_plain_health_request(
    req: hyper::Request<hyper::body::Incoming>,
) -> std::result::Result<HyperResponse, anyhow::Error> {
    if req.method() == hyper::Method::GET && req.uri().path() == "/health" {
        Ok(hyper::Response::new(Full::new(Bytes::from("ok"))))
    } else {
        Ok(hyper::Response::builder()
            .status(404)
            .body(Full::new(Bytes::new()))
            .unwrap())
    }
}

async fn handle_ops_request(
    req: hyper::Request<hyper::body::Incoming>,
    manifest_loaded: Arc<AtomicBool>,
    instance_count: Arc<AtomicU64>,
    drain_flag: Arc<AtomicBool>,
) -> std::result::Result<HyperResponse, anyhow::Error> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/ready") => {
            if manifest_loaded.load(Ordering::Acquire) {
                Ok(hyper::Response::new(Full::new(Bytes::from("ready"))))
            } else {
                Ok(hyper::Response::builder()
                    .status(503)
                    .body(Full::new(Bytes::from("manifest not loaded")))
                    .unwrap())
            }
        }
        (&hyper::Method::POST, "/drain") => {
            drain_flag.store(true, Ordering::Relaxed);
            tracing::info!("worker entered drain mode");
            Ok(hyper::Response::new(Full::new(Bytes::from("draining"))))
        }
        (&hyper::Method::GET, "/status") => {
            let body = serde_json::json!({
                "instances": instance_count.load(Ordering::Relaxed),
                "draining": drain_flag.load(Ordering::Relaxed),
            });
            let s = serde_json::to_string(&body).unwrap();
            Ok(hyper::Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(s)))
                .unwrap())
        }
        (&hyper::Method::GET, "/health") => {
            Ok(hyper::Response::new(Full::new(Bytes::from("good"))))
        }
        (&hyper::Method::GET, "/role") => {
            Ok(hyper::Response::new(Full::new(Bytes::from("worker"))))
        }
        _ => Ok(hyper::Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("not found")))
            .unwrap()),
    }
}

async fn handle_user_request(
    req: hyper::Request<hyper::body::Incoming>,
    worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
    instance_count: Arc<AtomicU64>,
    drain_flag: Arc<AtomicBool>,
    cache: S3BundleCache,
    apex_route: Option<Arc<ApexRoute>>,
) -> std::result::Result<HyperResponse, anyhow::Error> {
    if req.uri().path().starts_with("/__fn0_queue_task/") {
        return Ok(hyper::Response::builder()
            .status(403)
            .body(Full::new(Bytes::from("Forbidden")))
            .unwrap());
    }

    if drain_flag.load(Ordering::Relaxed) {
        return Ok(hyper::Response::builder()
            .status(503)
            .header("connection", "close")
            .body(Full::new(Bytes::from("draining")))
            .unwrap());
    }

    let _guard = InFlightGuard::new(instance_count);

    let host = req
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let host_no_port = host.split(':').next().unwrap_or("").to_string();
    let request_path = req.uri().path().to_string();

    let resolve_start = std::time::Instant::now();
    let project_id = match apex_route.as_deref() {
        Some(apex) if apex.domain == host_no_port => apex.project_id.clone(),
        _ => match cache.resolve_domain(&host_no_port).await {
            Some(sub) => sub,
            None => host_no_port
                .split('.')
                .next()
                .unwrap_or("unknown")
                .to_string(),
        },
    };
    fn0::telemetry::stage_duration("resolve_domain", resolve_start.elapsed());

    let mapped_req = req.map(|body| {
        UnsyncBoxBody::new(body)
            .map_err(|e: hyper::Error| anyhow::anyhow!(e))
            .boxed_unsync()
    });

    let (resp_tx, resp_rx) = oneshot::channel();
    let envelope = RequestEnvelope {
        project_id: project_id.clone(),
        req: mapped_req,
        resp_tx,
        enqueued_at: std::time::Instant::now(),
    };

    if let Err(err) = worker_pool::dispatch(&worker_senders, envelope) {
        match err {
            DispatchError::Full => {
                tracing::warn!(%project_id, "worker queue full");
                return Ok(hyper::Response::builder()
                    .status(503)
                    .body(Full::new(Bytes::from("Service Unavailable")))
                    .unwrap());
            }
            DispatchError::Closed => {
                tracing::error!(%project_id, "worker queue closed");
                return Ok(hyper::Response::builder()
                    .status(500)
                    .body(Full::new(Bytes::from("Internal Server Error")))
                    .unwrap());
            }
        }
    }

    let run_result = match tokio::time::timeout(REQUEST_DEADLINE, resp_rx).await {
        Ok(Ok(r)) => r,
        Ok(Err(_)) => {
            tracing::error!(%project_id, "worker dropped response channel");
            return Ok(hyper::Response::builder()
                .status(500)
                .body(Full::new(Bytes::from("Internal Server Error")))
                .unwrap());
        }
        Err(_) => {
            fn0::telemetry::request_deadline_exceeded();
            tracing::error!(%project_id, "request exceeded deadline");
            return Ok(hyper::Response::builder()
                .status(504)
                .header("connection", "close")
                .body(Full::new(Bytes::from("Gateway Timeout")))
                .unwrap());
        }
    };

    match run_result {
        Ok(resp) => {
            let (parts, body) = resp.into_parts();
            let collect_start = std::time::Instant::now();
            let collected: std::result::Result<http_body_util::Collected<Bytes>, anyhow::Error> =
                body.collect().await;
            fn0::telemetry::stage_duration("body_collect", collect_start.elapsed());
            let body_bytes = match collected {
                Ok(c) => c.to_bytes(),
                Err(err) => {
                    tracing::error!(%err, %project_id, "response body collect failed");
                    return Ok(hyper::Response::builder()
                        .status(502)
                        .body(Full::new(Bytes::from("Bad Gateway")))
                        .unwrap());
                }
            };
            Ok(hyper::Response::from_parts(parts, Full::new(body_bytes)))
        }
        Err(err) => {
            // Walk the chain: singleflight and the fetch path wrap this, and a
            // wrapped NotFound answered 502 instead of 404, which reads as a
            // broken deploy rather than an absent one.
            let not_found = err.chain().any(|cause| {
                matches!(
                    cause.downcast_ref::<fn0::cache::Error>(),
                    Some(fn0::cache::Error::NotFound)
                )
            });
            if not_found {
                return Ok(hyper::Response::builder()
                    .status(404)
                    .header("content-type", "text/plain; charset=utf-8")
                    .body(Full::new(Bytes::from(
                        "No application is deployed at this subdomain.",
                    )))
                    .unwrap());
            }
            // The cause goes in the message, not a field: the log pipeline
            // forwards message bodies and drops structured fields, so a field
            // here is invisible exactly when an outage makes it matter.
            tracing::error!(%project_id, path = %request_path, "Failed to run fn0: {err:#}");
            Ok(hyper::Response::builder()
                .status(502)
                .body(Full::new(Bytes::from("Bad Gateway")))
                .unwrap())
        }
    }
}
