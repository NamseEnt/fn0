pub(crate) fn write_flush_transaction(
    data_dir: &Path,
    transaction: &FlushTransaction,
) -> Result<(), String> {
    std::fs::create_dir_all(data_dir).map_err(|error| error.to_string())?;
    let path = data_dir.join(FLUSH_TRANSACTION_FILE);
    let temporary = data_dir.join(".flush.txn.tmp");
    let bytes = serde_json::to_vec(transaction).map_err(|error| error.to_string())?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    use std::io::Write;
    file.write_all(&bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    std::fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
    part::fsync_dir(data_dir).map_err(|error| error.to_string())
}

pub(crate) fn clear_flush_transaction(data_dir: &Path) -> Result<(), String> {
    let path = data_dir.join(FLUSH_TRANSACTION_FILE);
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "refusing symlinked flush transaction {}",
            path.display()
        )),
        Ok(metadata) if !metadata.is_file() => Err(format!(
            "flush transaction is not a file: {}",
            path.display()
        )),
        Ok(_) => {
            std::fs::remove_file(&path).map_err(|error| error.to_string())?;
            part::fsync_dir(data_dir).map_err(|error| error.to_string())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

impl ObjectStorage {
    /// Reconciles a flush that may have published one manifest but not the
    /// other. The journal checkpoint decides whether the publication is
    /// committed; no WAL data is lost when the transaction is rolled back.
    pub(crate) async fn reconcile_flush_transaction(
        &self,
        data_dir: &Path,
        checkpoint: u64,
    ) -> Result<(), String> {
        let path = data_dir.join(FLUSH_TRANSACTION_FILE);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.to_string()),
        };
        let transaction: FlushTransaction = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid flush transaction: {error}"))?;
        if checkpoint >= transaction.offset {
            return clear_flush_transaction(data_dir);
        }

        self.rollback_flush_transaction_value(data_dir, &transaction)
            .await
    }

    pub(crate) async fn rollback_flush_transaction(&self, data_dir: &Path) -> Result<(), String> {
        let path = data_dir.join(FLUSH_TRANSACTION_FILE);
        let bytes = std::fs::read(&path)
            .map_err(|error| format!("failed to read flush transaction for rollback: {error}"))?;
        let transaction: FlushTransaction = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid flush transaction: {error}"))?;
        self.rollback_flush_transaction_value(data_dir, &transaction)
            .await
    }

    async fn rollback_flush_transaction_value(
        &self,
        data_dir: &Path,
        transaction: &FlushTransaction,
    ) -> Result<(), String> {
        let log_ids: Vec<String> = transaction
            .log_parts
            .iter()
            .map(|part| part.id.clone())
            .collect();
        let state = self.load_catalog_state().await?;
        self.check_epoch(state.writer_epoch)?;
        let trace_ids: Vec<String> = transaction
            .trace_parts
            .iter()
            .map(|part| part.id.clone())
            .collect();
        let metric_ids: Vec<String> = transaction
            .metric_parts
            .iter()
            .map(|part| part.id.clone())
            .collect();
        let has_removed = state
            .manifest
            .parts
            .iter()
            .any(|part| log_ids.iter().any(|id| id == &part.id))
            || state
                .trace_manifest
                .parts
                .iter()
                .any(|part| trace_ids.iter().any(|id| id == &part.id))
            || state
                .metric_manifest
                .parts
                .iter()
                .any(|part| metric_ids.iter().any(|id| id == &part.id));
        if has_removed {
            self.commit_catalog_mutation(CatalogMutation {
                transaction_id: uuid::Uuid::new_v4().to_string(),
                writer_epoch: state.writer_epoch,
                log_removed: log_ids,
                trace_removed: trace_ids,
                metric_removed: metric_ids,
                ..Default::default()
            })
            .await?;
        }
        self.delete_part_objects(&transaction.log_parts).await?;
        self.delete_trace_part_objects(&transaction.trace_parts)
            .await?;
        self.delete_metric_part_objects(&transaction.metric_parts)
            .await?;

        let parts_root = data_dir.join("parts");
        let traces_root = data_dir.join("traces");
        let metrics_root = data_dir.join("metrics");
        let log_dirs = transaction
            .log_parts
            .iter()
            .map(|part| transaction_part_dir(&parts_root, &part.partition, &part.id))
            .collect::<Result<Vec<_>, _>>()?;
        let trace_dirs = transaction
            .trace_parts
            .iter()
            .map(|part| transaction_part_dir(&traces_root, &part.partition, &part.id))
            .collect::<Result<Vec<_>, _>>()?;
        let metric_dirs = transaction
            .metric_parts
            .iter()
            .map(|part| transaction_part_dir(&metrics_root, &part.partition, &part.id))
            .collect::<Result<Vec<_>, _>>()?;
        let mut dirs = log_dirs;
        dirs.extend(trace_dirs);
        dirs.extend(metric_dirs);
        crate::part::remove_part_dirs(&dirs)?;
        clear_flush_transaction(data_dir)
    }
}

fn transaction_part_dir(root: &Path, partition: &str, id: &str) -> Result<PathBuf, String> {
    if !is_safe_path_component(partition) || !is_safe_path_component(id) {
        return Err("flush transaction contains an unsafe part path".to_string());
    }
    Ok(root.join(partition).join(id))
}

fn write_upload_marker(part: &Part) -> Result<(), String> {
    let marker = part.dir.join(UPLOAD_MARKER_FILE);
    for _ in 0..3 {
        match std::fs::symlink_metadata(&marker) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "refusing symlinked upload marker {}",
                    marker.display()
                ));
            }
            Ok(metadata) if metadata.is_file() => {
                return part::fsync_dir(&part.dir).map_err(|error| error.to_string());
            }
            Ok(_) => {
                return Err(format!(
                    "upload marker path is not a file: {}",
                    marker.display()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                match std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&marker)
                {
                    Ok(file) => {
                        file.sync_all().map_err(|error| error.to_string())?;
                        return part::fsync_dir(&part.dir).map_err(|error| error.to_string());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => return Err(error.to_string()),
                }
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err(format!(
        "upload marker changed repeatedly while creating {}",
        marker.display()
    ))
}

fn remove_upload_marker(part: &Part) -> Result<(), String> {
    let marker = part.dir.join(UPLOAD_MARKER_FILE);
    match std::fs::symlink_metadata(&marker) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(format!(
                "refusing symlinked upload marker {}",
                marker.display()
            ));
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(format!(
                "upload marker path is not a file: {}",
                marker.display()
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    }
    match std::fs::remove_file(&marker) {
        Ok(()) => part::fsync_dir(&part.dir).map_err(|error| error.to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn remove_upload_markers_best_effort(parts: &[Part]) {
    for part in parts {
        if let Err(error) = remove_upload_marker(part) {
            // The manifest is already committed. Reporting publication as a
            // failure here would cause callers to retry durable rows. Startup
            // can safely remove a leftover marker from an active part.
            tracing::warn!(%error, part_id = %part.meta.id, "failed to remove committed upload marker");
        }
    }
}

fn collect_local_merge_groups(parts_root: &Path) -> Result<Vec<LocalMergeGroup>, String> {
    let mut candidates: BTreeMap<Vec<String>, Vec<PathBuf>> = BTreeMap::new();
    let partitions = match std::fs::read_dir(parts_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    for partition in partitions {
        let partition = partition.map_err(|error| error.to_string())?;
        if !partition.path().is_dir() || partition.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        for entry in std::fs::read_dir(partition.path()).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let dir = entry.path();
            if !dir.is_dir() || !dir.join(part::MERGE_TOMBSTONE_FILE).exists() {
                continue;
            }
            let old_dirs = match part::read_merge_tombstone_dirs(&dir, parts_root) {
                Ok(old_dirs) => old_dirs,
                Err(error) => {
                    tracing::warn!(%error, ?dir, "ignoring malformed local merge tombstone");
                    continue;
                }
            };
            let mut old_ids: Vec<String> = old_dirs
                .iter()
                .map(|old_dir| {
                    old_dir
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(str::to_owned)
                        .ok_or_else(|| {
                            format!("invalid merge input directory {}", old_dir.display())
                        })
                })
                .collect::<Result<_, _>>()?;
            old_ids.sort();
            old_ids.dedup();
            if old_ids.is_empty() {
                tracing::warn!(?dir, "ignoring merge tombstone without inputs");
                continue;
            }
            candidates.entry(old_ids).or_default().push(dir);
        }
    }

    let mut groups = Vec::new();
    for (old_ids, dirs) in candidates {
        // Current merges operate within one day partition and therefore emit
        // one replacement part. Two directories naming the same input set
        // are competing attempts, not parts of one transaction; selecting or
        // combining them would duplicate rows.
        if dirs.len() != 1 {
            return Err(format!(
                "multiple local merge replacements name the same inputs: {}",
                old_ids.join(", ")
            ));
        }
        let dir = &dirs[0];
        let replacement = match part::load_part(dir).and_then(crate::part::PartReader::open) {
            Ok(reader) => reader.part().clone(),
            Err(error) => {
                tracing::warn!(%error, ?dir, "local merge replacement is invalid; retaining inputs");
                continue;
            }
        };
        groups.push(LocalMergeGroup {
            old_ids,
            added: vec![replacement],
        });
    }
    Ok(groups)
}

fn topological_merge_order(groups: &[LocalMergeGroup]) -> Result<Vec<usize>, String> {
    let mut producer = HashMap::new();
    for (index, group) in groups.iter().enumerate() {
        for part in &group.added {
            if producer.insert(part.meta.id.as_str(), index).is_some() {
                return Err(format!(
                    "duplicate local merge replacement ID {}",
                    part.meta.id
                ));
            }
        }
    }

    let mut indegree = vec![0usize; groups.len()];
    let mut dependents = vec![Vec::new(); groups.len()];
    for (index, group) in groups.iter().enumerate() {
        let mut parents = HashSet::new();
        for old_id in &group.old_ids {
            if let Some(&parent) = producer.get(old_id.as_str())
                && parents.insert(parent)
            {
                indegree[index] += 1;
                dependents[parent].push(index);
            }
        }
    }

    let mut ready: VecDeque<usize> = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, &degree)| (degree == 0).then_some(index))
        .collect();
    let mut ordered = Vec::with_capacity(groups.len());
    while let Some(index) = ready.pop_front() {
        ordered.push(index);
        for &dependent in &dependents[index] {
            indegree[dependent] -= 1;
            if indegree[dependent] == 0 {
                ready.push_back(dependent);
            }
        }
    }
    if ordered.len() != groups.len() {
        return Err("local merge tombstones contain a cycle".to_string());
    }
    Ok(ordered)
}

fn merge_group_reaches_active_output(
    start: usize,
    groups: &[LocalMergeGroup],
    active_ids: &HashSet<&str>,
) -> bool {
    let mut pending: VecDeque<&str> = groups[start]
        .added
        .iter()
        .map(|part| part.meta.id.as_str())
        .collect();
    let mut visited = HashSet::new();

    while let Some(id) = pending.pop_front() {
        if !visited.insert(id) {
            continue;
        }
        if active_ids.contains(id) {
            return true;
        }
        for group in groups {
            if group.old_ids.iter().any(|old_id| old_id == id) {
                pending.extend(group.added.iter().map(|part| part.meta.id.as_str()));
            }
        }
    }
    false
}

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    let mut ids = HashSet::new();
    for part in &manifest.parts {
        if !is_safe_path_component(&part.id)
            || !is_safe_path_component(&part.partition)
            || !ids.insert(part.id.as_str())
        {
            return Err("manifest contains an invalid or duplicate part".to_string());
        }
    }
    Ok(())
}

fn validate_trace_manifest(manifest: &TraceManifest) -> Result<(), String> {
    let mut ids = HashSet::new();
    for part in &manifest.parts {
        if !is_safe_path_component(&part.id)
            || !is_safe_path_component(&part.partition)
            || !ids.insert(part.id.as_str())
        {
            return Err("trace manifest contains an invalid or duplicate part".to_string());
        }
    }
    Ok(())
}

fn validate_metric_manifest(manifest: &MetricManifest) -> Result<(), String> {
    let mut ids = HashSet::new();
    for part in &manifest.parts {
        if !is_safe_path_component(&part.id)
            || !is_safe_path_component(&part.partition)
            || !ids.insert(part.id.as_str())
        {
            return Err("metric manifest contains an invalid or duplicate part".to_string());
        }
    }
    Ok(())
}

fn metric_cache_part_dir(
    metrics_root: &Path,
    descriptor: &MetricManifestPart,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(metrics_root).map_err(|error| error.to_string())?;
    let canonical_root = validate_cache_root(metrics_root)?;
    let mut current = metrics_root.to_path_buf();
    let mut absent = false;
    for component in [&descriptor.partition, &descriptor.id] {
        if !is_safe_path_component(component) {
            return Err(format!("unsafe metric cache path component {component:?}"));
        }
        current.push(component);
        if absent {
            continue;
        }
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "refusing symlinked metric cache directory {}",
                    current.display()
                ));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(format!(
                    "metric cache path is not a directory: {}",
                    current.display()
                ));
            }
            Ok(_) => {}
            // Answering "where would this part live" must not create it --
            // see `ensure_safe_directory_chain`.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                absent = true;
                continue;
            }
            Err(error) => return Err(error.to_string()),
        }
        let canonical = std::fs::canonicalize(&current).map_err(|error| error.to_string())?;
        if !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "metric cache directory escapes root: {}",
                current.display()
            ));
        }
    }
    validate_metric_cache_files(&current)?;
    Ok(current)
}

fn validate_metric_cache_files(dir: &Path) -> Result<(), String> {
    for file in [
        SERIES_DATA_FILE,
        SERIES_INDEX_FILE,
        SERIES_BLOOM_FILE,
        SERIES_META_FILE,
        ".access",
    ] {
        let path = dir.join(file);
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "refusing symlinked metric cache file {}",
                    path.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn trace_cache_part_dir(
    traces_root: &Path,
    descriptor: &TraceManifestPart,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(traces_root).map_err(|error| error.to_string())?;
    let canonical_root = validate_cache_root(traces_root)?;
    let mut current = traces_root.to_path_buf();
    let mut absent = false;
    for component in [&descriptor.partition, &descriptor.id] {
        if !is_safe_path_component(component) {
            return Err(format!("unsafe trace cache path component {component:?}"));
        }
        current.push(component);
        if absent {
            continue;
        }
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "refusing symlinked trace cache directory {}",
                    current.display()
                ));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(format!(
                    "trace cache path is not a directory: {}",
                    current.display()
                ));
            }
            Ok(_) => {}
            // Answering "where would this part live" must not create it --
            // see `ensure_safe_directory_chain`.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                absent = true;
                continue;
            }
            Err(error) => return Err(error.to_string()),
        }
        let canonical = std::fs::canonicalize(&current).map_err(|error| error.to_string())?;
        if !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "trace cache directory escapes root: {}",
                current.display()
            ));
        }
    }
    validate_trace_cache_files(&current)?;
    Ok(current)
}

fn validate_trace_cache_files(dir: &Path) -> Result<(), String> {
    for file in [
        TRACE_DATA_FILE,
        TRACE_BLOOM_FILE,
        TRACE_META_FILE,
        ".access",
    ] {
        let path = dir.join(file);
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "refusing symlinked trace cache file {}",
                    path.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn is_safe_path_component(value: &str) -> bool {
    let mut components = Path::new(value).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}
