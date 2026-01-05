mod cache;
mod hmr;

use anyhow::{Context, Result};
pub use cache::SimpleCache;
use fn0::{CodeKind, DeploymentMap, Fn0};
use futures_util::{SinkExt, StreamExt};
pub use hmr::HmrBroadcaster;
use http_body_util::{BodyExt, Full, combinators::UnsyncBoxBody};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;

pub struct ServerConfig {
    pub port: u16,
    pub backend_path: String,
    pub frontend_path: String,
    pub public_dir: PathBuf,
    pub fe_dir: PathBuf,
    pub dev_mode: bool,
}

pub struct ServerHandle {
    pub cache: SimpleCache,
    pub hmr: HmrBroadcaster,
    _vite_child: Option<std::process::Child>,
    _ssr_adapter_child: Option<std::process::Child>,
}

pub async fn run(config: ServerConfig) -> Result<ServerHandle> {
    let mut deployment_map = DeploymentMap::new();
    deployment_map.register_code("backend", CodeKind::Wasm);
    deployment_map.register_code("frontend", CodeKind::Js);

    let cache = SimpleCache::new(config.backend_path.clone(), config.frontend_path.clone());
    let hmr = HmrBroadcaster::new();

    let (vite_port, vite_child) = if config.dev_mode {
        let port = find_available_port(5173)?;
        let child = start_vite(&config.fe_dir, port)?;
        wait_for_vite_ready(port).await?;
        println!("[vite] Dev server ready on port {}", port);
        (Some(port), Some(child))
    } else {
        (None, None)
    };

    let (ssr_adapter_port, ssr_adapter_child) = if config.dev_mode {
        let (port, child) = start_ssr_adapter(&config.fe_dir)?;
        (Some(port), Some(child))
    } else {
        (None, None)
    };

    let handle = ServerHandle {
        cache: cache.clone(),
        hmr: hmr.clone(),
        _vite_child: vite_child,
        _ssr_adapter_child: ssr_adapter_child,
    };

    let fn0 = Arc::new(Fn0::new(cache.clone(), cache, deployment_map));
    let public_dir = Arc::new(config.public_dir);

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr).await?;
    println!("Forte SSR server listening on http://{}", addr);

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

            tokio::spawn(async move {
                let io = TokioIo::new(socket);
                let conn = http1::Builder::new().serve_connection(
                    io,
                    service_fn(move |req| {
                        let fn0 = fn0_clone.clone();
                        let public_dir = public_dir_clone.clone();
                        let hmr = hmr_clone.clone();
                        handle_request(req, fn0, public_dir, hmr, vite_port, ssr_adapter_port)
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

fn find_available_port(start: u16) -> Result<u16> {
    use std::net::TcpListener;
    for port in start..=65535 {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Ok(port);
        }
    }
    anyhow::bail!("No available port found starting from {}", start)
}

fn start_vite(fe_dir: &Path, port: u16) -> Result<std::process::Child> {
    use std::process::{Command, Stdio};

    println!("[vite] Starting dev server on port {}...", port);
    let child = Command::new("npx")
        .arg("vite")
        .arg("--port")
        .arg(port.to_string())
        .arg("--strictPort")
        .current_dir(fe_dir)
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to start Vite dev server")?;

    Ok(child)
}

async fn wait_for_vite_ready(port: u16) -> Result<()> {
    use std::time::Duration;
    use tokio::time::sleep;

    let url = format!("http://localhost:{}", port);
    for _ in 0..50 {
        if let Ok(resp) = reqwest::get(&url).await
            && (resp.status().is_success() || resp.status().as_u16() == 404)
        {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }
    anyhow::bail!("Vite dev server failed to start within 5 seconds")
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    fn0: Arc<Fn0<SimpleCache>>,
    public_dir: Arc<PathBuf>,
    hmr: HmrBroadcaster,
    vite_port: Option<u16>,
    ssr_adapter_port: Option<u16>,
) -> Result<fn0::Response> {
    let uri = req.uri().clone();
    let path = uri.path();
    println!("Received {} {path}", req.method());

    if path == "/__hmr" {
        return handle_hmr_upgrade(req, hmr).await;
    }

    if let Some(vite_port) = vite_port
        && should_proxy_to_vite(path)
    {
        return proxy_to_vite(req, vite_port).await;
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

    if let Some(ssr_port) = ssr_adapter_port {
        println!("Calling SSR adapter on port {}", ssr_port);
        return call_ssr_adapter(ssr_port, &uri, backend_response).await;
    }

    println!("Preparing frontend request with backend response body");
    let frontend_request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(backend_response.into_body())?;

    println!("Calling frontend (ski::run)");
    match fn0.run("frontend", frontend_request).await {
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

fn should_proxy_to_vite(path: &str) -> bool {
    path.starts_with("/@vite/")
        || path.starts_with("/@react-refresh")
        || path.starts_with("/src/")
        || path.starts_with("/node_modules/")
}

async fn proxy_to_vite(
    req: Request<hyper::body::Incoming>,
    vite_port: u16,
) -> Result<fn0::Response> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    let url = format!(
        "http://127.0.0.1:{}{}",
        vite_port,
        uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
    );

    let client = reqwest::Client::new();
    let mut builder = client.request(method.clone(), &url);

    for (name, value) in headers.iter() {
        if name != "host"
            && let Ok(v) = value.to_str()
        {
            builder = builder.header(name.as_str(), v);
        }
    }

    let body_bytes = req.collect().await?.to_bytes();
    if !body_bytes.is_empty() {
        builder = builder.body(body_bytes.to_vec());
    }

    let resp = builder.send().await.context("Failed to proxy to Vite")?;

    let status = StatusCode::from_u16(resp.status().as_u16())?;
    let mut response_builder = Response::builder().status(status);

    for (name, value) in resp.headers().iter() {
        response_builder = response_builder.header(name.as_str(), value.as_bytes());
    }

    let body = resp.bytes().await?;
    Ok(response_builder.body(
        Full::new(body)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .boxed_unsync(),
    )?)
}

fn start_ssr_adapter(fe_dir: &Path) -> Result<(u16, std::process::Child)> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};

    println!("[ssr] Starting SSR adapter...");
    let mut child = Command::new("node")
        .arg("dev-server.mjs")
        .current_dir(fe_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start SSR adapter")?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line)
            && let Some(port) = json.get("port").and_then(|p| p.as_u64())
        {
            println!("[ssr] SSR adapter ready on port {}", port);
            return Ok((port as u16, child));
        }
    }

    anyhow::bail!("Failed to get SSR adapter port")
}

async fn call_ssr_adapter(
    port: u16,
    uri: &hyper::Uri,
    backend_response: fn0::Response,
) -> Result<fn0::Response> {
    let url = format!("http://127.0.0.1:{}/__ssr_render", port);

    let (_, body) = backend_response.into_parts();
    let body_bytes = body.collect().await?.to_bytes();
    let props: serde_json::Value = serde_json::from_slice(&body_bytes)?;

    let payload = serde_json::json!({
        "url": uri.to_string(),
        "props": props,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .context("Failed to call SSR adapter")?;

    let status = StatusCode::from_u16(resp.status().as_u16())?;
    let html = resp.bytes().await?;

    Ok(Response::builder()
        .status(status)
        .header("content-type", "text/html; charset=utf-8")
        .body(
            Full::new(html)
                .map_err(|e| anyhow::anyhow!("{e}"))
                .boxed_unsync(),
        )?)
}
