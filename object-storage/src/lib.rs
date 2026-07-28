//! User-facing object storage for fn0/Forte apps.
//!
//! Storage is split by access model, and the two are separate types rather than
//! two configurations of one:
//!
//! - [`private`] — the project's own bucket. Reachable only through the app or
//!   a presigned URL.
//! - [`public`] — served from the CDN at a stable URL that can be embedded in
//!   HTML. Everything written there is world-readable.
//!
//! Application code never sees the storage endpoint or credentials: each
//! namespace reads only its injected placeholder URL, and the worker's
//! object-storage hijack rewrites and signs every request out of band.
//!
//! Use each namespace's `memory()` in tests for an in-process backend with the
//! same API.

mod body;
mod http;
mod memory;
pub mod private;
pub mod public;
mod runtime;

pub use body::Body;

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
    /// `list` call. `None` means the listing is complete.
    pub next_cursor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[forte_sdk::test]
    async fn put_get_head_delete() {
        let bucket = private::memory();
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
    }

    #[forte_sdk::test]
    async fn public_put_returns_url() {
        let bucket = public::memory();
        let url = bucket
            .put("clips/intro.mp4", "video/mp4", &b"bytes"[..])
            .await
            .unwrap();

        assert_eq!(url, "memory://public/clips/intro.mp4");
        assert_eq!(url, bucket.url("clips/intro.mp4"));
        assert_eq!(
            bucket.head("clips/intro.mp4").await.unwrap(),
            Some(ObjectMetadata {
                size: 5,
                content_type: Some("video/mp4".to_string()),
                etag: None,
            })
        );
    }

    #[forte_sdk::test]
    async fn public_url_needs_no_stored_object() {
        let bucket = public::memory();
        assert_eq!(bucket.url("never/written"), "memory://public/never/written");
        assert_eq!(bucket.head("never/written").await.unwrap(), None);
    }
}
