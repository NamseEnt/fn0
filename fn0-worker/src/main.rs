mod quic;
mod websocket;

use adapt_cache::s3::S3AdaptCache;
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

#[derive(Debug, Clone)]
struct WasmCacheError(String);

impl std::fmt::Display for WasmCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for WasmCacheError {}

#[derive(Clone)]
struct WasmCache {
    inner: S3AdaptCache<WasmProxyPre, WasmCacheError>,
}

type JsCache = S3AdaptCache<String, FromUtf8Error>;

fn main() -> Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run())
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
        .unwrap_or_else(|_| "8080".to_string())
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

    let wasm_cache = WasmCache {
        inner: S3AdaptCache::new(s3_client.clone(), cwasm_bucket.clone(), None, 512 * 1024 * 1024),
    };
    let js_cache: JsCache =
        S3AdaptCache::new(s3_client, cwasm_bucket, Some("js".to_string()), 64 * 1024 * 1024);

    let deployment_map = DeploymentMap::new();
    let env_vars = Arc::new(RwLock::new(Vec::new()));
    let fn0 = Arc::new(Fn0::new(wasm_cache, js_cache, deployment_map, env_vars));

    let deployment_id = Arc::new(AtomicU64::new(0));
    let instance_count = Arc::new(AtomicU64::new(0));
    let graceful_shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let ws_port: u16 = std::env::var("WS_PORT")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .expect("WS_PORT must be a valid port");

    let quic_handle = tokio::spawn({
        let deployment_id = deployment_id.clone();
        let instance_count = instance_count.clone();
        let graceful_shutdown = graceful_shutdown.clone();
        let fn0 = fn0.clone();
        async move {
            if let Err(err) = quic::run_quic_server(quic_port, deployment_id, instance_count, graceful_shutdown, fn0).await {
                tracing::error!(%err, "QUIC server error");
            }
        }
    });

    let ws_handle = tokio::spawn({
        let deployment_id = deployment_id.clone();
        let instance_count = instance_count.clone();
        let graceful_shutdown = graceful_shutdown.clone();
        let fn0 = fn0.clone();
        async move {
            if let Err(err) = websocket::run_websocket_server(ws_port, deployment_id, instance_count, graceful_shutdown, fn0).await {
                tracing::error!(%err, "WebSocket server error");
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
        _ = ws_handle => {},
        _ = http_handle => {},
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received ctrl-c, shutting down");
        },
    }

    Ok(())
}

impl adapt_cache::AdaptCache<WasmProxyPre, fn0::wasmtime::Error> for WasmCache {
    async fn get(
        &self,
        id: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(WasmProxyPre, usize), fn0::wasmtime::Error> + Send,
    ) -> std::result::Result<WasmProxyPre, adapt_cache::Error<fn0::wasmtime::Error>> {
        let s3_key = format!("{id}.cwasm.zst");
        self.inner
            .get(&s3_key, |bytes| {
                let decompressed = zstd::decode_all(bytes.as_ref())
                    .map(Bytes::from)
                    .unwrap_or(bytes);
                convert(decompressed).map_err(|e| WasmCacheError(e.to_string()))
            })
            .await
            .map_err(|e| match e {
                adapt_cache::Error::NotFound => adapt_cache::Error::NotFound,
                adapt_cache::Error::StorageError(e) => adapt_cache::Error::StorageError(e),
                adapt_cache::Error::ConvertError(e) => {
                    adapt_cache::Error::StorageError(anyhow::anyhow!(e.0))
                }
                adapt_cache::Error::SingleflightLeaderFailed => {
                    adapt_cache::Error::SingleflightLeaderFailed
                }
            })
    }
}

async fn run_http_server(
    port: u16,
    fn0: Arc<Fn0<JsCache>>,
    graceful_shutdown: Arc<std::sync::atomic::AtomicBool>,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "HTTP server listening");

    loop {
        let (socket, _) = listener.accept().await?;
        let fn0 = fn0.clone();
        let graceful_shutdown = graceful_shutdown.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(socket);
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        let fn0 = fn0.clone();
                        let graceful_shutdown = graceful_shutdown.clone();
                        async move {
                            handle_request(req, fn0, graceful_shutdown).await
                        }
                    }),
                )
                .await
            {
                tracing::error!(%err, "Failed to serve connection");
            }
        });
    }
}

type HyperResponse = hyper::Response<Full<Bytes>>;

async fn handle_request(
    req: hyper::Request<hyper::body::Incoming>,
    fn0: Arc<Fn0<JsCache>>,
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
