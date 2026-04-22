use crate::args_parse::DeployContext;
use crate::deployment_cache::DeploymentId;
use crate::doc_db::{CwasmReady, Deployment};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::MissedTickBehavior;
use tracing::*;

const ROLLOUT_INTERVAL: Duration = Duration::from_secs(30);
const FANOUT_MAX_CONCURRENCY: usize = 4;

pub async fn run(ctx: Arc<DeployContext>, site_names: Vec<String>) {
    let mut interval = tokio::time::interval(ROLLOUT_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        for site_name in &site_names {
            if let Err(err) = reconcile_site(&ctx, site_name).await {
                warn!(%err, site = %site_name, "wasmtime_rollout: reconcile failed");
            }
        }
    }
}

async fn reconcile_site(ctx: &Arc<DeployContext>, site_name: &str) -> color_eyre::Result<()> {
    let target = match ctx.doc_db.get_worker_target(site_name).await? {
        Some(t) => t,
        None => return Ok(()),
    };
    let ready = ctx.doc_db.get_cwasm_ready(site_name).await?;

    if let Some(r) = &ready
        && r.fn0_wasmtime_version == target.fn0_wasmtime_version
    {
        return Ok(());
    }

    info!(
        site = %site_name,
        target_version = %target.fn0_wasmtime_version,
        current_ready = ?ready.as_ref().map(|r| r.fn0_wasmtime_version.as_str()),
        "Starting cwasm fan-out for new wasmtime version"
    );

    let active_subdomains = active_subdomains(ctx);

    if active_subdomains.is_empty() {
        ctx.doc_db
            .set_cwasm_ready(
                site_name,
                &CwasmReady {
                    fn0_wasmtime_version: target.fn0_wasmtime_version.clone(),
                },
            )
            .await?;
        info!(site = %site_name, "Fan-out trivially complete (no active subdomains)");
        return Ok(());
    }

    let semaphore = Arc::new(tokio::sync::Semaphore::new(FANOUT_MAX_CONCURRENCY));
    let mut handles = Vec::with_capacity(active_subdomains.len());
    for subdomain in active_subdomains {
        let ctx = ctx.clone();
        let version = target.fn0_wasmtime_version.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        handles.push(tokio::spawn(async move {
            let _permit = permit;
            let env_present = ctx
                .aws_s3
                .stat(&crate::cwasm_compile::env_key(&subdomain))
                .await
                .is_ok();
            let compile_ctx = crate::cwasm_compile::CompileContext {
                lambda_client: &ctx.lambda_client,
                wasm_bucket: crate::cwasm_compile::BucketRef {
                    op: &ctx.aws_s3,
                    name: &ctx.wasm_bucket,
                },
                cwasm_bucket: crate::cwasm_compile::BucketRef {
                    op: &ctx.cwasm_s3,
                    name: &ctx.cwasm_bucket,
                },
                fn0_wasmtime_version: &version,
            };
            let result =
                crate::cwasm_compile::compile_and_publish(&compile_ctx, &subdomain, env_present)
                    .await;
            (subdomain, result)
        }));
    }

    let mut all_ok = true;
    for handle in handles {
        match handle.await {
            Ok((subdomain, Ok(()))) => {
                info!(%subdomain, version = %target.fn0_wasmtime_version, "compiled");
            }
            Ok((subdomain, Err(err))) => {
                warn!(%err, %subdomain, version = %target.fn0_wasmtime_version, "compile failed");
                all_ok = false;
            }
            Err(err) => {
                warn!(%err, "fan-out task panicked");
                all_ok = false;
            }
        }
    }

    if all_ok {
        ctx.doc_db
            .set_cwasm_ready(
                site_name,
                &CwasmReady {
                    fn0_wasmtime_version: target.fn0_wasmtime_version.clone(),
                },
            )
            .await?;
        info!(
            site = %site_name,
            version = %target.fn0_wasmtime_version,
            "cwasm fan-out complete"
        );
    } else {
        warn!(
            site = %site_name,
            version = %target.fn0_wasmtime_version,
            "cwasm fan-out had failures; will retry next tick"
        );
    }

    Ok(())
}

fn active_subdomains(ctx: &Arc<DeployContext>) -> Vec<String> {
    let snapshot = ctx.deployment_cache.slice_updates(DeploymentId(0));
    let mut subs = Vec::new();
    for d in snapshot {
        if let Deployment::Deploy { subdomain, .. } = d {
            subs.push(subdomain);
        }
    }
    subs
}
