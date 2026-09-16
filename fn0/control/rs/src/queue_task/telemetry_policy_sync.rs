//! Applies the latest ProjectDoc telemetry policy to Signy.
//!
//! Queue payloads deliberately contain only the project id. The handler reads
//! ProjectDoc at execution time, so a delayed message cannot overwrite a newer
//! policy. A deletion tombstone wins over every registration attempt.

use crate::common::signy_tenant;
use crate::common::telemetry_policy_metrics;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub project_id: String,
}

pub async fn handle(input: Input) -> anyhow::Result<()> {
    let db = doc_db::turso();
    let project_id = input.project_id;
    let Some(policy) = begin_attempt(&db, &project_id).await? else {
        return Ok(());
    };
    let revision = policy.revision;
    telemetry_policy_metrics::sync_attempt();

    match signy_tenant::register_project(&project_id, &policy).await {
        Ok(signy_tenant::PolicySyncResult::Applied) => {
            telemetry_policy_metrics::sync_success();
            if finish_attempt(&db, &project_id, revision, None).await? {
                tracing::info!(%project_id, revision, "telemetry policy applied");
            } else {
                tracing::info!(%project_id, revision, "telemetry policy result superseded by a newer ProjectDoc");
            }
            Ok(())
        }
        Ok(signy_tenant::PolicySyncResult::Stale) => {
            telemetry_policy_metrics::sync_stale();
            if finish_attempt(&db, &project_id, revision, None).await? {
                tracing::warn!(%project_id, revision, "telemetry policy result is stale; Signy already has a newer revision");
            }
            Ok(())
        }
        Err(error) => {
            let detail = error.to_string();
            telemetry_policy_metrics::sync_failure();
            if detail.contains("409 Conflict") {
                telemetry_policy_metrics::sync_conflict();
            }
            let finalized =
                finish_attempt(&db, &project_id, revision, Some(detail.clone())).await?;
            if finalized {
                tracing::error!(%project_id, revision, %detail, "telemetry policy apply failed; queue will retry");
                Err(error)
            } else {
                tracing::info!(%project_id, revision, %detail, "telemetry policy failure superseded by a newer ProjectDoc");
                Ok(())
            }
        }
    }
}

/// Marks one queue attempt pending and returns the policy read in the same
/// transaction. A newer ProjectDoc or outbox revision wins over an older
/// redelivery before it can start a Signy write.
async fn begin_attempt(
    db: &doc_db::Database,
    project_id: &str,
) -> anyhow::Result<Option<TelemetryPolicy>> {
    let project_id = project_id.to_string();
    let result = db
        .trx(|trx| {
            let project_id = project_id.clone();
            async move {
                let Some(mut outbox) = trx
                    .get(TelemetryPolicyOutboxDocGet {
                        project_id: &project_id,
                    })
                    .await?
                else {
                    tracing::warn!(%project_id, "telemetry policy sync has no outbox record");
                    return trx.commit::<_, Option<TelemetryPolicy>>(None);
                };
                if let Some(tombstone) = trx
                    .get(ProjectDeletionDocGet {
                        project_id: &project_id,
                    })
                    .await?
                {
                    outbox.state = TelemetryPolicySyncState::BlockedByDeletion;
                    outbox.pending_since = None;
                    outbox.last_error = Some(format!("project deletion is {:?}", tombstone.state));
                    outbox.updated_at = now();
                    return trx.commit::<_, Option<TelemetryPolicy>>(None);
                }
                let Some(project) = trx
                    .get(ProjectDocGet {
                        project_id: &project_id,
                    })
                    .await?
                else {
                    outbox.state = TelemetryPolicySyncState::BlockedByDeletion;
                    outbox.pending_since = None;
                    outbox.last_error = Some("ProjectDoc is absent".to_string());
                    outbox.updated_at = now();
                    return trx.commit::<_, Option<TelemetryPolicy>>(None);
                };
                let Some(latest_revision) = latest_policy_revision(
                    outbox.policy_revision,
                    project.telemetry_policy.revision,
                ) else {
                    tracing::warn!(
                        %project_id,
                        outbox_revision = outbox.policy_revision,
                        project_revision = project.telemetry_policy.revision,
                        "telemetry policy outbox is ahead of ProjectDoc"
                    );
                    return trx.commit::<_, Option<TelemetryPolicy>>(None);
                };
                outbox.policy_revision = latest_revision;
                outbox.state = TelemetryPolicySyncState::Pending;
                if outbox.pending_since.is_none() {
                    outbox.pending_since = Some(outbox.updated_at);
                }
                outbox.attempts = outbox.attempts.saturating_add(1);
                outbox.last_error = None;
                outbox.updated_at = now();
                let policy = project.telemetry_policy.clone();
                trx.commit(Some(policy))
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(policy) => Ok(policy),
        doc_db::TrxResult::Cancelled(_) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => {
            anyhow::bail!("begin telemetry policy attempt conflict: {error:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

/// Finalizes an attempt only while both the ProjectDoc and outbox still name
/// the revision it sent. This prevents a slow old queue message from marking
/// a newer policy as applied or failed.
async fn finish_attempt(
    db: &doc_db::Database,
    project_id: &str,
    revision: u64,
    error: Option<String>,
) -> anyhow::Result<bool> {
    let project_id = project_id.to_string();
    let result = db
        .trx(|trx| {
            let project_id = project_id.clone();
            let error = error.clone();
            async move {
                let Some(mut outbox) = trx
                    .get(TelemetryPolicyOutboxDocGet {
                        project_id: &project_id,
                    })
                    .await?
                else {
                    return trx.commit::<_, bool>(false);
                };
                if trx
                    .get(ProjectDeletionDocGet {
                        project_id: &project_id,
                    })
                    .await?
                    .is_some()
                {
                    outbox.state = TelemetryPolicySyncState::BlockedByDeletion;
                    outbox.pending_since = None;
                    outbox.last_error = None;
                    outbox.updated_at = now();
                    return trx.commit(true);
                }
                let Some(project) = trx
                    .get(ProjectDocGet {
                        project_id: &project_id,
                    })
                    .await?
                else {
                    return trx.commit(false);
                };
                if !attempt_matches_current_policy(
                    outbox.policy_revision,
                    project.telemetry_policy.revision,
                    revision,
                ) {
                    return trx.commit(false);
                }
                outbox.state = if error.is_some() {
                    TelemetryPolicySyncState::Failed
                } else {
                    TelemetryPolicySyncState::Applied
                };
                if error.is_none() {
                    outbox.pending_since = None;
                } else if outbox.pending_since.is_none() {
                    outbox.pending_since = Some(outbox.updated_at);
                }
                outbox.last_error = error;
                outbox.updated_at = now();
                trx.commit(true)
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(finalized) => Ok(finalized),
        doc_db::TrxResult::Cancelled(_) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => {
            anyhow::bail!("finish telemetry policy attempt conflict: {error:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

fn attempt_matches_current_policy(
    outbox_revision: u64,
    project_revision: u64,
    attempted_revision: u64,
) -> bool {
    outbox_revision == attempted_revision && project_revision == attempted_revision
}

fn latest_policy_revision(outbox_revision: u64, project_revision: u64) -> Option<u64> {
    (outbox_revision <= project_revision).then_some(project_revision)
}

#[cfg(test)]
mod tests {
    use super::{attempt_matches_current_policy, latest_policy_revision};

    #[test]
    fn an_old_attempt_cannot_finalize_a_newer_project_policy() {
        assert!(!attempt_matches_current_policy(2, 2, 1));
        assert!(!attempt_matches_current_policy(1, 2, 1));
        assert!(attempt_matches_current_policy(2, 2, 2));
    }

    #[test]
    fn an_outbox_ahead_of_the_project_is_not_claimed_by_an_old_attempt() {
        assert!(!attempt_matches_current_policy(3, 2, 2));
    }

    #[test]
    fn multiple_outbox_deliveries_converge_on_the_latest_project_policy() {
        for outbox_revision in [1, 2, 3] {
            assert_eq!(latest_policy_revision(outbox_revision, 3), Some(3));
        }
    }
}
