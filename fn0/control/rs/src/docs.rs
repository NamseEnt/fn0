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

// Cloudflare rate-limits tag purges per account, not per zone or per project,
// so the budget is shared platform-wide and has to be accounted for in one
// place. A request identifies a project's one purge obligation for a given
// code version and phase, which is what lets a batch drained by one deploy's
// task satisfy another deploy's task.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CachePurgeRequest {
    pub project_id: String,
    pub code_version: u64,
    pub phase: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CompletedCachePurge {
    pub request: CachePurgeRequest,
    pub purged_at: DateTime,
}

#[forte_doc]
pub struct CachePurgeDoc {
    pub tokens: f64,
    pub refilled_at: DateTime,
    pub pending: Vec<CachePurgeRequest>,
    pub completed: Vec<CompletedCachePurge>,
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
    /// Credentials verified and buckets created, but the project's existing
    /// objects have not finished copying across. Reads and writes stay on the
    /// platform account until they have: switching first would 404 every
    /// object written before the account was connected.
    Migrating,
    Ok,
    /// The credentials no longer satisfy what the platform needs. The project
    /// keeps running on them until they actually fail, because a permission we
    /// cannot see is not the same as a permission that is gone.
    Degraded {
        missing: Vec<String>,
    },
}

/// The Cloudflare account a project's objects, assets, pages and custom domain
/// live in.
///
/// Deliberately absent: the account-wide token that created all of this. That
/// token can delete every bucket in the user's account and sign certificates
/// for their zone, and fn0 never receives it — the CLI uses it on the user's
/// own machine to provision, mints the two narrow credentials below, and
/// discards it. What is stored here is what a full compromise of the worker
/// fleet would yield: object access to three buckets, and cache purge.
///
/// Buckets are per Cloudflare account, not per project: one account's projects
/// share `asset_bucket` and `page_bucket` under `{project_id}/` prefixes, so a
/// user's zone needs one `static.` DNS record however many projects they run.
/// `object_bucket` is per project.
#[forte_doc]
pub struct ProjectCloudflareConfigDoc {
    #[pk]
    pub project_id: String,
    pub account_id: String,
    pub zone_id: String,
    pub zone_name: String,
    pub static_hostname: String,
    pub object_bucket: String,
    pub asset_bucket: String,
    pub page_bucket: String,
    /// R2 object read+write, scoped to the three buckets above. Cannot delete a
    /// bucket, cannot reach any other bucket, and cannot call the REST API at
    /// all — measured, not assumed.
    pub dataplane_access_key_id: String,
    pub dataplane_secret_ciphertext: String,
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
