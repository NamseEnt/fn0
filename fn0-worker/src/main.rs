mod bundle;
mod bundle_cache;
mod bundle_store;
mod quic;
mod telemetry;

use bundle::BundleFetcher;
use bundle_cache::BundleCache;
use bundle_store::BundleStore;
use bytes::Bytes;
use color_eyre::eyre::Result;
use fn0::{DeploymentMap, Fn0, WasmProxyPre};
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::string::FromUtf8Error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use base64::Engine;
use tokio_rustls::TlsAcceptor;

pub type WorkerFn0 = Fn0<BundleCache<String, FromUtf8Error>>;

pub fn read_pem_env(name: &str) -> Option<String> {
    if let Ok(v) = std::env::var(name) {
        return Some(v);
    }
    let b64 = std::env::var(format!("{name}_BASE64")).ok()?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(&b64).ok()?;
    String::from_utf8(bytes).ok()
}

fn main() -> Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    color_eyre::install()?;

    let otlp_endpoint = std::env::var("OTLP_ENDPOINT").expect("OTLP_ENDPOINT is required");
    let otlp_basic_auth = std::env::var("OTLP_BASIC_AUTH").ok();

    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();
    let telemetry_providers =
        telemetry::setup(&otlp_endpoint, otlp_basic_auth.as_deref())?;

    let result = rt.block_on(run());

    telemetry::shutdown(telemetry_providers)?;
    result
}

async fn run() -> Result<()> {
    let cwasm_bucket = std::env::var("CWASM_BUCKET").expect("CWASM_BUCKET is required");
    let s3_endpoint = std::env::var("S3_ENDPOINT").expect("S3_ENDPOINT is required");
    let s3_region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let quic_port: u16 = std::env::var("QUIC_PORT")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("QUIC_PORT must be a valid port");
    let http_port: u16 = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "443".to_string())
        .parse()
        .expect("HTTP_PORT must be a valid port");

    let s3_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_config::Region::new(s3_region))
        .endpoint_url(&s3_endpoint)
        .load()
        .await;
    let s3_client = aws_sdk_s3::Client::from_conf(
        aws_sdk_s3::config::Builder::from(&s3_config)
            .force_path_style(true)
            .build(),
    );

    let store = Arc::new(BundleStore::new());
    let wasm_cache: BundleCache<WasmProxyPre, fn0::wasmtime::Error> =
        BundleCache::new(store.clone());
    let js_cache: BundleCache<String, FromUtf8Error> = BundleCache::new(store.clone());

    let deployment_map = DeploymentMap::new();
    let env_vars = Arc::new(RwLock::new(Vec::new()));
    let fn0 = Arc::new(Fn0::new(
        wasm_cache.clone(),
        js_cache.clone(),
        deployment_map,
        env_vars,
    ));

    let bundle_fetcher = Arc::new(BundleFetcher::new(
        s3_client,
        cwasm_bucket,
        store.clone(),
        wasm_cache,
        js_cache,
        fn0.clone(),
    ));

    let deployment_id = Arc::new(AtomicU64::new(0));
    let instance_count = Arc::new(AtomicU64::new(0));
    let graceful_shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let quic_handle = tokio::spawn({
        let deployment_id = deployment_id.clone();
        let instance_count = instance_count.clone();
        let graceful_shutdown = graceful_shutdown.clone();
        let fn0 = fn0.clone();
        let bundle_fetcher = bundle_fetcher.clone();
        async move {
            if let Err(err) = quic::run_quic_server(quic_port, deployment_id, instance_count, graceful_shutdown, fn0, bundle_fetcher).await {
                tracing::error!(%err, "QUIC server error");
            }
        }
    });

    let http_handle = tokio::spawn({
        let fn0 = fn0.clone();
        let graceful_shutdown = graceful_shutdown.clone();
        async move {
            if let Err(err) = run_http_server(http_port, fn0, graceful_shutdown).await {
                tracing::error!(%err, "HTTP server error");
            }
        }
    });

    tokio::select! {
        _ = quic_handle => {},
        _ = http_handle => {},
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received ctrl-c, shutting down");
        },
    }

    Ok(())
}

async fn run_http_server(
    port: u16,
    fn0: Arc<WorkerFn0>,
    graceful_shutdown: Arc<std::sync::atomic::AtomicBool>,
) -> Result<()> {
    let tls_acceptor = match (read_pem_env("ORIGIN_CERT_PEM"), read_pem_env("ORIGIN_KEY_PEM")) {
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
        let (socket, _) = listener.accept().await?;
        let fn0 = fn0.clone();
        let graceful_shutdown = graceful_shutdown.clone();
        let tls_acceptor = tls_acceptor.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let fn0 = fn0.clone();
                let graceful_shutdown = graceful_shutdown.clone();
                async move { handle_request(req, fn0, graceful_shutdown).await }
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

type HyperResponse = hyper::Response<Full<Bytes>>;

async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    fn0: Arc<WorkerFn0>,
    graceful_shutdown: Arc<std::sync::atomic::AtomicBool>,
) -> std::result::Result<HyperResponse, anyhow::Error> {
    match req.uri().path() {
        "/health" => {
            let body = if graceful_shutdown.load(Ordering::Relaxed) {
                "graceful_shutting_down"
            } else {
                "good"
            };
            Ok(hyper::Response::new(Full::new(Bytes::from(body))))
        }
        "/role" => Ok(hyper::Response::new(Full::new(Bytes::from("worker")))),
        path if path.starts_with("/__forte_queue_task/") => {
            Ok(hyper::Response::builder()
                .status(403)
                .body(Full::new(Bytes::from("Forbidden")))
                .unwrap())
        }
        _ => {
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

            match fn0.run(&code_id, "/", mapped_req, None).await {
                Ok(resp) => {
                    let (parts, body) = resp.into_parts();
                    let collected: std::result::Result<http_body_util::Collected<Bytes>, anyhow::Error> = body.collect().await;
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
