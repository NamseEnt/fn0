use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::tenant::TenantId;
use crate::tenant_policy::PolicyError;

/// Admin request bodies are a single field, so anything larger is a mistake
/// rather than a bigger policy.
pub const MAX_ADMIN_BODY_BYTES: usize = 4 * 1024;

#[derive(Deserialize)]
struct RetentionRequest {
    revision: u64,
    retention: String,
    /// Bytes the tenant may keep stored, as a size such as `10GiB`. Optional
    /// so a control plane that only manages retention keeps working
    /// unchanged; omitting it clears any limit previously pushed for the
    /// tenant — the body is the whole policy, not a patch of it.
    #[serde(default)]
    max_stored_bytes: Option<String>,
    /// Per-signal overrides of `retention`, each optional and each cleared when
    /// omitted, for the same reason as `max_stored_bytes`.
    #[serde(default)]
    log_retention: Option<String>,
    #[serde(default)]
    trace_retention: Option<String>,
    #[serde(default)]
    metric_retention: Option<String>,
}

#[derive(Serialize)]
pub struct RetentionResponse {
    tenant: String,
    revision: u64,
    retention: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_stored_bytes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    log_retention: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_retention: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metric_retention: Option<String>,
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<PushOutcome>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum PushOutcome {
    Applied,
    Duplicate,
    Stale,
}

impl RetentionResponse {
    fn new(
        tenant: &TenantId,
        view: crate::tenant_policy::PolicyView,
        result: Option<PushOutcome>,
    ) -> Self {
        Self {
            tenant: tenant.as_str().to_string(),
            revision: view.revision,
            retention: view.retention,
            max_stored_bytes: view.max_stored_bytes,
            log_retention: view.log_retention,
            trace_retention: view.trace_retention,
            metric_retention: view.metric_retention,
            updated_at: rfc3339(view.updated_at),
            result,
        }
    }
}

#[derive(Serialize)]
pub struct TenantListResponse {
    tenants: Vec<RetentionResponse>,
}

/// `PUT …/tenants/{tenant}/retention` — the control plane's only way to change
/// retention, and the only way a tenant is onboarded at all: the pushed
/// policies are the tenant registry, so the moment this answers 200 the
/// tenant's requests are served. Answered only once the policy is durable, so
/// the caller's retry loop terminates on a real guarantee rather than on an
/// in-memory write.
pub async fn put_retention(
    State(state): State<Arc<AppState>>,
    Path(raw_tenant): Path<String>,
    body: axum::body::Bytes,
) -> Result<Json<RetentionResponse>, (StatusCode, String)> {
    let tenant = parse_tenant_for_change(&state, &raw_tenant)?;
    let request: RetentionRequest = serde_json::from_slice(&body).map_err(|error| {
        state.tenant_policy.record_rejected_push();
        (
            StatusCode::BAD_REQUEST,
            format!("invalid retention request body: {error}"),
        )
    })?;
    let result = state
        .tenant_policy
        .push_with_signal_retentions(
            &tenant,
            request.revision,
            &request.retention,
            request.max_stored_bytes.as_deref(),
            crate::tenant_policy::SignalRetentionRequest {
                logs: request.log_retention.as_deref(),
                traces: request.trace_retention.as_deref(),
                metrics: request.metric_retention.as_deref(),
            },
        )
        .await
        .map_err(into_http)?;
    let (view, outcome) = match result {
        crate::tenant_policy::PushResult::Applied(view) => (view, PushOutcome::Applied),
        crate::tenant_policy::PushResult::Duplicate(view) => (view, PushOutcome::Duplicate),
        crate::tenant_policy::PushResult::Stale(view) => (view, PushOutcome::Stale),
    };
    tracing::info!(%tenant, retention = %view.retention, "tenant policy updated");
    Ok(Json(RetentionResponse::new(&tenant, view, Some(outcome))))
}

pub async fn put_project_retention(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
    body: axum::body::Bytes,
) -> Result<Json<RetentionResponse>, (StatusCode, String)> {
    let tenant = parse_tenant_for_change(&state, &project_id)?;
    let request: RetentionRequest = serde_json::from_slice(&body).map_err(|error| {
        state.tenant_policy.record_rejected_push();
        (
            StatusCode::BAD_REQUEST,
            format!("invalid retention request body: {error}"),
        )
    })?;
    let result = state
        .tenant_policy
        .push_project_with_signal_retentions(
            &tenant,
            request.revision,
            &request.retention,
            request.max_stored_bytes.as_deref(),
            crate::tenant_policy::SignalRetentionRequest {
                logs: request.log_retention.as_deref(),
                traces: request.trace_retention.as_deref(),
                metrics: request.metric_retention.as_deref(),
            },
        )
        .await
        .map_err(into_http)?;
    let (view, outcome) = match result {
        crate::tenant_policy::PushResult::Applied(view) => (view, PushOutcome::Applied),
        crate::tenant_policy::PushResult::Duplicate(view) => (view, PushOutcome::Duplicate),
        crate::tenant_policy::PushResult::Stale(view) => (view, PushOutcome::Stale),
    };
    Ok(Json(RetentionResponse::new(&tenant, view, Some(outcome))))
}

#[derive(Deserialize)]
struct RevokeRequest {
    fence: u64,
}

pub async fn revoke_project_access(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
    body: axum::body::Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    let tenant = parse_tenant_for_change(&state, &project_id)?;
    let request: RevokeRequest = serde_json::from_slice(&body).map_err(|error| {
        state.tenant_policy.record_rejected_push();
        (
            StatusCode::BAD_REQUEST,
            format!("invalid revoke request body: {error}"),
        )
    })?;
    state
        .tenant_policy
        .revoke_project_access(&tenant, request.fence)
        .await
        .map_err(into_http)?;
    Ok(StatusCode::OK)
}

pub async fn get_retention(
    State(state): State<Arc<AppState>>,
    Path(raw_tenant): Path<String>,
) -> Result<Json<RetentionResponse>, (StatusCode, String)> {
    let tenant = parse_tenant(&raw_tenant)?;
    let view = state.tenant_policy.view(&tenant).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("no retention policy for tenant {tenant}"),
        )
    })?;
    Ok(Json(RetentionResponse::new(&tenant, view, None)))
}

/// `GET …/admin/tenants` — every tenant this instance serves, with the policy
/// each one was pushed. The pushed policies are the tenant registry, so this
/// is the control plane's reconciliation read: what it believes it onboarded
/// against what this instance actually holds.
pub async fn list_tenants(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TenantListResponse>, (StatusCode, String)> {
    // The routes are only mounted with a token, so the policy is enabled and
    // the snapshot exists; an empty instance still answers with an empty list.
    let tenants = state
        .tenant_policy
        .snapshot()
        .map(|policies| {
            policies
                .views()
                .map(|(tenant, view)| RetentionResponse::new(tenant, view, None))
                .collect()
        })
        .unwrap_or_default();
    Ok(Json(TenantListResponse { tenants }))
}

/// `DELETE` returns the tenant to *unknown*: its data is kept forever, and —
/// because the pushed policies are the tenant registry — its requests are
/// refused from here on. Deleting the data is `retention: "0"`, pushed before
/// this.
pub async fn delete_retention(
    State(state): State<Arc<AppState>>,
    Path(raw_tenant): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let tenant = parse_tenant_for_change(&state, &raw_tenant)?;
    state
        .tenant_policy
        .remove(&tenant)
        .await
        .map_err(into_http)?;
    tracing::info!(%tenant, "tenant retention policy removed; the tenant keeps its data");
    Ok(StatusCode::OK)
}

/// `GET …/tenants/{tenant}/usage` — what one tenant is currently costing this
/// instance.
///
/// Deliberately here rather than as labels on `/metrics`. That endpoint is
/// unauthenticated and process-wide by design, and a label per tenant would
/// multiply every series by the tenant count — on a workload whose whole point
/// is many small tenants, that is the cardinality problem this engine bounds
/// everywhere else. The reader that actually needs per-tenant numbers is the
/// control plane, which is already authenticated here and already asks per
/// tenant.
pub async fn get_usage(
    State(state): State<Arc<AppState>>,
    Path(raw_tenant): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let tenant = parse_tenant(&raw_tenant)?;

    let window = crate::part::MetadataWindow {
        start_ns: i64::MIN,
        end_ns: i64::MAX,
    };
    let on_disk = state.parts.stats(&tenant, window);
    let buffered = state.memtable.stats(&tenant, window);
    // What a storage plan charges for: the tenant's own extents in the shared
    // objects, every signal together. Asked of the quota rather than summed
    // here, so the number a customer is shown is the number they are refused
    // on. Read from the registries' running census rather than recomputed, so
    // this endpoint costs the same whether the tenant has one part or ten
    // thousand.
    let stored_bytes = state.tenant_quota.tenant_stored_bytes(&tenant);
    Ok(Json(serde_json::json!({
        "tenant": tenant.as_str(),
        "parts": state.parts.tenant_part_count(&tenant),
        "entries": on_disk.entries + buffered.entries,
        "bytes": on_disk.bytes + buffered.bytes,
        // `bytes` above counts logs, including what is buffered and therefore
        // not yet stored anywhere. `stored_bytes` is the durable total the
        // limit is compared against, so a control plane showing a customer
        // their usage reads this one and nothing else.
        "stored_bytes": stored_bytes,
        "max_stored_bytes": state.tenant_quota.max_stored_bytes_for(&tenant),
    })))
}

/// The tenant id arrives in the request path and ends up in an object key, so
/// it goes through the same allowlist as an ingest header rather than a
/// path-specific check.
fn parse_tenant(raw: &str) -> Result<TenantId, (StatusCode, String)> {
    TenantId::parse(raw).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid tenant id: {error}"),
        )
    })
}

/// A rejected id on a method that meant to change the policy. `GET` reads, so
/// a malformed id there is a bad read, not a rejected push.
fn parse_tenant_for_change(state: &AppState, raw: &str) -> Result<TenantId, (StatusCode, String)> {
    parse_tenant(raw).inspect_err(|_| state.tenant_policy.record_rejected_push())
}

fn into_http(error: PolicyError) -> (StatusCode, String) {
    match error {
        // Nothing was stored, so retrying the same request cannot help.
        PolicyError::Invalid(message) => (StatusCode::BAD_REQUEST, message),
        // Nothing was applied either. The control plane owns the retry, which
        // is what bounds the exposure of a delayed upgrade to a retry rather
        // than to an outage.
        PolicyError::Persist(message) => (StatusCode::SERVICE_UNAVAILABLE, message),
        PolicyError::RevisionConflict { revision } => (
            StatusCode::CONFLICT,
            format!("revision {revision} already has a different policy"),
        ),
        PolicyError::ProjectManaged => (
            StatusCode::CONFLICT,
            "tenant is managed by the project control plane".to_string(),
        ),
        PolicyError::AccessRevoked => (
            StatusCode::GONE,
            "project tenant access has been revoked".to_string(),
        ),
        PolicyError::FenceConflict { fence } => (
            StatusCode::CONFLICT,
            format!("access fence {fence} is already in force"),
        ),
    }
}

fn rfc3339(at: std::time::SystemTime) -> String {
    chrono::DateTime::<chrono::Utc>::from(at).to_rfc3339()
}

#[cfg(test)]
mod tests {
    include!("tests/admin.rs");
}
