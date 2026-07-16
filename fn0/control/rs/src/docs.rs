use forte_sdk::*;
use serde::{Deserialize, Serialize};

pub use doc_db::DbRequest;
pub use fn0_shared_schema::{
    WorkerManifestDoc, WorkerManifestDocGet, WorkerManifestDocPut, WorkerManifestDocQuery,
    WorkerProjectManifest,
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

#[forte_doc]
pub struct WaitlistDoc {
    #[sk]
    pub email: String,
    pub tier_interest: String,
    pub created_at: DateTime,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct SampledOperationCount {
    pub estimate: u64,
    pub lower_bound: u64,
    pub upper_bound: u64,
    pub is_valid: bool,
}

#[forte_doc]
pub struct ProjectOperationsUsageDoc {
    #[pk]
    pub project_id: String,
    #[sk]
    pub window_start_epoch_hour: i64,
    pub static_asset_reads: SampledOperationCount,
    pub static_asset_writes: SampledOperationCount,
    pub object_storage_reads: SampledOperationCount,
    pub object_storage_writes: SampledOperationCount,
    pub polled_at: DateTime,
}

#[forte_doc]
pub struct ProjectStorageSnapshotDoc {
    #[pk]
    pub project_id: String,
    #[sk]
    pub snapshot_epoch_hour: i64,
    pub static_asset_bytes: u64,
    pub static_asset_object_count: u64,
    pub object_storage_bytes: u64,
    pub object_storage_object_count: u64,
    pub taken_at: DateTime,
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
