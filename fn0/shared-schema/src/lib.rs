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
    /// Absent for projects whose objects live in the platform's own Cloudflare
    /// account; workers fall back to their process-wide target for those.
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
/// Three credentials rather than one because the platform account already
/// issues a separate token per store, and because it leaves room to hand a
/// worker a token scoped to a single bucket.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WorkerProjectStorage {
    pub account_id: String,
    pub region: String,
    pub object: WorkerR2Credential,
    pub public: WorkerR2Credential,
    pub public_bucket: String,
    /// CDN origin for `public_bucket`, without a trailing slash.
    pub public_base_url: String,
    pub page: WorkerR2Credential,
    pub page_bucket: String,
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
