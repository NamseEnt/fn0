use color_eyre::{eyre::eyre, Result};
use fn0::Fn0;
use http_body_util::BodyExt;
use hyper::{server::conn::http1, Request, StatusCode};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower::ServiceExt;
use tower_http::services::ServeDir;

pub async fn execute(port: Option<u16>, static_dir: Option<PathBuf>) -> Result<()> {
    println!("Starting local fn0 server...\n");

    let wasm_file = PathBuf::from("./dist/component.wasm");

    crate::commands::build::execute().await?;

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

    let static_service = static_dir.as_ref().map(|dir| ServeDir::new(dir));

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };

        let tower_service = ServiceBuilder::new().service(tower::util::service_fn({
            let code_id = code_id.clone();
            let fn0 = fn0.clone();
            let static_service = static_service.clone();

            move |req: Request<hyper::body::Incoming>| {
                let code_id = code_id.clone();
                let fn0 = fn0.clone();
                let static_service = static_service.clone();

                async move {
                    // Check if this is a static asset request (has file extension, not HTML)
                    let path = req.uri().path();
                    let is_static_asset = static_service.is_some()
                        && path.contains('.')
                        && !path.ends_with(".html")
                        && req.method() == hyper::Method::GET;

                    if is_static_asset {
                        // Try to serve static file
                        if let Some(service) = static_service {
                            match service.oneshot(req).await {
                                Ok(res) if res.status() != StatusCode::NOT_FOUND => {
                                    // File found, buffer the body and convert
                                    let (res_parts, res_body) = res.into_parts();

                                    match res_body.collect().await {
                                        Ok(collected) => {
                                            let bytes = collected.to_bytes();
                                            let body = fn0::Full::new(bytes)
                                                .map_err(|_| fn0::ErrorCode::InternalError(None));
                                            let wrapped = fn0::HyperOutgoingBody::new(body);
                                            return Ok(hyper::Response::from_parts(
                                                res_parts, wrapped,
                                            ));
                                        }
                                        Err(_) => {
                                            // Error reading body
                                            let body = fn0::Full::new(fn0::Bytes::from(
                                                "Error reading file",
                                            ))
                                            .map_err(|_| fn0::ErrorCode::InternalError(None));
                                            let wrapped = fn0::HyperOutgoingBody::new(body);
                                            let mut res = hyper::Response::new(wrapped);
                                            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                            return Ok(res);
                                        }
                                    }
                                }
                                _ => {
                                    // Static file not found, but req is consumed
                                    // Return 404 for static assets
                                    let body = fn0::Full::new(fn0::Bytes::from("Not Found"))
                                        .map_err(|_| fn0::ErrorCode::InternalError(None));
                                    let wrapped = fn0::HyperOutgoingBody::new(body);
                                    let mut res = hyper::Response::new(wrapped);
                                    *res.status_mut() = StatusCode::NOT_FOUND;
                                    return Ok(res);
                                }
                            }
                        }
                    }

                    // All other requests go to WASM (HTML, API, etc.)
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
