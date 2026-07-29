//! Per-project private object storage.
//!
//! Objects here are reachable only through the app or a presigned URL. Nothing
//! written to this bucket is publicly addressable; use [`crate::public`] for
//! objects a browser must fetch directly.

use crate::body::Body;
use crate::http::HttpBucket;
use crate::memory::MemoryBucket;
use crate::{Object, ObjectList, ObjectMetadata, Result};
use std::time::Duration;

/// Connects to the project's private object storage.
///
/// Panics if `FN0_OBJECT_STORAGE_URL` is not set — it is always injected by the
/// fn0 runtime, so its absence is a deployment fault, not a recoverable error.
pub fn bucket() -> PrivateBucket {
    let endpoint =
        std::env::var("FN0_OBJECT_STORAGE_URL").expect("FN0_OBJECT_STORAGE_URL must be set");
    PrivateBucket {
        inner: Backend::Http(HttpBucket::new(endpoint)),
    }
}

/// In-process private storage for tests. Holds objects in a `BTreeMap`.
pub fn memory() -> PrivateBucket {
    PrivateBucket {
        inner: Backend::Memory(MemoryBucket::new()),
    }
}

#[derive(Clone)]
pub struct PrivateBucket {
    inner: Backend,
}

#[derive(Clone)]
enum Backend {
    Http(HttpBucket),
    Memory(MemoryBucket),
}

impl PrivateBucket {
    /// Stores `body` at `key`, overwriting any existing object.
    ///
    /// Pass a `content_type` for anything a browser will fetch through
    /// [`PrivateBucket::presigned_get_url`]: no app is in that path to correct a
    /// wrong guess, and R2 serves no `Content-Type` at all when none was stored.
    /// `None` stores the object without one, which is what round-trips an
    /// [`Object`] that had none.
    pub async fn put(
        &self,
        key: &str,
        content_type: Option<&str>,
        body: impl Into<Body>,
    ) -> Result<()> {
        let body = body.into();
        match &self.inner {
            Backend::Http(b) => b.put(key, body, content_type).await,
            Backend::Memory(b) => b.put(key, body, content_type).await,
        }
    }

    /// Fetches the object at `key`, or `None` if it does not exist.
    ///
    /// The body is returned unread: call [`Body::bytes`] for the whole object,
    /// or hand it to [`PrivateBucket::put`] or an outgoing request to pipe it
    /// on without holding it in memory.
    pub async fn get(&self, key: &str) -> Result<Option<Object>> {
        match &self.inner {
            Backend::Http(b) => b.get(key).await,
            Backend::Memory(b) => b.get(key).await,
        }
    }

    /// Fetches object metadata without downloading the body.
    pub async fn head(&self, key: &str) -> Result<Option<ObjectMetadata>> {
        match &self.inner {
            Backend::Http(b) => b.head(key).await,
            Backend::Memory(b) => b.head(key).await,
        }
    }

    /// Deletes `key`. Succeeds whether or not the object existed.
    pub async fn delete(&self, key: &str) -> Result<()> {
        match &self.inner {
            Backend::Http(b) => b.delete(key).await,
            Backend::Memory(b) => b.delete(key).await,
        }
    }

    /// Lists objects whose key starts with `prefix`, in ascending key order.
    ///
    /// `after` resumes the listing after a given key (pass
    /// [`ObjectList::next_cursor`] from the previous page). At most `limit`
    /// entries are returned.
    pub async fn list(
        &self,
        prefix: &str,
        after: Option<&str>,
        limit: usize,
    ) -> Result<ObjectList> {
        match &self.inner {
            Backend::Http(b) => b.list(prefix, after, limit).await,
            Backend::Memory(b) => b.list(prefix, after, limit).await,
        }
    }

    /// Returns a URL for downloading `key` directly, without routing through
    /// the app — e.g. handed to a browser. Valid for `expires`; fn0 Cloud
    /// clamps longer durations to 5 minutes.
    pub async fn presigned_get_url(&self, key: &str, expires: Duration) -> Result<String> {
        match &self.inner {
            Backend::Http(b) => b.presigned_url(key, "GET", expires, None).await,
            Backend::Memory(b) => b.presigned_url(key, "GET", expires, None).await,
        }
    }

    /// Returns a URL for uploading directly to `key`, without routing through
    /// the app — e.g. a browser `PUT`. Valid for `expires`; fn0 Cloud clamps
    /// longer durations to 5 minutes.
    ///
    /// `content_length` binds the URL to an upload of exactly that many bytes;
    /// `None` accepts any size.
    pub async fn presigned_put_url(
        &self,
        key: &str,
        content_length: Option<u64>,
        expires: Duration,
    ) -> Result<String> {
        match &self.inner {
            Backend::Http(b) => b.presigned_url(key, "PUT", expires, content_length).await,
            Backend::Memory(b) => b.presigned_url(key, "PUT", expires, content_length).await,
        }
    }
}
