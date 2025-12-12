mod execute;
pub mod telemetry;

pub use bytes::Bytes;
use execute::Job;
use http_body_util::BodyExt;
pub use http_body_util::Full;
use hyper::Request;
use hyper::server::conn::http1;
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use measure_cpu_time::SystemClock;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
pub use wasmtime_wasi_http::{bindings::http::types::ErrorCode, body::HyperOutgoingBody};

pub struct Config {
    pub port: Option<u16>,
    pub wasm_path: Option<PathBuf>,
    pub otlp_endpoint: Option<String>,
}

pub async fn run<F>(config: Config, on_ready: F) -> color_eyre::Result<()>
where
    F: FnOnce(u16),
{
    let wasm_path = config.wasm_path.clone();
    let proxy_cache = match &config.wasm_path {
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

    // Setup telemetry
    let telemetry_providers = telemetry::setup_telemetry(config.otlp_endpoint)?;

    tokio::spawn({
        async move {
            execute::Executor::new(proxy_cache, job_rx, SystemClock, false)
                .run()
                .await;
        }
    });

    let listener = {
        let port = config.port.unwrap_or(3000);
        let increment_on_fail = config.port.is_none();
        open_tcp_listener(port, increment_on_fail)?
    };

    let actual_port = listener.local_addr()?.port();

    let code_id = wasm_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .ok_or_else(|| color_eyre::eyre::eyre!("Invalid wasm_path"))?
        .to_string();

    on_ready(actual_port);

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Failed to accept connection: {:?}", e);
                continue;
            }
        };

        let tower_service = ServiceBuilder::new().service(tower::util::service_fn({
            let code_id = code_id.clone();
            let job_tx = job_tx.clone();

            move |req: Request<hyper::body::Incoming>| {
                let code_id = code_id.clone();
                let job_tx = job_tx.clone();

                async move {
                    let (res_tx, res_rx) = tokio::sync::oneshot::channel();
                    if job_tx
                        .send(Job {
                            req,
                            res_tx,
                            code_id: code_id.clone(),
                        })
                        .await
                        .is_err()
                    {
                        telemetry::oneshot_drop_before_response(&code_id);
                        return Ok::<_, std::convert::Infallible>(internal_error_response());
                    };

                    match res_rx.await {
                        Ok(res) => Ok(res),
                        Err(_err) => Ok(internal_error_response()),
                    }
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

    telemetry::shutdown_telemetry(telemetry_providers)?;
    Ok(())
}

fn internal_error_response() -> execute::Response {
    let body = http_body_util::Full::new(Bytes::from("Internal Server Error"))
        .map_err(|_| ErrorCode::InternalError(None));
    let mut res = hyper::Response::new(HyperOutgoingBody::new(body));
    *res.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
    res
}

fn open_tcp_listener(mut port: u16, increment_on_fail: bool) -> color_eyre::Result<TcpListener> {
    loop {
        let socket = try_open_tcp_listener(port);
        if !increment_on_fail || socket.is_ok() {
            return socket.map_err(Into::into);
        }
        if port == u16::MAX {
            return Err(color_eyre::eyre::eyre!("Failed to open socket"));
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
