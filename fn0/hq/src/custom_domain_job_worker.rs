use std::sync::Arc;
use std::time::Duration;

use color_eyre::eyre::{Result, eyre};
use tokio::time::MissedTickBehavior;
use tracing::*;

use crate::args_parse::DeployContext;
use crate::doc_db::{
    CustomDomainAddJob, CustomDomainAddPhase, CustomDomainRemoveJob, CustomDomainRemovePhase,
    InsertCustomDomainOutcome,
};

const TICK_INTERVAL: Duration = Duration::from_secs(5);
const HEARTBEAT_STALE_SECS: i64 = 30;
const MAX_ATTEMPTS: u32 = 20;

pub async fn run(ctx: Arc<DeployContext>) {
    let mut interval = tokio::time::interval(TICK_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        if let Err(err) = tick(&ctx).await {
            warn!(%err, "custom_domain_job_worker tick failed");
        }
    }
}

async fn tick(ctx: &Arc<DeployContext>) -> Result<()> {
    let now = chrono::Utc::now();

    let add_jobs = ctx.doc_db.list_active_custom_domain_add_jobs().await?;
    for mut job in add_jobs {
        if !is_stale_add(&job, now, HEARTBEAT_STALE_SECS) {
            continue;
        }
        claim_add(ctx, &mut job, now).await?;
        run_add_until_blocked(ctx, job).await;
    }

    let remove_jobs = ctx.doc_db.list_active_custom_domain_remove_jobs().await?;
    for mut job in remove_jobs {
        if !is_stale_remove(&job, now, HEARTBEAT_STALE_SECS) {
            continue;
        }
        claim_remove(ctx, &mut job, now).await?;
        run_remove_until_blocked(ctx, job).await;
    }

    Ok(())
}

fn is_stale_add(
    job: &CustomDomainAddJob,
    now: chrono::DateTime<chrono::Utc>,
    ttl_secs: i64,
) -> bool {
    match &job.heartbeat_at {
        None => true,
        Some(hb) => match chrono::DateTime::parse_from_rfc3339(hb) {
            Ok(ts) => (now.timestamp() - ts.timestamp()) >= ttl_secs,
            Err(_) => true,
        },
    }
}

fn is_stale_remove(
    job: &CustomDomainRemoveJob,
    now: chrono::DateTime<chrono::Utc>,
    ttl_secs: i64,
) -> bool {
    match &job.heartbeat_at {
        None => true,
        Some(hb) => match chrono::DateTime::parse_from_rfc3339(hb) {
            Ok(ts) => (now.timestamp() - ts.timestamp()) >= ttl_secs,
            Err(_) => true,
        },
    }
}

async fn claim_add(
    ctx: &Arc<DeployContext>,
    job: &mut CustomDomainAddJob,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    let now_str = now.to_rfc3339();
    job.heartbeat_at = Some(now_str.clone());
    job.updated_at = now_str;
    ctx.doc_db.update_custom_domain_add_job(job).await
}

async fn claim_remove(
    ctx: &Arc<DeployContext>,
    job: &mut CustomDomainRemoveJob,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    let now_str = now.to_rfc3339();
    job.heartbeat_at = Some(now_str.clone());
    job.updated_at = now_str;
    ctx.doc_db.update_custom_domain_remove_job(job).await
}

async fn run_add_until_blocked(ctx: &Arc<DeployContext>, mut job: CustomDomainAddJob) {
    loop {
        if job.is_terminal() {
            return;
        }
        match advance_add_phase(ctx, &mut job).await {
            Ok(true) => {
                let now = chrono::Utc::now().to_rfc3339();
                job.heartbeat_at = Some(now.clone());
                job.updated_at = now;
                job.last_error = None;
                if let Err(err) = ctx.doc_db.update_custom_domain_add_job(&job).await {
                    warn!(%err, job_id = %job.job_id, "failed to persist add phase");
                    return;
                }
            }
            Ok(false) => return,
            Err(err) => {
                error!(%err, job_id = %job.job_id, phase = ?job.phase, "add phase failed");
                let now = chrono::Utc::now().to_rfc3339();
                job.attempts = job.attempts.saturating_add(1);
                job.last_error = Some(format!("{err:?}"));
                job.heartbeat_at = Some(now.clone());
                job.updated_at = now;
                if job.attempts >= MAX_ATTEMPTS {
                    job.phase = CustomDomainAddPhase::Failed;
                }
                if let Err(err) = ctx.doc_db.update_custom_domain_add_job(&job).await {
                    warn!(%err, job_id = %job.job_id, "failed to persist add error");
                }
                return;
            }
        }
    }
}

async fn run_remove_until_blocked(ctx: &Arc<DeployContext>, mut job: CustomDomainRemoveJob) {
    loop {
        if job.is_terminal() {
            return;
        }
        match advance_remove_phase(ctx, &mut job).await {
            Ok(true) => {
                let now = chrono::Utc::now().to_rfc3339();
                job.heartbeat_at = Some(now.clone());
                job.updated_at = now;
                job.last_error = None;
                if let Err(err) = ctx.doc_db.update_custom_domain_remove_job(&job).await {
                    warn!(%err, job_id = %job.job_id, "failed to persist remove phase");
                    return;
                }
            }
            Ok(false) => return,
            Err(err) => {
                error!(%err, job_id = %job.job_id, phase = ?job.phase, "remove phase failed");
                let now = chrono::Utc::now().to_rfc3339();
                job.attempts = job.attempts.saturating_add(1);
                job.last_error = Some(format!("{err:?}"));
                job.heartbeat_at = Some(now.clone());
                job.updated_at = now;
                if job.attempts >= MAX_ATTEMPTS {
                    job.phase = CustomDomainRemovePhase::Failed;
                }
                if let Err(err) = ctx.doc_db.update_custom_domain_remove_job(&job).await {
                    warn!(%err, job_id = %job.job_id, "failed to persist remove error");
                }
                return;
            }
        }
    }
}

async fn advance_add_phase(ctx: &Arc<DeployContext>, job: &mut CustomDomainAddJob) -> Result<bool> {
    match job.phase {
        CustomDomainAddPhase::Queued => {
            let hostname_id = if let Some(id) = job.cloudflare_hostname_id.clone() {
                if ctx
                    .cloudflare_saas
                    .get_custom_hostname(&id)
                    .await?
                    .is_some()
                {
                    id
                } else {
                    create_or_adopt_hostname(ctx, &job.domain).await?
                }
            } else {
                create_or_adopt_hostname(ctx, &job.domain).await?
            };
            job.cloudflare_hostname_id = Some(hostname_id);
            job.phase = CustomDomainAddPhase::CloudflareHostnameCreated;
            Ok(true)
        }
        CustomDomainAddPhase::CloudflareHostnameCreated => {
            let hostname_id = job
                .cloudflare_hostname_id
                .clone()
                .ok_or_else(|| eyre!("CloudflareHostnameCreated without hostname_id"))?;
            let outcome = ctx
                .doc_db
                .insert_custom_domain(&job.subdomain, &job.domain, &hostname_id)
                .await?;
            match outcome {
                InsertCustomDomainOutcome::Inserted | InsertCustomDomainOutcome::AlreadyExists => {
                    job.phase = CustomDomainAddPhase::DocPersisted;
                    Ok(true)
                }
                InsertCustomDomainOutcome::SubdomainTaken { existing_domain } => Err(eyre!(
                    "subdomain '{}' already has domain '{}'",
                    job.subdomain,
                    existing_domain
                )),
                InsertCustomDomainOutcome::DomainTaken { existing_subdomain } => Err(eyre!(
                    "domain '{}' already mapped to subdomain '{}'",
                    job.domain,
                    existing_subdomain
                )),
            }
        }
        CustomDomainAddPhase::DocPersisted => {
            ctx.custom_domain_cache.refresh().await;
            crate::deploy::spawn_immediate_push(ctx).await;
            job.phase = CustomDomainAddPhase::Pushed;
            Ok(true)
        }
        CustomDomainAddPhase::Pushed => {
            let hostname_id = job
                .cloudflare_hostname_id
                .clone()
                .ok_or_else(|| eyre!("Pushed without hostname_id"))?;
            let info = ctx
                .cloudflare_saas
                .get_custom_hostname(&hostname_id)
                .await?;
            let active = info.map(|h| h.status.is_active()).unwrap_or(false);
            if active {
                job.phase = CustomDomainAddPhase::Verified;
                return Ok(true);
            }
            Ok(false)
        }
        CustomDomainAddPhase::Verified => {
            job.phase = CustomDomainAddPhase::Done;
            Ok(true)
        }
        CustomDomainAddPhase::Done | CustomDomainAddPhase::Failed => Ok(false),
    }
}

async fn create_or_adopt_hostname(ctx: &Arc<DeployContext>, domain: &str) -> Result<String> {
    match ctx.cloudflare_saas.create_custom_hostname(domain).await {
        Ok(h) => Ok(h.id),
        Err(_) => {
            let existing = ctx
                .cloudflare_saas
                .find_custom_hostname_by_name(domain)
                .await?
                .ok_or_else(|| {
                    eyre!("create_custom_hostname failed and no existing match for {domain}")
                })?;
            Ok(existing.id)
        }
    }
}

async fn advance_remove_phase(
    ctx: &Arc<DeployContext>,
    job: &mut CustomDomainRemoveJob,
) -> Result<bool> {
    match job.phase {
        CustomDomainRemovePhase::Queued => {
            ctx.doc_db
                .delete_custom_domain(&job.subdomain, &job.domain)
                .await?;
            job.phase = CustomDomainRemovePhase::DocDeleted;
            Ok(true)
        }
        CustomDomainRemovePhase::DocDeleted => {
            ctx.custom_domain_cache.refresh().await;
            crate::deploy::spawn_immediate_push(ctx).await;
            job.phase = CustomDomainRemovePhase::Pushed;
            Ok(true)
        }
        CustomDomainRemovePhase::Pushed => {
            if let Some(id) = job.cloudflare_hostname_id.clone() {
                ctx.cloudflare_saas.delete_custom_hostname(&id).await?;
            } else if let Some(found) = ctx
                .cloudflare_saas
                .find_custom_hostname_by_name(&job.domain)
                .await?
            {
                ctx.cloudflare_saas
                    .delete_custom_hostname(&found.id)
                    .await?;
            }
            job.phase = CustomDomainRemovePhase::CloudflareHostnameDeleted;
            Ok(true)
        }
        CustomDomainRemovePhase::CloudflareHostnameDeleted => {
            job.phase = CustomDomainRemovePhase::Done;
            Ok(true)
        }
        CustomDomainRemovePhase::Done | CustomDomainRemovePhase::Failed => Ok(false),
    }
}
