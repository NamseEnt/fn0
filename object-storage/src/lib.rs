//! User-facing object storage for fn0/Forte apps.
//!
//! A [`Bucket`] exposes a small S3-style API (`put`/`get`/`head`/`delete`/`list`)
//! over a per-project object store. Application code never sees the storage
//! endpoint or credentials: [`bucket`] reads only the injected
//! `FN0_OBJECT_STORAGE_URL` placeholder, and the worker's object-storage hijack
//! rewrites and signs every request out of band.
//!
//! Use [`memory`] in tests for an in-process backend with the same API.

mod http;
mod memory;
mod runtime;

use bytes::Bytes;
use http::HttpBucket;
use memory::MemoryBucket;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("object storage transport error: {0}")]
    Transport(String),
    #[error("object storage returned status {status}: {message}")]
    UnexpectedStatus { status: u16, message: String },
    #[error("failed to parse object storage response: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Connects to the project's object storage.
///
/// Panics if `FN0_OBJECT_STORAGE_URL` is not set — it is always injected by the
/// fn0 runtime, so its absence is a deployment fault, not a recoverable error.
pub fn bucket() -> Bucket {
    let endpoint =
        std::env::var("FN0_OBJECT_STORAGE_URL").expect("FN0_OBJECT_STORAGE_URL must be set");
    Bucket {
        inner: BucketInner::Http(HttpBucket::new(endpoint)),
    }
}

/// In-process object storage for tests. Holds objects in a `BTreeMap`.
pub fn memory() -> Bucket {
    Bucket {
        inner: BucketInner::Memory(MemoryBucket::new()),
    }
}

#[derive(Clone)]
pub struct Bucket {
    inner: BucketInner,
}

#[derive(Clone)]
enum BucketInner {
    Http(HttpBucket),
    Memory(MemoryBucket),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectMetadata {
    pub size: u64,
    pub content_type: Option<String>,
    pub etag: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListEntry {
    pub key: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectList {
    pub entries: Vec<ListEntry>,
    /// When the result was truncated, the `after` value to pass to the next
    /// [`Bucket::list`] call. `None` means the listing is complete.
    pub next_cursor: Option<String>,
}

impl Bucket {
    /// Stores `data` at `key`, overwriting any existing object.
    pub async fn put(&self, key: &str, data: impl Into<Bytes>) -> Result<()> {
        self.put_with_content_type(key, data, None).await
    }

    /// Stores `data` at `key` with an explicit `Content-Type`.
    pub async fn put_with_content_type(
        &self,
        key: &str,
        data: impl Into<Bytes>,
        content_type: Option<&str>,
    ) -> Result<()> {
        let data = data.into();
        match &self.inner {
            BucketInner::Http(b) => b.put(key, data, content_type).await,
            BucketInner::Memory(b) => b.put(key, data, content_type).await,
        }
    }

    /// Fetches the object at `key`, or `None` if it does not exist.
    pub async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        match &self.inner {
            BucketInner::Http(b) => b.get(key).await,
            BucketInner::Memory(b) => b.get(key).await,
        }
    }

    /// Fetches object metadata without downloading the body.
    pub async fn head(&self, key: &str) -> Result<Option<ObjectMetadata>> {
        match &self.inner {
            BucketInner::Http(b) => b.head(key).await,
            BucketInner::Memory(b) => b.head(key).await,
        }
    }

    /// Deletes `key`. Succeeds whether or not the object existed.
    pub async fn delete(&self, key: &str) -> Result<()> {
        match &self.inner {
            BucketInner::Http(b) => b.delete(key).await,
            BucketInner::Memory(b) => b.delete(key).await,
        }
    }

    /// Lists objects whose key starts with `prefix`, in ascending key order.
    ///
    /// `after` resumes the listing after a given key (pass [`ObjectList::next_cursor`]
    /// from the previous page). At most `limit` entries are returned.
    pub async fn list(
        &self,
        prefix: &str,
        after: Option<&str>,
        limit: usize,
    ) -> Result<ObjectList> {
        match &self.inner {
            BucketInner::Http(b) => b.list(prefix, after, limit).await,
            BucketInner::Memory(b) => b.list(prefix, after, limit).await,
        }
    }

    /// Returns a URL for downloading `key` directly, without routing through
    /// the app — e.g. handed to a browser. Valid for `expires`; fn0 Cloud
    /// clamps longer durations to 5 minutes.
    pub async fn presigned_get_url(&self, key: &str, expires: Duration) -> Result<String> {
        match &self.inner {
            BucketInner::Http(b) => b.presigned_url(key, "GET", expires, None).await,
            BucketInner::Memory(b) => b.presigned_url(key, "GET", expires, None).await,
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
            BucketInner::Http(b) => b.presigned_url(key, "PUT", expires, content_length).await,
            BucketInner::Memory(b) => b.presigned_url(key, "PUT", expires, content_length).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[forte_sdk::test]
    async fn put_get_head_delete() {
        let bucket = memory();
        assert_eq!(bucket.get("a").await.unwrap(), None);
        assert_eq!(bucket.head("a").await.unwrap(), None);

        bucket
            .put_with_content_type("a", &b"hello"[..], Some("text/plain"))
            .await
            .unwrap();

        assert_eq!(
            bucket.get("a").await.unwrap().as_deref(),
            Some(&b"hello"[..])
        );
        assert_eq!(
            bucket.head("a").await.unwrap(),
            Some(ObjectMetadata {
                size: 5,
                content_type: Some("text/plain".to_string()),
                etag: None,
            })
        );

        bucket.delete("a").await.unwrap();
        assert_eq!(bucket.get("a").await.unwrap(), None);
        bucket.delete("a").await.unwrap();
    }

    #[forte_sdk::test]
    async fn list_prefix_after_limit() {
        let bucket = memory();
        for key in ["img/1", "img/2", "img/3", "doc/1"] {
            bucket.put(key, &b"x"[..]).await.unwrap();
        }

        let page = bucket.list("img/", None, 2).await.unwrap();
        assert_eq!(
            page.entries
                .iter()
                .map(|e| e.key.as_str())
                .collect::<Vec<_>>(),
            ["img/1", "img/2"]
        );
        assert_eq!(page.next_cursor.as_deref(), Some("img/2"));

        let page = bucket.list("img/", Some("img/2"), 2).await.unwrap();
        assert_eq!(
            page.entries
                .iter()
                .map(|e| e.key.as_str())
                .collect::<Vec<_>>(),
            ["img/3"]
        );
        assert_eq!(page.next_cursor, None);
    }

    #[forte_sdk::test]
    async fn memory_presigned_urls() {
        let bucket = memory();
        assert_eq!(
            bucket
                .presigned_get_url("a/b.png", Duration::from_secs(60))
                .await
                .unwrap(),
            "memory://get/a/b.png"
        );
        assert_eq!(
            bucket
                .presigned_put_url("a/b.png", None, Duration::from_secs(60))
                .await
                .unwrap(),
            "memory://put/a/b.png"
        );
        assert_eq!(
            bucket
                .presigned_put_url("a/b.png", Some(1024), Duration::from_secs(60))
                .await
                .unwrap(),
            "memory://put/a/b.png"
        );
    }
}
