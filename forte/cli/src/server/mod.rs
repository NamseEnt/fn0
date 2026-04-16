mod cache;
pub mod vite_dev;

use anyhow::Result;
pub use cache::SimpleCache;
use fn0::{Deployment, DeploymentMap, EnvVars, Fn0};
use http_body_util::{BodyExt, Full, combinators::UnsyncBoxBody};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::UnixStream;

pub struct ServerConfig {
    pub port: u16,
    pub backend_path: String,
    pub frontend_path: String,
    pub public_dir: PathBuf,
    pub project_root: PathBuf,
    pub dev_mode: bool,
    pub vite_socket_path: Option<PathBuf>,
    pub env_vars: EnvVars,
}

pub struct ServerHandle {
    pub cache: SimpleCache,
    pub fn0: Arc<Fn0<SimpleCache>>,
}

pub const DEV_CODE_ID: &str = "app";

pub fn load_env_file(project_root: &Path) -> Vec<(String, String)> {
    let env_path = project_root.join(".env");
    let mut vars = Vec::new();

    let Ok(content) = std::fs::read_to_string(&env_path) else {
        return vars;
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            vars.push((key.trim().to_string(), value.trim().to_string()));
        }
    }

    vars
}

pub fn create_env_vars(project_root: &Path) -> EnvVars {
    let mut map = HashMap::new();
    map.insert(DEV_CODE_ID.to_string(), load_env_file(project_root));
    Arc::new(RwLock::new(map))
}

async fn load_public_assets(public_dir: &Path) -> HashMap<String, Vec<u8>> {
    let mut assets = HashMap::new();
    if !public_dir.exists() {
        return assets;
    }
    let mut stack = vec![public_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(mut rd) = tokio::fs::read_dir(&dir).await else {
            continue;
        };
        while let Ok(Some(entry)) = rd.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(bytes) = tokio::fs::read(&path).await {
                if let Ok(rel) = path.strip_prefix(public_dir) {
                    let key = format!("/{}", rel.to_string_lossy().replace('\\', "/"));
                    assets.insert(key, bytes);
                }
            }
        }
    }
    assets
}

pub async fn run(config: ServerConfig) -> Result<ServerHandle> {
    let mut deployment_map = DeploymentMap::new();
    deployment_map.register_deployment(
        "app",
        Deployment::Forte {
            frontend_script_path: config.frontend_path.clone(),
        },
    );

    let cache = SimpleCache::new(config.backend_path.clone(), config.frontend_path.clone());

    let vite_socket_path = config.vite_socket_path.map(Arc::new);

    let fn0 = Arc::new(Fn0::new(
        cache.clone(),
        cache.clone(),
        deployment_map,
        config.env_vars,
    ));

    let public_dir = Arc::new(config.public_dir);

    if vite_socket_path.is_none() {
        let assets = load_public_assets(&public_dir).await;
        fn0.set_public_assets("app", assets);
    }

    let handle = ServerHandle {
        cache: cache.clone(),
        fn0: fn0.clone(),
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://localhost:{}", config.port);

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
            let vite_socket_path_clone = vite_socket_path.clone();

            tokio::spawn(async move {
                let io = TokioIo::new(socket);
                let conn = http1::Builder::new().serve_connection(
                    io,
                    service_fn(move |req| {
                        let fn0 = fn0_clone.clone();
                        let public_dir = public_dir_clone.clone();
                        let vite_socket = vite_socket_path_clone.clone();
                        handle_request(req, fn0, public_dir, vite_socket)
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
    vite_socket_path: Option<Arc<PathBuf>>,
) -> Result<fn0::Response> {
    let uri = req.uri().clone();
    let path = uri.path();
    let path_with_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or(path);

    if path.starts_with("/__forte_queue_task/") {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(
                Full::new(bytes::Bytes::from("Forbidden"))
                    .map_err(|e| anyhow::anyhow!("{e}"))
                    .boxed_unsync(),
            )
            .unwrap());
    }

    if should_proxy_to_vite(path) {
        if let Some(socket_path) = &vite_socket_path {
            return proxy_to_vite_uds(socket_path, path_with_query).await;
        }
    }

    if vite_socket_path.is_some() {
        if let Some(static_response) = try_serve_static(&public_dir, path).await {
            return Ok(static_response);
        }
    }

    let mapped_req = req.map(|body| {
        UnsyncBoxBody::new(body)
            .map_err(|e| anyhow::anyhow!(e))
            .boxed_unsync()
    });

    if let Some(socket_path) = &vite_socket_path {
        let original_headers = mapped_req.headers().clone();

        let backend_response = match fn0.run_backend_only("app", mapped_req).await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Backend error: {:?}", e);
                return Err(anyhow::anyhow!("Backend error: {:?}", e));
            }
        };

        let backend_status = backend_response.status();

        if backend_status.is_redirection() {
            return Ok(backend_response);
        }

        if backend_status.is_client_error() || backend_status.is_server_error() {
            let (parts, body) = backend_response.into_parts();
            let body_bytes = body.collect().await?.to_bytes();
            let body_str = String::from_utf8_lossy(&body_bytes);
            if backend_status != StatusCode::NOT_FOUND {
                eprintln!("Backend error: {} {} - {}", backend_status, path, body_str);
            }

            return Ok(fn0::Response::from_parts(
                parts,
                UnsyncBoxBody::new(body_str.to_string())
                    .map_err(|e| anyhow::anyhow!(e))
                    .boxed_unsync(),
            ));
        }

        if path.starts_with("/__forte_hook/") || path.starts_with("/__forte_action/") {
            return Ok(backend_response);
        }

        let backend_set_cookies: Vec<_> = backend_response
            .headers()
            .get_all(http::header::SET_COOKIE)
            .iter()
            .cloned()
            .collect();

        let (_, body) = backend_response.into_parts();
        let body_bytes = body.collect().await?.to_bytes();
        let props: serde_json::Value = serde_json::from_slice(&body_bytes)?;

        let cookie_header = original_headers
            .get(http::header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let host = original_headers
            .get(http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");
        let full_url = format!("http://{}{}", host, uri);
        let mut ssr_response =
            call_vite_ssr_uds(socket_path, &full_url, props, cookie_header).await?;

        for cookie_value in backend_set_cookies {
            ssr_response
                .headers_mut()
                .append(http::header::SET_COOKIE, cookie_value);
        }

        return Ok(ssr_response);
    }

    match fn0.run("app", "", mapped_req, None).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            eprintln!("Request error: {:?}", e);
            Err(anyhow::anyhow!("Request error: {:?}", e))
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

fn should_proxy_to_vite(path: &str) -> bool {
    path.starts_with("/src/")
        || path.starts_with("/@vite/")
        || path.starts_with("/@id/")
        || path.starts_with("/@fs/")
        || path.starts_with("/__vite")
        || path.starts_with("/node_modules/")
        || path == "/@react-refresh"
}

#[cfg(unix)]
async fn proxy_to_vite_uds(socket_path: &Path, path: &str) -> Result<fn0::Response> {
    let mut stream = UnixStream::connect(socket_path).await?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        path
    );
    stream.write_all(request.as_bytes()).await?;

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes).await?;

    parse_http_response(&response_bytes)
}

#[cfg(unix)]
async fn call_vite_ssr_uds(
    socket_path: &Path,
    url: &str,
    props: serde_json::Value,
    cookie: Option<String>,
) -> Result<fn0::Response> {
    let mut stream = UnixStream::connect(socket_path).await?;

    let body = serde_json::json!({
        "url": url,
        "props": props,
        "cookie": cookie,
    })
    .to_string();

    let request = format!(
        "POST /__ssr_render HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(request.as_bytes()).await?;

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes).await?;

    parse_http_response(&response_bytes)
}

fn parse_http_response(response_bytes: &[u8]) -> Result<fn0::Response> {
    let response_str = String::from_utf8_lossy(response_bytes);

    let header_end = response_str
        .find("\r\n\r\n")
        .ok_or_else(|| anyhow::anyhow!("Invalid HTTP response"))?;

    let header_part = &response_str[..header_end];
    let body_start = header_end + 4;
    let raw_body = &response_bytes[body_start..];

    let mut lines = header_part.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing status line"))?;

    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);

    let mut builder = Response::builder().status(status_code);
    let mut is_chunked = false;

    for line in lines {
        if let Some((key, value)) = line.split_once(": ") {
            let key_lower = key.to_lowercase();
            if key_lower == "transfer-encoding" && value.to_lowercase().contains("chunked") {
                is_chunked = true;
            }
            if key_lower != "transfer-encoding" && key_lower != "content-length" {
                builder = builder.header(key, value);
            }
        }
    }

    let body = if is_chunked || looks_like_chunked(raw_body) {
        decode_chunked_body(raw_body)
    } else {
        raw_body.to_vec()
    };

    let body_bytes = bytes::Bytes::from(body);
    let body_len = body_bytes.len();
    Ok(builder
        .header(hyper::header::CONTENT_LENGTH, body_len.to_string())
        .body(
            Full::new(body_bytes)
                .map_err(|e| anyhow::anyhow!("{e}"))
                .boxed_unsync(),
        )?)
}

fn looks_like_chunked(data: &[u8]) -> bool {
    let crlf_pos = data.windows(2).position(|w| w == b"\r\n");
    if let Some(pos) = crlf_pos {
        if pos > 0 && pos <= 8 {
            let potential_size = &data[..pos];
            if let Ok(s) = std::str::from_utf8(potential_size) {
                return usize::from_str_radix(s.trim(), 16).is_ok();
            }
        }
    }
    false
}

fn decode_chunked_body(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        let chunk_size_end = data[pos..]
            .windows(2)
            .position(|w| w == b"\r\n")
            .map(|p| pos + p);

        let Some(chunk_size_end) = chunk_size_end else {
            break;
        };

        let chunk_size_str = String::from_utf8_lossy(&data[pos..chunk_size_end]);
        let chunk_size = usize::from_str_radix(chunk_size_str.trim(), 16).unwrap_or(0);

        if chunk_size == 0 {
            break;
        }

        let chunk_start = chunk_size_end + 2;
        let chunk_end = chunk_start + chunk_size;

        if chunk_end <= data.len() {
            result.extend_from_slice(&data[chunk_start..chunk_end]);
        }

        pos = chunk_end + 2;
    }

    result
}
