use std::sync::Arc;
use std::time::Duration;

use color_eyre::eyre::{Result, eyre};
use tokio::time::MissedTickBehavior;
use tracing::*;

use crate::args_parse::DeployContext;
use crate::doc_db::{RenameJob, RenameJobPhase};

const TICK_INTERVAL: Duration = Duration::from_secs(5);
const HEARTBEAT_STALE_SECS: i64 = 30;
const MAX_ATTEMPTS: u32 = 20;
const VERIFY_TIMEOUT_SECS: i64 = 120;

pub async fn run(ctx: Arc<DeployContext>) {
    let mut interval = tokio::time::interval(TICK_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        if let Err(err) = tick(&ctx).await {
            warn!(%err, "rename_job_worker tick failed");
        }
    }
}

async fn tick(ctx: &Arc<DeployContext>) -> Result<()> {
    let jobs = ctx.doc_db.list_active_rename_jobs().await?;
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

fn is_stale(job: &RenameJob, now: chrono::DateTime<chrono::Utc>, ttl_secs: i64) -> bool {
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
    job: &mut RenameJob,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<bool> {
    let now_str = now.to_rfc3339();
    job.heartbeat_at = Some(now_str.clone());
    job.updated_at = now_str;
    ctx.doc_db.update_rename_job(job).await?;
    Ok(true)
}

async fn run_until_blocked(ctx: &Arc<DeployContext>, mut job: RenameJob) {
    loop {
        if job.is_terminal() {
            return;
        }
        match advance_one_phase(ctx, &mut job).await {
            Ok(true) => {
                let now = chrono::Utc::now().to_rfc3339();
                job.heartbeat_at = Some(now.clone());
                job.updated_at = now;
                job.last_error = None;
                if let Err(err) = ctx.doc_db.update_rename_job(&job).await {
                    warn!(%err, job_id = %job.job_id, "failed to persist rename_job phase");
                    return;
                }
            }
            Ok(false) => return,
            Err(err) => {
                error!(%err, job_id = %job.job_id, phase = ?job.phase, "rename_job phase failed");
                let now = chrono::Utc::now().to_rfc3339();
                job.attempts = job.attempts.saturating_add(1);
                job.last_error = Some(format!("{err:?}"));
                job.heartbeat_at = Some(now.clone());
                job.updated_at = now;
                if job.attempts >= MAX_ATTEMPTS {
                    job.phase = RenameJobPhase::Failed;
                }
                if let Err(err) = ctx.doc_db.update_rename_job(&job).await {
                    warn!(%err, job_id = %job.job_id, "failed to persist rename_job error");
                }
                return;
            }
        }
    }
}

async fn advance_one_phase(ctx: &Arc<DeployContext>, job: &mut RenameJob) -> Result<bool> {
    match job.phase {
        RenameJobPhase::Queued => {
            if job.code_version.is_none() {
                let info = ctx
                    .doc_db
                    .latest_active_deployment_for(&job.old_subdomain)
                    .await?;
                let (code_id, code_version) = info.ok_or_else(|| {
                    eyre!(
                        "no active deployment found for old subdomain '{}'",
                        job.old_subdomain
                    )
                })?;
                job.code_id = code_id;
                job.code_version = Some(code_version);
            }
            if job.build_id.is_none() {
                job.build_id = ctx.doc_db.get_forte_build(&job.old_subdomain).await?;
            }
            job.phase = RenameJobPhase::OldWritesBlocked;
            Ok(true)
        }
        RenameJobPhase::OldWritesBlocked => {
            crate::turso_admin::set_block_writes(&ctx.forte_db, &job.old_subdomain, true).await?;
            job.phase = RenameJobPhase::NewDbSeeded;
            Ok(true)
        }
        RenameJobPhase::NewDbSeeded => {
            crate::turso_admin::create_database_from(
                &ctx.forte_db,
                &job.new_subdomain,
                &job.old_subdomain,
            )
            .await?;
            job.phase = RenameJobPhase::NewEnvUploaded;
            Ok(true)
        }
        RenameJobPhase::NewEnvUploaded => {
            copy_bundle_object(
                ctx,
                &crate::cwasm_compile::raw_key(&job.old_subdomain),
                &crate::cwasm_compile::raw_key(&job.new_subdomain),
                true,
            )
            .await?;
            let env_present = copy_bundle_object(
                ctx,
                &crate::cwasm_compile::env_key(&job.old_subdomain),
                &crate::cwasm_compile::env_key(&job.new_subdomain),
                false,
            )
            .await?;
            if !env_present {
                job.env_ciphertext = None;
            } else {
                job.env_ciphertext = Some(String::new());
            }
            job.phase = RenameJobPhase::NewCwasmCompiled;
            Ok(true)
        }
        RenameJobPhase::NewCwasmCompiled => {
            let target_versions = collect_target_versions(ctx).await?;
            let env_present = job.env_ciphertext.is_some();
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
                    &job.new_subdomain,
                    env_present,
                )
                .await?;
            }
            job.phase = RenameJobPhase::NewVersioned;
            Ok(true)
        }
        RenameJobPhase::NewVersioned => {
            let v = ctx.doc_db.next_code_version(job.code_id).await?;
            job.code_version = Some(v);
            job.phase = RenameJobPhase::NewDeploymentInserted;
            Ok(true)
        }
        RenameJobPhase::NewDeploymentInserted => {
            let code_version = job
                .code_version
                .ok_or_else(|| eyre!("NewDeploymentInserted phase without code_version"))?;
            let exists = ctx
                .doc_db
                .deployment_exists(&job.new_subdomain, job.code_id, code_version)
                .await?;
            if !exists {
                ctx.doc_db
                    .insert_deployment(&job.new_subdomain, job.code_id, code_version)
                    .await?;
            }
            job.phase = RenameJobPhase::NewBuildRegistered;
            Ok(true)
        }
        RenameJobPhase::NewBuildRegistered => {
            if let Some(build_id) = job.build_id.clone() {
                if job.old_build_ids.is_none() {
                    let mut olds = ctx
                        .doc_db
                        .register_build(&job.new_subdomain, &build_id)
                        .await?;
                    olds.retain(|id| id != &build_id);
                    job.old_build_ids = Some(olds);
                }
            }
            job.phase = RenameJobPhase::NewDeploymentPushed;
            Ok(true)
        }
        RenameJobPhase::NewDeploymentPushed => {
            ctx.deployment_cache.refresh().await;
            if job.generation.is_none() {
                job.generation = Some(ctx.deployment_cache.last_deployment_id());
            }
            crate::deploy::spawn_immediate_push(ctx).await;
            job.phase = RenameJobPhase::NewDeploymentVerified;
            Ok(true)
        }
        RenameJobPhase::NewDeploymentVerified => {
            let target_generation = job
                .generation
                .ok_or_else(|| eyre!("verify phase without generation"))?;
            if all_sites_have_verified_host(ctx, target_generation).await? {
                job.phase = RenameJobPhase::ProjectRenamed;
                return Ok(true);
            }
            let started_ts = chrono::DateTime::parse_from_rfc3339(&job.updated_at)
                .map(|t| t.timestamp())
                .unwrap_or_else(|_| chrono::Utc::now().timestamp());
            let now_ts = chrono::Utc::now().timestamp();
            if now_ts - started_ts >= VERIFY_TIMEOUT_SECS {
                warn!(
                    job_id = %job.job_id,
                    "rename verify timeout reached; advancing without all-sites verification"
                );
                job.phase = RenameJobPhase::ProjectRenamed;
                return Ok(true);
            }
            Ok(false)
        }
        RenameJobPhase::ProjectRenamed => {
            ctx.doc_db
                .rename_project(
                    &job.github_username,
                    &job.old_project_name,
                    &job.new_project_name,
                )
                .await?;
            job.phase = RenameJobPhase::OldUndeployed;
            Ok(true)
        }
        RenameJobPhase::OldUndeployed => {
            let job_id = format!("rename-undeploy-{}", job.job_id);
            let payload = serde_json::json!({
                "subdomain": job.old_subdomain,
            })
            .to_string();
            ctx.doc_db
                .insert_undeployment_with_job(&job.old_subdomain, &job_id, &payload)
                .await?;
            ctx.doc_db.clear_build(&job.old_subdomain).await?;
            ctx.doc_db
                .enqueue_r2_subdomain_delete(&job.old_subdomain)
                .await?;
            job.phase = RenameJobPhase::OldEnvDeleted;
            Ok(true)
        }
        RenameJobPhase::OldEnvDeleted => {
            let _ = ctx
                .aws_s3
                .delete(&crate::cwasm_compile::env_key(&job.old_subdomain))
                .await;
            let _ = ctx
                .aws_s3
                .delete(&crate::cwasm_compile::raw_key(&job.old_subdomain))
                .await;
            ctx.deployment_cache.refresh().await;
            crate::deploy::spawn_immediate_push(ctx).await;
            job.phase = RenameJobPhase::OldDbDestroyed;
            Ok(true)
        }
        RenameJobPhase::OldDbDestroyed => {
            crate::turso_admin::destroy_database(&ctx.forte_db, &job.old_subdomain).await?;
            job.phase = RenameJobPhase::Done;
            Ok(true)
        }
        RenameJobPhase::Done | RenameJobPhase::Failed => Ok(false),
    }
}

async fn copy_bundle_object(
    ctx: &Arc<DeployContext>,
    src_key: &str,
    dst_key: &str,
    required: bool,
) -> Result<bool> {
    match ctx.aws_s3.read(src_key).await {
        Ok(buf) => {
            ctx.aws_s3
                .write(dst_key, buf)
                .await
                .map_err(|e| eyre!("copy {src_key} -> {dst_key}: write failed: {e}"))?;
            Ok(true)
        }
        Err(err) => {
            if required {
                Err(eyre!(
                    "copy {src_key} -> {dst_key}: read failed: {err}"
                ))
            } else {
                Ok(false)
            }
        }
    }
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

async fn all_sites_have_verified_host(
    ctx: &Arc<DeployContext>,
    target_generation: u64,
) -> Result<bool> {
    if ctx.sites.is_empty() {
        return Ok(true);
    }
    for site in &ctx.sites {
        let statuses = ctx.doc_db.list_host_statuses(site.name()).await?;
        let any_verified = statuses
            .iter()
            .any(|s| s.healthy && s.generation.unwrap_or(0) >= target_generation);
        if !any_verified {
            return Ok(false);
        }
    }
    Ok(true)
}
