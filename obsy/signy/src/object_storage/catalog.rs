/// Immutable part objects plus an append-only, replicated catalog journal.
///
/// `prefix` is the path component of SIGNY_OBJECT_STORE_URL. Credentials
/// and endpoint settings are consumed by object_store from the process
/// environment (AWS_*, OBJECT_STORE_* and backend-specific keys).
pub struct ObjectStorage {
    store: Arc<dyn ObjectStore>,
    /// Requests issued through `store`, counted by kind. See
    /// [`counting_store`] for why the count and not the amount.
    ops: Arc<ObjectStoreOps>,
    prefix: ObjectPath,
    manifest_update: tokio::sync::Mutex<()>,
    catalog_state: tokio::sync::Mutex<Option<CatalogState>>,
    /// The local filesystem backend does not implement conditional updates.
    /// It is only exposed as a single-process development backend, where the
    /// process-local mutex plus LocalFileSystem's staged rename gives us an
    /// atomic replacement.
    local_manifest_overwrite: bool,
    /// Told once, when the epoch check first fires.
    ///
    /// Held here rather than returned to each caller so that every writer —
    /// flush, merge, retention, the final force-flush — reacts identically
    /// without any of them having to know what fencing is. They all see an
    /// ordinary catalog error; the drain has already begun by then.
    fence_sink: std::sync::OnceLock<Arc<crate::shutdown::ShutdownState>>,
    /// This instance's claim on the prefix, or 0 while unclaimed.
    ///
    /// The architecture assumes one writer; nothing used to enforce it, so two
    /// processes on the same prefix each believed they owned it. The catalog
    /// generation create stops a lost update but not two divergent local WALs, and not one
    /// instance's retention expiring a part the other has just registered.
    writer_epoch: AtomicU64,
    /// Catalog files checksummed while restoring. The expensive part of a
    /// startup at scale is this, not the store round trips: every part's bloom,
    /// stream index and metadata are read and verified from local disk. Counted
    /// so a test can catch a redundant pass without running at the scale where
    /// the seconds show.
    #[cfg(test)]
    catalog_validations: AtomicU64,
}

/// Coordinates all mutations of the local object-store cache. Readers hold a
/// shared guard while their Parquet files are open; restore and eviction hold
/// an exclusive guard.
pub struct RemoteCache {
    pub storage: Arc<ObjectStorage>,
    pub parts_root: PathBuf,
    /// Object-store failures since the last success.
    ///
    /// This was a health bit plus a failure generation, where a success cleared
    /// the bit only if its operation had started in the current generation. The
    /// guard was aimed at a real hazard — a slow success must not clear a newer
    /// failure — but it made a *single* failed request mean "the store is
    /// down", and that is what `/ready` reads.
    ///
    /// Measured at a 3% injected write-error rate, which the engine survives
    /// with no ingest errors and no lost data: `remote_healthy` flipped 14-17
    /// times a minute and read false 34-59% of the time. An orchestrator
    /// watching that pulls the instance in and out of service over an error
    /// rate that cost nothing.
    remote_failures: Arc<AtomicU32>,
    cache_healthy: Arc<AtomicBool>,
}

/// Consecutive failures that constitute an outage rather than a bad request.
///
/// A store that is genuinely unreachable fails everything, so it crosses this
/// in the time of a few operations. An isolated failure between successes never
/// does.
///
/// What is given up is the old generation guard: a slow success that started
/// before an outage now resets the count, delaying detection by one more round
/// of failures. That is a bounded delay rather than the indefinite masking the
/// guard was written to prevent, because during a real outage failures vastly
/// outnumber stale successes.
const REMOTE_FAILURE_THRESHOLD: u32 = 3;

impl RemoteCache {
    pub fn new(storage: Arc<ObjectStorage>, parts_root: PathBuf) -> Self {
        Self {
            storage,
            parts_root,
            remote_failures: Arc::new(AtomicU32::new(0)),
            cache_healthy: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.is_remote_healthy() && self.is_cache_healthy()
    }

    pub fn is_remote_healthy(&self) -> bool {
        self.consecutive_remote_failures() < REMOTE_FAILURE_THRESHOLD
    }

    /// Failure pressure below the threshold, which the health flag hides by
    /// design. An operator watching this sees a store degrading before it is
    /// declared down.
    pub fn consecutive_remote_failures(&self) -> u32 {
        self.remote_failures.load(Ordering::Acquire)
    }

    pub fn is_cache_healthy(&self) -> bool {
        self.cache_healthy.load(Ordering::Acquire)
    }

    /// The store answered. Callers report the outcome of the operation they
    /// just finished; nothing has to be captured beforehand.
    pub fn record_remote_success(&self) {
        self.remote_failures.store(0, Ordering::Release);
    }

    pub fn record_remote_failure(&self) {
        let _ =
            self.remote_failures
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |failures| {
                    Some(failures.saturating_add(1))
                });
    }

    pub fn mark_cache_healthy(&self) {
        self.cache_healthy.store(true, Ordering::Release);
    }

    pub fn mark_cache_unhealthy(&self) {
        self.cache_healthy.store(false, Ordering::Release);
    }

    pub fn trace_parts_root(&self) -> PathBuf {
        self.parts_root
            .parent()
            .map(|parent| parent.join("traces"))
            .unwrap_or_else(|| PathBuf::from("traces"))
    }

    pub fn metric_parts_root(&self) -> PathBuf {
        self.parts_root
            .parent()
            .map(|parent| parent.join("metrics"))
            .unwrap_or_else(|| PathBuf::from("metrics"))
    }

    /// Drive the remote to unhealthy in one call, for tests that need the
    /// state rather than the path into it.
    #[cfg(test)]
    pub fn mark_unhealthy(&self) {
        for _ in 0..REMOTE_FAILURE_THRESHOLD {
            self.record_remote_failure();
        }
    }
}

impl ObjectStorage {
    /// See `catalog_validations`.
    #[cfg(test)]
    pub fn catalog_validations(&self) -> u64 {
        self.catalog_validations.load(Ordering::Acquire)
    }

    #[cfg(test)]
    fn record_catalog_validation(&self) {
        self.catalog_validations.fetch_add(1, Ordering::AcqRel);
    }

    #[cfg(not(test))]
    fn record_catalog_validation(&self) {}

    pub fn from_url(url: &str) -> Result<Self, String> {
        let url =
            url::Url::parse(url).map_err(|error| format!("invalid object-store URL: {error}"))?;
        let local_manifest_overwrite = url.scheme() == "file";
        if local_manifest_overwrite {
            tracing::warn!(
                %url,
                "object store is a local filesystem path: catalog updates use overwrite instead \
            of compare-and-swap and are only safe for a single process on a local disk. Do not use this for \
            production or on shared/network storage."
            );
        }
        let options = normalized_object_store_options(std::env::vars());
        let (store, prefix) = object_store::parse_url_opts(&url, options)
            .map_err(|error| format!("failed to configure object store: {error}"))?;
        // Tier B load gate: wrap the real store with seeded latency/fault
        // injection only when the load knobs are present. Absent the knobs this
        // is a no-op and the object-store construction path is unchanged.
        let store: Arc<dyn ObjectStore> = match fault_store::FaultConfig::from_env()? {
            Some(config) => Arc::new(fault_store::LatencyFaultStore::new(
                Arc::from(store),
                config,
            )),
            None => Arc::from(store),
        };
        Ok(Self::wrapping(store, prefix, local_manifest_overwrite))
    }

    /// The one place the store is installed, so the operation counter cannot be
    /// left off a backend by adding a constructor that forgets it.
    fn wrapping(
        store: Arc<dyn ObjectStore>,
        prefix: ObjectPath,
        local_manifest_overwrite: bool,
    ) -> Self {
        let ops = Arc::new(ObjectStoreOps::default());
        Self {
            store: Arc::new(CountingStore::new(store, ops.clone())),
            ops,
            prefix,
            manifest_update: tokio::sync::Mutex::new(()),
            catalog_state: tokio::sync::Mutex::new(None),
            local_manifest_overwrite,
            fence_sink: std::sync::OnceLock::new(),
            writer_epoch: AtomicU64::new(0),
            #[cfg(test)]
            catalog_validations: AtomicU64::new(0),
        }
    }

    /// Object-store requests this process has issued, by kind.
    pub fn operation_counts(&self) -> ObjectStoreOpCounts {
        self.ops.snapshot()
    }

    #[cfg(test)]
    pub fn in_memory() -> Self {
        Self::wrapping(
            Arc::new(object_store::memory::InMemory::new()),
            ObjectPath::from("signy-test"),
            false,
        )
    }

    /// Two handles over one backing store: what two processes pointed at the
    /// same prefix actually look like.
    #[cfg(test)]
    pub fn sharing_store_for_test(store: Arc<dyn ObjectStore>) -> Arc<Self> {
        Arc::new(Self::wrapping(store, ObjectPath::from("signy-test"), false))
    }

    /// An in-memory store whose every write fails, for the paths that must
    /// report a failure rather than apply a change that is not durable.
    #[cfg(test)]
    pub fn in_memory_with_failing_writes() -> Self {
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        Self::wrapping(
            Arc::new(fault_store::LatencyFaultStore::new(
                inner,
                fault_store::FaultConfig::for_test(0, 0, 0, 1.0, 1),
            )),
            ObjectPath::from("signy-test"),
            false,
        )
    }

    /// Wrap an arbitrary object store, used by tests to inject fault-injecting
    /// backends. The store is treated as a full conditional-put backend (no
    /// local overwrite shortcut), exercising the real CAS manifest path.
    #[cfg(test)]
    pub fn from_store(store: Arc<dyn ObjectStore>, prefix: &str) -> Self {
        Self::wrapping(store, ObjectPath::from(prefix), false)
    }

    fn path(&self, relative: &str) -> ObjectPath {
        if self.prefix.as_ref().is_empty() {
            ObjectPath::from(relative)
        } else {
            ObjectPath::from(format!("{}/{relative}", self.prefix.as_ref()))
        }
    }

    fn manifest_path(&self) -> ObjectPath {
        self.path(MANIFEST_FILE)
    }

    fn trace_manifest_path(&self) -> ObjectPath {
        self.path(TRACE_MANIFEST_FILE)
    }

    fn part_path(&self, part: &ManifestPart, file: &str) -> ObjectPath {
        self.path(&format!("parts/{}/{}/{}", part.partition, part.id, file))
    }

    fn tenant_policy_path(&self, tenant: &str) -> ObjectPath {
        self.path(&format!("{TENANT_POLICY_PREFIX}/{tenant}.json"))
    }

    fn trace_part_path(&self, part: &TraceManifestPart, file: &str) -> ObjectPath {
        self.path(&format!(
            "trace_parts/{}/{}/{}",
            part.partition, part.id, file
        ))
    }

    fn metric_manifest_path(&self) -> ObjectPath {
        self.path(METRIC_MANIFEST_FILE)
    }

    fn metric_part_path(&self, part: &MetricManifestPart, file: &str) -> ObjectPath {
        self.path(&format!(
            "metric_parts/{}/{}/{}",
            part.partition, part.id, file
        ))
    }

    pub async fn load_manifest(&self) -> Result<Manifest, String> {
        Ok(self.load_catalog_state().await?.manifest)
    }

    /// Refuse to start when the configured store does not actually enforce
    /// conditional writes.
    ///
    /// This does not test `object_store` — whether `AmazonS3` implements
    /// `If-Match` correctly is that crate's problem and its test suite's. What
    /// it tests is **our configuration**: `from_url` hands the environment
    /// straight to `object_store`, and nothing else checks that the store it
    /// built back actually does compare-and-swap. Get
    /// `OBJECT_STORE_CONDITIONAL_PUT` wrong and every manifest guarantee in
    /// this engine — lost-update protection, merge input revalidation, writer
    /// fencing — silently rests on nothing.
    ///
    /// The check is the *negative* path. A positive one proves nothing: the
    /// first catalog write of a fresh prefix succeeds whether or not the
    /// condition was honoured. What must hold is that a write which should be
    /// rejected **is** rejected.
    pub async fn verify_conditional_put(&self) -> Result<(), String> {
        if self.local_manifest_overwrite {
            // `file://` is a declared single-process development backend that
            // deliberately opts out of CAS, and `from_url` already warns.
            return Ok(());
        }
        let probe = self.path("_preflight/conditional-put-probe");
        // Any leftover from an aborted earlier boot would make the first
        // create fail for the wrong reason.
        let _ = self.store.delete(&probe).await;

        let created = self
            .store
            .put_opts(
                &probe,
                Bytes::from_static(b"signy conditional-put probe").into(),
                PutOptions {
                    mode: PutMode::Create,
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| {
                format!("conditional-put preflight could not write its probe object: {error}")
            })?;

        let outcome = self.probe_rejections(&probe, created).await;
        // Best effort: a leftover probe only costs the next boot one delete.
        let _ = self.store.delete(&probe).await;
        outcome
    }

    pub fn verify_catalog_protection(&self) -> Result<(), String> {
        if self.local_manifest_overwrite {
            return Ok(());
        }
        let configured = std::env::var("SIGNY_OBJECT_STORE_CATALOG_LOCKED")
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes"
                )
            })
            .unwrap_or(false);
        if configured {
            Ok(())
        } else {
            Err("catalog protection is not asserted; configure an R2 Bucket Lock rule for the catalog/ prefix and set SIGNY_OBJECT_STORE_CATALOG_LOCKED=true".to_string())
        }
    }

    async fn probe_rejections(&self, probe: &ObjectPath, created: PutResult) -> Result<(), String> {
        let recreated = self
            .store
            .put_opts(
                probe,
                Bytes::from_static(b"second create").into(),
                PutOptions {
                    mode: PutMode::Create,
                    ..Default::default()
                },
            )
            .await;
        if recreated.is_ok() {
            return Err(Self::preflight_failure(
                "a second PutMode::Create over an existing object succeeded",
            ));
        }

        // A version that was valid and no longer is: exactly the shape of the
        // lost update the catalog generation create exists to prevent.
        let stale = UpdateVersion {
            e_tag: created.e_tag.clone(),
            version: created.version.clone(),
        };
        if stale.e_tag.is_none() && stale.version.is_none() {
            return Err(Self::preflight_failure(
                "the store returned neither an ETag nor a version, so no write can ever be conditioned on one",
            ));
        }
        self.store
            .put_opts(
                probe,
                Bytes::from_static(b"overwrite").into(),
                PutOptions {
                    mode: PutMode::Overwrite,
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| format!("conditional-put preflight could not overwrite: {error}"))?;
        let updated = self
            .store
            .put_opts(
                probe,
                Bytes::from_static(b"stale update").into(),
                PutOptions {
                    mode: PutMode::Update(stale),
                    ..Default::default()
                },
            )
            .await;
        if updated.is_ok() {
            return Err(Self::preflight_failure(
                "a PutMode::Update carrying a superseded version succeeded",
            ));
        }
        Ok(())
    }

    fn preflight_failure(what_happened: &str) -> String {
        format!(
            "the configured object store does not enforce conditional writes: {what_happened}. \
Every catalog guarantee in this engine depends on conditional object creation, so refusing to start is the \
only safe response. For S3-compatible stores set OBJECT_STORE_CONDITIONAL_PUT=etag; for a local \
single-process development store use a file:// URL, which opts out of CAS deliberately."
        )
    }

    /// Where a detected fence is reported. Set once, during startup.
    pub fn set_fence_sink(&self, shutdown: Arc<crate::shutdown::ShutdownState>) {
        let _ = self.fence_sink.set(shutdown);
    }

    pub fn writer_epoch(&self) -> u64 {
        self.writer_epoch.load(Ordering::Acquire)
    }

    /// Take ownership of the prefix, and report the epoch taken.
    ///
    /// Called once at startup, before any worker runs. The catalog commit
    /// carries the writer epoch for every signal in one logical generation.
    pub async fn claim_writer_epoch(&self) -> Result<u64, String> {
        let state = self.load_catalog_state().await?;
        let epoch = state
            .writer_epoch
            .checked_add(1)
            .ok_or_else(|| "writer epoch overflow".to_string())?;
        let next = self
            .commit_catalog_mutation(CatalogMutation {
                transaction_id: uuid::Uuid::new_v4().to_string(),
                writer_epoch: epoch,
                claim_writer: true,
                ..Default::default()
            })
            .await?;
        self.writer_epoch
            .store(next.writer_epoch, Ordering::Release);
        tracing::info!(
            epoch = next.writer_epoch,
            "claimed the object-store writer epoch"
        );
        Ok(next.writer_epoch)
    }

    #[cfg(test)]
    fn put_mode(&self, version: Option<UpdateVersion>) -> PutMode {
        match version {
            Some(_) if self.local_manifest_overwrite => PutMode::Overwrite,
            Some(version) => PutMode::Update(version),
            None => PutMode::Create,
        }
    }

    /// Refuse to write when the loaded manifest is owned by someone else.
    ///
    /// Checked on every CAS rather than periodically, so the fence lands on
    /// the first write after a takeover instead of at the next poll.
    fn check_epoch(&self, observed: u64) -> Result<(), String> {
        let held = self.writer_epoch();
        if held == 0 || observed == held {
            return Ok(());
        }
        if let Some(shutdown) = self.fence_sink.get() {
            shutdown.mark_fenced();
        }
        Err(format!(
            "{FENCED_ERROR}: this instance holds writer epoch {held} but the manifest now carries \
{observed}"
        ))
    }

    pub async fn load_trace_manifest(&self) -> Result<TraceManifest, String> {
        Ok(self.load_catalog_state().await?.trace_manifest)
    }

    pub async fn publish_trace_parts(&self, added: &[TracePart]) -> Result<TraceManifest, String> {
        for part in added {
            let id = part.meta.id.clone();
            TracePartReader::open(part.clone())
                .map_err(|error| format!("refusing to publish invalid trace part {id}: {error}"))?;
        }
        for part in added {
            self.upload_trace_part(part).await?;
        }

        let state = self.load_catalog_state().await?;
        self.check_epoch(state.writer_epoch)?;
        let descriptors: Vec<TraceManifestPart> = added
            .iter()
            .map(|part| TraceManifestPart {
                id: part.meta.id.clone(),
                partition: part.meta.partition.clone(),
            })
            .collect();
        for descriptor in &descriptors {
            if let Some(existing) = state
                .trace_manifest
                .parts
                .iter()
                .find(|item| item.id == descriptor.id)
                && existing != descriptor
            {
                return Err(format!(
                    "trace manifest part ID collision: {}",
                    descriptor.id
                ));
            }
        }
        if descriptors
            .iter()
            .all(|descriptor| state.trace_manifest.parts.contains(descriptor))
        {
            return Ok(state.trace_manifest);
        }
        let next = self
            .commit_catalog_mutation(CatalogMutation {
                transaction_id: uuid::Uuid::new_v4().to_string(),
                writer_epoch: state.writer_epoch,
                trace_added: descriptors,
                ..Default::default()
            })
            .await?;
        Ok(next.trace_manifest)
    }

    /// Uploads a trace compaction's replacement and swaps it for its inputs in
    /// one catalog commit, so there is no generation in which both the inputs
    /// and the replacement answer a query. Idempotent: a replay after a crash
    /// that already committed finds the replacement present and the inputs
    /// gone, and changes nothing. An input that is missing while the
    /// replacement is not yet present means something else, such as retention,
    /// removed it first, and the caller must step aside.
    pub async fn replace_trace_parts(
        &self,
        added: &[TracePart],
        removed: &[TraceManifestPart],
    ) -> Result<TraceManifest, String> {
        for part in added {
            let id = part.meta.id.clone();
            TracePartReader::open(part.clone())
                .map_err(|error| format!("refusing to publish invalid trace part {id}: {error}"))?;
        }
        for part in added {
            self.upload_trace_part(part).await?;
        }

        let state = self.load_catalog_state().await?;
        self.check_epoch(state.writer_epoch)?;
        let removed_ids: Vec<String> = removed.iter().map(|part| part.id.clone()).collect();
        let descriptors: Vec<TraceManifestPart> = added
            .iter()
            .map(|part| TraceManifestPart {
                id: part.meta.id.clone(),
                partition: part.meta.partition.clone(),
            })
            .collect();
        let present_removed = state
            .trace_manifest
            .parts
            .iter()
            .filter(|part| removed_ids.iter().any(|id| id == &part.id))
            .count();
        let all_added_present = descriptors
            .iter()
            .all(|descriptor| state.trace_manifest.parts.contains(descriptor));
        if present_removed == 0 && all_added_present {
            return Ok(state.trace_manifest);
        }
        if present_removed != removed_ids.len() {
            return Err(format!(
                "{INPUTS_CHANGED_ERROR}: expected {} input trace parts, found {present_removed}",
                removed_ids.len()
            ));
        }
        for descriptor in &descriptors {
            if let Some(existing) = state
                .trace_manifest
                .parts
                .iter()
                .find(|item| item.id == descriptor.id)
                && existing != descriptor
            {
                return Err(format!(
                    "trace manifest part ID collision: {}",
                    descriptor.id
                ));
            }
        }
        let next = self
            .commit_catalog_mutation(CatalogMutation {
                transaction_id: uuid::Uuid::new_v4().to_string(),
                writer_epoch: state.writer_epoch,
                trace_added: descriptors,
                trace_removed: removed_ids,
                ..Default::default()
            })
            .await?;
        Ok(next.trace_manifest)
    }

    /// Removes trace descriptors from the manifest using the same CAS
    /// semantics as publication. The immutable objects are deleted only after
    /// the manifest no longer exposes them.
    pub async fn remove_trace_parts(
        &self,
        removed: &[TraceManifestPart],
    ) -> Result<TraceManifest, String> {
        if removed.is_empty() {
            return self.load_trace_manifest().await;
        }
        let state = self.load_catalog_state().await?;
        self.check_epoch(state.writer_epoch)?;
        let removed_ids: Vec<String> = removed.iter().map(|part| part.id.clone()).collect();
        let present = state
            .trace_manifest
            .parts
            .iter()
            .filter(|part| removed_ids.iter().any(|id| id == &part.id))
            .count();
        if present == 0 {
            return Ok(state.trace_manifest);
        }
        let next = self
            .commit_catalog_mutation(CatalogMutation {
                transaction_id: uuid::Uuid::new_v4().to_string(),
                writer_epoch: state.writer_epoch,
                trace_removed: removed_ids,
                ..Default::default()
            })
            .await?;
        Ok(next.trace_manifest)
    }

    pub async fn load_metric_manifest(&self) -> Result<MetricManifest, String> {
        Ok(self.load_catalog_state().await?.metric_manifest)
    }

    /// Uploads immutable metric part files, then atomically adds and removes
    /// their descriptors in one manifest CAS. Modeled on the log `publish`
    /// rather than the trace one because the metric compactor replaces its
    /// inputs, and a replacement whose add and remove land in two generations
    /// has a window where both the inputs and the output answer queries.
    pub async fn publish_metric_parts(
        &self,
        added: &[SeriesPart],
        removed: &[MetricManifestPart],
    ) -> Result<MetricManifest, String> {
        for part in added {
            let id = part.meta.id.clone();
            SeriesPartReader::open(part.clone()).map_err(|error| {
                format!("refusing to publish invalid metric part {id}: {error}")
            })?;
        }
        for part in added {
            self.upload_metric_part(part).await?;
        }

        let state = self.load_catalog_state().await?;
        self.check_epoch(state.writer_epoch)?;
        let removed_ids: Vec<String> = removed.iter().map(|part| part.id.clone()).collect();
        let descriptors: Vec<MetricManifestPart> =
            added.iter().map(MetricManifestPart::from).collect();
        if !removed_ids.is_empty() {
            let present_removed = state
                .metric_manifest
                .parts
                .iter()
                .filter(|part| removed_ids.iter().any(|id| id == &part.id))
                .count();
            let all_added_present = descriptors
                .iter()
                .all(|descriptor| state.metric_manifest.parts.contains(descriptor));
            if present_removed == 0 && all_added_present {
                return Ok(state.metric_manifest);
            }
            if !descriptors.is_empty() && present_removed != removed_ids.len() {
                return Err(format!(
                    "{INPUTS_CHANGED_ERROR}: expected {} input metric parts, found {present_removed}",
                    removed_ids.len()
                ));
            }
        }
        for descriptor in &descriptors {
            if let Some(existing) = state
                .metric_manifest
                .parts
                .iter()
                .find(|item| item.id == descriptor.id)
                && existing != descriptor
                && !removed_ids.iter().any(|id| id == &descriptor.id)
            {
                return Err(format!(
                    "metric manifest part ID collision: {}",
                    descriptor.id
                ));
            }
        }
        if removed_ids.is_empty()
            && descriptors
                .iter()
                .all(|descriptor| state.metric_manifest.parts.contains(descriptor))
        {
            return Ok(state.metric_manifest);
        }
        let next = self
            .commit_catalog_mutation(CatalogMutation {
                transaction_id: uuid::Uuid::new_v4().to_string(),
                writer_epoch: state.writer_epoch,
                metric_added: descriptors,
                metric_removed: removed_ids,
                ..Default::default()
            })
            .await?;
        Ok(next.metric_manifest)
    }

    /// Removal alone, idempotent per id for the same reason the trace removal
    /// is: retention retries must remove what is left rather than wedge.
    pub async fn remove_metric_parts(
        &self,
        removed: &[MetricManifestPart],
    ) -> Result<MetricManifest, String> {
        if removed.is_empty() {
            return self.load_metric_manifest().await;
        }
        self.publish_metric_parts(&[], removed).await
    }

    pub async fn delete_metric_part_objects(
        &self,
        parts: &[MetricManifestPart],
    ) -> Result<(), String> {
        for part in parts {
            for file in METRIC_PART_FILES {
                match self.store.delete(&self.metric_part_path(part, file)).await {
                    Ok(()) | Err(object_store::Error::NotFound { .. }) => {}
                    Err(error) => {
                        return Err(format!(
                            "failed to delete remote metric part {}/{} file {file}: {error}",
                            part.partition, part.id
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn delete_part_objects(&self, parts: &[ManifestPart]) -> Result<(), String> {
        for part in parts {
            for file in PART_FILES {
                match self.store.delete(&self.part_path(part, file)).await {
                    Ok(()) | Err(object_store::Error::NotFound { .. }) => {}
                    Err(error) => {
                        return Err(format!(
                            "failed to delete remote part {}/{} file {file}: {error}",
                            part.partition, part.id
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn delete_trace_part_objects(
        &self,
        parts: &[TraceManifestPart],
    ) -> Result<(), String> {
        for part in parts {
            for file in TRACE_PART_FILES {
                match self.store.delete(&self.trace_part_path(part, file)).await {
                    Ok(()) | Err(object_store::Error::NotFound { .. }) => {}
                    Err(error) => {
                        return Err(format!(
                            "failed to delete remote trace part {}/{} file {file}: {error}",
                            part.partition, part.id
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Store one tenant policy at a greater monotonic revision. Project queue
    /// messages may skip revisions; the policy body carries the revision for
    /// restart visibility, while the object version supplies the cross-process
    /// CAS that a local mutex cannot.
    pub async fn put_tenant_policy_revision(
        &self,
        tenant: &str,
        body: Vec<u8>,
        revision: u64,
    ) -> Result<(), String> {
        let path = self.tenant_policy_path(tenant);
        let existing = match self.store.get(&path).await {
            Ok(result) => {
                let e_tag = result.meta.e_tag.clone();
                let version = result.meta.version.clone();
                let bytes = result.bytes().await.map_err(|error| {
                    format!("failed to read the policy for tenant {tenant}: {error}")
                })?;
                Some((e_tag, version, bytes.to_vec()))
            }
            Err(object_store::Error::NotFound { .. }) => None,
            Err(error) => {
                return Err(format!("failed to read the policy for tenant {tenant}: {error}"));
            }
        };
        let current_revision = existing
            .as_ref()
            .and_then(|(_, _, bytes)| {
                serde_json::from_slice::<serde_json::Value>(bytes)
                    .ok()?
                    .get("revision")
                    .and_then(serde_json::Value::as_u64)
            })
            .unwrap_or(0);
        if let Some((e_tag, version, current_body)) = existing {
            if current_revision == revision && current_body == body {
                return Ok(());
            }
            if revision < current_revision {
                return Err(format!(
                    "tenant policy revision conflict for {tenant}: current={current_revision}, requested={revision}"
                ));
            }
            self.store
                .put_opts(
                    &path,
                    body.into(),
                    PutOptions {
                        mode: PutMode::Update(UpdateVersion { e_tag, version }),
                        ..Default::default()
                    },
                )
                .await
                .map(|_| ())
                .map_err(|error| {
                    format!("failed to update the policy for tenant {tenant}: {error}")
                })
        } else {
            self.store
                .put_opts(
                    &path,
                    body.into(),
                    PutOptions {
                        mode: PutMode::Create,
                        ..Default::default()
                    },
                )
                .await
                .map(|_| ())
                .map_err(|error| {
                    format!("failed to create the policy for tenant {tenant}: {error}")
                })
        }
    }

    pub async fn delete_tenant_policy(&self, tenant: &str) -> Result<(), String> {
        match self.store.delete(&self.tenant_policy_path(tenant)).await {
            Ok(()) | Err(object_store::Error::NotFound { .. }) => Ok(()),
            Err(error) => Err(format!(
                "failed to delete the policy for tenant {tenant}: {error}"
            )),
        }
    }

    /// Every stored policy, as `(file name, body)`. Read once at startup; a
    /// failure here is fatal, so it never returns a partial listing.
    pub async fn load_tenant_policies(&self) -> Result<Vec<(String, Vec<u8>)>, String> {
        use futures_util::StreamExt;

        let prefix = self.path(TENANT_POLICY_PREFIX);
        let mut locations = Vec::new();
        let mut stream = self.store.list(Some(&prefix));
        while let Some(item) = stream.next().await {
            let meta =
                item.map_err(|error| format!("failed to list the tenant policies: {error}"))?;
            locations.push(meta.location);
        }
        let mut policies = Vec::new();
        for location in locations {
            let name = location
                .filename()
                .ok_or_else(|| format!("tenant policy object {location} has no file name"))?
                .to_string();
            let bytes = self
                .store
                .get(&location)
                .await
                .map_err(|error| format!("failed to read tenant policy {location}: {error}"))?
                .bytes()
                .await
                .map_err(|error| format!("failed to read tenant policy {location}: {error}"))?;
            policies.push((name, bytes.to_vec()));
        }
        Ok(policies)
    }

    fn delete_request_path(&self, tenant: &str, request_id: &str) -> ObjectPath {
        self.path(&format!(
            "{DELETE_REQUEST_PREFIX}/{tenant}/{request_id}.json"
        ))
    }

    /// One object per request, for the same reason as one object per policy: a
    /// submission is a single unconditional write with nothing to contend on.
    pub async fn put_delete_request(
        &self,
        tenant: &str,
        request_id: &str,
        body: Vec<u8>,
    ) -> Result<(), String> {
        self.store
            .put(&self.delete_request_path(tenant, request_id), body.into())
            .await
            .map(|_| ())
            .map_err(|error| {
                format!("failed to store delete request {request_id} for tenant {tenant}: {error}")
            })
    }

    pub async fn remove_delete_request(
        &self,
        tenant: &str,
        request_id: &str,
    ) -> Result<(), String> {
        match self
            .store
            .delete(&self.delete_request_path(tenant, request_id))
            .await
        {
            Ok(()) | Err(object_store::Error::NotFound { .. }) => Ok(()),
            Err(error) => Err(format!(
                "failed to remove delete request {request_id} for tenant {tenant}: {error}"
            )),
        }
    }

    /// Every stored request body. Read once at startup; a failure here is fatal,
    /// because starting with a subset would serve data a tenant asked to have
    /// deleted.
    pub async fn load_delete_requests(&self) -> Result<Vec<Vec<u8>>, String> {
        use futures_util::StreamExt;

        let prefix = self.path(DELETE_REQUEST_PREFIX);
        let mut locations = Vec::new();
        let mut stream = self.store.list(Some(&prefix));
        while let Some(item) = stream.next().await {
            let meta =
                item.map_err(|error| format!("failed to list the delete requests: {error}"))?;
            locations.push(meta.location);
        }
        let mut requests = Vec::new();
        for location in locations {
            let bytes = self
                .store
                .get(&location)
                .await
                .map_err(|error| format!("failed to read delete request {location}: {error}"))?
                .bytes()
                .await
                .map_err(|error| format!("failed to read delete request {location}: {error}"))?;
            requests.push(bytes.to_vec());
        }
        Ok(requests)
    }

}

/// When one object was first seen outside the active set, which scan cycle saw
/// it last, and how large it is. The first sighting is what the grace period is
/// measured from, the cycle is how a completed cycle recognizes an entry whose
/// object is gone, and the size is what the per-pass byte budget and the dry
/// run report.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct OrphanLedgerEntry {
    first_seen: chrono::DateTime<chrono::Utc>,
    last_seen_cycle: u64,
    #[serde(default)]
    bytes: u64,
}

/// How far the current scan cycle has walked the part prefixes, and which cycle
/// that is.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct OrphanScanCursor {
    prefix_index: usize,
    after: Option<String>,
    cycle: u64,
}

impl OrphanScanCursor {
    fn first() -> Self {
        Self {
            prefix_index: 0,
            after: None,
            cycle: 0,
        }
    }

    fn start_next_cycle(&mut self) {
        self.prefix_index = 0;
        self.after = None;
        self.cycle += 1;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OrphanLedger {
    version: u32,
    scan: OrphanScanCursor,
    orphans: BTreeMap<String, OrphanLedgerEntry>,
}

impl OrphanLedger {
    fn empty() -> Self {
        Self {
            version: ORPHAN_LEDGER_VERSION,
            scan: OrphanScanCursor::first(),
            orphans: BTreeMap::new(),
        }
    }
}

/// A ledger written before the scan became resumable holds first sightings
/// alone, and dropping it would hand every object in it a fresh grace.
#[derive(Deserialize)]
#[serde(untagged)]
enum StoredOrphanLedger {
    Current(OrphanLedger),
    FirstSightings(BTreeMap<String, String>),
}

pub struct OrphanCollectionOptions {
    pub grace_period: std::time::Duration,
    pub max_runtime: std::time::Duration,
    pub max_scanned_objects: usize,
    pub max_deleted_objects: usize,
    pub max_deleted_bytes: u64,
    /// Report what the pass would delete and delete nothing. The ledger is
    /// still written, so a dry run ages first sightings exactly as a real pass
    /// does and the run that follows it deletes what the dry run reported.
    pub dry_run: bool,
}

impl OrphanCollectionOptions {
    #[cfg(test)]
    fn unbounded(grace_period: std::time::Duration) -> Self {
        Self {
            grace_period,
            max_runtime: std::time::Duration::from_secs(3600),
            max_scanned_objects: usize::MAX,
            max_deleted_objects: usize::MAX,
            max_deleted_bytes: u64::MAX,
            dry_run: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OrphanCollection {
    pub scanned_objects: usize,
    pub scan_cycles_completed: usize,
    pub candidate_objects: usize,
    pub candidate_bytes: u64,
    pub deleted_objects: usize,
    pub deleted_bytes: u64,
    pub delete_errors: usize,
    pub ledger_entries: usize,
}

impl ObjectStorage {
    /// Every object the three manifests name, which is what the collector must
    /// never delete. Read before anything else in a pass: a failure here ends
    /// the pass, so an unreadable catalog can never be mistaken for an empty
    /// active set.
    async fn active_object_keys(&self) -> Result<HashSet<String>, String> {
        let manifest = self.load_manifest().await?;
        let trace_manifest = self.load_trace_manifest().await?;
        let metric_manifest = self.load_metric_manifest().await?;
        let mut active = HashSet::new();
        for part in &manifest.parts {
            for file in PART_FILES {
                active.insert(self.part_path(part, file).to_string());
            }
        }
        for part in &trace_manifest.parts {
            for file in TRACE_PART_FILES {
                active.insert(self.trace_part_path(part, file).to_string());
            }
        }
        for part in &metric_manifest.parts {
            for file in METRIC_PART_FILES {
                active.insert(self.metric_part_path(part, file).to_string());
            }
        }
        Ok(active)
    }

    /// Deletes immutable objects that are absent from all three manifests only
    /// after a grace period. Retention removes manifest visibility first; this
    /// pass is the crash-safe, delayed physical garbage collector.
    pub async fn collect_orphans(
        &self,
        options: &OrphanCollectionOptions,
    ) -> Result<OrphanCollection, String> {
        self.collect_orphans_at(options, chrono::Utc::now(), tokio::time::Instant::now())
            .await
    }

    /// Delete objects no manifest names any more, once they have been out of
    /// the active set for `options.grace_period`.
    ///
    /// What the grace is for, now that retirement unregisters before it writes
    /// the manifest and a reader can no longer plan a part the manifest has
    /// dropped:
    ///
    /// * a restore or merge already past its plan, holding descriptors from
    ///   the manifest it read;
    /// * local cache state a crash left behind, which a restart reconciles
    ///   against the store rather than against what it last had in memory;
    /// * a manifest consumer that is not this process at all;
    /// * a store whose listing lags its writes, where an object can be
    ///   absent from `active` because the manifest read was stale;
    /// * the gap between one lifecycle step and the next -- a publish whose
    ///   objects are up but whose manifest write has not landed is held by
    ///   the ledger's first sighting.
    ///
    /// So the value is a bound on how far behind the slowest of those may be,
    /// not a guess.
    ///
    /// **A pass is bounded and resumable.** It walks the part prefixes from
    /// the cursor the last pass left, saves the ledger as it goes, and stops
    /// on whichever budget runs out first. That is not an optimization: the
    /// collector that had one deadline around the whole pass deleted
    /// sequentially until it was cut off, and everything it had learned about
    /// objects it saw for the first time died with the pass, so a store large
    /// enough to spend the deadline on listing alone never collected anything
    /// again.
    ///
    /// **Two sightings are needed, not one.** Deletion reads the ledger as the
    /// pass loaded it, so an object this pass saw for the first time is never
    /// also deleted by it, however old the object is.
    pub(crate) async fn collect_orphans_at(
        &self,
        options: &OrphanCollectionOptions,
        now: chrono::DateTime<chrono::Utc>,
        started_at: tokio::time::Instant,
    ) -> Result<OrphanCollection, String> {
        use futures_util::StreamExt;

        let active = self.active_object_keys().await?;
        let cutoff = now
            - chrono::Duration::from_std(options.grace_period)
                .map_err(|error| format!("invalid garbage-collection grace period: {error}"))?;
        let mut ledger = self.load_orphan_ledger().await?;
        let deletable: Vec<String> = ledger
            .orphans
            .iter()
            .filter(|(key, entry)| entry.first_seen < cutoff && !active.contains(key.as_str()))
            .map(|(key, _)| key.clone())
            .collect();

        let deadline = started_at + options.max_runtime;
        let mut outcome = OrphanCollection::default();
        let mut scanned_since_save = 0usize;
        // A listing that fails mid-cycle still leaves the sightings it made
        // worth keeping, so the pass saves them before it reports.
        let mut scan_error = None;
        while outcome.scanned_objects < options.max_scanned_objects
            && tokio::time::Instant::now() < deadline
        {
            let Some(prefix_name) = ORPHAN_PREFIXES.get(ledger.scan.prefix_index).copied() else {
                // The cycle closed, so an entry no listing touched during it
                // stands for an object that is no longer in the store. Cycles
                // are counted rather than timed: a sighting and the reset that
                // follows it share one pass's timestamp, so a comparison of
                // times cannot tell them apart.
                ledger
                    .orphans
                    .retain(|_, entry| entry.last_seen_cycle == ledger.scan.cycle);
                ledger.scan.start_next_cycle();
                outcome.scan_cycles_completed += 1;
                break;
            };
            let prefix = self.path(prefix_name);
            let mut stream = match ledger.scan.after.clone() {
                Some(after) => self
                    .store
                    .list_with_offset(Some(&prefix), &ObjectPath::from(after)),
                None => self.store.list(Some(&prefix)),
            };
            let mut prefix_exhausted = true;
            while let Some(item) = stream.next().await {
                let meta = match item {
                    Ok(meta) => meta,
                    Err(error) => {
                        scan_error = Some(format!("failed to list object store: {error}"));
                        prefix_exhausted = false;
                        break;
                    }
                };
                let key = meta.location.to_string();
                if active.contains(key.as_str()) {
                    // Back in the active set, so any grace it had started is
                    // no longer about anything.
                    ledger.orphans.remove(&key);
                } else {
                    ledger
                        .orphans
                        .entry(key.clone())
                        .and_modify(|entry| {
                            entry.last_seen_cycle = ledger.scan.cycle;
                            entry.bytes = meta.size;
                        })
                        .or_insert(OrphanLedgerEntry {
                            first_seen: now,
                            last_seen_cycle: ledger.scan.cycle,
                            bytes: meta.size,
                        });
                }
                ledger.scan.after = Some(key);
                outcome.scanned_objects += 1;
                scanned_since_save += 1;
                if scanned_since_save >= ORPHAN_LEDGER_SAVE_INTERVAL {
                    self.store_orphan_ledger(&ledger).await?;
                    scanned_since_save = 0;
                }
                if outcome.scanned_objects >= options.max_scanned_objects
                    || tokio::time::Instant::now() >= deadline
                {
                    prefix_exhausted = false;
                    break;
                }
            }
            drop(stream);
            if prefix_exhausted {
                ledger.scan.prefix_index += 1;
                ledger.scan.after = None;
            }
            if scan_error.is_some() {
                break;
            }
        }

        let mut deletable_bytes = 0u64;
        let mut within_budget = Vec::new();
        let mut within_budget_bytes = 0u64;
        for key in &deletable {
            let bytes = ledger.orphans.get(key).map_or(0, |entry| entry.bytes);
            deletable_bytes += bytes;
            if within_budget.len() >= options.max_deleted_objects
                || (!within_budget.is_empty()
                    && within_budget_bytes + bytes > options.max_deleted_bytes)
            {
                continue;
            }
            within_budget_bytes += bytes;
            within_budget.push(key.clone());
        }
        outcome.candidate_objects = deletable.len();
        outcome.candidate_bytes = deletable_bytes;

        if !options.dry_run {
            for chunk in within_budget.chunks(ORPHAN_DELETE_CHUNK) {
                if tokio::time::Instant::now() >= deadline {
                    break;
                }
                let locations = futures_util::stream::iter(
                    chunk
                        .iter()
                        .map(|key| Ok(ObjectPath::from(key.clone())))
                        .collect::<Vec<_>>(),
                )
                .boxed();
                let mut deletions = self.store.delete_stream(locations);
                while let Some(deletion) = deletions.next().await {
                    let deleted = match deletion {
                        Ok(location) => location.to_string(),
                        // Already gone is the state this pass wanted.
                        Err(object_store::Error::NotFound { path, .. }) => path,
                        Err(error) => {
                            outcome.delete_errors += 1;
                            tracing::warn!(%error, "failed to delete an orphan object");
                            continue;
                        }
                    };
                    if let Some(entry) = ledger.orphans.remove(&deleted) {
                        outcome.deleted_bytes += entry.bytes;
                    }
                    outcome.deleted_objects += 1;
                }
                if outcome.delete_errors >= ORPHAN_DELETE_ERROR_LIMIT {
                    break;
                }
            }
        }

        outcome.ledger_entries = ledger.orphans.len();
        self.store_orphan_ledger(&ledger).await?;
        if let Some(error) = scan_error {
            return Err(error);
        }
        if outcome.delete_errors >= ORPHAN_DELETE_ERROR_LIMIT {
            return Err(format!(
                "gave up after {} failed orphan deletions",
                outcome.delete_errors
            ));
        }
        Ok(outcome)
    }

    #[cfg(test)]
    pub(crate) async fn garbage_collect_orphans_at(
        &self,
        grace_period: std::time::Duration,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<usize, String> {
        let options = OrphanCollectionOptions::unbounded(grace_period);
        Ok(self
            .collect_orphans_at(&options, now, tokio::time::Instant::now())
            .await?
            .deleted_objects)
    }

    /// The collector's own record of when each object left the active set.
    ///
    /// A first sighting is an upper bound on the retirement it stands for --
    /// the object left the manifest at or before the pass that noticed -- so
    /// reading it this way can only hold an object longer, never delete one
    /// early. It lives beside the manifests rather than under the part
    /// prefixes so that the collector never lists its own bookkeeping.
    async fn load_orphan_ledger(&self) -> Result<OrphanLedger, String> {
        let bytes = match self.store.get(&self.path(GC_ORPHANS_FILE)).await {
            Ok(result) => result
                .bytes()
                .await
                .map_err(|error| format!("failed to read the orphan ledger: {error}"))?,
            Err(object_store::Error::NotFound { .. }) => return Ok(OrphanLedger::empty()),
            Err(error) => return Err(format!("failed to load the orphan ledger: {error}")),
        };
        let stored: StoredOrphanLedger = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid orphan ledger: {error}"))?;
        Ok(match stored {
            StoredOrphanLedger::Current(ledger) => ledger,
            StoredOrphanLedger::FirstSightings(sightings) => OrphanLedger {
                orphans: sightings
                    .into_iter()
                    .filter_map(|(key, at)| {
                        chrono::DateTime::parse_from_rfc3339(&at).ok().map(|at| {
                            (
                                key,
                                OrphanLedgerEntry {
                                    first_seen: at.with_timezone(&chrono::Utc),
                                    last_seen_cycle: 0,
                                    bytes: 0,
                                },
                            )
                        })
                    })
                    .collect(),
                ..OrphanLedger::empty()
            },
        })
    }

    async fn store_orphan_ledger(&self, ledger: &OrphanLedger) -> Result<(), String> {
        let body = serde_json::to_vec(ledger)
            .map_err(|error| format!("failed to encode the orphan ledger: {error}"))?;
        self.store
            .put_opts(
                &self.path(GC_ORPHANS_FILE),
                body.into(),
                PutOptions {
                    mode: PutMode::Overwrite,
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| format!("failed to store the orphan ledger: {error}"))?;
        Ok(())
    }
}
