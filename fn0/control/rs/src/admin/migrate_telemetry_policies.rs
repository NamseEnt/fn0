//! One-shot pre-schema migration for the required ProjectDoc policy field.
//!
//! Existing policy values come from Signy. A project without a Signy policy is
//! recorded in TelemetryPolicyMigrationExceptionDoc and is never backfilled.
//! This task must report zero exceptions before the required ProjectDoc schema
//! is rolled out.

use crate::common::signy_tenant;
use crate::common::telemetry_policy_metrics;
use crate::docs::*;
use doc_db::{DocGet, DocKey, Document};
use forte_sdk::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Deserialize)]
pub struct Input;

pub async fn handle(Input: Input) -> anyhow::Result<()> {
    let db = doc_db::database();
    let mut after: Option<(String, String)> = None;
    let mut migrated = 0_u64;
    let mut exceptions = 0_u64;
    loop {
        let page = db
            .scan(
                after.as_ref().map(|(pk, sk)| (pk.as_str(), sk.as_str())),
                256,
            )
            .await?;
        if page.is_empty() {
            break;
        }
        for (pk, _sk, bytes) in &page {
            if !pk.starts_with("ProjectDoc/") {
                continue;
            }
            let value: serde_json::Value = serde_json::from_slice(bytes)
                .map_err(|error| anyhow::anyhow!("invalid {pk}: {error}"))?;
            let Some(project_id) = value
                .get("project_id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
            else {
                anyhow::bail!("ProjectDoc {pk} has no project_id");
            };
            if let Some(policy_value) = value.get("telemetry_policy")
                && let Ok(policy) = serde_json::from_value::<TelemetryPolicy>(policy_value.clone())
            {
                match signy_tenant::read_policy(&project_id).await? {
                    Some(signy_policy) if same_policy_meaning(&policy, &signy_policy) => {
                        ensure_outbox(&db, &project_id, &policy).await?;
                        (TelemetryPolicyMigrationExceptionDocDelete {
                            project_id: &project_id,
                        })
                        .send_with(&db)
                        .await?;
                        continue;
                    }
                    Some(_) => {
                        record_exception(
                            &db,
                            &project_id,
                            "ProjectDoc policy differs from the registered Signy policy",
                        )
                        .await?;
                        exceptions += 1;
                        continue;
                    }
                    None => {
                        record_exception(&db, &project_id, "Signy has no explicit tenant policy")
                            .await?;
                        exceptions += 1;
                        continue;
                    }
                }
            }
            // A legacy document used effective per-signal fields. It is not
            // interpreted as the new schema: the actual Signy policy below is
            // the only source for the migration value.
            let Some(mut policy) = signy_tenant::read_policy(&project_id).await? else {
                exceptions += 1;
                record_exception(&db, &project_id, "Signy has no explicit tenant policy").await?;
                tracing::error!(%project_id, "ProjectDoc migration requires an explicit Signy policy");
                continue;
            };
            // Claim the unchanged values at the revision Signy already has.
            // Signy permits this metadata-only claim once, so a crash and
            // rerun cannot keep incrementing the revision.
            policy.revision = policy.revision.max(1);
            // Claim the unchanged values before ProjectDoc is written. Once
            // this succeeds, the generic operator PUT cannot mutate this
            // tenant and the normal outbox retry can safely resend the same
            // revision. The migration changes only revision metadata; all
            // retention and storage values came from Signy above.
            if matches!(
                signy_tenant::register_project(&project_id, &policy).await?,
                signy_tenant::PolicySyncResult::Stale
            ) {
                record_exception(
                    &db,
                    &project_id,
                    "Signy policy advanced during migration; rerun after policy sync",
                )
                .await?;
                exceptions += 1;
                continue;
            }
            match write_migrated_project_doc(&db, &project_id, &policy).await? {
                MigrationWriteResult::Migrated => migrated += 1,
                MigrationWriteResult::AlreadyCurrent => {}
            }
            (TelemetryPolicyMigrationExceptionDocDelete {
                project_id: &project_id,
            })
            .send_with(&db)
            .await?;
            tracing::info!(%project_id, revision = policy.revision, "migrated ProjectDoc telemetry policy from Signy");
        }
        let Some((pk, sk, _)) = page.last() else {
            break;
        };
        after = Some((pk.clone(), sk.clone()));
    }
    tracing::info!(
        migrated,
        exceptions,
        "ProjectDoc telemetry policy migration finished"
    );
    telemetry_policy_metrics::migration_exceptions(exceptions);
    if exceptions > 0 {
        anyhow::bail!("{exceptions} projects require explicit telemetry policy migration");
    }
    Ok(())
}

async fn ensure_outbox(
    db: &doc_db::Database,
    project_id: &str,
    policy: &TelemetryPolicy,
) -> anyhow::Result<()> {
    let project_id = project_id.to_string();
    let policy = policy.clone();
    match db
        .trx(|trx| {
            let project_id = project_id.clone();
            let policy = policy.clone();
            async move {
                let Some(mut outbox) = trx
                    .get(TelemetryPolicyOutboxDocGet {
                        project_id: &project_id,
                    })
                    .await?
                else {
                    let timestamp = now();
                    trx.create(TelemetryPolicyOutboxDoc {
                        project_id,
                        policy_revision: policy.revision,
                        policy: Some(policy.clone()),
                        state: TelemetryPolicySyncState::Pending,
                        attempts: 0,
                        last_error: None,
                        pending_since: Some(timestamp),
                        updated_at: timestamp,
                    })?;
                    return trx.commit(());
                };
                if outbox.policy_revision < policy.revision
                    || (outbox.policy_revision == policy.revision && outbox.policy.is_none())
                {
                    let timestamp = now();
                    outbox.policy_revision = policy.revision;
                    outbox.policy = Some(policy.clone());
                    outbox.state = TelemetryPolicySyncState::Pending;
                    if outbox.pending_since.is_none() {
                        outbox.pending_since = Some(timestamp);
                    }
                    outbox.last_error = None;
                    outbox.updated_at = timestamp;
                }
                trx.commit(())
            }
        })
        .await
    {
        doc_db::TrxResult::Committed(()) => Ok(()),
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => anyhow::bail!("migration outbox conflict: {error:?}"),
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MigrationWriteResult {
    Migrated,
    AlreadyCurrent,
}

/// Writes a legacy ProjectDoc only if it is still legacy at commit time.
///
/// The migration reads Signy outside this transaction because that is an
/// external service. Re-reading the raw document inside an optimistic
/// transaction prevents that stale external read from replacing a policy that
/// a concurrent control writer has already stored.
async fn write_migrated_project_doc(
    db: &doc_db::Database,
    project_id: &str,
    policy: &TelemetryPolicy,
) -> anyhow::Result<MigrationWriteResult> {
    let project_id = project_id.to_string();
    let policy = policy.clone();
    match db
        .trx(|trx| {
            let project_id = project_id.clone();
            let policy = policy.clone();
            async move {
                let (Some(mut project), outbox) = trx
                    .get((
                        RawProjectDocGet {
                            project_id: project_id.clone(),
                        },
                        TelemetryPolicyOutboxDocGet {
                            project_id: &project_id,
                        },
                    ))
                    .await?
                else {
                    return trx.commit(MigrationWriteResult::AlreadyCurrent);
                };
                let current_policy = project
                    .value
                    .get("telemetry_policy")
                    .cloned()
                    .and_then(|value| serde_json::from_value::<TelemetryPolicy>(value).ok());
                if let Some(current_policy) = current_policy {
                    let current_revision = current_policy.revision;
                    match outbox {
                        Some(mut outbox)
                            if outbox.policy_revision < current_revision
                                || (outbox.policy_revision == current_revision
                                    && outbox.policy.is_none()) =>
                        {
                            let timestamp = now();
                            outbox.policy_revision = current_revision;
                            outbox.policy = Some(current_policy.clone());
                            outbox.state = TelemetryPolicySyncState::Pending;
                            if outbox.pending_since.is_none() {
                                outbox.pending_since = Some(timestamp);
                            }
                            outbox.last_error = None;
                            outbox.updated_at = timestamp;
                        }
                        Some(_) => {}
                        None => {
                            let timestamp = now();
                            trx.create(TelemetryPolicyOutboxDoc {
                                project_id,
                                policy_revision: current_revision,
                                policy: Some(current_policy.clone()),
                                state: TelemetryPolicySyncState::Pending,
                                attempts: 0,
                                last_error: None,
                                pending_since: Some(timestamp),
                                updated_at: timestamp,
                            })?;
                        }
                    }
                    return trx.commit(MigrationWriteResult::AlreadyCurrent);
                }
                if outbox
                    .as_ref()
                    .is_some_and(|outbox| outbox.policy_revision > policy.revision)
                {
                    anyhow::bail!(
                        "migration refused to move ProjectDoc policy backwards for {project_id}"
                    );
                }
                project.value["telemetry_policy"] = serde_json::to_value(&policy)?;
                let timestamp = now();
                match outbox {
                    Some(mut outbox) => {
                        outbox.policy_revision = policy.revision;
                        outbox.policy = Some(policy.clone());
                        outbox.state = TelemetryPolicySyncState::Pending;
                        if outbox.pending_since.is_none() {
                            outbox.pending_since = Some(timestamp);
                        }
                        outbox.last_error = None;
                        outbox.updated_at = timestamp;
                    }
                    None => {
                        trx.create(TelemetryPolicyOutboxDoc {
                            project_id,
                            policy_revision: policy.revision,
                            policy: Some(policy.clone()),
                            state: TelemetryPolicySyncState::Pending,
                            attempts: 0,
                            last_error: None,
                            pending_since: Some(timestamp),
                            updated_at: timestamp,
                        })?;
                    }
                }
                trx.commit(MigrationWriteResult::Migrated)
            }
        })
        .await
    {
        doc_db::TrxResult::Committed(result) => Ok(result),
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(error) => {
            anyhow::bail!("migration ProjectDoc conflict: {error:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}

#[derive(Clone)]
struct RawProjectDoc {
    value: serde_json::Value,
}

impl Serialize for RawProjectDoc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RawProjectDoc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            value: serde_json::Value::deserialize(deserializer)?,
        })
    }
}

impl Document for RawProjectDoc {
    fn key(&self) -> DocKey {
        let project_id = self
            .value
            .get("project_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        DocKey::new(format!("ProjectDoc/project_id={project_id}"), "")
    }
}

struct RawProjectDocGet {
    project_id: String,
}

impl DocGet for RawProjectDocGet {
    type Doc = RawProjectDoc;

    fn key(&self) -> DocKey {
        DocKey::new(format!("ProjectDoc/project_id={}", self.project_id), "")
    }
}

async fn record_exception(
    db: &doc_db::Database,
    project_id: &str,
    reason: &str,
) -> anyhow::Result<()> {
    (TelemetryPolicyMigrationExceptionDocPut(TelemetryPolicyMigrationExceptionDoc {
        project_id: project_id.to_string(),
        reason: reason.to_string(),
        observed_at: now(),
    }))
    .send_with(db)
    .await?;
    tracing::error!(%project_id, %reason, "ProjectDoc migration requires operator resolution");
    Ok(())
}

fn same_policy_meaning(left: &TelemetryPolicy, right: &TelemetryPolicy) -> bool {
    left.base_retention == right.base_retention
        && left.log_retention_override == right.log_retention_override
        && left.trace_retention_override == right.trace_retention_override
        && left.metric_retention_override == right.metric_retention_override
        && left.max_stored_bytes == right.max_stored_bytes
}

#[cfg(test)]
mod tests {
    use super::{MigrationWriteResult, same_policy_meaning, write_migrated_project_doc};
    use crate::docs::TelemetryPolicy;

    #[test]
    fn missing_overrides_are_not_equal_to_materialized_effective_values() {
        let inherited = TelemetryPolicy {
            revision: 1,
            base_retention: "30d".to_string(),
            log_retention_override: None,
            trace_retention_override: None,
            metric_retention_override: None,
            max_stored_bytes: "512MiB".to_string(),
        };
        let materialized = TelemetryPolicy {
            log_retention_override: Some("30d".to_string()),
            trace_retention_override: Some("30d".to_string()),
            metric_retention_override: Some("30d".to_string()),
            ..inherited.clone()
        };
        assert!(!same_policy_meaning(&inherited, &materialized));
    }

    #[test]
    fn migration_does_not_overwrite_a_newer_project_policy() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let current = serde_json::json!({
                "project_id": "acme",
                "owner_github_id": 1,
                "name": "acme",
                "created_at": "2026-09-16T00:00:00Z",
                "telemetry_policy": {
                    "revision": 3,
                    "base_retention": "7d",
                    "log_retention_override": null,
                    "trace_retention_override": null,
                    "metric_retention_override": null,
                    "max_stored_bytes": "1GiB"
                }
            });
            db.put(
                "ProjectDoc/project_id=acme",
                "",
                &serde_json::to_vec(&current).unwrap(),
            )
            .await
            .unwrap();
            let stale = TelemetryPolicy {
                revision: 1,
                base_retention: "30d".to_string(),
                log_retention_override: None,
                trace_retention_override: None,
                metric_retention_override: None,
                max_stored_bytes: "512MiB".to_string(),
            };

            assert_eq!(
                write_migrated_project_doc(&db, "acme", &stale)
                    .await
                    .unwrap(),
                MigrationWriteResult::AlreadyCurrent
            );
            let stored = db
                .get("ProjectDoc/project_id=acme", "")
                .await
                .unwrap()
                .unwrap();
            let stored: serde_json::Value = serde_json::from_slice(&stored).unwrap();
            assert_eq!(stored["telemetry_policy"]["revision"], 3);
            assert_eq!(stored["telemetry_policy"]["base_retention"], "7d");
        });
    }
}
