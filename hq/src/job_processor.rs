use aws_sdk_s3::Client as S3Client;
use crate::doc_db::DocDb;
use std::time::Duration;
use tracing::*;

pub async fn run(doc_db: DocDb, s3_client: S3Client) {
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let job = match doc_db.claim_job().await {
            Ok(Some(job)) => job,
            Ok(None) => continue,
            Err(err) => {
                warn!(%err, "Failed to claim job");
                continue;
            }
        };

        info!(job_id = %job.id, task = %job.task_name, "Processing job");

        let result = match job.task_name.as_str() {
            "delete_wasm" => process_delete_wasm(&s3_client, &job.payload).await,
            other => {
                warn!(task = other, "Unknown job task");
                Ok(())
            }
        };

        match result {
            Ok(()) => {
                if let Err(err) = doc_db.complete_job(&job.id).await {
                    warn!(%err, job_id = %job.id, "Failed to complete job");
                }
            }
            Err(err) => {
                warn!(%err, job_id = %job.id, "Job failed");
                if job.retry_count + 1 >= job.max_retries {
                    if let Err(err) = doc_db.fail_job(&job.id).await {
                        warn!(%err, job_id = %job.id, "Failed to remove dead job");
                    }
                } else if let Err(err) = doc_db.retry_job(&job.id).await {
                    warn!(%err, job_id = %job.id, "Failed to retry job");
                }
            }
        }
    }
}

async fn process_delete_wasm(s3_client: &S3Client, payload: &str) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    struct DeleteWasmPayload {
        s3_key: String,
        bucket: String,
    }

    let payload: DeleteWasmPayload =
        serde_json::from_str(payload).map_err(|e| format!("Invalid payload: {}", e))?;

    s3_client
        .delete_object()
        .bucket(&payload.bucket)
        .key(&payload.s3_key)
        .send()
        .await
        .map_err(|e| format!("S3 delete failed: {}", e))?;

    info!(key = %payload.s3_key, "Deleted WASM from S3");

    Ok(())
}
