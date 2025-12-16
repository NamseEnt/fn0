mod dns;
mod health_checker;
mod host_id;
mod host_infra;
mod reaper;
mod telemetry;

use color_eyre::eyre::Result;
use dashmap::DashMap;
use health_checker::*;
use host_id::*;
use host_infra::*;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let provider = telemetry::setup_otlp()?;

    rt.block_on(async move {
        // let host_infra = Arc::new(host_infra::oci::OciHostInfra::new());
        // let host_info_map = Arc::new(DashMap::new());
        // let health_check_map = Arc::new(DashMap::new());

        let shutdown_signal = async {
            tokio::signal::ctrl_c().await?;
            Ok(())
        };

        // let sync_host_info_map_future =
        //     host_infra::run_sync_host_info_map(host_infra.clone(), host_info_map.clone());
        // let health_checker_future =
        //     health_checker::run(host_info_map.clone(), health_check_map.clone());
        // let reaper_future = reaper::run(
        //     host_infra.clone(),
        //     host_info_map.clone(),
        //     health_check_map.clone(),
        // );
        // let dns_sync_ips_future = dns::sync_ips(health_check_map.clone());

        let result = tokio::select! {
            result = shutdown_signal => { result }
            result = web_server() => { result }
            // result = sync_host_info_map_future => { result }
            // result = health_checker_future => { result }
            // result = reaper_future => { result }
            // result = dns_sync_ips_future => { result }
        };

        telemetry::on_shutdown(provider)?;

        result
    })?;
    Ok(())
}

async fn web_server() -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
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

async fn route(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>> {
    match req.uri().path() {
        "/health" => Ok(Response::new(Full::new(Bytes::from("ok")))),
        _ => Ok(Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("not found")))
            .unwrap()),
    }
}
