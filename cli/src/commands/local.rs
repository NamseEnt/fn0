use color_eyre::{eyre::eyre, Result};
use fn0::Fn0;
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, server::conn::http1, Request, Response, StatusCode};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use mime_guess::from_path;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use wasmtime_wasi_http::body::HyperOutgoingBody;
use wasmtime_wasi_http::bindings::http::types::ErrorCode;

pub async fn execute(port: Option<u16>) -> Result<()> {
    println!("Starting local fn0 server...\n");

    let wasm_file = PathBuf::from("./dist/component.wasm");

    crate::commands::build::execute().await?;

    let cli_config = crate::config::Config::load("fn0.toml").map_err(|e| {
        eyre!(
            "Failed to load fn0.toml: {}. Make sure you're in a fn0 project directory.",
            e
        )
    })?;

    let static_dir = match cli_config.language_env {
        crate::config::LanguageEnvironment::TypescriptBunHono => None,
        crate::config::LanguageEnvironment::TypescriptBunAstro => Some(PathBuf::from("dist/client")),
    };

    let config = fn0::Config {
        port,
        wasm_path: Some(wasm_file),
        otlp_endpoint: None,
    };

    println!("\nServer starting...\n");

    let listener = {
        let port = config.port.unwrap_or(3000);
        let increment_on_fail = config.port.is_none();
        open_tcp_listener(port, increment_on_fail)?
    };

    println!(
        "Server on http://localhost:{}",
        listener.local_addr().unwrap().port()
    );

    let code_id = config
        .wasm_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .ok_or_else(|| eyre!("Invalid wasm_path"))?
        .to_string();

    let fn0 = std::sync::Arc::new(Fn0::new(config).await?);

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };

        let tower_service = ServiceBuilder::new().service(tower::util::service_fn({
            let code_id = code_id.clone();
            let fn0 = fn0.clone();
            let static_dir = static_dir.clone();

            move |req: Request<hyper::body::Incoming>| {
                let code_id = code_id.clone();
                let fn0 = fn0.clone();
                let static_dir = static_dir.clone();

                async move {
                    if let Some(ref dir) = static_dir {
                        if let Some(static_resp) = try_serve_static(dir, req.uri().path()).await {
                            return Ok(static_resp);
                        }
                    }

                    fn0.run(code_id, req).await
                }
            }
        }));

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    TokioIo::new(stream),
                    TowerToHyperService::new(tower_service),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn try_serve_static(
    base_dir: &Path,
    req_path: &str,
) -> Option<Response<HyperOutgoingBody>> {
    let path_str = req_path.trim_start_matches('/');
    let mut file_path = base_dir.join(path_str);

    if let Ok(canonical_base) = base_dir.canonicalize() {
        if let Ok(canonical_file) = file_path.canonicalize() {
            if !canonical_file.starts_with(&canonical_base) {
                return None;
            }
        }
    }

    if file_path.is_dir() {
        file_path.push("index.html");
    }

    if let Ok(file_content) = tokio::fs::read(&file_path).await {
        let mime_type = from_path(&file_path).first_or_octet_stream();

        let body = Full::new(Bytes::from(file_content))
            .map_err(|_| ErrorCode::InternalError(None));

        let mut response = Response::new(HyperOutgoingBody::new(body));
        *response.status_mut() = StatusCode::OK;
        response.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            mime_type.as_ref().parse().ok()?,
        );

        if path_str.starts_with("_astro/") {
            response.headers_mut().insert(
                hyper::header::CACHE_CONTROL,
                "public, max-age=31536000, immutable".parse().ok()?,
            );
        }

        return Some(response);
    }

    None
}

fn open_tcp_listener(mut port: u16, increment_on_fail: bool) -> Result<TcpListener> {
    loop {
        let socket = try_open_tcp_listener(port);
        if !increment_on_fail || socket.is_ok() {
            return socket.map_err(Into::into);
        }
        if port == u16::MAX {
            return Err(eyre!("Failed to open socket"));
        }
        port += 1;
    }
}

fn try_open_tcp_listener(port: u16) -> std::io::Result<TcpListener> {
    let addr: SocketAddr = format!("[::]:{port}").parse().unwrap();
    let domain = Domain::for_address(addr);
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;

    socket.set_only_v6(false)?;
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    socket.listen(128)?;

    let std_listener: std::net::TcpListener = socket.into();
    std_listener.set_nonblocking(true)?;
    let listener = TcpListener::from_std(std_listener)?;

    Ok(listener)
}
