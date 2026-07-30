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

/// Credentials for one R2 endpoint. One per store rather than one per project,
/// because the platform account issues a different token to each and because it
/// lets a per-project token be scoped to a single bucket.
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

/// The bucket a project's public objects and deployed static assets live in,
/// and the origin its CDN serves them from.
#[derive(Clone, PartialEq, Eq)]
pub struct PublicStorageTarget {
    pub credentials: R2Credentials,
    pub bucket: String,
    /// Without a trailing slash.
    pub base_url: String,
}

/// Resolves the private object-storage credentials for a project. The bucket
/// itself is not resolved: it is derived from the project id, and bucket names
/// are scoped to an account, so the derivation holds in a user's account
/// exactly as it does in the platform's.
pub trait ObjectStorageResolver: Send + Sync {
    fn resolve(&self, project_id: &str) -> Option<Arc<R2Credentials>>;
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

impl ObjectStorageResolver for StaticResolver<R2Credentials> {
    fn resolve(&self, _project_id: &str) -> Option<Arc<R2Credentials>> {
        Some(self.target.clone())
    }
}

impl PublicStorageResolver for StaticResolver<PublicStorageTarget> {
    fn resolve(&self, _project_id: &str) -> Option<Arc<PublicStorageTarget>> {
        Some(self.target.clone())
    }
}
