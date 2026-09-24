//! Applies the policy snapshot stored in the telemetry outbox to Signy.
//!
//! Queue payloads identify a durable outbox record. The outbox stores the
//! complete policy snapshot, so a delayed message cannot overwrite a newer
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
    let db = doc_db::database();
    let project_id = input.project_id;
    let Some(policy) = begin_attempt(&db, &project_id).await? else {
        return Ok(());
    };
    let revision = policy.revision;
    telemetry_policy_metrics::sync_attempt();

    match signy_tenant::register_project(&project_id, &policy).await {
        Ok(signy_tenant::PolicySyncResult::Applied) => {
            telemetry_policy_metrics::sync_success();
            if finish_attempt(&db, &project_id, revision, None, false).await? {
                tracing::info!(%project_id, revision, "telemetry policy applied");
            } else {
                tracing::info!(%project_id, revision, "telemetry policy result superseded by a newer outbox revision");
            }
            Ok(())
        }
        Ok(signy_tenant::PolicySyncResult::Stale) => {
            telemetry_policy_metrics::sync_stale();
            if finish_attempt(&db, &project_id, revision, None, false).await? {
                tracing::warn!(%project_id, revision, "telemetry policy result is stale; Signy already has a newer revision");
            }
            Ok(())
        }
        Err(error) => {
            let detail = error.to_string();
            let permanent = is_permanent_sync_error(&detail);
            telemetry_policy_metrics::sync_failure();
            if detail.contains("409 Conflict") {
                telemetry_policy_metrics::sync_conflict();
            }
            let finalized =
                finish_attempt(&db, &project_id, revision, Some(detail.clone()), permanent).await?;
            if finalized {
                if permanent {
                    telemetry_policy_metrics::sync_dead_letter();
                    tracing::error!(%project_id, revision, %detail, "telemetry policy apply moved to dead letter");
                    Ok(())
                } else {
                    tracing::error!(%project_id, revision, %detail, "telemetry policy apply failed; queue will retry");
                    Err(error)
                }
            } else {
                tracing::info!(%project_id, revision, %detail, "telemetry policy failure superseded by a newer outbox revision");
                Ok(())
            }
        }
    }
}

/// Marks one queue attempt pending and returns the policy stored in the same
/// outbox transaction. A newer outbox revision wins over an older redelivery
/// before it can start a Signy write.
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
                if matches!(
                    outbox.state,
                    TelemetryPolicySyncState::Applied
                        | TelemetryPolicySyncState::DeadLetter
                        | TelemetryPolicySyncState::BlockedByDeletion
                ) {
                    return trx.commit::<_, Option<TelemetryPolicy>>(None);
                }
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
                let policy = match outbox.policy.clone() {
                    Some(policy) => policy,
                    None => {
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
                        outbox.policy_revision = project.telemetry_policy.revision;
                        outbox.policy = Some(project.telemetry_policy.clone());
                        project.telemetry_policy.clone()
                    }
                };
                outbox.state = TelemetryPolicySyncState::Pending;
                if outbox.pending_since.is_none() {
                    outbox.pending_since = Some(outbox.updated_at);
                }
                outbox.attempts = outbox.attempts.saturating_add(1);
                outbox.last_error = None;
                outbox.updated_at = now();
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

/// Finalizes an attempt only while the outbox still names the revision it sent.
/// This prevents a slow old queue message from marking a newer policy as
/// applied or failed.
async fn finish_attempt(
    db: &doc_db::Database,
    project_id: &str,
    revision: u64,
    error: Option<String>,
    permanent: bool,
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
                if !attempt_matches_current_policy(&outbox, revision) {
                    return trx.commit(false);
                }
                outbox.state = if error.is_some() {
                    if permanent {
                        TelemetryPolicySyncState::DeadLetter
                    } else {
                        TelemetryPolicySyncState::Failed
                    }
                } else {
                    TelemetryPolicySyncState::Applied
                };
                if error.is_none() || permanent {
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

fn is_permanent_sync_error(detail: &str) -> bool {
    [
        "400 Bad Request",
        "401 Unauthorized",
        "403 Forbidden",
        "404 Not Found",
        "405 Method Not Allowed",
        "409 Conflict",
        "410 Gone",
        "415 Unsupported Media Type",
        "422 Unprocessable Entity",
    ]
    .iter()
    .any(|status| detail.contains(status))
}

fn attempt_matches_current_policy(
    outbox: &TelemetryPolicyOutboxDoc,
    attempted_revision: u64,
) -> bool {
    outbox.policy_revision == attempted_revision
        && outbox
            .policy
            .as_ref()
            .is_some_and(|policy| policy.revision == attempted_revision)
}

#[cfg(test)]
mod tests {
    use super::{attempt_matches_current_policy, is_permanent_sync_error};
    use crate::docs::{TelemetryPolicy, TelemetryPolicyOutboxDoc, TelemetryPolicySyncState};
    use forte_sdk::now;

    fn outbox(revision: u64) -> TelemetryPolicyOutboxDoc {
        TelemetryPolicyOutboxDoc {
            project_id: "project".to_string(),
            policy_revision: revision,
            policy: Some(TelemetryPolicy {
                revision,
                base_retention: "30d".to_string(),
                log_retention_override: None,
                trace_retention_override: None,
                metric_retention_override: None,
                max_stored_bytes: "512MiB".to_string(),
            }),
            state: TelemetryPolicySyncState::Pending,
            attempts: 0,
            last_error: None,
            pending_since: None,
            updated_at: now(),
        }
    }

    #[test]
    fn an_old_attempt_cannot_finalize_a_newer_project_policy() {
        assert!(!attempt_matches_current_policy(&outbox(2), 1));
        assert!(attempt_matches_current_policy(&outbox(2), 2));
    }

    #[test]
    fn only_non_retryable_http_failures_move_to_dead_letter() {
        assert!(is_permanent_sync_error(
            "signy answered 409 Conflict: conflict"
        ));
        assert!(is_permanent_sync_error(
            "signy answered 400 Bad Request: invalid"
        ));
        assert!(!is_permanent_sync_error(
            "signy answered 503 Service Unavailable"
        ));
        assert!(!is_permanent_sync_error("request timed out"));
    }
}
