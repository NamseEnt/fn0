mod args;
mod args_parse;
mod deployment_cache;
mod dns;
mod host_connection;
mod host_id;
mod host_provider;
mod random_sleep;
mod site;
mod telemetry;

use args::HqArgs;
use color_eyre::eyre::{Result, eyre};
use host_id::*;
use host_provider::*;
use http_body_util::Full;
use hyper::{Request, Response, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, task::JoinSet};
use tracing::*;

use crate::args_parse::HqArgsParsed;

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async move {
        let telemetry_providers = telemetry::setup_otlp()?;
        let HqArgsParsed {
            sites,
            deployment_cache,
        } = HqArgs::parse().await?;

        let mut set = JoinSet::new();

        set.spawn(async move {
            deployment_cache.run_sync().await;
            Ok(())
        });
        for mut site in sites {
            set.spawn(async move {
                site.run().await;
                Ok(())
            });
        }
        set.spawn(async {
            tokio::signal::ctrl_c().await?;
            Ok(())
        });
        set.spawn(web_server());

        let result = set.join_next().await.unwrap().map_err(|err| eyre!(err));

        telemetry::on_shutdown(telemetry_providers)?;

        result
    })?
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
        "/health" => {
            info!("health check");
            Ok(Response::new(Full::new(Bytes::from("ok"))))
        }
        _ => Ok(Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("not found")))
            .unwrap()),
    }
}
