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
}

#[forte_doc]
pub struct WebSocketSingletonReconcileCursorDoc {
    pub after_project_id: Option<String>,
    pub after_singleton_id: Option<String>,
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
    pub redirect_uri: String,
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
