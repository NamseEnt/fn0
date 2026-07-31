//! Per-project resolution of the R2 account, buckets and credentials a
//! request's object-storage traffic is signed against.
//!
//! Every storage hijack used to close over one credential set taken from the
//! process environment, which is correct only while every project's objects
//! live in the platform's own Cloudflare account. Projects that bring their own
//! account need the target chosen inside the request path, in the same trust
//! boundary that already knows the project id.
//!
//! [`StaticResolver`] preserves the single-tenant behaviour for the published
//! crate and for self-hosting: one target, handed to every project.

use std::sync::Arc;

/// Credentials for one R2 endpoint.
#[derive(Clone, PartialEq, Eq)]
pub struct R2Credentials {
    pub endpoint_host: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

impl R2Credentials {
    pub fn for_account(
        account_id: &str,
        region: String,
        access_key_id: String,
        secret_access_key: String,
    ) -> Self {
        Self {
            endpoint_host: format!("{account_id}.r2.cloudflarestorage.com"),
            region,
            access_key_id,
            secret_access_key,
        }
    }
}

/// The bucket a project's private objects live in.
#[derive(Clone, PartialEq, Eq)]
pub struct PrivateObjectStorageTarget {
    pub credentials: R2Credentials,
    pub bucket: String,
}

/// The bucket a project's public objects live in, and the origin its CDN serves
/// them from.
#[derive(Clone, PartialEq, Eq)]
pub struct PublicStorageTarget {
    pub credentials: R2Credentials,
    pub bucket: String,
    /// Without a trailing slash.
    pub base_url: String,
}

/// Resolves where a project's private objects live, bucket included. The bucket
/// name is provisioned on the owner's own account and travels with the rest of
/// their configuration; deriving it here instead would put the naming rule in
/// two crates that version apart from each other.
pub trait ObjectStorageResolver: Send + Sync {
    fn resolve(&self, project_id: &str) -> Option<Arc<PrivateObjectStorageTarget>>;
}

pub trait PublicStorageResolver: Send + Sync {
    fn resolve(&self, project_id: &str) -> Option<Arc<PublicStorageTarget>>;
}

/// Hands the same target to every project.
pub struct StaticResolver<T> {
    target: Arc<T>,
}

impl<T> StaticResolver<T> {
    pub fn new(target: T) -> Self {
        Self {
            target: Arc::new(target),
        }
    }
}

impl ObjectStorageResolver for StaticResolver<PrivateObjectStorageTarget> {
    fn resolve(&self, _project_id: &str) -> Option<Arc<PrivateObjectStorageTarget>> {
        Some(self.target.clone())
    }
}

impl PublicStorageResolver for StaticResolver<PublicStorageTarget> {
    fn resolve(&self, _project_id: &str) -> Option<Arc<PublicStorageTarget>> {
        Some(self.target.clone())
    }
}
