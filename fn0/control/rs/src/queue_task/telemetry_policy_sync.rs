//! Applies the latest ProjectDoc telemetry policy to Signy.
//!
//! Queue payloads deliberately contain only the project id. The handler reads
//! ProjectDoc at execution time, so a delayed message cannot overwrite a newer
//! policy. A deletion tombstone wins over every registration attempt.

use crate::common::signy_tenant;
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
    let Some(mut outbox) = (TelemetryPolicyOutboxDocGet {
        project_id: &project_id,
    })
    .send_with(&db)
    .await?
    else {
        tracing::warn!(%project_id, "telemetry policy sync has no outbox record");
        return Ok(());
    };

    if let Some(tombstone) = (ProjectDeletionDocGet {
        project_id: &project_id,
    })
    .send_with(&db)
    .await?
    {
        outbox.state = TelemetryPolicySyncState::BlockedByDeletion;
        outbox.last_error = Some(format!("project deletion is {:?}", tombstone.state));
        outbox.updated_at = now();
        (TelemetryPolicyOutboxDocPut(outbox)).send_with(&db).await?;
        return Ok(());
    }

    let Some(project) = (ProjectDocGet {
        project_id: &project_id,
    })
    .send_with(&db)
    .await?
    else {
        outbox.state = TelemetryPolicySyncState::BlockedByDeletion;
        outbox.last_error = Some("ProjectDoc is absent".to_string());
        outbox.updated_at = now();
        (TelemetryPolicyOutboxDocPut(outbox)).send_with(&db).await?;
        return Ok(());
    };

    outbox.attempts = outbox.attempts.saturating_add(1);
    outbox.policy_revision = project.telemetry_policy.revision;
    outbox.state = TelemetryPolicySyncState::Pending;
    outbox.last_error = None;
    outbox.updated_at = now();
    (TelemetryPolicyOutboxDocPut(outbox.clone()))
        .send_with(&db)
        .await?;

    match signy_tenant::register_project(&project_id, &project.telemetry_policy).await {
        Ok(()) => {
            outbox.state = TelemetryPolicySyncState::Applied;
            outbox.last_error = None;
            outbox.updated_at = now();
            (TelemetryPolicyOutboxDocPut(outbox)).send_with(&db).await?;
            tracing::info!(%project_id, revision = project.telemetry_policy.revision, "telemetry policy applied");
            Ok(())
        }
        Err(error) => {
            let detail = error.to_string();
            outbox.state = TelemetryPolicySyncState::Failed;
            outbox.last_error = Some(detail.clone());
            outbox.updated_at = now();
            (TelemetryPolicyOutboxDocPut(outbox)).send_with(&db).await?;
            tracing::error!(%project_id, %detail, "telemetry policy apply failed; queue will retry");
            Err(error)
        }
    }
}
