use std::sync::Arc;
use std::time::Duration;

use color_eyre::eyre::{Result, eyre};
use tokio::time::MissedTickBehavior;
use tracing::*;

use crate::args_parse::DeployContext;
use crate::doc_db::{DeployJob, DeployJobPhase};

const TICK_INTERVAL: Duration = Duration::from_secs(5);
const HEARTBEAT_STALE_SECS: i64 = 30;
const MAX_ATTEMPTS: u32 = 20;

pub async fn run(ctx: Arc<DeployContext>) {
    let mut interval = tokio::time::interval(TICK_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        if let Err(err) = tick(&ctx).await {
            warn!(%err, "deploy_job_worker tick failed");
        }
    }
}

async fn tick(ctx: &Arc<DeployContext>) -> Result<()> {
    let jobs = ctx.doc_db.list_active_deploy_jobs().await?;
    let now = chrono::Utc::now();
    for mut job in jobs {
        if !is_stale(&job, now, HEARTBEAT_STALE_SECS) {
            continue;
        }
        if !claim(ctx, &mut job, now).await? {
            continue;
        }
        run_until_blocked(ctx, job).await;
    }
    Ok(())
}

fn is_stale(job: &DeployJob, now: chrono::DateTime<chrono::Utc>, ttl_secs: i64) -> bool {
    match &job.heartbeat_at {
        None => true,
        Some(hb) => match chrono::DateTime::parse_from_rfc3339(hb) {
            Ok(ts) => (now.timestamp() - ts.timestamp()) >= ttl_secs,
            Err(_) => true,
        },
    }
}

async fn claim(
    ctx: &Arc<DeployContext>,
    job: &mut DeployJob,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<bool> {
    let now_str = now.to_rfc3339();
    job.heartbeat_at = Some(now_str.clone());
    job.updated_at = now_str;
    ctx.doc_db.update_deploy_job(job).await?;
    Ok(true)
}

async fn run_until_blocked(ctx: &Arc<DeployContext>, mut job: DeployJob) {
    loop {
        if job.is_terminal() {
            return;
        }
        match advance_one_phase(ctx, &mut job).await {
            Ok(true) => {
                // phase advanced; continue driving this job
                let now = chrono::Utc::now().to_rfc3339();
                job.heartbeat_at = Some(now.clone());
                job.updated_at = now;
                job.last_error = None;
                if let Err(err) = ctx.doc_db.update_deploy_job(&job).await {
                    warn!(%err, job_id = %job.job_id, "failed to persist deploy_job phase");
                    return;
                }
            }
            Ok(false) => return,
            Err(err) => {
                error!(%err, job_id = %job.job_id, phase = ?job.phase, "deploy_job phase failed");
                let now = chrono::Utc::now().to_rfc3339();
                job.attempts = job.attempts.saturating_add(1);
                job.last_error = Some(format!("{err:?}"));
                job.heartbeat_at = Some(now.clone());
                job.updated_at = now;
                if job.attempts >= MAX_ATTEMPTS {
                    job.phase = DeployJobPhase::Failed;
                }
                if let Err(err) = ctx.doc_db.update_deploy_job(&job).await {
                    warn!(%err, job_id = %job.job_id, "failed to persist deploy_job error");
                }
                return;
            }
        }
    }
}

async fn advance_one_phase(ctx: &Arc<DeployContext>, job: &mut DeployJob) -> Result<bool> {
    match job.phase {
        DeployJobPhase::Queued => {
            let env_key = crate::cwasm_compile::env_key(&job.subdomain);
            match &job.env_ciphertext {
                Some(b64) => {
                    use base64::{Engine, engine::general_purpose::STANDARD};
                    let ciphertext = STANDARD
                        .decode(b64.as_bytes())
                        .map_err(|e| eyre!("bad env ciphertext base64: {e}"))?;
                    ctx.aws_s3.write(&env_key, ciphertext).await?;
                }
                None => {
                    let _ = ctx.aws_s3.delete(&env_key).await;
                }
            }
            job.phase = DeployJobPhase::EnvUploaded;
            Ok(true)
        }
        DeployJobPhase::EnvUploaded => {
            run_cwasm_compile(ctx, job).await?;
            job.phase = DeployJobPhase::CwasmCompiled;
            Ok(true)
        }
        DeployJobPhase::CwasmCompiled => {
            crate::turso_admin::ensure_database(&ctx.forte_db, &job.subdomain).await?;
            job.phase = DeployJobPhase::DbEnsured;
            Ok(true)
        }
        DeployJobPhase::DbEnsured => {
            if job.code_version.is_none() {
                let v = ctx.doc_db.next_code_version(job.code_id).await?;
                job.code_version = Some(v);
            }
            job.phase = DeployJobPhase::Versioned;
            Ok(true)
        }
        DeployJobPhase::Versioned => {
            let code_version = job
                .code_version
                .ok_or_else(|| eyre!("versioned phase without code_version"))?;
            let exists = ctx
                .doc_db
                .deployment_exists(&job.subdomain, job.code_id, code_version)
                .await?;
            if !exists {
                ctx.doc_db
                    .insert_deployment(&job.subdomain, job.code_id, code_version)
                    .await?;
            }
            job.phase = DeployJobPhase::Deployed;
            Ok(true)
        }
        DeployJobPhase::Deployed => {
            if let Some(build_id) = job.build_id.clone() {
                if job.old_build_ids.is_none() {
                    let mut olds = ctx
                        .doc_db
                        .register_build(&job.subdomain, &build_id)
                        .await?;
                    olds.retain(|id| id != &build_id);
                    job.old_build_ids = Some(olds);
                }
            }
            job.phase = DeployJobPhase::BuildRegistered;
            Ok(true)
        }
        DeployJobPhase::BuildRegistered => {
            if let Some(olds) = job.old_build_ids.clone() {
                for old in olds {
                    if let Err(err) = ctx
                        .doc_db
                        .enqueue_r2_prefix_delete(&job.subdomain, &old)
                        .await
                    {
                        warn!(%err, %old, "enqueue r2 prefix delete failed; will retry next phase tick");
                        return Err(err.into());
                    }
                }
            }
            job.phase = DeployJobPhase::R2GcEnqueued;
            Ok(true)
        }
        DeployJobPhase::R2GcEnqueued => {
            ctx.deployment_cache.refresh().await;
            if job.generation.is_none() {
                job.generation = Some(ctx.deployment_cache.last_deployment_id());
            }
            crate::deploy::spawn_immediate_push(ctx).await;
            job.phase = DeployJobPhase::Pushed;
            Ok(true)
        }
        DeployJobPhase::Pushed => {
            job.phase = DeployJobPhase::Done;
            Ok(true)
        }
        DeployJobPhase::Done | DeployJobPhase::Failed => Ok(false),
    }
}

async fn run_cwasm_compile(ctx: &Arc<DeployContext>, job: &DeployJob) -> Result<()> {
    let target_versions = collect_target_versions(ctx).await?;
    for version in &target_versions {
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
            fn0_wasmtime_version: version,
        };
        crate::cwasm_compile::compile_and_publish(
            &compile_ctx,
            &job.subdomain,
            job.env_ciphertext.is_some(),
        )
        .await?;
    }
    Ok(())
}

async fn collect_target_versions(
    ctx: &Arc<DeployContext>,
) -> Result<std::collections::BTreeSet<String>> {
    let mut versions = std::collections::BTreeSet::new();
    for site in &ctx.sites {
        let target = ctx
            .doc_db
            .get_worker_target(site.name())
            .await?
            .ok_or_else(|| eyre!("worker-target not set for site '{}'", site.name()))?;
        versions.insert(target.fn0_wasmtime_version);
        if let Some(last) = ctx.doc_db.get_worker_last_stable(site.name()).await? {
            versions.insert(last.fn0_wasmtime_version);
        }
    }
    Ok(versions)
}

