mod admin;
mod args;
mod args_parse;
mod cwasm_compile;
mod deploy;
mod deploy_job_worker;
mod rename;
mod rename_job_worker;
mod deployment_cache;
mod dns;
mod dns_sync;
mod doc_db;
mod docs;
mod env_crypto;
mod forte_r2;
mod host_id;
mod host_provider;
mod job_processor;
mod lambda;
mod self_dns;
mod site;
mod ssh;
mod ssh_pool;
mod telemetry;
mod turso_admin;
mod wasmtime_rollout;

use args::HqArgs;
use args_parse::DeployContext;
use color_eyre::eyre::{Result, eyre};
use http_body_util::Full;
use hyper::{Method, Request, Response, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, task::JoinSet};
use tracing::*;

use crate::args_parse::HqArgsParsed;

fn main() -> Result<()> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
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

        let mut set = JoinSet::new();

        {
            let deployment_cache = deployment_cache.clone();
            set.spawn(async move {
                deployment_cache.run_sync().await;
                Ok(())
            });
        }

        {
            let doc_db = deploy_context.doc_db.clone();
            let s3 = deploy_context.cwasm_s3.clone();
            let forte_r2 = deploy_context.forte_r2.clone();
            set.spawn(async move {
                job_processor::run(doc_db, s3, forte_r2).await;
                Ok(())
            });
        }

        let site_names: Vec<String> = sites.iter().map(|s| s.name().to_string()).collect();

        for site in sites {
            set.spawn(async move {
                site.run().await;
                Ok(())
            });
        }

        {
            let doc_db = deploy_context.doc_db.clone();
            let deployment_cache = deployment_cache.clone();
            let names = site_names.clone();
            set.spawn(async move {
                dns_sync::run(dns_provider, doc_db, names, deployment_cache).await;
                Ok(())
            });
        }

        {
            let ctx = deploy_context.clone();
            let names = site_names.clone();
            set.spawn(async move {
                wasmtime_rollout::run(ctx, names).await;
                Ok(())
            });
        }

        {
            let ctx = deploy_context.clone();
            set.spawn(async move {
                deploy_job_worker::run(ctx).await;
                Ok(())
            });
        }

        {
            let ctx = deploy_context.clone();
            set.spawn(async move {
                rename_job_worker::run(ctx).await;
                Ok(())
            });
        }

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
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        let ctx = ctx.clone();
                        async move { route(req, ctx).await }
                    }),
                )
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
        (&Method::POST, "/admin/grant") => Ok(admin::handle_admin_grant(req, ctx).await),
        (&Method::POST, "/deploy/start") => Ok(deploy::handle_deploy_start(req, ctx).await),
        (&Method::POST, "/deploy/r2/sign") => Ok(deploy::handle_deploy_r2_sign(req, ctx).await),
        (&Method::POST, "/deploy/finish") => Ok(deploy::handle_deploy_finish(req, ctx).await),
        (&Method::POST, "/deploy/destroy") => Ok(deploy::handle_deploy_destroy(req, ctx).await),
        (&Method::GET, "/deploy/status") => Ok(deploy::handle_deploy_status(req, ctx).await),
        (&Method::POST, "/rename/start") => Ok(rename::handle_rename_start(req, ctx).await),
        (&Method::GET, "/rename/status") => Ok(rename::handle_rename_status(req, ctx).await),
        _ => Ok(Response::builder()
            .status(404)
            .body(Full::new(Bytes::from("not found")))
            .unwrap()),
    }
}
