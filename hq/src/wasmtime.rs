use std::sync::Arc;

use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, body::Bytes};
use serde::{Deserialize, Serialize};

use crate::args_parse::DeployContext;
use doc_db::{CompileJobPayload, DocDbResult, LatestCode, MigrationProgress, MigrationStatus};

#[derive(Deserialize)]
struct WasmtimeChangeRequest {
    target_wasmtime_id: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct WasmtimeChangeResponse {
    active_wasmtime_id: String,
    preparing_wasmtime_id: Option<String>,
    migration_status: MigrationStatus,
    total_target_count: u64,
    ready_count: u64,
    failed_count: u64,
    retrying_count: u64,
    recent_errors: Vec<String>,
    target_wasmtime_id: Option<String>,
}

fn json_response<T: Serialize>(status: u16, body: &T) -> Response<Full<Bytes>> {
    let json = serde_json::to_string(body).unwrap();
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap()
}

pub fn active_artifact_key(subdomain: &str) -> String {
    format!("bundles/{subdomain}.tar.zst")
}

pub fn migration_artifact_key(wasmtime_id: &str, code_id: u64, code_version: u64) -> String {
    format!("compiled/{wasmtime_id}/{code_id}/{code_version}/bundle.tar.zst")
}

pub fn lambda_name_for(ctx: &DeployContext, wasmtime_id: &str) -> Option<String> {
    ctx.wasmtime_lambdas.get(wasmtime_id).cloned()
}

pub async fn compile_targets_for_deploy(
    ctx: &DeployContext,
    code_id: u64,
    subdomain: &str,
    code_version: u64,
    source_bundle_key: &str,
) -> DocDbResult<Vec<CompileJobPayload>> {
    let state = ctx.doc_db.get_migration_state().await?;
    let mut targets = vec![CompileJobPayload {
        code_id,
        subdomain: subdomain.to_string(),
        code_version,
        source_bundle_key: source_bundle_key.to_string(),
        target_wasmtime_id: state.active_wasmtime_id.clone(),
        output_artifact_key: active_artifact_key(subdomain),
    }];

    if state.migration_status == MigrationStatus::Running {
        if let Some(preparing_wasmtime_id) = state.preparing_wasmtime_id {
            targets.push(CompileJobPayload {
                code_id,
                subdomain: subdomain.to_string(),
                code_version,
                source_bundle_key: source_bundle_key.to_string(),
                output_artifact_key: migration_artifact_key(
                    &preparing_wasmtime_id,
                    code_id,
                    code_version,
                ),
                target_wasmtime_id: preparing_wasmtime_id,
            });
        }
    }

    Ok(targets)
}

pub async fn refresh_migration_status(ctx: &DeployContext) -> DocDbResult<MigrationProgress> {
    let progress = ctx.doc_db.get_migration_progress().await?;
    let Some(target_wasmtime_id) = progress.state.preparing_wasmtime_id.clone() else {
        return Ok(progress);
    };

    if progress.state.migration_status == MigrationStatus::Running {
        if progress.failed_count > 0 {
            ctx.doc_db.mark_migration_failed(&target_wasmtime_id).await?;
            return ctx.doc_db.get_migration_progress().await;
        }

        if progress.ready_count == progress.total_latest_codes {
            ctx.doc_db.mark_migration_prepared(&target_wasmtime_id).await?;
            return ctx.doc_db.get_migration_progress().await;
        }
    }

    Ok(progress)
}

pub async fn reconcile_running_migration(ctx: &DeployContext) -> DocDbResult<MigrationProgress> {
    let state = ctx.doc_db.get_migration_state().await?;
    if state.migration_status != MigrationStatus::Running {
        return refresh_migration_status(ctx).await;
    }

    let Some(target_wasmtime_id) = state.preparing_wasmtime_id else {
        return refresh_migration_status(ctx).await;
    };

    let latest_codes = ctx.doc_db.list_latest_codes().await?;
    let payloads: Vec<_> = latest_codes
        .into_iter()
        .map(|latest| compile_payload_for_latest(latest, &target_wasmtime_id))
        .collect();
    ctx.doc_db.ensure_compile_jobs(&payloads).await?;
    refresh_migration_status(ctx).await
}

pub async fn handle_change_post(
    req: Request<hyper::body::Incoming>,
    ctx: Arc<DeployContext>,
) -> Response<Full<Bytes>> {
    let body = match req.collect().await {
        Ok(b) => b.to_bytes(),
        Err(_) => {
            return json_response(
                400,
                &ErrorResponse {
                    error: "Failed to read body".to_string(),
                },
            );
        }
    };

    let request: WasmtimeChangeRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => {
            return json_response(
                400,
                &ErrorResponse {
                    error: "Invalid request body".to_string(),
                },
            );
        }
    };

    if !ctx
        .wasmtime_lambdas
        .contains_key(&request.target_wasmtime_id)
    {
        return json_response(
            400,
            &ErrorResponse {
                error: format!(
                    "Unknown wasmtime_id: {}",
                    request.target_wasmtime_id
                ),
            },
        );
    }

    let state = match ctx.doc_db.get_migration_state().await {
        Ok(state) => state,
        Err(err) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to read migration state: {err}"),
                },
            );
        }
    };

    if state.active_wasmtime_id == request.target_wasmtime_id {
        return json_response(
            400,
            &ErrorResponse {
                error: "target_wasmtime_id is already active".to_string(),
            },
        );
    }

    if state.migration_status == MigrationStatus::Running {
        return json_response(
            409,
            &ErrorResponse {
                error: "Another wasmtime change is already running".to_string(),
            },
        );
    }

    match ctx
        .doc_db
        .compare_and_start_migration(&request.target_wasmtime_id)
        .await
    {
        Ok(true) => {}
        Ok(false) => {
            return json_response(
                409,
                &ErrorResponse {
                    error: "Another wasmtime change is already running".to_string(),
                },
            );
        }
        Err(err) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to start migration: {err}"),
                },
            );
        }
    }

    let latest_codes = match ctx.doc_db.list_latest_codes().await {
        Ok(codes) => codes,
        Err(err) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to list latest codes: {err}"),
                },
            );
        }
    };

    let payloads: Vec<_> = latest_codes
        .iter()
        .cloned()
        .map(|latest| compile_payload_for_latest(latest, &request.target_wasmtime_id))
        .collect();

    if let Err(err) = ctx.doc_db.ensure_compile_jobs(&payloads).await {
        return json_response(
            500,
            &ErrorResponse {
                error: format!("Failed to enqueue compile jobs: {err}"),
            },
        );
    }

    let progress = match refresh_migration_status(&ctx).await {
        Ok(progress) => progress,
        Err(err) => {
            return json_response(
                500,
                &ErrorResponse {
                    error: format!("Failed to read migration progress: {err}"),
                },
            );
        }
    };

    json_response(
        200,
        &WasmtimeChangeResponse {
            active_wasmtime_id: progress.state.active_wasmtime_id,
            preparing_wasmtime_id: progress.state.preparing_wasmtime_id.clone(),
            migration_status: progress.state.migration_status,
            total_target_count: progress.total_latest_codes,
            ready_count: progress.ready_count,
            failed_count: progress.failed_count,
            retrying_count: progress.retrying_count,
            recent_errors: progress.recent_errors,
            target_wasmtime_id: Some(request.target_wasmtime_id),
        },
    )
}

pub async fn handle_change_get(ctx: Arc<DeployContext>) -> Response<Full<Bytes>> {
    match refresh_migration_status(&ctx).await {
        Ok(progress) => json_response(200, &change_response(progress)),
        Err(err) => json_response(
            500,
            &ErrorResponse {
                error: format!("Failed to read migration progress: {err}"),
            },
        ),
    }
}

fn change_response(progress: MigrationProgress) -> WasmtimeChangeResponse {
    WasmtimeChangeResponse {
        target_wasmtime_id: progress.state.preparing_wasmtime_id.clone(),
        active_wasmtime_id: progress.state.active_wasmtime_id,
        preparing_wasmtime_id: progress.state.preparing_wasmtime_id,
        migration_status: progress.state.migration_status,
        total_target_count: progress.total_latest_codes,
        ready_count: progress.ready_count,
        failed_count: progress.failed_count,
        retrying_count: progress.retrying_count,
        recent_errors: progress.recent_errors,
    }
}

fn compile_payload_for_latest(latest: LatestCode, target_wasmtime_id: &str) -> CompileJobPayload {
    CompileJobPayload {
        code_id: latest.code_id,
        subdomain: latest.subdomain.clone(),
        code_version: latest.latest_code_version,
        source_bundle_key: latest.latest_source_bundle_key.clone(),
        target_wasmtime_id: target_wasmtime_id.to_string(),
        output_artifact_key: migration_artifact_key(
            target_wasmtime_id,
            latest.code_id,
            latest.latest_code_version,
        ),
    }
}
