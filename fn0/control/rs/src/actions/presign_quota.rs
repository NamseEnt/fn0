//! Hourly presign quota enforcement (issue #11).
//!
//! Runs from `cron_on_tick` right after `usage_metering`, over the same
//! project set. Sums metered object-storage Class A/B operations and
//! worker-reported presign mint counts, compares them against the `quota.rs`
//! defaults (or `ProjectQuotaOverridesDoc`), and publishes the breaching
//! projects in `PresignBlockedDoc`. Workers poll that doc and refuse to mint
//! presigned URLs for listed projects.
//!
//! Release is automatic and needs no separate code path: a fresh evaluation
//! every hour drops a project from the blocked list as soon as its latest
//! hourly window is back under the hourly caps and the rolling 30-day sums
//! are back under the monthly caps.

use crate::common::admin;
use crate::docs::*;
use crate::quota;
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub enum Output {
    Ok {
        evaluated_count: u64,
        blocked_count: u64,
    },
    Unauthorized,
    Error {
        message: String,
    },
}

pub struct EnforcementStats {
    pub evaluated: u64,
    pub blocked: u64,
}

struct EffectiveQuota {
    presign_mint_per_month: u64,
    class_a_per_hour: u64,
    class_a_per_month: u64,
    class_b_per_hour: u64,
    class_b_per_month: u64,
}

impl EffectiveQuota {
    fn resolve(overrides: Option<&ProjectQuotaOverridesDoc>) -> Self {
        Self {
            presign_mint_per_month: overrides
                .and_then(|o| o.presign_mint_per_month)
                .unwrap_or(quota::PRESIGN_MINT_PER_MONTH),
            class_a_per_hour: overrides
                .and_then(|o| o.object_storage_class_a_per_hour)
                .unwrap_or(quota::OBJECT_STORAGE_CLASS_A_PER_HOUR),
            class_a_per_month: overrides
                .and_then(|o| o.object_storage_class_a_per_month)
                .unwrap_or(quota::OBJECT_STORAGE_CLASS_A_PER_MONTH),
            class_b_per_hour: overrides
                .and_then(|o| o.object_storage_class_b_per_hour)
                .unwrap_or(quota::OBJECT_STORAGE_CLASS_B_PER_HOUR),
            class_b_per_month: overrides
                .and_then(|o| o.object_storage_class_b_per_month)
                .unwrap_or(quota::OBJECT_STORAGE_CLASS_B_PER_MONTH),
        }
    }
}

pub async fn run_enforcement(project_ids: &BTreeSet<String>) -> anyhow::Result<EnforcementStats> {
    let db = doc_db::turso();
    let now = forte_sdk::now();
    let current_epoch_hour = now.timestamp() / 3600;
    let latest_window = current_epoch_hour - 1;
    let month_start = current_epoch_hour - quota::QUOTA_ROLLING_WINDOW_HOURS;

    let previously_blocked: BTreeSet<String> = (PresignBlockedDocGet {})
        .send_with(&db)
        .await?
        .map(|doc| doc.blocked_project_ids.into_iter().collect())
        .unwrap_or_default();

    let mut blocked: BTreeSet<String> = BTreeSet::new();

    for project_id in project_ids {
        let overrides = (ProjectQuotaOverridesDocGet {
            project_id: project_id.as_str(),
        })
        .send_with(&db)
        .await?;
        let effective = EffectiveQuota::resolve(overrides.as_ref());

        let usage: Vec<ProjectOperationsUsageDoc> = (ProjectOperationsUsageDocQuery {
            project_id: project_id.as_str(),
            window_start_epoch_hour: Some(month_start - 1),
            limit: None,
        })
        .send_with(&db)
        .await?;

        let mut class_a_month: u64 = 0;
        let mut class_b_month: u64 = 0;
        let mut class_a_latest: u64 = 0;
        let mut class_b_latest: u64 = 0;
        for doc in &usage {
            class_a_month += doc.object_storage_writes.estimate;
            class_b_month += doc.object_storage_reads.estimate;
            if doc.window_start_epoch_hour == latest_window {
                class_a_latest = doc.object_storage_writes.estimate;
                class_b_latest = doc.object_storage_reads.estimate;
            }
        }

        let mint_docs: Vec<PresignMintCountDoc> = (PresignMintCountDocQuery {
            project_id: project_id.as_str(),
            window_epoch_hour: Some(month_start),
            writer_id: None,
            limit: None,
        })
        .send_with(&db)
        .await?;
        let mint_month: u64 = mint_docs.iter().map(|doc| doc.minted).sum();

        let over_quota = class_b_latest > effective.class_b_per_hour
            || class_b_month > effective.class_b_per_month
            || class_a_latest > effective.class_a_per_hour
            || class_a_month > effective.class_a_per_month
            || mint_month > effective.presign_mint_per_month;
        if over_quota {
            blocked.insert(project_id.clone());
            if !previously_blocked.contains(project_id) {
                tracing::warn!(
                    %project_id,
                    class_a_latest,
                    class_a_month,
                    class_b_latest,
                    class_b_month,
                    mint_month,
                    "presign quota exceeded; blocking presign minting"
                );
            }
        } else if previously_blocked.contains(project_id) {
            tracing::info!(%project_id, "presign quota back under limits; unblocking");
        }
    }

    PresignBlockedDocPut(PresignBlockedDoc {
        updated_epoch_hour: current_epoch_hour,
        blocked_project_ids: blocked.iter().cloned().collect(),
    })
    .send_with(&db)
    .await?;

    Ok(EnforcementStats {
        evaluated: project_ids.len() as u64,
        blocked: blocked.len() as u64,
    })
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }
    let project_ids = match crate::actions::usage_metering::run_metering().await {
        Ok(stats) => stats.project_ids,
        Err(e) => {
            return Output::Error {
                message: format!("metering: {e}"),
            };
        }
    };
    match run_enforcement(&project_ids).await {
        Ok(stats) => Output::Ok {
            evaluated_count: stats.evaluated,
            blocked_count: stats.blocked,
        },
        Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
