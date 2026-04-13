mod args;
mod args_parse;
mod deploy;
mod deployment_cache;
mod dns;
mod dns_sync;
mod host_connection;
mod host_id;
mod host_provider;
mod job_processor;
mod random_sleep;
mod self_dns;
mod site;
mod ssh;
mod telemetry;
mod wasmtime_migration;

use args::HqArgs;
use args_parse::DeployContext;
use color_eyre::eyre::{Result, eyre};
use host_id::*;
use host_provider::*;
use http_body_util::Full;
use hyper::{Method, Request, Response, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, task::JoinSet};
use tracing::*;

use crate::args_parse::HqArgsParsed;

fn main() -> Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async move {
        let telemetry_providers = telemetry::setup_otlp()?;
        let HqArgsParsed {
            sites,
            deployment_cache,
            deploy_context,
            self_dns_args,
            dns_provider,
        } = HqArgs::parse().await?;

        self_dns::register(self_dns_args).await?;

        deploy_context.doc_db.ensure_job_tables().await?;
        deploy_context.doc_db.ensure_wasmtime_tables().await?;

        let initial_deployments = deploy_context
            .doc_db
            .all_deployments()
            .await
            .unwrap_or_default();
        if let Err(err) = wasmtime_migration::ensure_migration(
            &deploy_context.doc_db,
            &initial_deployments,
        )
        .await
        {
            warn!(%err, "Failed to ensure wasmtime migration on startup");
        }

        let mut set = JoinSet::new();

        set.spawn(async move {
            deployment_cache.run_sync().await;
            Ok(())
        });

        {
            let doc_db = deploy_context.doc_db.clone();
            let s3_client = deploy_context.s3_client.clone();
            let wasm_bucket = deploy_context.wasm_bucket.clone();
            let cwasm_bucket = deploy_context.cwasm_bucket.clone();
            set.spawn(async move {
                job_processor::run(doc_db, s3_client, wasm_bucket, cwasm_bucket).await;
                Ok(())
            });
        }
        let all_host_connections: Vec<_> = sites
            .iter()
            .map(|s| s.host_connections.clone())
            .collect();

        for mut site in sites {
            set.spawn(async move {
                site.run().await;
                Ok(())
            });
        }

        set.spawn(async move {
            dns_sync::run(dns_provider, all_host_connections).await;
            Ok(())
        });
        set.spawn(async {
            tokio::signal::ctrl_c().await?;
            Ok(())
        });
        set.spawn(web_server(deploy_context));

        let result = set.join_next().await.unwrap().map_err(|err| eyre!(err));

        telemetry::on_shutdown(telemetry_providers)?;

        result
    })?
}

async fn web_server(deploy_context: Arc<DeployContext>) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let ctx = deploy_context.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req| {
                    let ctx = ctx.clone();
                    async move { route(req, ctx).await }
                }))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn route(
    req: Request<hyper::body::Incoming>,
    ctx: Arc<DeployContext>,
) -> Result<Response<Full<Bytes>>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => {
            info!("health check");
            Ok(Response::new(Full::new(Bytes::from("ok"))))
        }
        (&Method::POST, "/deploy/start") => {
            Ok(deploy::handle_deploy_start(req, ctx).await)
        }
        (&Method::POST, "/deploy/finish") => {
            Ok(deploy::handle_deploy_finish(req, ctx).await)
        }
        (&Method::POST, "/deploy/destroy") => {
            Ok(deploy::handle_deploy_destroy(req, ctx).await)
        }
        _ => Ok(Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("not found")))
            .unwrap()),
    }
}
