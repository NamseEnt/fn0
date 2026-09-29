#[allow(clippy::too_many_arguments)]
fn write_meta(
    path: &Path,
    id: &str,
    partition: &str,
    rows: &[Row],
    row_group_size: usize,
    metadata_columns: &[(String, u64)],
    parsed_columns: &[(String, u64)],
    row_group_ranges: &[ByteRange],
) -> io::Result<()> {
    let n = rows.len();
    let bounds = row_group_bounds(rows, row_group_size);
    // Rows are ordered by `(tenant, timestamp)`, so the part-wide extremes are
    // no longer the first and last row.
    let min_ts = rows.iter().map(|r| r.timestamp_ns).min().unwrap_or_default();
    let max_ts = rows.iter().map(|r| r.timestamp_ns).max().unwrap_or_default();
    let row_group_min_ts: Vec<i64> = bounds
        .iter()
        .map(|(start, end)| {
            rows[*start..*end]
                .iter()
                .map(|row| row.timestamp_ns)
                .min()
                .unwrap_or_default()
        })
        .collect();
    let row_group_max_ts: Vec<i64> = bounds
        .iter()
        .map(|(start, end)| {
            rows[*start..*end]
                .iter()
                .map(|row| row.timestamp_ns)
                .max()
                .unwrap_or_default()
        })
        .collect();
    let row_group_rows: Vec<u32> = bounds
        .iter()
        .map(|(start, end)| (end - start) as u32)
        .collect();
    // Whether the group is one non-decreasing run of timestamps. Rows are
    // sorted by `(tenant, timestamp)`, so this is true by construction for a
    // freshly written part; the bit is kept so the reader's early stop stays
    // an explicitly recorded property rather than an assumption.
    let row_group_ts_monotonic: Vec<bool> = bounds
        .iter()
        .map(|(start, end)| {
            rows[*start..*end]
                .windows(2)
                .all(|pair| pair[0].timestamp_ns <= pair[1].timestamp_ns)
        })
        .collect();
    let mut tenants = tenant_segments(rows, &bounds, row_group_ranges);

    let dir = path
        .parent()
        .ok_or_else(|| io::Error::other("metadata path has no parent"))?;
    // One pass over the body for both checksums. The segments are contiguous,
    // ordered and non-overlapping — rows are sorted by tenant and row groups
    // never straddle a boundary — so the per-segment digests fall out of the
    // same read the part-wide one already needed.
    let data_crc32 = checksum_data_and_segments(&dir.join(DATA_FILE), &mut tenants)?;
    let integrity = PartIntegrity {
        data_crc32,
        index_crc32: file_crc32(&dir.join(INDEX_FILE))?,
        metadata_crc32: 0,
    };
    let mut meta = MetaFile {
        id: id.to_string(),
        partition: partition.to_string(),
        min_ts_ns: min_ts,
        max_ts_ns: max_ts,
        row_count: n as u64,
        row_group_count: bounds.len() as u32,
        row_group_min_ts,
        row_group_max_ts,
        row_group_rows,
        row_group_ts_monotonic,
        tenants,
        materialized_bytes: rows.iter().map(Row::materialized_bytes).sum(),
        metadata_columns: metadata_columns.to_vec(),
        parsed_columns: parsed_columns.to_vec(),
        integrity,
    };
    meta.integrity.metadata_crc32 = metadata_crc32(&meta).map_err(io::Error::other)?;
    // Compact, not pretty. `tenants` and the two row-group timestamp arrays
    // all scale with tenant breadth, and pretty-printing puts every element of
    // them on its own indented line — whitespace that startup then parses for
    // every part. The checksum is taken over the canonical `to_vec` form, so
    // the file's layout was never part of what it verifies.
    let s = serde_json::to_string(&meta).map_err(io::Error::other)?;
    fs::write(path, s)?;
    sync_file(path)?;
    Ok(())
}

/// Build the per-tenant index from `(tenant, timestamp)`-sorted rows whose
/// row-group boundaries already respect tenant boundaries.
///
/// `crc32` is left zero here and filled in by `checksum_data_and_segments`,
/// which is the only place that reads the body.
fn tenant_segments(
    rows: &[Row],
    bounds: &[(usize, usize)],
    row_group_ranges: &[ByteRange],
) -> Vec<TenantSegment> {
    let mut segments: Vec<TenantSegment> = Vec::new();
    for (row_group, (start, end)) in bounds.iter().enumerate() {
        let group = &rows[*start..*end];
        let tenant = &group[0].tenant;
        let min_ts_ns = group.iter().map(|r| r.timestamp_ns).min().unwrap_or_default();
        let max_ts_ns = group.iter().map(|r| r.timestamp_ns).max().unwrap_or_default();
        let range = row_group_ranges.get(row_group).copied().unwrap_or_default();
        match segments.last_mut() {
            Some(segment) if segment.tenant == *tenant => {
                segment.row_group_end = row_group as u32 + 1;
                segment.row_count += group.len() as u64;
                segment.min_ts_ns = segment.min_ts_ns.min(min_ts_ns);
                segment.max_ts_ns = segment.max_ts_ns.max(max_ts_ns);
                segment.bytes.end = segment.bytes.end.max(range.end);
            }
            _ => segments.push(TenantSegment {
                tenant: tenant.clone(),
                row_group_start: row_group as u32,
                row_group_end: row_group as u32 + 1,
                row_count: group.len() as u64,
                min_ts_ns,
                max_ts_ns,
                bytes: range,
                crc32: 0,
            }),
        }
    }
    segments
}

/// Checksums the body once, filling in each segment's digest on the way past.
///
/// Segments are contiguous, ordered and non-overlapping, so a single forward
/// read covers all of them. Reading the file once per segment instead would
/// make flush cost O(tenants) reads of the whole part.
fn checksum_data_and_segments(path: &Path, segments: &mut [TenantSegment]) -> io::Result<u32> {
    let mut file = fs::File::open(path)?;
    let mut whole = crc32fast::Hasher::new();
    let mut digests = vec![crc32fast::Hasher::new(); segments.len()];
    let mut buffer = [0u8; 64 * 1024];
    let mut offset = 0u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let chunk = &buffer[..read];
        whole.update(chunk);
        let chunk_end = offset.saturating_add(read as u64);
        for (segment, digest) in segments.iter().zip(digests.iter_mut()) {
            let start = segment.bytes.start.max(offset);
            let end = segment.bytes.end.min(chunk_end);
            if start < end {
                digest.update(&chunk[(start - offset) as usize..(end - offset) as usize]);
            }
        }
        offset = chunk_end;
    }
    for (segment, digest) in segments.iter_mut().zip(digests) {
        segment.crc32 = digest.finalize();
    }
    Ok(whole.finalize())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PartIntegrity {
    data_crc32: u32,
    index_crc32: u32,
    metadata_crc32: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct MetaFile {
    /// Absent in parts written before versioning, which `serde` maps to 0 —
    /// a value this build never writes, so those parts are reported as an
    /// unsupported version rather than as corruption.
    #[serde(default)]
    id: String,
    partition: String,
    min_ts_ns: i64,
    max_ts_ns: i64,
    row_count: u64,
    row_group_count: u32,
    row_group_min_ts: Vec<i64>,
    row_group_max_ts: Vec<i64>,
    row_group_rows: Vec<u32>,
    row_group_ts_monotonic: Vec<bool>,
    tenants: Vec<TenantSegment>,
    materialized_bytes: u64,
    /// The metadata keys this part stores as `_sm:` columns, sorted, each with
    /// the number of rows carrying it. The counts are what let a merge choose
    /// its output's columns deterministically without reading a row.
    metadata_columns: Vec<(String, u64)>,
    /// The same for `_pf:` columns: the fields `| json` extracts from the
    /// part's lines, stored beside the line it still keeps.
    parsed_columns: Vec<(String, u64)>,
    integrity: PartIntegrity,
}

fn file_crc32(path: &Path) -> io::Result<u32> {
    let mut file = fs::File::open(path)?;
    let mut hasher = crc32fast::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize())
}

fn metadata_crc32(meta: &MetaFile) -> Result<u32, String> {
    let mut canonical = meta.clone();
    canonical.integrity.metadata_crc32 = 0;
    let bytes = serde_json::to_vec(&canonical).map_err(|error| error.to_string())?;
    Ok(crc32fast::hash(&bytes))
}

#[derive(Serialize, Deserialize)]
struct MergeTombstone {
    old_dirs: Vec<PathBuf>,
}

pub fn load_part(dir: &Path) -> Result<Part, String> {
    let _arena = crate::memprof::enter(crate::memprof::Arena::PartMeta);
    let meta_str = fs::read_to_string(dir.join(META_FILE)).map_err(|e| e.to_string())?;
    let meta_file: MetaFile = serde_json::from_str(&meta_str).map_err(|e| e.to_string())?;
    let actual_metadata_crc = metadata_crc32(&meta_file)?;
    if actual_metadata_crc != meta_file.integrity.metadata_crc32 {
        return Err(format!(
            "metadata checksum mismatch: expected {}, got {}",
            meta_file.integrity.metadata_crc32, actual_metadata_crc
        ));
    }
    validate_meta_file(dir, &meta_file)?;
    let meta = PartMeta {
        id: meta_file.id,
        partition: meta_file.partition,
        min_ts_ns: meta_file.min_ts_ns,
        max_ts_ns: meta_file.max_ts_ns,
        row_count: meta_file.row_count,
        row_group_count: meta_file.row_group_count,
        row_group_min_ts: meta_file.row_group_min_ts,
        row_group_max_ts: meta_file.row_group_max_ts,
        row_group_rows: meta_file.row_group_rows,
        row_group_ts_monotonic: meta_file.row_group_ts_monotonic,
        tenants: meta_file.tenants,
        materialized_bytes: meta_file.materialized_bytes,
        metadata_columns: meta_file.metadata_columns,
        parsed_columns: meta_file.parsed_columns,
        meta_bytes: meta_str.len() as u64,
        integrity: meta_file.integrity,
    };
    Ok(Part {
        dir: dir.to_path_buf(),
        meta,
    })
}

fn validate_meta_file(dir: &Path, meta: &MetaFile) -> Result<(), String> {
    let dir_id = dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("invalid part directory name: {}", dir.display()))?;
    if meta.id != dir_id {
        return Err(format!(
            "part metadata id {} does not match directory {dir_id}",
            meta.id
        ));
    }
    let dir_partition = dir
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("invalid part partition directory: {}", dir.display()))?;
    if meta.partition != dir_partition {
        return Err(format!(
            "part metadata partition {} does not match directory {dir_partition}",
            meta.partition
        ));
    }
    if meta.row_count == 0 || meta.row_group_count == 0 {
        return Err("part metadata must contain at least one row and row group".to_string());
    }
    if meta.min_ts_ns > meta.max_ts_ns
        || partition_of(meta.min_ts_ns) != meta.partition
        || partition_of(meta.max_ts_ns) != meta.partition
    {
        return Err("part metadata has an invalid timestamp range".to_string());
    }
    let row_group_count = meta.row_group_count as usize;
    if meta.row_group_min_ts.len() != row_group_count
        || meta.row_group_max_ts.len() != row_group_count
        || meta.row_group_min_ts.iter().min() != Some(&meta.min_ts_ns)
        || meta.row_group_max_ts.iter().max() != Some(&meta.max_ts_ns)
    {
        return Err("part metadata has inconsistent row-group bounds".to_string());
    }
    if meta.row_group_rows.len() != row_group_count
        || meta.row_group_ts_monotonic.len() != row_group_count
        || meta.row_group_rows.iter().map(|&n| n as u64).sum::<u64>() != meta.row_count
        || meta.row_group_rows.contains(&0)
    {
        return Err("part metadata has inconsistent row-group row counts".to_string());
    }
    if meta
        .metadata_columns
        .windows(2)
        .any(|pair| pair[0].0 >= pair[1].0)
        || meta.metadata_columns.len() > MAX_METADATA_COLUMNS
        || meta
            .parsed_columns
            .windows(2)
            .any(|pair| pair[0].0 >= pair[1].0)
        || meta.parsed_columns.len() > MAX_METADATA_COLUMNS
    {
        return Err("part metadata columns are not sorted, unique and capped".to_string());
    }
    for index in 0..row_group_count {
        if meta.row_group_min_ts[index] > meta.row_group_max_ts[index] {
            return Err("part metadata row-group bounds are not sorted".to_string());
        }
    }
    validate_tenant_segments(meta)?;
    Ok(())
}

/// The tenant index is the isolation boundary for a shared part, so it is
/// validated as strictly as the rest of the metadata: it must be sorted,
/// gap-free, and cover exactly the part's row groups and rows. Anything less
/// would let a malformed part expose one tenant's row groups to another.
fn validate_tenant_segments(meta: &MetaFile) -> Result<(), String> {
    if meta.tenants.is_empty() {
        return Err("part metadata has no tenant segments".to_string());
    }
    let mut expected_start = 0u32;
    let mut total_rows = 0u64;
    for (index, segment) in meta.tenants.iter().enumerate() {
        if index > 0 && meta.tenants[index - 1].tenant >= segment.tenant {
            return Err("part tenant segments are not sorted by tenant".to_string());
        }
        if segment.row_group_start != expected_start || segment.row_group_end <= segment.row_group_start
        {
            return Err("part tenant segments do not tile the row groups".to_string());
        }
        if segment.row_group_end > meta.row_group_count {
            return Err("part tenant segment exceeds the row-group count".to_string());
        }
        let groups = segment.row_group_start as usize..segment.row_group_end as usize;
        let segment_min = meta.row_group_min_ts[groups.clone()]
            .iter()
            .min()
            .copied()
            .unwrap_or_default();
        let segment_max = meta.row_group_max_ts[groups.clone()]
            .iter()
            .max()
            .copied()
            .unwrap_or_default();
        if segment.min_ts_ns != segment_min || segment.max_ts_ns != segment_max {
            return Err("part tenant segment timestamps do not match its row groups".to_string());
        }
        // A group's own bounds must be consistent; whether the group is one
        // ordered run is recorded per group in `row_group_ts_monotonic`.
        for row_group in groups.clone() {
            if meta.row_group_min_ts[row_group] > meta.row_group_max_ts[row_group] {
                return Err("part row group timestamps are inverted".to_string());
            }
        }
        if segment.row_count == 0 {
            return Err("part tenant segment is empty".to_string());
        }
        total_rows = total_rows.saturating_add(segment.row_count);
        expected_start = segment.row_group_end;
    }
    if expected_start != meta.row_group_count {
        return Err("part tenant segments do not cover every row group".to_string());
    }
    if total_rows != meta.row_count {
        return Err("part tenant segment row counts do not sum to the part row count".to_string());
    }
    Ok(())
}

pub fn discover_parts(parts_root: &Path) -> Result<Vec<Part>, String> {
    let mut parts = Vec::new();
    let partitions = match fs::read_dir(parts_root) {
        Ok(partitions) => partitions,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(parts),
        Err(error) => {
            return Err(format!(
                "failed to read parts root {}: {error}",
                parts_root.display()
            ));
        }
    };
    let partition_entries: Vec<_> = partitions.collect::<Result<_, _>>().map_err(|e| {
        format!(
            "failed to enumerate parts root {}: {e}",
            parts_root.display()
        )
    })?;

    // Pass 1 only reads and validates merge markers. Deleting while walking
    // the directory tree can erase an intermediate part's tombstone before it
    // is inspected, which would let an older generation reappear. Build the
    // complete replacement graph first and clean it only after its transitive
    // closure is known.
    let mut tombstoned_dirs: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    let mut invalid_merge_dirs: std::collections::HashSet<PathBuf> =
        std::collections::HashSet::new();
    let mut tombstone_edges: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut valid_replacements: Vec<PathBuf> = Vec::new();

    for partition_entry in &partition_entries {
        let partition_path = partition_entry.path();
        if !partition_path.is_dir() {
            continue;
        }
        let name = partition_entry.file_name();
        if name == ".tmp" {
            continue;
        }
        let part_entries: Vec<_> = fs::read_dir(&partition_path)
            .map_err(|e| format!("failed to read partition {}: {e}", partition_path.display()))?
            .collect::<Result<_, _>>()
            .map_err(|e| {
                format!(
                    "failed to enumerate partition {}: {e}",
                    partition_path.display()
                )
            })?;

        for part_entry in &part_entries {
            let part_dir = part_entry.path();
            if !part_dir.is_dir() {
                continue;
            }
            if !part_dir.join(META_FILE).exists() {
                continue;
            }
            if !part_dir.join(MERGE_TOMBSTONE_FILE).exists() {
                continue;
            }
            match read_merge_tombstone_dirs(&part_dir, parts_root) {
                Ok(old_dirs) => {
                    let part_key = canonical_path(&part_dir);
                    tombstone_edges.insert(part_key, old_dirs);
                    // Do not delete the old parts until the replacement part
                    // itself can be opened. A corrupt replacement must not
                    // turn a recoverable merge into data loss on restart.
                    let replacement_valid = match load_part(&part_dir).and_then(PartReader::open) {
                        Ok(_) => true,
                        Err(e) => {
                            tracing::warn!(
                                error = %e,
                                ?part_dir,
                                "discover: invalid merge replacement; keeping old parts"
                            );
                            false
                        }
                    };
                    if !replacement_valid {
                        invalid_merge_dirs.insert(canonical_path(&part_dir));
                        continue;
                    }
                    valid_replacements.push(part_dir);
                }
                Err(e) => {
                    // A malformed marker is treated conservatively: skip the
                    // replacement and retain all ordinary parts.
                    invalid_merge_dirs.insert(canonical_path(&part_dir));
                    tracing::warn!(error = %e, ?part_dir, "discover: failed to read merge tombstone; keeping old parts");
                }
            }
        }
    }

    let mut pending_old_dirs = std::collections::VecDeque::new();
    for replacement_dir in &valid_replacements {
        let replacement_key = canonical_path(replacement_dir);
        if let Some(old_dirs) = tombstone_edges.get(&replacement_key) {
            pending_old_dirs.extend(old_dirs.iter().cloned());
        }
    }

    let mut cleanup_dirs = Vec::new();
    while let Some(old_dir) = pending_old_dirs.pop_front() {
        let old_key = canonical_path(&old_dir);
        if !tombstoned_dirs.insert(old_key.clone()) {
            continue;
        }
        cleanup_dirs.push(old_dir);
        if let Some(previous_generation) = tombstone_edges.get(&old_key) {
            pending_old_dirs.extend(previous_generation.iter().cloned());
        }
    }

    if let Err(error) = remove_part_dirs(&cleanup_dirs) {
        tracing::warn!(
            error = %error,
            "discover: tombstoned part cleanup incomplete"
        );
        // Keep all surviving markers. A later restart reconstructs the same
        // closure and retries any paths that could not be removed this time.
    } else {
        for replacement_dir in &valid_replacements {
            if replacement_dir.exists()
                && let Err(error) = remove_merge_tombstone(replacement_dir)
            {
                tracing::warn!(
                    %error,
                    ?replacement_dir,
                    "discover: failed to remove merge tombstone file"
                );
            }
        }
    }

    // Pass 2 loads only the surviving generation. Tombstoned paths remain
    // hidden even when their physical deletion failed.
    for partition_entry in &partition_entries {
        let partition_path = partition_entry.path();
        if !partition_path.is_dir() {
            continue;
        }
        let name = partition_entry.file_name();
        if name == ".tmp" {
            continue;
        }
        let part_entries = fs::read_dir(&partition_path)
            .map_err(|e| format!("failed to read partition {}: {e}", partition_path.display()))?;
        for part_entry in part_entries {
            let part_entry = part_entry.map_err(|e| {
                format!(
                    "failed to enumerate partition {}: {e}",
                    partition_path.display()
                )
            })?;
            let part_dir = part_entry.path();
            if !part_dir.is_dir() {
                continue;
            }
            if !part_dir.exists() {
                continue;
            }
            if !part_dir.join(META_FILE).exists() {
                return Err(format!("part is missing metadata: {}", part_dir.display()));
            }
            let part_key = canonical_path(&part_dir);
            if tombstoned_dirs.contains(&part_key) || invalid_merge_dirs.contains(&part_key) {
                continue;
            }
            let part = load_part(&part_dir)
                .map_err(|e| format!("failed to load part {}: {e}", part_dir.display()))?;
            parts.push(part);
        }
    }
    Ok(parts)
}

pub fn cleanup_tmp(parts_root: &Path) -> Result<(), String> {
    let root_metadata = fs::symlink_metadata(parts_root).map_err(|error| error.to_string())?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(format!(
            "refusing unsafe parts root {}",
            parts_root.display()
        ));
    }
    let canonical_root = fs::canonicalize(parts_root).map_err(|error| error.to_string())?;
    let tmp = parts_root.join(".tmp");
    let metadata = match fs::symlink_metadata(&tmp) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("refusing unsafe tmp directory {}", tmp.display()));
    }
    let canonical_tmp = fs::canonicalize(&tmp).map_err(|error| error.to_string())?;
    if !canonical_tmp.starts_with(&canonical_root) {
        return Err(format!(
            "tmp directory escapes parts root: {}",
            tmp.display()
        ));
    }
    fs::remove_dir_all(&tmp).map_err(|error| error.to_string())?;
    fsync_dir(parts_root).map_err(|error| error.to_string())
}

