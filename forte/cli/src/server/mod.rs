mod cache;
mod fetch_handler;
mod hmr;
mod transform_server;

use anyhow::Result;
pub use cache::SimpleCache;
use fetch_handler::ForteFetchHandler;
use fn0::{CodeKind, DeploymentMap, Fn0};
use futures_util::{SinkExt, StreamExt};
pub use hmr::{HmrBroadcaster, HmrModuleUpdate};
use http_body_util::{BodyExt, Full, combinators::UnsyncBoxBody};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
pub use transform_server::TransformServer;

use crate::deps::DependencyMap;

pub struct ServerConfig {
    pub port: u16,
    pub backend_path: String,
    pub frontend_path: String,
    pub public_dir: PathBuf,
    pub project_root: PathBuf,
    pub dev_mode: bool,
    pub dep_map: DependencyMap,
}

pub struct ServerHandle {
    pub cache: SimpleCache,
    pub hmr: HmrBroadcaster,
    pub transform_server: Option<Arc<TransformServer>>,
}

pub async fn run(config: ServerConfig) -> Result<ServerHandle> {
    let mut deployment_map = DeploymentMap::new();
    deployment_map.register_code("backend", CodeKind::Wasm);
    deployment_map.register_code("frontend", CodeKind::Js);

    let cache = SimpleCache::new(config.backend_path.clone(), config.frontend_path.clone());
    let hmr = HmrBroadcaster::new();

    let transform_server = if config.dev_mode {
        Some(Arc::new(TransformServer::new(&config.project_root, config.dep_map.clone())))
    } else {
        None
    };

    let handle = ServerHandle {
        cache: cache.clone(),
        hmr: hmr.clone(),
        transform_server: transform_server.clone(),
    };

    let fn0 = Arc::new(Fn0::new(cache.clone(), cache, deployment_map));
    let public_dir = Arc::new(config.public_dir);

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr).await?;
    println!("Forte dev server listening on http://{}", addr);

    tokio::spawn(async move {
        loop {
            let (socket, _) = match listener.accept().await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                    continue;
                }
            };
            let fn0_clone = fn0.clone();
            let public_dir_clone = public_dir.clone();
            let hmr_clone = hmr.clone();
            let transform_server_clone = transform_server.clone();

            tokio::spawn(async move {
                let io = TokioIo::new(socket);
                let conn = http1::Builder::new().serve_connection(
                    io,
                    service_fn(move |req| {
                        let fn0 = fn0_clone.clone();
                        let public_dir = public_dir_clone.clone();
                        let hmr = hmr_clone.clone();
                        let ts = transform_server_clone.clone();
                        handle_request(req, fn0, public_dir, hmr, ts)
                    }),
                );
                if let Err(err) = conn.with_upgrades().await {
                    eprintln!("Failed to serve connection: {}", err);
                }
            });
        }
    });

    Ok(handle)
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    fn0: Arc<Fn0<SimpleCache>>,
    public_dir: Arc<PathBuf>,
    hmr: HmrBroadcaster,
    transform_server: Option<Arc<TransformServer>>,
) -> Result<fn0::Response> {
    let uri = req.uri().clone();
    let path = uri.path();
    println!("Received {} {path}", req.method());

    if path == "/__hmr" {
        return handle_hmr_upgrade(req, hmr).await;
    }

    if let Some(ts) = &transform_server {
        if should_transform(path) {
            return serve_transformed(ts, path).await;
        }
    }

    if let Some(static_response) = try_serve_static(&public_dir, path).await {
        return Ok(static_response);
    }

    let backend_response = match fn0
        .run(
            "backend",
            req.map(|body| {
                UnsyncBoxBody::new(body)
                    .map_err(|e| anyhow::anyhow!(e))
                    .boxed_unsync()
            }),
            None,
        )
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Backend error: {:?}", e);
            return Err(anyhow::anyhow!("Backend error: {:?}", e));
        }
    };

    let backend_status = backend_response.status();

    println!("Backend response status: {}", backend_status);

    if !backend_status.is_success() {
        let (parts, body) = backend_response.into_parts();
        let body_bytes = body.collect().await?.to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);
        eprintln!("Backend error response body: {}", body_str);

        return Ok(fn0::Response::from_parts(
            parts,
            UnsyncBoxBody::new(body_str.to_string())
                .map_err(|e| anyhow::anyhow!(e))
                .boxed_unsync(),
        ));
    }

    if path.starts_with("/__forte_hook/") {
        return Ok(backend_response);
    }

    println!("Preparing frontend request with backend response body");
    let frontend_request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(backend_response.into_body())?;

    println!("Calling frontend (ski::run)");
    let fetch_handler = Arc::new(ForteFetchHandler::new(fn0.clone()));
    match fn0.run("frontend", frontend_request, Some(fetch_handler)).await {
        Ok(resp) => {
            println!("Frontend response status: {}", resp.status());
            Ok(resp)
        }
        Err(e) => {
            eprintln!("Frontend error: {:?}", e);
            Err(e)
        }
    }
}

fn should_transform(path: &str) -> bool {
    path.starts_with("/src/")
        || path.starts_with("/.forte/deps/")
        || path == "/__hmr-client.js"
        || path == "/__react-refresh.js"
        || path == "/__error-overlay.js"
}

async fn serve_transformed(
    ts: &TransformServer,
    path: &str,
) -> Result<fn0::Response> {
    match ts.serve_module(path).await {
        Ok(resp) => {
            let (parts, body) = resp.into_parts();
            Ok(Response::from_parts(
                parts,
                UnsyncBoxBody::new(body).boxed_unsync(),
            ))
        }
        Err(e) => {
            eprintln!("[transform] Error serving {}: {:?}", path, e);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(
                    Full::new(bytes::Bytes::from(format!("Transform error: {}", e)))
                        .map_err(|e| anyhow::anyhow!("{e}"))
                        .boxed_unsync(),
                )?)
        }
    }
}

async fn try_serve_static(public_dir: &PathBuf, path: &str) -> Option<fn0::Response> {
    let file_path = if path == "/favicon.ico" {
        public_dir.join("favicon.ico")
    } else if path.starts_with("/public/") {
        let relative_path = path.strip_prefix("/public/").unwrap_or(path);
        public_dir.join(relative_path)
    } else {
        return None;
    };

    if !file_path.starts_with(public_dir) {
        return Some(
            Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(
                    Full::new(bytes::Bytes::from("Forbidden"))
                        .map_err(|e| anyhow::anyhow!("{e}"))
                        .boxed_unsync(),
                )
                .unwrap(),
        );
    }

    match tokio::fs::read(&file_path).await {
        Ok(contents) => {
            let content_type = get_content_type(&file_path);
            println!(
                "[static] Serving {} ({})",
                file_path.display(),
                content_type
            );

            Some(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", content_type)
                    .header("cache-control", "public, max-age=3600")
                    .body(
                        Full::new(bytes::Bytes::from(contents))
                            .map_err(|e| anyhow::anyhow!("{e}"))
                            .boxed_unsync(),
                    )
                    .unwrap(),
            )
        }
        Err(_) => Some(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(
                    Full::new(bytes::Bytes::from("Not Found"))
                        .map_err(|e| anyhow::anyhow!("{e}"))
                        .boxed_unsync(),
                )
                .unwrap(),
        ),
    }
}

fn get_content_type(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("webp") => "image/webp",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("eot") => "application/vnd.ms-fontobject",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("pdf") => "application/pdf",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        _ => "application/octet-stream",
    }
}

async fn handle_hmr_upgrade(
    req: Request<hyper::body::Incoming>,
    hmr: HmrBroadcaster,
) -> Result<fn0::Response> {
    let ws_key = req
        .headers()
        .get("sec-websocket-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let Some(ws_key) = ws_key else {
        return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(
            Full::new(bytes::Bytes::from("Missing WebSocket key"))
                .map_err(|e| anyhow::anyhow!("{e}"))
                .boxed_unsync(),
        )?);
    };

    let accept_key = derive_accept_key(ws_key.as_bytes());

    hmr.send_connected();

    tokio::spawn(async move {
        match hyper::upgrade::on(req).await {
            Ok(upgraded) => {
                let ws_stream = tokio_tungstenite::WebSocketStream::from_raw_socket(
                    TokioIo::new(upgraded),
                    tokio_tungstenite::tungstenite::protocol::Role::Server,
                    None,
                )
                .await;

                let (mut ws_tx, mut ws_rx) = ws_stream.split();
                let mut hmr_rx = hmr.subscribe();

                loop {
                    tokio::select! {
                        msg = hmr_rx.recv() => {
                            match msg {
                                Ok(data) => {
                                    if ws_tx.send(Message::Text(data)).await.is_err() {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                        ws_msg = ws_rx.next() => {
                            match ws_msg {
                                Some(Ok(Message::Close(_))) | None => break,
                                Some(Ok(Message::Ping(data))) => {
                                    let _ = ws_tx.send(Message::Pong(data)).await;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket upgrade error: {}", e);
            }
        }
    });

    Ok(Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header("upgrade", "websocket")
        .header("connection", "Upgrade")
        .header("sec-websocket-accept", accept_key)
        .body(
            Full::new(bytes::Bytes::new())
                .map_err(|e| anyhow::anyhow!("{e}"))
                .boxed_unsync(),
        )?)
}
