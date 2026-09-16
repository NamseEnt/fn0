//! One-shot pre-schema migration for the required ProjectDoc policy field.
//!
//! Existing policy values come from Signy. A project without a Signy policy is
//! recorded in TelemetryPolicyMigrationExceptionDoc and is never backfilled.
//! This task must report zero exceptions before the required ProjectDoc schema
//! is rolled out.

use crate::common::signy_tenant;
use crate::docs::*;
use forte_sdk::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input;

pub async fn handle(Input: Input) -> anyhow::Result<()> {
    let db = doc_db::turso();
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
        for (pk, sk, bytes) in &page {
            if !pk.starts_with("ProjectDoc/") {
                continue;
            }
            let mut value: serde_json::Value = serde_json::from_slice(bytes)
                .map_err(|error| anyhow::anyhow!("invalid {pk}: {error}"))?;
            let Some(project_id) = value
                .get("project_id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
            else {
                anyhow::bail!("ProjectDoc {pk} has no project_id");
            };
            if let Some(policy_value) = value.get("telemetry_policy") {
                let policy: TelemetryPolicy = serde_json::from_value(policy_value.clone())
                    .map_err(|error| {
                        anyhow::anyhow!("invalid telemetry_policy in {pk}: {error}")
                    })?;
                if (TelemetryPolicyOutboxDocGet {
                    project_id: &project_id,
                })
                .send_with(&db)
                .await?
                .is_none()
                {
                    (TelemetryPolicyOutboxDocPut(TelemetryPolicyOutboxDoc {
                        project_id: project_id.clone(),
                        policy_revision: policy.revision,
                        state: TelemetryPolicySyncState::Pending,
                        attempts: 0,
                        last_error: None,
                        updated_at: now(),
                    }))
                    .send_with(&db)
                    .await?;
                }
                (TelemetryPolicyMigrationExceptionDocDelete {
                    project_id: &project_id,
                })
                .send_with(&db)
                .await?;
                continue;
            }
            let Some(mut policy) = signy_tenant::read_policy(&project_id).await? else {
                exceptions += 1;
                (TelemetryPolicyMigrationExceptionDocPut(TelemetryPolicyMigrationExceptionDoc {
                    project_id: project_id.clone(),
                    reason: "Signy has no explicit tenant policy".to_string(),
                    observed_at: now(),
                }))
                .send_with(&db)
                .await?;
                tracing::error!(%project_id, "ProjectDoc migration requires an explicit Signy policy");
                continue;
            };
            // The migration claims the unchanged values under the next
            // revision. This works for both legacy Signy documents without a
            // revision (0 -> 1) and policies that an operator pushed after
            // the revision field was introduced (N -> N+1).
            policy.revision = policy.revision.saturating_add(1).max(1);
            // Claim the unchanged values before ProjectDoc is written. Once
            // this succeeds, the generic operator PUT cannot mutate this
            // tenant and the normal outbox retry can safely resend the same
            // revision. The migration changes only revision metadata; all
            // retention and storage values came from Signy above.
            signy_tenant::register_project(&project_id, &policy).await?;
            value["telemetry_policy"] = serde_json::to_value(&policy)?;
            db.put(pk, sk, &serde_json::to_vec(&value)?).await?;
            (TelemetryPolicyOutboxDocPut(TelemetryPolicyOutboxDoc {
                project_id: project_id.clone(),
                policy_revision: policy.revision,
                state: TelemetryPolicySyncState::Pending,
                attempts: 0,
                last_error: None,
                updated_at: now(),
            }))
            .send_with(&db)
            .await?;
            (TelemetryPolicyMigrationExceptionDocDelete {
                project_id: &project_id,
            })
            .send_with(&db)
            .await?;
            migrated += 1;
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
    if exceptions > 0 {
        anyhow::bail!("{exceptions} projects require explicit telemetry policy migration");
    }
    Ok(())
}
