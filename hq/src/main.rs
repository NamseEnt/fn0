mod watchdog;

use color_eyre::eyre::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use watchdog::host_infra::HostInfra;

fn main() -> Result<()> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async move {
            tokio::try_join!(web_server(), run_watchdog_loop())
        })?;
    Ok(())
}

async fn run_watchdog_loop() -> Result<()> {
    let host_infra = Arc::new(watchdog::host_infra::oci::OciHostInfra::new().await);

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let host_infra_clone = host_infra.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            match host_infra_clone.get_host_infos().await {
                Ok(host_infos) => {
                    if let Err(e) = tx.send(host_infos).await {
                        eprintln!("Failed to send host infos: {e}");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get host infos: {e:?}");
                }
            }
        }
    });

    let health_recorder = Arc::new(watchdog::health_recorder::InMemoryHealthRecorder::new());
    let dns = Arc::new(watchdog::dns::cloudflare::CloudflareDns::new(None).await);

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            }
            _ = async {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
            } => {}
        }
    }

    Ok(())
}

async fn web_server() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(route))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn route(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    match req.uri().path() {
        "/health" => Ok(Response::new(Full::new(Bytes::from("ok")))),
        _ => Ok(Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("not found")))
            .unwrap()),
    }
}
