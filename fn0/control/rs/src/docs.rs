use forte_sdk::*;
use serde::{Deserialize, Serialize};

pub use doc_db::DbRequest;
pub use fn0_shared_schema::{
    WorkerCertManifestDoc, WorkerCertManifestDocGet, WorkerCertManifestDocPut, WorkerHostnameCert,
    WorkerManifestDoc, WorkerManifestDocGet, WorkerManifestDocPut, WorkerManifestDocQuery,
    WorkerProjectManifest, WorkerProjectStorage, WorkerR2Credential,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct CliTokenEntry {
    pub id: String,
    pub label: String,
    pub created_at: DateTime,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WebSessionEntry {
    pub token: String,
    pub created_at: DateTime,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectIndexEntry {
    pub project_id: String,
    pub name: String,
}

#[forte_doc]
pub struct UserDoc {
    #[pk]
    pub github_id: i64,
    pub github_login: String,
    pub created_at: DateTime,
    pub cli_tokens: Vec<CliTokenEntry>,
    pub web_sessions: Vec<WebSessionEntry>,
    pub projects: Vec<ProjectIndexEntry>,
}

#[forte_doc]
pub struct ProjectDoc {
    #[pk]
    pub project_id: String,
    pub owner_github_id: i64,
    pub name: String,
    pub created_at: DateTime,
    pub telemetry_policy: TelemetryPolicy,
}

/// The concrete policy selected by control for one project. These are actual
/// Signy values, never a fallback marker or a migration sentinel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TelemetryPolicy {
    pub revision: u64,
    /// The tenant-level retention. A signal without an override inherits it.
    pub base_retention: String,
    /// `None` is meaningful: it preserves Signy's signal inheritance rather
    /// than materialising the current effective value.
    pub log_retention_override: Option<String>,
    pub trace_retention_override: Option<String>,
    pub metric_retention_override: Option<String>,
    pub max_stored_bytes: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryPolicySyncState {
    Pending,
    Applied,
    Failed,
    DeadLetter,
    BlockedByDeletion,
}

#[forte_doc]
pub struct TelemetryPolicyOutboxDoc {
    #[pk]
    pub project_id: String,
    pub policy_revision: u64,
    #[serde(default)]
    pub policy: Option<TelemetryPolicy>,
    pub state: TelemetryPolicySyncState,
    pub attempts: u64,
    pub last_error: Option<String>,
    /// The first time the current pending cycle was observed. This is kept
    /// separately from `updated_at`, which changes on every retry.
    #[serde(default)]
    pub pending_since: Option<DateTime>,
    pub updated_at: DateTime,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectDeletionState {
    RevokePending,
    AccessRevoked,
    TeardownComplete,
}

#[forte_doc]
pub struct ProjectDeletionDoc {
    #[pk]
    pub project_id: String,
    pub fence: u64,
    pub state: ProjectDeletionState,
    /// The start of the durable revoke-pending interval, independent of retry
    /// updates to `updated_at`.
    #[serde(default)]
    pub revoke_pending_since: Option<DateTime>,
    pub updated_at: DateTime,
    pub last_error: Option<String>,
}

/// Projects whose old ProjectDoc cannot be upgraded because Signy has no
/// explicit policy. This is an operator exception, not a policy value.
#[forte_doc]
pub struct TelemetryPolicyMigrationExceptionDoc {
    #[pk]
    pub project_id: String,
    pub reason: String,
    pub observed_at: DateTime,
}

#[forte_doc]
pub struct CompiledBundleDoc {
    #[pk]
    pub project_id: String,
    #[sk]
    pub code_version: u64,
    pub created_at: DateTime,
    pub fn0_wasmtime_versions: Vec<String>,
}

#[forte_doc]
pub struct Fn0WasmtimeVersionDoc {
    pub active: String,
    pub pending: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CronJob {
    pub function: String,
    pub every_minutes: u32,
}

#[forte_doc]
pub struct CronConfigDoc {
    #[sk]
    pub project_id: String,
    pub jobs: Vec<CronJob>,
    pub updated_at: DateTime,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WebSocketSingletonDeclaration {
    pub singleton_id: String,
    pub route_path: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketSingletonRuntimeState {
    Preparing,
    #[default]
    Active,
    Terminating,
}

#[forte_doc]
pub struct WebSocketSingletonConfigDoc {
    #[pk]
    pub project_id: String,
    #[sk]
    pub code_version: u64,
    pub declarations: Vec<WebSocketSingletonDeclaration>,
}

#[forte_doc]
pub struct WebSocketSingletonRuntimeDoc {
    #[pk]
    pub project_id: String,
    #[sk]
    pub singleton_id: String,
    pub code_version: u64,
    #[serde(default)]
    pub claim_token: String,
    pub connection_id: String,
    pub lease_expires_at: DateTime,
    #[serde(default)]
    pub state: WebSocketSingletonRuntimeState,
}

#[forte_doc]
pub struct WebSocketSingletonReconcileCursorDoc {
    pub after_project_id: Option<String>,
    pub after_singleton_id: Option<String>,
}

/// Monthly compute egress a project may send. Operators change it by editing this document in
/// the control database; `new_project` writes the platform default. A project without this
/// document is refused all egress.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonthlyEgressLimit {
    Bytes(u64),
    Unlimited,
}

#[forte_doc]
pub struct ProjectEgressQuotaDoc {
    #[pk]
    pub project_id: String,
    pub monthly_egress_limit: MonthlyEgressLimit,
}

/// Bytes handed to workers as egress credit in one UTC calendar month (`YYYY-MM`). Credit is
/// charged when it is granted, so a worker that exits with unused credit still counts it.
#[forte_doc]
pub struct ProjectEgressUsageDoc {
    #[pk]
    pub project_id: String,
    #[sk]
    pub month: String,
    pub granted_bytes: u64,
}

#[forte_doc]
pub struct WaitlistDoc {
    #[sk]
    pub email: String,
    pub tier_interest: String,
    pub created_at: DateTime,
}

#[forte_doc]
pub struct CliAuthorizationCodeDoc {
    #[pk]
    pub code: String,
    pub github_id: i64,
    pub code_challenge: String,
    pub redirect_uri: Option<String>,
    pub label: String,
    pub expires_at: DateTime,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum CloudflareConnectionState {
    Ok,
    /// The credentials no longer satisfy what the platform needs. The project
    /// keeps running on them until they actually fail, because a permission we
    /// cannot see is not the same as a permission that is gone.
    Degraded {
        missing: Vec<String>,
    },
}

/// The Cloudflare account a project's objects, assets, cached HTML and custom
/// domain live in.
///
/// Deliberately absent: the account-wide token that created all of this. That
/// token can delete every bucket in the user's account and sign certificates
/// for their zone, and fn0 never receives it — the CLI uses it on the user's
/// own machine to provision, mints the narrow credentials below, and discards
/// it.
///
/// Every bucket is this project's alone, and the two that are served publicly
/// answer on a hostname that is their own name in the owner's zone. Nothing is
/// shared between projects, so no key prefix carries tenancy and no listing
/// walks another project's objects.
#[forte_doc]
pub struct ProjectCloudflareConfigDoc {
    #[pk]
    pub project_id: String,
    pub account_id: String,
    pub zone_id: String,
    pub zone_name: String,
    pub frontend_asset_hostname: String,
    pub public_object_storage_hostname: String,
    pub private_object_storage_bucket: String,
    pub public_object_storage_bucket: String,
    pub frontend_asset_bucket: String,
    /// R2 object read+write on the two object-storage buckets. The only R2
    /// credential published to the fleet, so
    /// this is what a full compromise of the workers would yield.
    pub worker_access_key_id: String,
    pub worker_secret_ciphertext: String,
    /// R2 object read+write on the frontend-asset bucket alone, and never sent
    /// to a worker. Asset GC deletes on a schedule; scoping it here is what
    /// bounds a mistake in it to artifacts a redeploy rebuilds.
    pub frontend_asset_access_key_id: String,
    pub frontend_asset_secret_ciphertext: String,
    /// Cache purge on this one zone, and nothing else. Runtime needs it because
    /// a public object write purges its edge copy, and that happens on a
    /// request rather than on the user's machine.
    pub purge_token_ciphertext: String,
    pub state: CloudflareConnectionState,
    pub checked_at: DateTime,
    /// Bumped on every credential or bucket change so workers can skip
    /// re-decrypting a target they already hold.
    pub config_version: u64,
}
