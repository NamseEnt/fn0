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
    /// Absent only between a project's creation and its owner connecting a
    /// Cloudflare account; a worker cannot serve the project until it is set.
    #[serde(default)]
    pub storage: Option<WorkerProjectStorage>,
}

/// One R2 token as it travels to the worker: the key id in the clear, the
/// secret only as KMS ciphertext, so a leaked manifest row is not a usable
/// credential.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WorkerR2Credential {
    pub access_key_id: String,
    pub secret_ciphertext: String,
}

/// Where one project's objects live, as the worker sees it.
///
/// One credential, scoped to exactly the three buckets named here. The
/// frontend-asset bucket is deliberately outside it: nothing in the worker
/// serves assets — the CDN does, straight off the bucket — so a fleet-wide
/// credential able to rewrite a deployed frontend would be reach with no use
/// for it.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WorkerProjectStorage {
    pub account_id: String,
    pub region: String,
    pub credential: WorkerR2Credential,
    pub private_object_storage_bucket: String,
    pub public_object_storage_bucket: String,
    /// CDN origin for `public_object_storage_bucket`, without a trailing slash.
    pub public_object_storage_base_url: String,
    /// Bumped by control on every credential or bucket change, so a worker can
    /// skip re-decrypting a target it already holds.
    pub config_version: u64,
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

/// A certificate the worker serves for one custom hostname, issued through the
/// project owner's own Cloudflare Origin CA. Only valid for the Cloudflare edge
/// to origin leg, which is the only leg the worker terminates.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WorkerHostnameCert {
    pub project_id: String,
    pub cert_pem: String,
    pub key_ciphertext: String,
    pub not_after_epoch_seconds: i64,
}

/// Certificates live outside `WorkerManifestDoc` because every worker polls
/// that document once a second and a PEM per project is a different order of
/// magnitude from a bucket name. Custom hostnames are far fewer than projects.
#[forte_doc]
pub struct WorkerCertManifestDoc {
    pub cert_version: u64,
    pub certs: HashMap<String, WorkerHostnameCert>,
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
pub struct WebSocketConnectionDoc {
    #[sk]
    pub connection_id: String,
    pub project_id: String,
    pub worker_id: String,
    pub endpoint: String,
}

#[forte_doc]
pub struct WebSocketDirectoryGcCursorDoc {
    pub after_connection_id: Option<String>,
}
