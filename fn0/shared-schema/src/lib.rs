use forte_macros::forte_doc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use doc_db::DbRequest;

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkerProjectManifest {
    pub code_version: u64,
    /// The project's registered domain. A project answers on it and nothing
    /// else, so an entry without one cannot receive a request; the worker
    /// serves nothing for an empty value.
    ///
    /// Deserialization tolerates the pre-rename shape (`custom_domain`, and
    /// `null` from projects that never registered one) so a manifest written
    /// before this field was required does not poison the whole document.
    /// New control writes only `domain`, so roll the worker fleet out before
    /// control: an old worker cannot read a manifest that new control wrote.
    #[serde(
        alias = "custom_domain",
        default,
        deserialize_with = "deserialize_domain"
    )]
    pub domain: String,
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

fn deserialize_domain<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct DomainVisitor;
    impl<'de> serde::de::Visitor<'de> for DomainVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or null")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<String, E> {
            Ok(value.to_string())
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<String, E> {
            Ok(String::new())
        }

        fn visit_some<D: serde::Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<String, D::Error> {
            Deserialize::deserialize(deserializer)
        }
    }
    deserializer.deserialize_option(DomainVisitor)
}

#[cfg(test)]
mod tests {
    use super::WorkerProjectManifest;

    #[test]
    fn domain_reads_the_legacy_custom_domain_key() {
        let entry: WorkerProjectManifest =
            serde_json::from_str(r#"{"code_version":0,"custom_domain":"app.example.com"}"#)
                .unwrap();
        assert_eq!(entry.domain, "app.example.com");
    }

    #[test]
    fn null_legacy_domain_reads_as_empty() {
        let entry: WorkerProjectManifest =
            serde_json::from_str(r#"{"code_version":0,"custom_domain":null}"#).unwrap();
        assert_eq!(entry.domain, "");
    }

    #[test]
    fn domain_serializes_under_its_own_name() {
        let entry: WorkerProjectManifest =
            serde_json::from_str(r#"{"code_version":0,"domain":"app.example.com"}"#).unwrap();
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains(r#""domain":"app.example.com""#));
        assert!(!json.contains("custom_domain"));
    }
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
