//! Publicly readable object storage.
//!
//! Objects here are served straight from the CDN at a stable URL, so they can
//! be embedded in HTML that outlives any signature. Writing to a key overwrites
//! it in place and invalidates the edge copy, which is what makes the URL safe
//! to persist in a cached page.
//!
//! Everything stored here is world-readable. Key naming is a convention, not
//! access control — put anything that must stay private in [`crate::private`].
//!
//! Browsers are never allowed to hold a copy: objects are stored with
//! `Cache-Control: public, max-age=0, s-maxage=<edge ttl>`, so the edge serves
//! the bytes and the browser revalidates on every request. A cache purge
//! reaches the edge but could never reach a browser, so this is what makes an
//! overwrite visible to returning visitors immediately.

use crate::body::Body;
use crate::http::{self, HttpBucket};
use crate::memory::MemoryBucket;
use crate::{ObjectList, ObjectMetadata, Result};

/// Connects to the project's public object storage.
///
/// Panics if `FN0_PUBLIC_STORAGE_URL` or `FN0_PUBLIC_STORAGE_BASE_URL` is not
/// set — both are always injected by the fn0 runtime, so their absence is a
/// deployment fault, not a recoverable error.
pub fn bucket() -> PublicBucket {
    let endpoint =
        std::env::var("FN0_PUBLIC_STORAGE_URL").expect("FN0_PUBLIC_STORAGE_URL must be set");
    let base_url = std::env::var("FN0_PUBLIC_STORAGE_BASE_URL")
        .expect("FN0_PUBLIC_STORAGE_BASE_URL must be set");
    PublicBucket {
        inner: Backend::Http(HttpBucket::new(endpoint)),
        base_url: base_url.trim_end_matches('/').to_string(),
    }
}

/// In-process public storage for tests. Holds objects in a `BTreeMap` and
/// hands out `memory://` URLs, so [`PublicBucket::url`] stays stable without a
/// live CDN.
pub fn memory() -> PublicBucket {
    PublicBucket {
        inner: Backend::Memory(MemoryBucket::new()),
        base_url: "memory://public".to_string(),
    }
}

#[derive(Clone)]
pub struct PublicBucket {
    inner: Backend,
    base_url: String,
}

#[derive(Clone)]
enum Backend {
    Http(HttpBucket),
    Memory(MemoryBucket),
}

impl PublicBucket {
    /// Stores `body` at `key` and returns its public URL, overwriting any
    /// existing object.
    ///
    /// `content_type` is required: the object is fetched by a browser with no
    /// app in the path to correct a wrong guess.
    ///
    /// Returns once the object is written and its edge copy has been queued for
    /// invalidation, not once the edge is consistent. Until that drains, the
    /// edge may still serve the previous bytes.
    pub async fn put(
        &self,
        key: &str,
        content_type: &str,
        body: impl Into<Body>,
    ) -> Result<String> {
        let body = body.into();
        match &self.inner {
            Backend::Http(b) => b.put(key, body, Some(content_type)).await?,
            Backend::Memory(b) => b.put(key, body, Some(content_type)).await?,
        }
        Ok(self.url(key))
    }

    /// Deletes `key` and invalidates its edge copy. Succeeds whether or not the
    /// object existed.
    pub async fn delete(&self, key: &str) -> Result<()> {
        match &self.inner {
            Backend::Http(b) => b.delete(key).await,
            Backend::Memory(b) => b.delete(key).await,
        }
    }

    /// The public URL for `key`, whether or not an object is stored there. Pure
    /// string building — no request is made.
    pub fn url(&self, key: &str) -> String {
        format!("{}/{}", self.base_url, http::encode_path(key))
    }

    /// Fetches object metadata without downloading the body.
    pub async fn head(&self, key: &str) -> Result<Option<ObjectMetadata>> {
        match &self.inner {
            Backend::Http(b) => b.head(key).await,
            Backend::Memory(b) => b.head(key).await,
        }
    }

    /// Lists objects whose key starts with `prefix`, in ascending key order.
    ///
    /// `after` resumes the listing after a given key (pass
    /// [`ObjectList::next_cursor`] from the previous page). At most `limit`
    /// entries are returned.
    pub async fn list(&self, prefix: &str, after: Option<&str>, limit: usize) -> Result<ObjectList> {
        match &self.inner {
            Backend::Http(b) => b.list(prefix, after, limit).await,
            Backend::Memory(b) => b.list(prefix, after, limit).await,
        }
    }
}
