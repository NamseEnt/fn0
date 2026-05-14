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
