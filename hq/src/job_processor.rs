use std::{sync::Arc, time::Duration};

use aws_sdk_lambda::{primitives::Blob, types::InvocationType};
use aws_sdk_s3::Client as S3Client;
use doc_db::CompileJobPayload;
use tracing::*;

use crate::{args_parse::DeployContext, wasmtime};

pub async fn run(ctx: Arc<DeployContext>) {
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let job = match ctx.doc_db.claim_job().await {
            Ok(Some(job)) => job,
            Ok(None) => continue,
            Err(err) => {
                warn!(%err, "Failed to claim job");
                continue;
            }
        };

        info!(job_id = %job.id, task = %job.task_name, "Processing job");

        let outcome = match job.task_name.as_str() {
            "delete_wasm" => process_delete_wasm(&ctx.s3_client, &job.payload)
                .await
                .map(|_| None)
                .map_err(|err| (err, None)),
            "compile_for_wasmtime" => process_compile_for_wasmtime(&ctx, &job.payload)
                .await
                .map(Some)
                .map_err(|(err, payload)| (err, payload)),
            other => {
                warn!(task = other, "Unknown job task");
                Ok(None)
            }
        };

        match outcome {
            Ok(maybe_payload) => {
                if let Err(err) = ctx.doc_db.complete_job(&job.id).await {
                    warn!(%err, job_id = %job.id, "Failed to complete job");
                }

                if maybe_payload.is_some() {
                    if let Err(err) = wasmtime::refresh_migration_status(&ctx).await {
                        warn!(%err, job_id = %job.id, "Failed to refresh migration status");
                    }
                }
            }
            Err((err, maybe_payload)) => {
                warn!(%err, job_id = %job.id, "Job failed");

                let is_terminal = job.retry_count + 1 >= job.max_retries;
                if let Some(ref payload) = maybe_payload {
                    let state_result = if is_terminal {
                        ctx.doc_db.mark_compile_failed(&payload, &err).await
                    } else {
                        ctx.doc_db.mark_compile_pending_error(&payload, &err).await
                    };

                    if let Err(state_err) = state_result {
                        warn!(
                            %state_err,
                            job_id = %job.id,
                            code_id = payload.code_id,
                            code_version = payload.code_version,
                            wasmtime_id = %payload.target_wasmtime_id,
                            "Failed to update compiled artifact state"
                        );
                    }
                }

                if is_terminal {
                    if let Err(err) = ctx.doc_db.fail_job(&job.id).await {
                        warn!(%err, job_id = %job.id, "Failed to remove dead job");
                    }
                } else if let Err(err) = ctx.doc_db.retry_job(&job.id).await {
                    warn!(%err, job_id = %job.id, "Failed to retry job");
                }

                if maybe_payload.is_some() {
                    if let Err(err) = wasmtime::refresh_migration_status(&ctx).await {
                        warn!(%err, job_id = %job.id, "Failed to refresh migration status");
                    }
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

async fn process_compile_for_wasmtime(
    ctx: &DeployContext,
    payload_str: &str,
) -> Result<CompileJobPayload, (String, Option<CompileJobPayload>)> {
    let payload: CompileJobPayload = serde_json::from_str(payload_str)
        .map_err(|e| (format!("Invalid compile payload: {}", e), None))?;

    ctx.doc_db
        .mark_compile_processing(&payload)
        .await
        .map_err(|e| (format!("Failed to mark processing: {}", e), Some(payload.clone())))?;

    let lambda_name = wasmtime::lambda_name_for(ctx, &payload.target_wasmtime_id)
        .ok_or_else(|| {
            (
                format!(
                    "No Lambda registered for wasmtime_id={}",
                    payload.target_wasmtime_id
                ),
                Some(payload.clone()),
            )
        })?;

    let request_payload = serde_json::to_vec(&serde_json::json!({
        "code_id": payload.code_id,
        "code_version": payload.code_version,
        "subdomain": payload.subdomain,
        "source_bundle_key": payload.source_bundle_key,
        "target_wasmtime_id": payload.target_wasmtime_id,
        "output_artifact_key": payload.output_artifact_key,
    }))
    .unwrap();

    info!(
        code_id = payload.code_id,
        code_version = payload.code_version,
        wasmtime_id = %payload.target_wasmtime_id,
        lambda_name = %lambda_name,
        artifact_key = %payload.output_artifact_key,
        "Invoking compile lambda"
    );

    let response = ctx
        .lambda_client
        .invoke()
        .function_name(&lambda_name)
        .invocation_type(InvocationType::RequestResponse)
        .payload(Blob::new(request_payload))
        .send()
        .await
        .map_err(|e| (format!("Lambda invoke failed: {}", e), Some(payload.clone())))?;

    if let Some(function_error) = response.function_error() {
        let response_payload = response
            .payload()
            .map(|bytes| String::from_utf8_lossy(bytes.as_ref()).to_string())
            .unwrap_or_default();
        return Err((
            format!("Lambda function error: {function_error}; payload={response_payload}"),
            Some(payload),
        ));
    }

    #[derive(serde::Deserialize)]
    struct LambdaCompileResponse {
        ok: Option<bool>,
        artifact_key: Option<String>,
        error: Option<String>,
        message: Option<String>,
        target_wasmtime_id: Option<String>,
    }

    if let Some(response_payload) = response.payload() {
        if !response_payload.as_ref().is_empty() {
            let parsed: LambdaCompileResponse = serde_json::from_slice(response_payload.as_ref())
                .map_err(|e| {
                    (
                        format!("Invalid lambda response: {}", e),
                        Some(payload.clone()),
                    )
                })?;

            if let Some(target_wasmtime_id) = parsed.target_wasmtime_id {
                if target_wasmtime_id != payload.target_wasmtime_id {
                    return Err((
                        format!(
                            "Lambda target mismatch: expected {}, got {}",
                            payload.target_wasmtime_id, target_wasmtime_id
                        ),
                        Some(payload),
                    ));
                }
            }

            if parsed.ok == Some(false) {
                return Err((
                    parsed
                        .error
                        .or(parsed.message)
                        .unwrap_or_else(|| "Lambda reported failure".to_string()),
                    Some(payload),
                ));
            }

            if let Some(artifact_key) = parsed.artifact_key {
                if artifact_key != payload.output_artifact_key {
                    return Err((
                        format!(
                            "Lambda artifact key mismatch: expected {}, got {}",
                            payload.output_artifact_key, artifact_key
                        ),
                        Some(payload),
                    ));
                }
            }
        }
    }

    ctx.doc_db
        .mark_compile_ready(&payload)
        .await
        .map_err(|e| (format!("Failed to mark ready: {}", e), Some(payload.clone())))?;

    Ok(payload)
}
