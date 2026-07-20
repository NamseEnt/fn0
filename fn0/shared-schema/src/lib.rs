use forte_macros::forte_doc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use doc_db::DbRequest;

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkerProjectManifest {
    pub code_version: u64,
    pub custom_domain: Option<String>,
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
