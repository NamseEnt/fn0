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
//! Reading is split by which copy answers, because the two cannot be made to
//! agree: [`PublicBucket::get_from_origin`] reads the bucket and always sees
//! the app's own write, [`PublicBucket::get_from_cdn`] reads the edge and can
//! answer with a previous version until an invalidation drains.
//!
//! Browsers are never allowed to hold a copy: objects are stored with
//! `Cache-Control: public, max-age=0, s-maxage=<edge ttl>`, so the edge serves
//! the bytes and the browser revalidates on every request. A cache purge
//! reaches the edge but could never reach a browser, so this is what makes an
//! overwrite visible to returning visitors immediately.

use std::time::Duration;

use crate::body::Body;
use crate::http::{self, HttpBucket};
use crate::memory::MemoryBucket;
use crate::{Object, ObjectList, ObjectMetadata, Result};

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
///
/// There is no edge here, so [`PublicBucket::get_from_cdn`] reads the same
/// store as [`PublicBucket::get_from_origin`] and never answers with a previous
/// version. A test that needs to prove staleness handling needs a real edge.
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

    /// Fetches the object at `key` from the bucket, or `None` if it does not
    /// exist.
    ///
    /// Reads what is stored, not what the edge is serving: an object is
    /// readable here the moment [`Self::put`] returns, before the invalidation
    /// queued by that `put` has drained. Use this whenever the app must see its
    /// own write.
    ///
    /// The body is returned unread — see [`Object`] for what that allows.
    pub async fn get_from_origin(&self, key: &str) -> Result<Option<Object>> {
        match &self.inner {
            Backend::Http(b) => b.get(key).await,
            Backend::Memory(b) => b.get(key).await,
        }
    }

    /// Fetches the object at `key` from the CDN, or `None` if it does not
    /// exist.
    ///
    /// An edge hit is served without touching the bucket, which is what makes
    /// this the cheaper read for an object fetched far more often than it is
    /// written. The bytes travel compressed when the CDN can compress them; the
    /// body handed back is always the stored bytes.
    ///
    /// **This can serve a previous version.** The edge holds a copy for up to a
    /// year and only an invalidation replaces it, so a read that follows a
    /// [`Self::put`], a [`Self::delete`] or a presigned upload can answer with
    /// the bytes from before it. [`Self::get_from_origin`] is the one that
    /// cannot.
    ///
    /// [`Self::head`] and [`Self::list`] always read the bucket, so they can
    /// report an object this call is still serving the previous version of.
    pub async fn get_from_cdn(&self, key: &str) -> Result<Option<Object>> {
        match &self.inner {
            Backend::Http(b) => b.get_through_cdn(key).await,
            Backend::Memory(b) => b.get(key).await,
        }
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

    /// The exact `Cache-Control` an upload through [`Self::presigned_put_url`]
    /// must send. It is part of the signature, so R2 rejects anything else.
    pub const UPLOAD_CACHE_CONTROL: &'static str = "public, max-age=0, s-maxage=31536000";

    /// Returns a URL for uploading directly to `key`, without the bytes passing
    /// through the app. Valid for `expires`; fn0 Cloud clamps longer durations
    /// to 5 minutes. `content_length` binds the URL to an upload of exactly that
    /// many bytes; `None` accepts any size.
    ///
    /// `Cache-Control` and `Content-Type` are part of the signature, so the
    /// uploader cannot choose them — a browser-cacheable `max-age` would leave
    /// copies that no purge could ever reach.
    ///
    /// Unlike [`Self::put`], this does **not** invalidate the edge copy: the
    /// write never reaches the platform. Overwriting a key that is already
    /// published requires a matching [`Self::purge`], or the edge keeps serving
    /// the previous bytes for up to a year.
    pub async fn presigned_put_url(
        &self,
        key: &str,
        content_type: &str,
        content_length: Option<u64>,
        expires: Duration,
    ) -> Result<String> {
        match &self.inner {
            Backend::Http(b) => {
                b.presigned_public_put_url(key, content_type, expires, content_length)
                    .await
            }
            Backend::Memory(b) => b.presigned_url(key, "PUT", expires, content_length).await,
        }
    }

    /// Invalidates the edge copy of `key`. [`Self::put`] and [`Self::delete`] do
    /// this already; this exists for writes that bypassed the platform through
    /// [`Self::presigned_put_url`].
    ///
    /// Returns once the invalidation is queued, not once the edge is
    /// consistent.
    pub async fn purge(&self, key: &str) -> Result<()> {
        match &self.inner {
            Backend::Http(b) => b.purge(key).await,
            Backend::Memory(b) => b.purge(key).await,
        }
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
}
