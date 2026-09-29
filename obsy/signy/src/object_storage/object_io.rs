/// How much of a part file is resident while it is being uploaded.
///
/// Matched to `flush_chunk_bytes`, which is the size the engine already builds
/// and writes parts in, and comfortably above the 5 MiB floor `object_store`
/// documents for every part of a multipart upload but the last.
const UPLOAD_CHUNK_BYTES: u64 = 32 * 1024 * 1024;

/// Above this a streamed upload is logged at info. Below it the line is a
/// debug: a day of merge output is thousands of ordinary uploads, and the
/// question a log is kept for is whether a large one ever meets a memory peak.
const NOTEWORTHY_UPLOAD_BYTES: u64 = 100 * 1024 * 1024;

/// Uploads running at once, so a memory peak can be read against how many
/// bounded buffers were live rather than against one.
static UPLOADS_IN_FLIGHT: AtomicU32 = AtomicU32::new(0);

struct UploadInFlight;

impl UploadInFlight {
    fn enter() -> (Self, u32) {
        let count = UPLOADS_IN_FLIGHT.fetch_add(1, Ordering::Relaxed) + 1;
        (Self, count)
    }
}

impl Drop for UploadInFlight {
    fn drop(&mut self) {
        UPLOADS_IN_FLIGHT.fetch_sub(1, Ordering::Relaxed);
    }
}

impl ObjectStorage {
    /// One immutable object, uploaded without ever holding it whole.
    ///
    /// At or under [`UPLOAD_CHUNK_BYTES`] the file goes up in a single
    /// conditional put, which is what makes an existing key detectable, and is
    /// the path almost every part file takes. A larger file is streamed a
    /// chunk at a time, because a publish that costs its file size is what put
    /// resident memory at the cgroup limit. `put_multipart` carries no put
    /// mode, so the existing-key check is explicit there instead of a
    /// conditional create. That window has no second writer in it: part keys
    /// carry a uuid, and a competing writer is fenced at the manifest CAS
    /// before it can publish.
    async fn upload_object(
        &self,
        local_path: &Path,
        len: u64,
        object_path: &ObjectPath,
        kind: &str,
        subject: &str,
    ) -> Result<(), String> {
        let started = std::time::Instant::now();
        let (_in_flight, in_flight) = UploadInFlight::enter();
        if len <= UPLOAD_CHUNK_BYTES {
            let bytes = tokio::fs::read(local_path).await.map_err(|error| {
                format!(
                    "failed to read {kind} file {}: {error}",
                    local_path.display()
                )
            })?;
            match self
                .store
                .put_opts(
                    object_path,
                    bytes.into(),
                    PutOptions {
                        mode: PutMode::Create,
                        ..Default::default()
                    },
                )
                .await
            {
                Ok(_) => {}
                Err(object_store::Error::AlreadyExists { .. }) => {
                    self.verify_existing_object(local_path, len, object_path, subject)
                        .await?;
                }
                Err(error) => return Err(format!("failed to upload {subject}: {error}")),
            }
            tracing::debug!(
                subject,
                bytes = len,
                in_flight,
                upload_ms = started.elapsed().as_secs_f64() * 1000.0,
                "upload put the file whole"
            );
            return Ok(());
        }

        if self.store.head(object_path).await.is_ok() {
            self.verify_existing_object(local_path, len, object_path, subject)
                .await?;
            tracing::info!(
                subject,
                bytes = len,
                chunk_bytes = UPLOAD_CHUNK_BYTES,
                chunks = 0,
                in_flight,
                upload_ms = started.elapsed().as_secs_f64() * 1000.0,
                "upload found the file already stored"
            );
            return Ok(());
        }

        let mut upload = self
            .store
            .put_multipart(object_path)
            .await
            .map_err(|error| format!("failed to start the upload for {subject}: {error}"))?;
        let chunks = match self.stream_into(local_path, len, &mut upload).await {
            Ok(chunks) => chunks,
            Err(error) => {
                tracing::warn!(subject, bytes = len, %error, "aborting a streamed upload");
                upload.abort().await.ok();
                return Err(format!("failed to upload {subject}: {error}"));
            }
        };
        if let Err(error) = upload.complete().await {
            upload.abort().await.ok();
            return Err(format!(
                "failed to finish the upload for {subject}: {error}"
            ));
        }
        let upload_ms = started.elapsed().as_secs_f64() * 1000.0;
        // Every part over the chunk streams, which is most merge output. The
        // ones worth a line are the large, the slow and the overlapping --
        // those are what a memory peak has to be read against.
        if len >= NOTEWORTHY_UPLOAD_BYTES || upload_ms > 1000.0 || in_flight > 1 {
            tracing::info!(
                subject,
                bytes = len,
                chunk_bytes = UPLOAD_CHUNK_BYTES,
                chunks,
                in_flight,
                upload_ms,
                "upload streamed the file"
            );
        } else {
            tracing::debug!(
                subject,
                bytes = len,
                chunk_bytes = UPLOAD_CHUNK_BYTES,
                chunks,
                in_flight,
                upload_ms,
                "upload streamed the file"
            );
        }
        Ok(())
    }

    /// Feed the upload one chunk at a time, awaiting each. Parts are sent
    /// rather than queued so that exactly one chunk is resident: awaiting them
    /// together would be faster and would cost the file size again.
    async fn stream_into(
        &self,
        local_path: &Path,
        len: u64,
        upload: &mut Box<dyn MultipartUpload>,
    ) -> Result<u32, String> {
        let mut file = tokio::fs::File::open(local_path)
            .await
            .map_err(|error| format!("{}: {error}", local_path.display()))?;
        let mut sent = 0;
        let mut chunks = 0;
        while sent < len {
            let want = UPLOAD_CHUNK_BYTES.min(len - sent) as usize;
            let mut chunk = vec![0u8; want];
            file.read_exact(&mut chunk)
                .await
                .map_err(|error| format!("{}: {error}", local_path.display()))?;
            upload
                .put_part(chunk.into())
                .await
                .map_err(|error| error.to_string())?;
            sent += want as u64;
            chunks += 1;
        }
        Ok(chunks)
    }

    /// The bytes already under this key must be the ones this publish would
    /// write, which is the resume-after-a-crash case. Both sides are read in
    /// [`UPLOAD_CHUNK_BYTES`] ranges so that the check costs a chunk rather
    /// than two copies of the object.
    async fn verify_existing_object(
        &self,
        local_path: &Path,
        len: u64,
        object_path: &ObjectPath,
        subject: &str,
    ) -> Result<(), String> {
        let remote =
            self.store.head(object_path).await.map_err(|error| {
                format!("failed to verify existing object for {subject}: {error}")
            })?;
        if remote.size != len {
            return Err(format!("immutable object collision for {subject}"));
        }
        let mut file = tokio::fs::File::open(local_path)
            .await
            .map_err(|error| format!("failed to re-read {subject}: {error}"))?;
        let mut offset = 0;
        while offset < len {
            let want = UPLOAD_CHUNK_BYTES.min(len - offset);
            let stored = self
                .store
                .get_opts(
                    object_path,
                    GetOptions {
                        range: Some(GetRange::Bounded(offset..offset + want)),
                        ..Default::default()
                    },
                )
                .await
                .map_err(|error| format!("failed to read existing object for {subject}: {error}"))?
                .bytes()
                .await
                .map_err(|error| {
                    format!("failed to read existing object for {subject}: {error}")
                })?;
            let mut chunk = vec![0u8; want as usize];
            file.read_exact(&mut chunk)
                .await
                .map_err(|error| format!("failed to re-read {subject}: {error}"))?;
            if stored.as_ref() != chunk.as_slice() {
                tracing::warn!(subject, offset, "stored object differs from the local file");
                return Err(format!("immutable object collision for {subject}"));
            }
            offset += want;
        }
        Ok(())
    }
}

impl ObjectStorage {
    async fn upload_metric_part(&self, part: &SeriesPart) -> Result<(), String> {
        let descriptor = MetricManifestPart::from(part);
        for file in METRIC_PART_FILES {
            let local_path = part.dir.join(file);
            let metadata = std::fs::symlink_metadata(&local_path).map_err(|error| {
                format!(
                    "failed to inspect metric part file {}: {error}",
                    local_path.display()
                )
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(format!(
                    "refusing unsafe metric part file {}",
                    local_path.display()
                ));
            }
            self.upload_object(
                &local_path,
                metadata.len(),
                &self.metric_part_path(&descriptor, file),
                "metric part",
                &format!("metric part {} file {file}", part.meta.id),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn restore_metric_catalog(
        &self,
        metrics_root: &Path,
    ) -> Result<MetricManifest, String> {
        let manifest = self.load_metric_manifest().await?;
        std::fs::create_dir_all(metrics_root).map_err(|error| error.to_string())?;
        for descriptor in &manifest.parts {
            let dir = metric_cache_part_dir(metrics_root, descriptor)?;
            if crate::series_part::load_series_part(&dir)
                .ok()
                .and_then(|part| SeriesPartReader::open_cached(part).ok())
                .is_some()
            {
                continue;
            }
            self.download_metric_part(descriptor, metrics_root, false)
                .await?;
        }
        Ok(manifest)
    }

    /// The metric side of `reconcile_trace_local_cache`, on the same terms:
    /// metric parts are immutable, so an interrupted upload retries safely
    /// against byte-verified existing objects. Interrupted compactions replay
    /// first — their manifest replacement is idempotent, so every crash
    /// window converges before the local-only publish scan can see a
    /// half-replaced tier.
    pub async fn reconcile_metric_local_cache(
        &self,
        metrics_root: &Path,
    ) -> Result<MetricManifest, String> {
        self.replay_metric_compactions(metrics_root).await?;
        let manifest = self.restore_metric_catalog(metrics_root).await?;
        validate_cache_tree_no_symlinks(metrics_root)?;
        let active: HashMap<&str, &MetricManifestPart> = manifest
            .parts
            .iter()
            .map(|part| (part.id.as_str(), part))
            .collect();
        let mut unpublished = Vec::new();
        for part in discover_series_parts(metrics_root)? {
            let descriptor = MetricManifestPart::from(&part);
            if let Some(existing) = active.get(descriptor.id.as_str()) {
                if **existing != descriptor {
                    return Err(format!(
                        "local metric part {} conflicts with the remote manifest",
                        descriptor.id
                    ));
                }
                continue;
            }
            SeriesPartReader::open(part.clone()).map_err(|error| {
                format!(
                    "local metric part {} is not fully cached and is absent from the remote manifest: {error}",
                    descriptor.id
                )
            })?;
            unpublished.push(part);
        }
        if !unpublished.is_empty() {
            self.publish_metric_parts(&unpublished, &[]).await?;
        }
        self.restore_metric_catalog(metrics_root).await
    }

    /// Replays every pending compaction commit record against the remote
    /// manifest, then finishes it locally the way the crashed pass would
    /// have: replacement published (idempotently), inputs deleted, record
    /// cleared. A record whose replacement never became durable only clears.
    async fn replay_metric_compactions(&self, metrics_root: &Path) -> Result<(), String> {
        for (path, record) in crate::series_merge::read_records(metrics_root)? {
            let mut new_parts = Vec::new();
            let mut new_dirs = Vec::new();
            let mut durable = true;
            for relative in &record.new {
                let dir = crate::series_merge::record_dir(metrics_root, relative)?;
                match crate::series_part::load_series_part(&dir) {
                    Ok(part) => new_parts.push(part),
                    Err(_) => durable = false,
                }
                new_dirs.push(dir);
            }
            if durable {
                let mut input_descriptors = Vec::new();
                let mut input_dirs = Vec::new();
                for relative in &record.inputs {
                    let dir = crate::series_merge::record_dir(metrics_root, relative)?;
                    let (partition, id) = relative
                        .split_once('/')
                        .ok_or_else(|| format!("malformed compaction input {relative:?}"))?;
                    input_descriptors.push(MetricManifestPart {
                        id: id.to_string(),
                        partition: partition.to_string(),
                    });
                    input_dirs.push(dir);
                }
                match self
                    .publish_metric_parts(&new_parts, &input_descriptors)
                    .await
                {
                    Ok(_) => {
                        self.delete_metric_part_objects(&input_descriptors).await?;
                        crate::part::remove_part_dirs(&existing_dirs(&input_dirs))?;
                    }
                    // Neither the inputs nor the replacement are in the
                    // manifest, so the manifest has already retired both.
                    // Keeping either on disk would get it published again as
                    // a part a local-only run left behind.
                    Err(error) if is_inputs_changed_error(&error) => {
                        tracing::warn!(%error, "dropping a metric compaction the manifest already retired");
                        crate::part::remove_part_dirs(&existing_dirs(&new_dirs))?;
                        crate::part::remove_part_dirs(&existing_dirs(&input_dirs))?;
                    }
                    Err(error) => return Err(error),
                }
            } else {
                crate::part::remove_part_dirs(&existing_dirs(&new_dirs))?;
            }
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
            }
        }
        Ok(())
    }

    pub async fn restore_metric_parts(
        &self,
        metrics_root: &Path,
        ids: &HashSet<String>,
    ) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let manifest = self.load_metric_manifest().await?;
        let mut restored = HashSet::new();
        for descriptor in &manifest.parts {
            if !ids.contains(&descriptor.id) {
                continue;
            }
            let dir = metric_cache_part_dir(metrics_root, descriptor)?;
            if crate::series_part::load_series_part(&dir)
                .ok()
                .and_then(|part| SeriesPartReader::open(part).ok())
                .is_none()
            {
                self.download_metric_part(descriptor, metrics_root, true)
                    .await?;
            }
            restored.insert(descriptor.id.clone());
        }
        let missing: Vec<_> = ids.difference(&restored).cloned().collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "metric parts are no longer present in the object-store manifest: {}",
                missing.join(", ")
            ))
        }
    }

    async fn download_metric_part(
        &self,
        descriptor: &MetricManifestPart,
        metrics_root: &Path,
        include_data: bool,
    ) -> Result<(), String> {
        let final_dir = metric_cache_part_dir(metrics_root, descriptor)?;
        let staging = StagingDir::create(metrics_root, &descriptor.partition, &descriptor.id)?;
        let temp_dir = staging.path();
        let files: &[&str] = if include_data {
            &METRIC_PART_FILES
        } else {
            &METRIC_CATALOG_FILES
        };
        for file in files {
            let bytes = self
                .store
                .get(&self.metric_part_path(descriptor, file))
                .await
                .map_err(|error| {
                    format!(
                        "failed to download metric part {} file {file}: {error}",
                        descriptor.id
                    )
                })?
                .bytes()
                .await
                .map_err(|error| {
                    format!(
                        "failed to read metric part {} file {file}: {error}",
                        descriptor.id
                    )
                })?;
            tokio::fs::write(temp_dir.join(file), bytes)
                .await
                .map_err(|error| {
                    format!(
                        "failed to cache metric part {} file {file}: {error}",
                        descriptor.id
                    )
                })?;
        }
        let downloaded = crate::series_part::load_series_part(temp_dir)?;
        if downloaded.meta.id != descriptor.id || downloaded.meta.partition != descriptor.partition
        {
            return Err(format!(
                "downloaded metric part {} metadata mismatch",
                descriptor.id
            ));
        }
        SeriesPartReader::open_cached(downloaded)?;
        if include_data {
            SeriesPartReader::open(crate::series_part::load_series_part(temp_dir)?)?;
        }
        match std::fs::remove_dir_all(&final_dir) {
            Ok(()) => {}
            // Either nothing was cached here yet, or a concurrent restore of
            // the same immutable part committed first; both are the state this
            // line wanted.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
        if let Some(parent) = final_dir.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::rename(temp_dir, &final_dir).map_err(|error| error.to_string())?;
        Ok(())
    }

    async fn upload_trace_part(&self, part: &TracePart) -> Result<(), String> {
        let descriptor = TraceManifestPart {
            id: part.meta.id.clone(),
            partition: part.meta.partition.clone(),
        };
        for file in TRACE_PART_FILES {
            let local_path = part.dir.join(file);
            let metadata = std::fs::symlink_metadata(&local_path).map_err(|error| {
                format!(
                    "failed to inspect trace part file {}: {error}",
                    local_path.display()
                )
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(format!(
                    "refusing unsafe trace part file {}",
                    local_path.display()
                ));
            }
            self.upload_object(
                &local_path,
                metadata.len(),
                &self.trace_part_path(&descriptor, file),
                "trace part",
                &format!("trace part {} file {file}", part.meta.id),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn restore_trace_catalog(&self, traces_root: &Path) -> Result<TraceManifest, String> {
        let manifest = self.load_trace_manifest().await?;
        std::fs::create_dir_all(traces_root).map_err(|error| error.to_string())?;
        for descriptor in &manifest.parts {
            let dir = trace_cache_part_dir(traces_root, descriptor)?;
            if crate::trace_part::load_trace_part(&dir)
                .ok()
                .and_then(|part| TracePartReader::open_cached(part).ok())
                .is_some()
            {
                continue;
            }
            self.download_trace_part(descriptor, traces_root, false)
                .await?;
        }
        Ok(manifest)
    }

    /// Restores the remote trace catalog and publishes trace parts left by a
    /// local-only run or by a crash before the trace manifest CAS. Trace parts
    /// are immutable, so retrying an interrupted upload is safe: existing
    /// remote objects are verified byte-for-byte by `upload_trace_part`.
    pub async fn reconcile_trace_local_cache(
        &self,
        traces_root: &Path,
    ) -> Result<TraceManifest, String> {
        self.replay_trace_compactions(traces_root).await?;
        let manifest = self.restore_trace_catalog(traces_root).await?;
        validate_cache_tree_no_symlinks(traces_root)?;
        let active: HashMap<&str, &TraceManifestPart> = manifest
            .parts
            .iter()
            .map(|part| (part.id.as_str(), part))
            .collect();
        let mut unpublished = Vec::new();
        for part in discover_trace_parts(traces_root)? {
            let descriptor = TraceManifestPart {
                id: part.meta.id.clone(),
                partition: part.meta.partition.clone(),
            };
            if let Some(existing) = active.get(descriptor.id.as_str()) {
                if **existing != descriptor {
                    return Err(format!(
                        "local trace part {} conflicts with the remote manifest",
                        descriptor.id
                    ));
                }
                continue;
            }
            TracePartReader::open(part.clone()).map_err(|error| {
                format!(
                    "local trace part {} is not fully cached and is absent from the remote manifest: {error}",
                    descriptor.id
                )
            })?;
            unpublished.push(part);
        }
        if !unpublished.is_empty() {
            self.publish_trace_parts(&unpublished).await?;
        }
        self.restore_trace_catalog(traces_root).await
    }

    /// Finishes every trace compaction a crash interrupted, the way the pass
    /// would have: the replacement swapped in for its inputs, idempotently,
    /// then the local inputs removed and the record cleared. A replacement
    /// that never became durable is removed with its record and the inputs
    /// stay. Runs before the unpublished-part scan, which would otherwise
    /// publish a replacement beside the inputs it replaces.
    async fn replay_trace_compactions(&self, traces_root: &Path) -> Result<(), String> {
        for (path, record) in crate::trace_merge::read_records(traces_root)? {
            let mut new_parts = Vec::new();
            let mut new_dirs = Vec::new();
            let mut durable = true;
            for relative in &record.new {
                let dir = crate::trace_merge::record_dir(traces_root, relative)?;
                match crate::trace_part::load_trace_part(&dir)
                    .and_then(|part| TracePartReader::open(part.clone()).map(|_| part))
                {
                    Ok(part) => new_parts.push(part),
                    Err(_) => durable = false,
                }
                if dir.exists() {
                    new_dirs.push(dir);
                }
            }
            if durable {
                let mut input_descriptors = Vec::new();
                let mut input_dirs = Vec::new();
                for relative in &record.inputs {
                    input_dirs.push(crate::trace_merge::record_dir(traces_root, relative)?);
                    input_descriptors.push(crate::trace_merge::record_input_descriptor(relative)?);
                }
                match self.replace_trace_parts(&new_parts, &input_descriptors).await {
                    Ok(_) => crate::part::remove_part_dirs(&existing_dirs(&input_dirs))?,
                    // See `replay_metric_compactions`: the manifest retired
                    // both, and neither may be left for the unpublished scan.
                    Err(error) if is_inputs_changed_error(&error) => {
                        tracing::warn!(%error, "dropping a trace compaction the manifest already retired");
                        crate::part::remove_part_dirs(&new_dirs)?;
                        crate::part::remove_part_dirs(&existing_dirs(&input_dirs))?;
                    }
                    Err(error) => return Err(error),
                }
            } else {
                crate::part::remove_part_dirs(&new_dirs)?;
            }
            crate::trace_merge::remove_record(&path)?;
        }
        Ok(())
    }

    pub async fn restore_trace_parts(
        &self,
        traces_root: &Path,
        ids: &HashSet<String>,
    ) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let manifest = self.load_trace_manifest().await?;
        let mut restored = HashSet::new();
        for descriptor in &manifest.parts {
            if !ids.contains(&descriptor.id) {
                continue;
            }
            let dir = trace_cache_part_dir(traces_root, descriptor)?;
            if crate::trace_part::load_trace_part(&dir)
                .ok()
                .and_then(|part| TracePartReader::open(part).ok())
                .is_none()
            {
                self.download_trace_part(descriptor, traces_root, true)
                    .await?;
            }
            restored.insert(descriptor.id.clone());
        }
        let missing: Vec<_> = ids.difference(&restored).cloned().collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "trace parts are no longer present in the object-store manifest: {}",
                missing.join(", ")
            ))
        }
    }

    async fn download_trace_part(
        &self,
        descriptor: &TraceManifestPart,
        traces_root: &Path,
        include_data: bool,
    ) -> Result<(), String> {
        let final_dir = trace_cache_part_dir(traces_root, descriptor)?;
        let staging = StagingDir::create(traces_root, &descriptor.partition, &descriptor.id)?;
        let temp_dir = staging.path();
        let files: &[&str] = if include_data {
            &TRACE_PART_FILES
        } else {
            &TRACE_CATALOG_FILES
        };
        for file in files {
            let bytes = self
                .store
                .get(&self.trace_part_path(descriptor, file))
                .await
                .map_err(|error| {
                    format!(
                        "failed to download trace part {} file {file}: {error}",
                        descriptor.id
                    )
                })?
                .bytes()
                .await
                .map_err(|error| {
                    format!(
                        "failed to read trace part {} file {file}: {error}",
                        descriptor.id
                    )
                })?;
            tokio::fs::write(temp_dir.join(file), bytes)
                .await
                .map_err(|error| {
                    format!(
                        "failed to cache trace part {} file {file}: {error}",
                        descriptor.id
                    )
                })?;
        }
        let downloaded = crate::trace_part::load_trace_part(temp_dir)?;
        if downloaded.meta.id != descriptor.id || downloaded.meta.partition != descriptor.partition
        {
            return Err(format!(
                "downloaded trace part {} metadata mismatch",
                descriptor.id
            ));
        }
        TracePartReader::open_cached(downloaded)?;
        if include_data {
            TracePartReader::open(crate::trace_part::load_trace_part(temp_dir)?)?;
        }
        match std::fs::remove_dir_all(&final_dir) {
            Ok(()) => {}
            // Either nothing was cached here yet, or a concurrent restore of the
            // same immutable part committed first. Both are the state this line
            // was trying to produce.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
        if let Some(parent) = final_dir.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::rename(temp_dir, &final_dir).map_err(|error| error.to_string())?;
        Ok(())
    }

    /// Publishes local parts that are not yet visible in the manifest.
    /// `upload_part` verifies existing immutable keys, so this also resumes an
    /// upload interrupted before the manifest CAS without accepting unrelated
    /// bytes under the same part ID.
    pub async fn publish_local_only_parts(
        &self,
        local_parts: &[Part],
        manifest: &Manifest,
    ) -> Result<Manifest, String> {
        let active_ids: HashSet<&str> =
            manifest.parts.iter().map(|part| part.id.as_str()).collect();
        let mut unpublished = Vec::new();
        for part in local_parts {
            if active_ids.contains(part.meta.id.as_str()) {
                remove_upload_marker(part)?;
                continue;
            }
            let marker = part.dir.join(UPLOAD_MARKER_FILE);
            if marker.exists() {
                unpublished.push(part.clone());
                continue;
            }

            let descriptor = ManifestPart::from(part);
            let mut remote_files = 0usize;
            for file in PART_FILES {
                match self.store.head(&self.part_path(&descriptor, file)).await {
                    Ok(_) => remote_files += 1,
                    Err(object_store::Error::NotFound { .. }) => {}
                    Err(error) => {
                        return Err(format!(
                            "failed to inspect remote part {} file {file}: {error}",
                            part.meta.id
                        ));
                    }
                }
            }
            if remote_files == PART_FILES.len() {
                tracing::warn!(
                    part_id = %part.meta.id,
                    "local part is absent from the manifest but has a complete remote object set; preserving it as a stale generation"
                );
                continue;
            }
            write_upload_marker(part)?;
            unpublished.push(part.clone());
        }
        if unpublished.is_empty() {
            Ok(manifest.clone())
        } else {
            let published = self.publish(&unpublished, &[]).await?;
            for part in &unpublished {
                remove_upload_marker(part)?;
            }
            Ok(published)
        }
    }

    pub async fn publish_flush_parts(
        &self,
        log_parts: &[Part],
        trace_parts: &[TracePart],
        metric_parts: &[SeriesPart],
    ) -> Result<(Manifest, TraceManifest, MetricManifest), String> {
        for part in log_parts {
            let id = part.meta.id.clone();
            crate::part::PartReader::open(part.clone())
                .map_err(|error| format!("refusing to publish invalid part {id}: {error}"))?;
            write_upload_marker(part)?;
        }
        for part in trace_parts {
            let id = part.meta.id.clone();
            TracePartReader::open(part.clone())
                .map_err(|error| format!("refusing to publish invalid trace part {id}: {error}"))?;
        }
        for part in metric_parts {
            let id = part.meta.id.clone();
            SeriesPartReader::open(part.clone()).map_err(|error| {
                format!("refusing to publish invalid metric part {id}: {error}")
            })?;
        }
        for part in log_parts {
            self.upload_part(part).await?;
        }
        for part in trace_parts {
            self.upload_trace_part(part).await?;
        }
        for part in metric_parts {
            self.upload_metric_part(part).await?;
        }

        let state = self.load_catalog_state().await?;
        self.check_epoch(state.writer_epoch)?;
        let log_added: Vec<ManifestPart> = log_parts.iter().map(ManifestPart::from).collect();
        let trace_added: Vec<TraceManifestPart> = trace_parts
            .iter()
            .map(|part| TraceManifestPart {
                id: part.meta.id.clone(),
                partition: part.meta.partition.clone(),
            })
            .collect();
        let metric_added: Vec<MetricManifestPart> =
            metric_parts.iter().map(MetricManifestPart::from).collect();
        for descriptor in &log_added {
            if let Some(existing) = state
                .manifest
                .parts
                .iter()
                .find(|part| part.id == descriptor.id)
                && existing != descriptor
            {
                return Err(format!("manifest part ID collision: {}", descriptor.id));
            }
        }
        for descriptor in &trace_added {
            if let Some(existing) = state
                .trace_manifest
                .parts
                .iter()
                .find(|part| part.id == descriptor.id)
                && existing != descriptor
            {
                return Err(format!(
                    "trace manifest part ID collision: {}",
                    descriptor.id
                ));
            }
        }
        for descriptor in &metric_added {
            if let Some(existing) = state
                .metric_manifest
                .parts
                .iter()
                .find(|part| part.id == descriptor.id)
                && existing != descriptor
            {
                return Err(format!(
                    "metric manifest part ID collision: {}",
                    descriptor.id
                ));
            }
        }
        let all_present = log_added
            .iter()
            .all(|part| state.manifest.parts.contains(part))
            && trace_added
                .iter()
                .all(|part| state.trace_manifest.parts.contains(part))
            && metric_added
                .iter()
                .all(|part| state.metric_manifest.parts.contains(part));
        let next = if all_present {
            state
        } else {
            self.commit_catalog_mutation(CatalogMutation {
                transaction_id: uuid::Uuid::new_v4().to_string(),
                writer_epoch: state.writer_epoch,
                log_added,
                trace_added,
                metric_added,
                ..Default::default()
            })
            .await?
        };
        remove_upload_markers_best_effort(log_parts);
        Ok((next.manifest, next.trace_manifest, next.metric_manifest))
    }

    /// Uploads immutable part files, then atomically adds/removes their IDs in
    /// the manifest. Uploaded objects that lose a CAS race are harmless and
    /// will be collected by a future retention pass.
    pub async fn publish(
        &self,
        added: &[Part],
        removed_ids: &[String],
    ) -> Result<Manifest, String> {
        // The manifest is the source of truth. Never make a part visible
        // remotely until the exact local files have passed the same read-back
        // validation required by the registry.
        for part in added {
            let id = part.meta.id.clone();
            crate::part::PartReader::open(part.clone())
                .map_err(|error| format!("refusing to publish invalid part {id}: {error}"))?;
        }
        // Persist intent before the first immutable object is written. If the
        // process dies after the last object upload but before the manifest
        // CAS, startup can distinguish this transaction from an inactive old
        // generation and safely finish it.
        for part in added {
            write_upload_marker(part)?;
        }
        for part in added {
            self.upload_part(part).await?;
        }

        let state = self.load_catalog_state().await?;
        self.check_epoch(state.writer_epoch)?;
        let removed: Vec<String> = removed_ids.to_vec();
        let descriptors: Vec<ManifestPart> = added.iter().map(ManifestPart::from).collect();
        if !removed.is_empty() {
            let present_removed = state
                .manifest
                .parts
                .iter()
                .filter(|part| removed.iter().any(|id| id == &part.id))
                .count();
            let all_added_present = descriptors
                .iter()
                .all(|descriptor| state.manifest.parts.contains(descriptor));
            if present_removed == 0 && all_added_present {
                remove_upload_markers_best_effort(added);
                return Ok(state.manifest);
            }
            if !descriptors.is_empty() && present_removed != removed.len() {
                return Err(format!(
                    "{INPUTS_CHANGED_ERROR}: expected {} input parts, found {present_removed}",
                    removed.len()
                ));
            }
        }
        for descriptor in &descriptors {
            if let Some(existing) = state
                .manifest
                .parts
                .iter()
                .find(|item| item.id == descriptor.id)
                && existing != descriptor
                && !removed.iter().any(|id| id == &descriptor.id)
            {
                return Err(format!("manifest part ID collision: {}", descriptor.id));
            }
        }
        if removed.is_empty()
            && descriptors
                .iter()
                .all(|descriptor| state.manifest.parts.contains(descriptor))
        {
            remove_upload_markers_best_effort(added);
            return Ok(state.manifest);
        }
        let next = self
            .commit_catalog_mutation(CatalogMutation {
                transaction_id: uuid::Uuid::new_v4().to_string(),
                writer_epoch: state.writer_epoch,
                log_added: descriptors,
                log_removed: removed,
                ..Default::default()
            })
            .await?;
        remove_upload_markers_best_effort(added);
        Ok(next.manifest)
    }
}

/// `remove_part_dirs` fsyncs each parent, which fails for a part that was
/// never written and so has no partition directory.
fn existing_dirs(dirs: &[PathBuf]) -> Vec<PathBuf> {
    dirs.iter().filter(|dir| dir.exists()).cloned().collect()
}
