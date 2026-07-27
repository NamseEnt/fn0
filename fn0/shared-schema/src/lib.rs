use forte_macros::forte_doc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use doc_db::DbRequest;

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkerProjectManifest {
    pub code_version: u64,
    pub custom_domain: Option<String>,
    #[serde(default = "default_static_cache_state")]
    pub static_cache_state: String,
    #[serde(default)]
    pub pending_code_version: Option<u64>,
}

pub const STATIC_CACHE_STATE_ACTIVE: &str = "active";
pub const STATIC_CACHE_STATE_PRE_PURGE: &str = "pre_purge";
pub const STATIC_CACHE_STATE_ACTIVATING: &str = "activating";

fn default_static_cache_state() -> String {
    STATIC_CACHE_STATE_ACTIVE.to_string()
}

#[forte_doc]
pub struct WorkerManifestDoc {
    pub manifest_version: u64,
    pub project_manifests: HashMap<String, WorkerProjectManifest>,
}

#[forte_doc]
pub struct WorkerHostStatusDoc {
    #[sk]
    pub host_id: String,
    pub addr: String,
    pub active_image_ref: Option<String>,
    pub reported_at: i64,
}

#[forte_doc]
pub struct PresignBlockedDoc {
    pub updated_epoch_hour: i64,
    pub blocked_project_ids: Vec<String>,
}

#[forte_doc]
pub struct PresignMintCountDoc {
    #[pk]
    pub project_id: String,
    #[sk]
    pub window_epoch_hour: i64,
    #[sk]
    pub writer_id: String,
    pub minted: u64,
}
