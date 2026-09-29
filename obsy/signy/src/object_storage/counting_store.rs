//! Counts object-store operations by kind.
//!
//! This design has been shaped end to end by one number: Cloudflare R2 charges
//! per operation, and per-tenant objects were rejected on that basis alone. The
//! rejection was arithmetic on an estimate, and nothing since has checked the
//! estimate against what the engine actually issues. Amounts cannot be measured
//! locally — a local backend has no price — but *counts* are a property of this
//! code rather than of the backend, so they measure the same on `file://` as on
//! R2.
//!
//! Operation kinds are reported raw rather than pre-multiplied into a bill.
//! Which kinds are billed how is the backend's policy and it changes; what this
//! engine owns is how many of each it issues per flush, merge and retention
//! cycle.
//!
//! **Bytes are counted for the same reason and they are the same kind of
//! number.** A shared multi-tenant part holds every tenant's rows, and a cold
//! read restores the *whole object* — so one tenant's query pays for the other
//! tenants' bytes. That is the premise behind "add Parquet range reads", and
//! until now it was a premise: nothing counted the bytes, so the size of the
//! waste was read off the code rather than measured. Which byte ranges this
//! engine asks for is a property of this engine, so it measures the same on
//! `file://` as on R2 — the same argument that makes the counts portable.
//! `ranged_gets` is zero today by construction, and it stays that way:
//! measured, the whole-object restore is read by 5.66 distinct tenants before
//! eviction, so fetching each tenant's range instead is 5.66 requests where
//! this is one — the work it was the before-number for is struck in `todo.md`
//! for that reason. What the counter is now is a tripwire. A non-zero value
//! means something started asking for ranges, and the decision above says
//! nothing should have.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use futures_util::StreamExt;
use futures_util::stream::BoxStream;
use object_store::path::Path;
use object_store::{
    GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta, ObjectStore,
    PutMultipartOptions, PutOptions, PutPayload, PutResult,
};

#[derive(Debug, Default)]
pub struct ObjectStoreOps {
    pub puts: AtomicU64,
    pub multipart_puts: AtomicU64,
    pub gets: AtomicU64,
    /// GETs that asked for a byte range rather than a whole object. **Zero,
    /// and expected to stay zero** — it was the before-number for Parquet
    /// range reads, and that work was measured and decided against
    /// (`todo.md`), so a non-zero reading now means the engine started
    /// spending requests to save bytes the backend does not bill.
    pub ranged_gets: AtomicU64,
    pub deletes: AtomicU64,
    pub lists: AtomicU64,
    /// Objects the list streams yielded. A backend pages a listing, so the
    /// request count is a function of this and the page size, not of `lists`.
    pub listed_objects: AtomicU64,
    pub copies: AtomicU64,
    /// Bytes the backend agreed to return, taken from the response's own range
    /// rather than by consuming the body — a caller that drops a `GetResult`
    /// unread still cost the transfer on a real backend, and a caller that
    /// streams it must not be charged twice.
    pub get_bytes: AtomicU64,
    /// Bytes handed to the backend to store, counted before the call so a
    /// failed upload still shows what it tried to move.
    pub put_bytes: AtomicU64,
    /// The same two totals, split by what was being read or written.
    ///
    /// Aggregates cannot answer the question this meter was added for. A part
    /// restore and a manifest rewrite are both bytes, and only one of them is
    /// what range reads would shrink — reading a single total and attributing
    /// it to restores is the arithmetic-on-an-estimate this file's own header
    /// warns about.
    pub get_bytes_by_kind: PathBytes,
    pub put_bytes_by_kind: PathBytes,
}

/// Bytes per object class. The classes are the storage layout's own top-level
/// split, so a path that matches none of them is a path this accounting has
/// not been taught about and is counted as `other` rather than dropped.
#[derive(Debug, Default)]
pub struct PathBytes {
    pub manifest: AtomicU64,
    pub part: AtomicU64,
    pub trace_part: AtomicU64,
    pub other: AtomicU64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PathByteCounts {
    pub manifest: u64,
    pub part: u64,
    pub trace_part: u64,
    pub other: u64,
}

/// Which class a key belongs to, decided by the prefixes `catalog.rs` builds.
///
/// Both manifests are one class: they are the same mechanism — one whole
/// object rewritten per commit — and separating them would suggest the cost
/// differs by domain when what it varies with is the commit rate.
fn classify(location: &Path) -> ObjectClass {
    let key = location.as_ref();
    if key.ends_with("manifest.json") || key.contains("/catalog/") || key.starts_with("catalog/") {
        ObjectClass::Manifest
    } else if key.contains("/parts/") || key.starts_with("parts/") {
        ObjectClass::Part
    } else if key.contains("/trace_parts/") || key.starts_with("trace_parts/") {
        ObjectClass::TracePart
    } else {
        ObjectClass::Other
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ObjectClass {
    Manifest,
    Part,
    TracePart,
    Other,
}

impl PathBytes {
    fn add(&self, class: ObjectClass, bytes: u64) {
        let counter = match class {
            ObjectClass::Manifest => &self.manifest,
            ObjectClass::Part => &self.part,
            ObjectClass::TracePart => &self.trace_part,
            ObjectClass::Other => &self.other,
        };
        counter.fetch_add(bytes, Ordering::Relaxed);
    }

    fn snapshot(&self) -> PathByteCounts {
        PathByteCounts {
            manifest: self.manifest.load(Ordering::Relaxed),
            part: self.part.load(Ordering::Relaxed),
            trace_part: self.trace_part.load(Ordering::Relaxed),
            other: self.other.load(Ordering::Relaxed),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjectStoreOpCounts {
    pub puts: u64,
    pub multipart_puts: u64,
    pub gets: u64,
    pub ranged_gets: u64,
    pub deletes: u64,
    pub lists: u64,
    pub listed_objects: u64,
    pub copies: u64,
    pub get_bytes: u64,
    pub put_bytes: u64,
    pub get_bytes_by_kind: PathByteCounts,
    pub put_bytes_by_kind: PathByteCounts,
}

impl ObjectStoreOps {
    pub fn snapshot(&self) -> ObjectStoreOpCounts {
        ObjectStoreOpCounts {
            puts: self.puts.load(Ordering::Relaxed),
            multipart_puts: self.multipart_puts.load(Ordering::Relaxed),
            gets: self.gets.load(Ordering::Relaxed),
            ranged_gets: self.ranged_gets.load(Ordering::Relaxed),
            deletes: self.deletes.load(Ordering::Relaxed),
            lists: self.lists.load(Ordering::Relaxed),
            listed_objects: self.listed_objects.load(Ordering::Relaxed),
            copies: self.copies.load(Ordering::Relaxed),
            get_bytes: self.get_bytes.load(Ordering::Relaxed),
            put_bytes: self.put_bytes.load(Ordering::Relaxed),
            get_bytes_by_kind: self.get_bytes_by_kind.snapshot(),
            put_bytes_by_kind: self.put_bytes_by_kind.snapshot(),
        }
    }
}

/// Counts every operation reaching `inner`, including the ones issued by
/// `ObjectStore`'s provided methods: `head`, `get_range` and `get` all route
/// through `get_opts`, and `put` through `put_opts`.
#[derive(Debug)]
pub struct CountingStore {
    inner: Arc<dyn ObjectStore>,
    ops: Arc<ObjectStoreOps>,
}

impl CountingStore {
    pub fn new(inner: Arc<dyn ObjectStore>, ops: Arc<ObjectStoreOps>) -> Self {
        Self { inner, ops }
    }
}

impl std::fmt::Display for CountingStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "CountingStore({})", self.inner)
    }
}

#[async_trait]
impl ObjectStore for CountingStore {
    async fn put_opts(
        &self,
        location: &Path,
        payload: PutPayload,
        opts: PutOptions,
    ) -> object_store::Result<PutResult> {
        // Counted before the call, and regardless of the outcome: a rejected
        // conditional put is a request the backend served and bills for.
        self.ops.puts.fetch_add(1, Ordering::Relaxed);
        let bytes = payload.content_length() as u64;
        self.ops.put_bytes.fetch_add(bytes, Ordering::Relaxed);
        self.ops.put_bytes_by_kind.add(classify(location), bytes);
        self.inner.put_opts(location, payload, opts).await
    }

    async fn put_multipart_opts(
        &self,
        location: &Path,
        opts: PutMultipartOptions,
    ) -> object_store::Result<Box<dyn MultipartUpload>> {
        self.ops.multipart_puts.fetch_add(1, Ordering::Relaxed);
        self.inner.put_multipart_opts(location, opts).await
    }

    async fn get_opts(
        &self,
        location: &Path,
        options: GetOptions,
    ) -> object_store::Result<GetResult> {
        self.ops.gets.fetch_add(1, Ordering::Relaxed);
        if options.range.is_some() {
            self.ops.ranged_gets.fetch_add(1, Ordering::Relaxed);
        }
        let result = self.inner.get_opts(location, options).await;
        if let Ok(got) = &result {
            // The response's range, not the object's size: for a whole-object
            // GET the two are equal, and for a ranged one only this is the
            // transfer. A failed GET moved nothing and is counted as a
            // request but not as bytes.
            let bytes = got.range.end - got.range.start;
            self.ops.get_bytes.fetch_add(bytes, Ordering::Relaxed);
            self.ops.get_bytes_by_kind.add(classify(location), bytes);
        }
        result
    }

    async fn delete(&self, location: &Path) -> object_store::Result<()> {
        self.ops.deletes.fetch_add(1, Ordering::Relaxed);
        self.inner.delete(location).await
    }

    fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, object_store::Result<ObjectMeta>> {
        self.ops.lists.fetch_add(1, Ordering::Relaxed);
        let ops = self.ops.clone();
        self.inner
            .list(prefix)
            .inspect(move |item| {
                if item.is_ok() {
                    ops.listed_objects.fetch_add(1, Ordering::Relaxed);
                }
            })
            .boxed()
    }

    fn list_with_offset(
        &self,
        prefix: Option<&Path>,
        offset: &Path,
    ) -> BoxStream<'static, object_store::Result<ObjectMeta>> {
        self.ops.lists.fetch_add(1, Ordering::Relaxed);
        let ops = self.ops.clone();
        self.inner
            .list_with_offset(prefix, offset)
            .inspect(move |item| {
                if item.is_ok() {
                    ops.listed_objects.fetch_add(1, Ordering::Relaxed);
                }
            })
            .boxed()
    }

    async fn list_with_delimiter(&self, prefix: Option<&Path>) -> object_store::Result<ListResult> {
        self.ops.lists.fetch_add(1, Ordering::Relaxed);
        let result = self.inner.list_with_delimiter(prefix).await;
        if let Ok(listed) = &result {
            self.ops
                .listed_objects
                .fetch_add(listed.objects.len() as u64, Ordering::Relaxed);
        }
        result
    }

    async fn copy(&self, from: &Path, to: &Path) -> object_store::Result<()> {
        self.ops.copies.fetch_add(1, Ordering::Relaxed);
        self.inner.copy(from, to).await
    }

    async fn copy_if_not_exists(&self, from: &Path, to: &Path) -> object_store::Result<()> {
        self.ops.copies.fetch_add(1, Ordering::Relaxed);
        self.inner.copy_if_not_exists(from, to).await
    }
}
