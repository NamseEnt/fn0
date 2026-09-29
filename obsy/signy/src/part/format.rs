/// Where the flush build spends its 98%.
///
/// The rate ladder of 2026-08-13 put this engine's capacity ceiling in the
/// flush loop, and the loop's phase table then put 6.12 of a 6.46-second pass
/// inside this file — so what a customer can offer this server is decided by
/// the four spans below, and none of them had a number. They live in a
/// `static` because `flush_rows_internal` takes no metrics handle and is
/// called by both the flush and the merge; **only the flush path observes**,
/// discriminated by the `merge_old_dirs` argument that is already there, so a
/// merge rewrite cannot smear the distribution this exists to read.
///
/// Counted per part rather than per pass: a flush cuts its snapshot into
/// chunks, so these see ~3.4 observations for each `signy_flush_build_ms`.
pub static FLUSH_BUILD: FlushBuildMetrics = FlushBuildMetrics::new();

pub struct FlushBuildMetrics {
    /// Sorting and deduplicating one part's rows by `(tenant, ts)`.
    pub sort: crate::metrics::LatencyHistogram,
    /// Everything schema-shaped between the sort and the write: the metadata
    /// column census, and `parse_rows` — which runs the JSON parser over every
    /// line in the part.
    pub parse: crate::metrics::LatencyHistogram,
    /// `write_part_files` in total: Arrow build, Parquet encode with zstd, the
    /// trigram and metadata blooms, `index.bin`, `meta.json`, and the fsyncs
    /// that make them durable. Its own three are below.
    pub write: crate::metrics::LatencyHistogram,
    /// `write_parquet`: Arrow arrays, dictionary encoding, zstd, the fsync.
    pub parquet: crate::metrics::LatencyHistogram,
    /// `write_index`: the trigram and metadata blooms over every line, into
    /// `index.bin`.
    pub index: crate::metrics::LatencyHistogram,
    /// `write_meta`: the part's `meta.json`.
    pub meta: crate::metrics::LatencyHistogram,
    /// The commit tail — tombstone, directory rename, parent fsync.
    pub commit: crate::metrics::LatencyHistogram,
}

impl FlushBuildMetrics {
    const fn new() -> Self {
        Self {
            sort: crate::metrics::LatencyHistogram::new(),
            parse: crate::metrics::LatencyHistogram::new(),
            write: crate::metrics::LatencyHistogram::new(),
            parquet: crate::metrics::LatencyHistogram::new(),
            index: crate::metrics::LatencyHistogram::new(),
            meta: crate::metrics::LatencyHistogram::new(),
            commit: crate::metrics::LatencyHistogram::new(),
        }
    }
}

pub fn partition_of(ts_ns: i64) -> String {
    let secs = ts_ns.div_euclid(1_000_000_000);
    let dt = chrono::DateTime::from_timestamp(secs, 0).unwrap_or_default();
    dt.format("%Y-%m-%d").to_string()
}

fn gen_part_id(min_ts_ns: i64) -> String {
    let secs = min_ts_ns.div_euclid(1_000_000_000);
    let dt = chrono::DateTime::from_timestamp(secs, 0).unwrap_or_default();
    format!("{}-{}", dt.format("%Y%m%dT%H%M%S"), uuid::Uuid::new_v4())
}

pub fn rows_from_snapshot(snapshot: &MemTableSnapshot) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    for (tenant, entries) in snapshot {
        for e in entries {
            rows.push(Row::from_entry(tenant, e));
        }
    }
    sort_rows(&mut rows);
    rows
}

/// Flush a snapshot through the batch writer in chunks of at most roughly
/// `chunk_bytes` materialized, instead of copying the whole snapshot into one
/// `Vec<Row>` first.
///
/// The whole-snapshot copy was the largest flush transient:
/// `docs/MEMORY_ATTRIBUTION.md` measured the flush arena at ~3.3x the memtable
/// it was writing, because the copy, the per-partition parsed-field maps and
/// the part-wide index buffers were all alive at once. Every one of those is
/// sized by its input, so bounding the input bounds them all.
///
/// Order and dedup are [`sort_rows`]'s, reproduced without a global sort:
/// tenants are visited in name order — the prefix of [`Row::sort_key`] — and
/// each tenant's entries are emitted in `(timestamp, line, metadata)` order,
/// the suffix. The snapshot is shared with concurrent queries, so entries are
/// ordered through a side index rather than sorted in place. A duplicate pair
/// always shares a tenant, so skipping an entry equal to the tenant's
/// previously emitted one is the same dedup even when a chunk cut falls
/// between the two.
///
/// A chunk that fails rolls back every part this call already committed, so
/// the caller sees the all-or-nothing flush it always had.
pub fn flush_snapshot_chunked(
    snapshot: &MemTableSnapshot,
    parts_root: &Path,
    row_group_size: usize,
    chunk_bytes: u64,
) -> io::Result<Vec<Part>> {
    let mut tenants: Vec<(&TenantId, &Vec<LogEntry>)> = snapshot
        .iter()
        .filter(|(_, entries)| !entries.is_empty())
        .collect();
    tenants.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

    let mut parts: Vec<Part> = Vec::new();
    let mut chunk: Vec<Row> = Vec::new();
    let mut chunk_used: u64 = 0;

    fn cut(
        chunk: &mut Vec<Row>,
        parts: &mut Vec<Part>,
        parts_root: &Path,
        row_group_size: usize,
    ) -> io::Result<()> {
        let rows = std::mem::take(chunk);
        match flush_rows(rows, parts_root, row_group_size) {
            Ok(new_parts) => {
                parts.extend(new_parts);
                Ok(())
            }
            Err(error) => {
                let committed: Vec<PathBuf> = parts.iter().map(|part| part.dir.clone()).collect();
                rollback_committed(&committed);
                Err(error)
            }
        }
    }

    for (tenant, entries) in tenants {
        let mut order: Vec<u32> = (0..entries.len() as u32).collect();
        order.sort_by(|&a, &b| {
            let ea = &entries[a as usize];
            let eb = &entries[b as usize];
            (ea.timestamp_ns, &ea.line, &ea.structured_metadata).cmp(&(
                eb.timestamp_ns,
                &eb.line,
                &eb.structured_metadata,
            ))
        });
        let mut prev: Option<u32> = None;
        for &i in &order {
            let entry = &entries[i as usize];
            if let Some(p) = prev {
                let previous = &entries[p as usize];
                if previous.timestamp_ns == entry.timestamp_ns
                    && previous.line == entry.line
                    && previous.structured_metadata == entry.structured_metadata
                {
                    continue;
                }
            }
            prev = Some(i);
            let row = Row::from_entry(tenant, entry);
            chunk_used = chunk_used.saturating_add(row.materialized_bytes());
            chunk.push(row);
            if chunk_used >= chunk_bytes {
                cut(&mut chunk, &mut parts, parts_root, row_group_size)?;
                chunk_used = 0;
            }
        }
    }
    if !chunk.is_empty() {
        cut(&mut chunk, &mut parts, parts_root, row_group_size)?;
    }
    Ok(parts)
}

/// Sorts into layout order and drops entries that are copies of one another.
///
/// Delivery is at-least-once. A push that was durably written but whose
/// response never reached the client is retried, and a crash between a flush
/// and its checkpoint replays a WAL suffix that is already in a part — both
/// produce a second copy of an entry that is identical in every field. Loki
/// resolves this the same way: one timestamp and one identical line is one
/// entry.
///
/// What is given up is two genuinely distinct entries that share a tenant, a
/// nanosecond, a line and its metadata. At nanosecond resolution that is a
/// collision of things already indistinguishable to a reader.
///
/// Every part is written through here — flush, merge and the retention rewrite
/// — so a duplicate that a flush could not see (its twin is in an older part)
/// is removed the first time the two are merged.
fn sort_rows(rows: &mut Vec<Row>) {
    rows.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
    rows.dedup_by(|left, right| left.sort_key() == right.sort_key());
}

pub fn flush_rows(
    rows: Vec<Row>,
    parts_root: &Path,
    row_group_size: usize,
) -> io::Result<Vec<Part>> {
    flush_rows_internal(rows, parts_root, row_group_size, None)
}

/// Flush rows while carrying a merge tombstone into every committed part.
///
/// The tombstone is written and fsynced inside the temporary part directory
/// before that directory is renamed into the visible partition directory.
/// This makes a visible merged part self-describing even if the process dies
/// immediately after the rename.
pub fn flush_rows_with_merge_tombstone(
    rows: Vec<Row>,
    parts_root: &Path,
    row_group_size: usize,
    old_dirs: &[PathBuf],
) -> io::Result<Vec<Part>> {
    flush_rows_internal(rows, parts_root, row_group_size, Some(old_dirs))
}

/// Write one part from a sorted, deduplicated row stream.
///
/// The merge counterpart of [`flush_rows_with_merge_tombstone`], and the reason
/// [`StreamingPartWriter`] exists: the batch form holds the whole group as a
/// `Vec<Row>` first, which `docs/MEMORY_ATTRIBUTION.md` measured at 829 of 847
/// live megabytes at the settle peak.
///
/// `metadata_keys`/`parsed_keys` and `partition` come from the inputs'
/// `meta.json`, not from the rows, because the schema names those columns and
/// has to exist before the first row group.
///
/// An input set that turns out to be empty leaves no part and no tombstone,
/// the same as flushing an empty `Vec`.
#[allow(clippy::too_many_arguments)]
pub fn flush_row_stream_with_merge_tombstone(
    rows: &mut MergedRows,
    keep: &mut dyn FnMut(&Row) -> bool,
    parts_root: &Path,
    partition: &str,
    metadata_keys: Vec<String>,
    parsed_keys: Vec<String>,
    row_group_size: usize,
    old_dirs: &[PathBuf],
) -> io::Result<(Vec<Part>, u64, u64)> {
    let tmp_root = parts_root.join(".tmp");
    fs::create_dir_all(&tmp_root)?;
    let staging = tmp_root.join(format!("merge-{}", uuid::Uuid::new_v4()));
    if staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    fs::create_dir_all(&staging)?;

    let mut writer =
        StreamingPartWriter::create(&staging, metadata_keys, parsed_keys, row_group_size)?;
    let mut dropped = 0u64;
    let mut kept = 0u64;
    loop {
        let row = match rows.next_row() {
            Ok(Some(row)) => row,
            Ok(None) => break,
            Err(error) => {
                let _ = fs::remove_dir_all(&staging);
                return Err(io::Error::other(error));
            }
        };
        if !keep(&row) {
            dropped += 1;
            continue;
        }
        kept += 1;
        if let Err(error) = writer.push(row) {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    }
    if writer.is_empty() {
        let _ = fs::remove_dir_all(&staging);
        return Ok((Vec::new(), dropped, kept));
    }

    let part_id = gen_part_id(writer.min_ts_ns());
    if let Err(error) = writer.finish(&staging, &part_id, partition) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    if let Err(error) = write_merge_tombstone(&staging, parts_root, old_dirs) {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    let final_dir = parts_root.join(partition).join(&part_id);
    if let Some(parent) = final_dir.parent() {
        fs::create_dir_all(parent)?;
    }
    if final_dir.exists() {
        let _ = fs::remove_dir_all(&staging);
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("part dir already exists: {}", final_dir.display()),
        ));
    }
    fs::rename(&staging, &final_dir)?;
    if let Some(parent) = final_dir.parent() {
        fsync_dir(parent)?;
    }
    fsync_dir(parts_root)?;
    let part = load_part(&final_dir).map_err(|error| {
        rollback_committed(std::slice::from_ref(&final_dir));
        io::Error::other(error)
    })?;
    Ok((vec![part], dropped, kept))
}

/// Whether this call's phases enter [`FLUSH_BUILD`].
///
/// Only the flush path does. A merge rewrite runs the identical code with a
/// tombstone, and letting it in would blend two workloads into the one
/// distribution that exists to explain the flush ceiling — with nothing in the
/// numbers to say it had happened. The discriminator is the argument that was
/// already here, named so the meaning is testable without a global counter that
/// every other test's flush also moves.
fn measures_build(merge_old_dirs: Option<&[PathBuf]>) -> bool {
    merge_old_dirs.is_none()
}

fn flush_rows_internal(
    rows: Vec<Row>,
    parts_root: &Path,
    row_group_size: usize,
    merge_old_dirs: Option<&[PathBuf]>,
) -> io::Result<Vec<Part>> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let tmp_root = parts_root.join(".tmp");
    fs::create_dir_all(&tmp_root)?;

    let mut by_partition: BTreeMap<String, Vec<Row>> = BTreeMap::new();
    for row in rows {
        let p = partition_of(row.timestamp_ns);
        by_partition.entry(p).or_default().push(row);
    }

    let measured = measures_build(merge_old_dirs);
    let mut parts = Vec::new();
    let mut committed_dirs: Vec<PathBuf> = Vec::new();
    for (partition, mut part_rows) in by_partition {
        let sort_started = std::time::Instant::now();
        sort_rows(&mut part_rows);
        if measured {
            FLUSH_BUILD.sort.observe(sort_started.elapsed());
        }
        let part_id = gen_part_id(
            part_rows
                .iter()
                .map(|row| row.timestamp_ns)
                .min()
                .unwrap_or_default(),
        );

        let tmp_dir = tmp_root.join(&part_id);
        if tmp_dir.exists() {
            let _ = fs::remove_dir_all(&tmp_dir);
        }
        if let Err(e) = fs::create_dir_all(&tmp_dir) {
            rollback_committed(&committed_dirs);
            return Err(e);
        }

        let parse_started = std::time::Instant::now();
        let metadata_columns = select_metadata_columns(metadata_column_counts(&part_rows));
        let parsed_rows = parse_rows(&part_rows);
        let parsed_columns = select_metadata_columns(parsed_column_counts(&parsed_rows));
        if measured {
            FLUSH_BUILD.parse.observe(parse_started.elapsed());
        }
        let write_started = std::time::Instant::now();
        if let Err(e) = write_part_files(
            &tmp_dir,
            &part_id,
            &partition,
            &part_rows,
            &parsed_rows,
            &metadata_columns,
            &parsed_columns,
            row_group_size,
            measured,
        ) {
            rollback_committed(&committed_dirs);
            return Err(e);
        }

        if measured {
            FLUSH_BUILD.write.observe(write_started.elapsed());
        }
        let commit_started = std::time::Instant::now();
        if let Some(old_dirs) = merge_old_dirs
            && let Err(e) = write_merge_tombstone(&tmp_dir, parts_root, old_dirs)
        {
            let _ = fs::remove_dir_all(&tmp_dir);
            rollback_committed(&committed_dirs);
            return Err(e);
        }

        let final_dir = parts_root.join(&partition).join(&part_id);
        if let Some(parent) = final_dir.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            rollback_committed(&committed_dirs);
            return Err(e);
        }
        if final_dir.exists() {
            let _ = fs::remove_dir_all(&tmp_dir);
            rollback_committed(&committed_dirs);
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("part dir already exists: {}", final_dir.display()),
            ));
        }
        if let Err(e) = fs::rename(&tmp_dir, &final_dir) {
            rollback_committed(&committed_dirs);
            return Err(e);
        }
        // Fsync the parent (partition) directory and parts_root to make the rename durable.
        if let Some(parent) = final_dir.parent()
            && let Err(e) = fsync_dir(parent)
        {
            rollback_committed(&committed_dirs);
            return Err(e);
        }
        if let Err(e) = fsync_dir(parts_root) {
            rollback_committed(&committed_dirs);
            return Err(e);
        }
        committed_dirs.push(final_dir.clone());
        if measured {
            FLUSH_BUILD.commit.observe(commit_started.elapsed());
        }

        let part = match load_part(&final_dir) {
            Ok(p) => p,
            Err(e) => {
                rollback_committed(&committed_dirs);
                return Err(io::Error::other(e));
            }
        };
        parts.push(part);
    }
    Ok(parts)
}

fn rollback_committed(committed_dirs: &[PathBuf]) {
    for dir in committed_dirs.iter().rev() {
        if dir.exists()
            && let Err(e) = fs::remove_dir_all(dir)
        {
            tracing::warn!(error = %e, ?dir, "rollback: failed to remove committed part dir");
        }
    }
}

/// How many rows of the part carry each metadata key.
fn metadata_column_counts(rows: &[Row]) -> BTreeMap<String, u64> {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for row in rows {
        for (name, _) in &row.structured_metadata {
            *counts.entry(name.clone()).or_default() += 1;
        }
    }
    counts
}

/// One `| json` extraction per row, shared by the column-key selection, the
/// column fill and nothing else — the parse was the whole added write cost of
/// the `_pf:` columns, and running it once instead of twice is half of it
/// back.
pub fn parse_rows(rows: &[Row]) -> Vec<Option<BTreeMap<String, String>>> {
    rows.iter()
        .map(|row| crate::logql::parsed_json_fields(&row.line))
        .collect()
}

/// How many rows' lines `| json` would extract each field from.
fn parsed_column_counts(parsed: &[Option<BTreeMap<String, String>>]) -> BTreeMap<String, u64> {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for fields in parsed.iter().flatten() {
        for name in fields.keys() {
            *counts.entry(name.clone()).or_default() += 1;
        }
    }
    counts
}

/// Which metadata keys become columns in this part: the `MAX_METADATA_COLUMNS`
/// most frequent, returned **sorted by key** with their row counts.
///
/// Frequency rather than first-N-alphabetical, so an adversarial tenant
/// churning key names cannot push `trace_id` out of a column; ties break on
/// the key so the choice is deterministic. The counts travel into `meta.json`
/// because a merge must choose its output's columns *before* reading a row —
/// and summing recorded counts is what keeps that choice deterministic too.
/// Keys past the cap stay in the residual blob column; the invariant the read
/// path relies on is that a columnized key never also appears in the residual.
pub(crate) fn select_metadata_columns(counts: BTreeMap<String, u64>) -> Vec<(String, u64)> {
    let mut by_count: Vec<(String, u64)> = counts.into_iter().collect();
    by_count.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    by_count.truncate(MAX_METADATA_COLUMNS);
    by_count.sort_by(|a, b| a.0.cmp(&b.0));
    by_count
}

#[allow(clippy::too_many_arguments)]
fn write_part_files(
    dir: &Path,
    id: &str,
    partition: &str,
    rows: &[Row],
    parsed_rows: &[Option<BTreeMap<String, String>>],
    metadata_columns: &[(String, u64)],
    parsed_columns: &[(String, u64)],
    row_group_size: usize,
    measured: bool,
) -> io::Result<()> {
    let parquet_started = std::time::Instant::now();
    let row_group_ranges = write_parquet(
        &dir.join(DATA_FILE),
        rows,
        parsed_rows,
        metadata_columns,
        parsed_columns,
        row_group_size,
    )?;
    if measured {
        FLUSH_BUILD.parquet.observe(parquet_started.elapsed());
    }
    let index_started = std::time::Instant::now();
    write_index(&dir.join(INDEX_FILE), rows, parsed_rows, row_group_size)?;
    if measured {
        FLUSH_BUILD.index.observe(index_started.elapsed());
    }
    let meta_started = std::time::Instant::now();
    let result = write_meta(
        &dir.join(META_FILE),
        id,
        partition,
        rows,
        row_group_size,
        metadata_columns,
        parsed_columns,
        &row_group_ranges,
    );
    if measured {
        FLUSH_BUILD.meta.observe(meta_started.elapsed());
    }
    result
}

/// Row-group boundaries for a `(tenant, timestamp)`-sorted row set. A row
/// group never spans two tenants.
fn row_group_bounds(rows: &[Row], row_group_size: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut segment_start = 0usize;
    while segment_start < rows.len() {
        let mut segment_end = segment_start;
        while segment_end < rows.len() && rows[segment_end].tenant == rows[segment_start].tenant {
            segment_end += 1;
        }
        let mut start = segment_start;
        while start < segment_end {
            let end = (start + row_group_size).min(segment_end);
            out.push((start, end));
            start = end;
        }
        segment_start = segment_end;
    }
    out
}

/// The Parquet column a metadata key is stored in.
///
/// The `_sm:` prefix keeps the metadata namespace disjoint from the reserved
/// columns that precede it in the schema, so the mapping is injective in both
/// directions.
fn metadata_column_name(key: &str) -> String {
    format!("_sm:{key}")
}

/// The Parquet column a `| json`-extracted field is stored in. Same
/// injectivity argument as [`metadata_column_name`].
fn parsed_column_name(key: &str) -> String {
    format!("_pf:{key}")
}

fn part_schema(metadata_keys: &[String], parsed_keys: &[String]) -> Arc<Schema> {
    let mut fields = vec![
        Field::new(TENANT_COLUMN, DataType::Utf8, false),
        Field::new("timestamp_ns", DataType::Int64, false),
        Field::new("_msg", DataType::Utf8, false),
    ];
    for key in metadata_keys {
        fields.push(Field::new(metadata_column_name(key), DataType::Utf8, true));
    }
    for key in parsed_keys {
        fields.push(Field::new(parsed_column_name(key), DataType::Utf8, true));
    }
    fields.push(Field::new("structured_metadata", DataType::Utf8, true));
    Arc::new(Schema::new(fields))
}

fn row_group_batch(
    schema: &Arc<Schema>,
    rows: &[Row],
    parsed_rows: &[Option<BTreeMap<String, String>>],
    metadata_keys: &[String],
    parsed_keys: &[String],
) -> io::Result<RecordBatch> {
    let tenants: Vec<&str> = rows.iter().map(|r| r.tenant.as_str()).collect();
    let ts: Vec<i64> = rows.iter().map(|r| r.timestamp_ns).collect();
    let msg: Vec<&str> = rows.iter().map(|r| r.line.as_str()).collect();
    // The residual: only the pairs whose key did not make a column. Most rows
    // have none, so the common write path runs no serde at all — and the read
    // path depends on the split being exact: a columnized key must never also
    // appear in a row's residual.
    let sm: Vec<Option<String>> = rows
        .iter()
        .map(|r| {
            debug_assert!(
                r.structured_metadata
                    .windows(2)
                    .all(|pair| pair[0].0 < pair[1].0),
                "structured metadata must be canonical before it reaches a part"
            );
            let residual: Vec<&(String, String)> = r
                .structured_metadata
                .iter()
                .filter(|(name, _)| metadata_keys.binary_search(name).is_err())
                .collect();
            if residual.is_empty() {
                None
            } else {
                serde_json::to_string(&residual).ok()
            }
        })
        .collect();

    let mut columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(tenants)),
        Arc::new(Int64Array::from(ts)),
        Arc::new(StringArray::from(msg)),
    ];
    for key in metadata_keys {
        let vals: Vec<Option<&str>> = rows
            .iter()
            .map(|r| {
                r.structured_metadata
                    .binary_search_by(|(name, _)| name.as_str().cmp(key))
                    .ok()
                    .map(|index| r.structured_metadata[index].1.as_str())
            })
            .collect();
        columns.push(Arc::new(StringArray::from(vals)));
    }
    for key in parsed_keys {
        let vals: Vec<Option<&str>> = parsed_rows
            .iter()
            .map(|fields| {
                fields
                    .as_ref()
                    .and_then(|fields| fields.get(key))
                    .map(String::as_str)
            })
            .collect();
        columns.push(Arc::new(StringArray::from(vals)));
    }
    columns.push(Arc::new(StringArray::from(sm)));

    RecordBatch::try_new(schema.clone(), columns).map_err(io::Error::other)
}

/// The one place both writers get their Parquet properties, so the streaming
/// and the batch path cannot drift into different files.
///
/// Statistics are per column on purpose: `timestamp_ns` keeps page-level
/// min/max because time is the axis the page index can actually prune — rows
/// are piecewise time-ordered inside a group. Every string column stays at
/// chunk level: a page's min/max over random hex like `trace_id` spans the
/// alphabet and prunes nothing, and the page index would carry (and load)
/// those dead bounds for every page of every wide column.
fn part_writer_properties(row_group_size: usize) -> WriterProperties {
    WriterProperties::builder()
        .set_max_row_group_row_count(Some(row_group_size))
        .set_compression(Compression::ZSTD(ZstdLevel::default()))
        .set_statistics_enabled(EnabledStatistics::Chunk)
        .set_column_statistics_enabled(ColumnPath::from("timestamp_ns"), EnabledStatistics::Page)
        // Without a row bound a whole 8192-row group's timestamp chunk fits
        // one default-sized page, and one page per chunk makes the page index
        // exactly as coarse as the group bounds it exists to refine — measured
        // as a two-second window costing what the whole range costs. At 1024
        // rows a page, a stream's run inside a group carries usefully tight
        // per-page time bounds; the cost is more pages everywhere, which the
        // write bench and the disk axis price.
        .set_data_page_row_count_limit(crate::part::BLOOM_WINDOW_ROWS)
        .build()
}

#[allow(clippy::too_many_arguments)]
/// Where each row group landed in the body, off the metadata the writer just
/// wrote.
///
/// Shared by the batch and the streaming writer so the two cannot disagree
/// about what a row group's extent is.
fn row_group_byte_ranges(metadata: &parquet::file::metadata::ParquetMetaData) -> Vec<ByteRange> {
    metadata
        .row_groups()
        .iter()
        .map(|row_group| {
            // A row group's column chunks are contiguous, but the order they
            // are reported in is the schema's, not the file's, so the extent is
            // taken as the span rather than as first-to-last.
            let mut start = u64::MAX;
            let mut end = 0u64;
            for column in row_group.columns() {
                let (offset, length) = column.byte_range();
                start = start.min(offset);
                end = end.max(offset.saturating_add(length));
            }
            ByteRange { start, end }
        })
        .collect()
}

fn write_parquet(
    path: &Path,
    rows: &[Row],
    parsed_rows: &[Option<BTreeMap<String, String>>],
    metadata_columns: &[(String, u64)],
    parsed_columns: &[(String, u64)],
    row_group_size: usize,
) -> io::Result<Vec<ByteRange>> {
    let metadata_keys: Vec<String> = metadata_columns
        .iter()
        .map(|(key, _)| key.clone())
        .collect();
    let parsed_keys: Vec<String> = parsed_columns.iter().map(|(key, _)| key.clone()).collect();
    let schema = part_schema(&metadata_keys, &parsed_keys);
    let bounds = row_group_bounds(rows, row_group_size);

    let file = fs::File::create(path)?;
    let props = part_writer_properties(row_group_size);
    let mut writer =
        ArrowWriter::try_new(file, schema.clone(), Some(props)).map_err(io::Error::other)?;
    // The sidecars address row groups by ordinal, so the Parquet row groups
    // must be exactly `bounds`. Flushing after each batch pins the boundary
    // instead of letting the writer choose one that straddles a tenant.
    for (start, end) in &bounds {
        let batch = row_group_batch(
            &schema,
            &rows[*start..*end],
            &parsed_rows[*start..*end],
            &metadata_keys,
            &parsed_keys,
        )?;
        writer.write(&batch).map_err(io::Error::other)?;
        writer.flush().map_err(io::Error::other)?;
    }
    let metadata = writer.close().map_err(io::Error::other)?;
    sync_file(path)?;
    crate::page_cache::drop_cache(path);

    let ranges = row_group_byte_ranges(&metadata);
    if ranges.len() != bounds.len() {
        return Err(io::Error::other(format!(
            "parquet wrote {} row groups for {} planned boundaries",
            ranges.len(),
            bounds.len()
        )));
    }
    Ok(ranges)
}

fn encode_blooms(
    rows: &[Row],
    parsed_rows: &[Option<BTreeMap<String, String>>],
    row_group_size: usize,
) -> io::Result<Vec<u8>> {
    let bounds = row_group_bounds(rows, row_group_size);
    let mut buf = Vec::new();
    buf.extend_from_slice(BLOOM_MAGIC);
    buf.extend_from_slice(&(bounds.len() as u32).to_le_bytes());
    for (start, end) in &bounds {
        buf.extend_from_slice(&encode_group_blooms(
            &rows[*start..*end],
            &parsed_rows[*start..*end],
        )?);
    }
    Ok(buf)
}

/// One row group's filters, length-prefixed, exactly as they sit in the
/// `BTF5` section: the group's line/trigram bloom, then one exact-field
/// sub-bloom per [`BLOOM_WINDOW_ROWS`]-row window.
///
/// Windows are what turned an admitted group from an 8192-row decode into a
/// ~1024-row one: `docs/COMPARISON.md`'s `metadata_rare` decomposed to
/// ~0.28 ms of fixed cost plus ~1.7 ms per admitted group, all of it the
/// narrow pass over rows the per-group filter could not exclude. The bits
/// are linear in the token count, so splitting one filter into eight costs
/// only headers — the pruning granularity is close to free.
///
/// A group with no exact token anywhere writes a window count of zero — the
/// absent-section semantics the reader prunes on — and a token-less window
/// inside a token-bearing group writes a zero length, which prunes that
/// window the same way.
///
/// Split out of [`encode_blooms`] so a writer that never holds the whole part
/// can produce the same bytes a row group at a time. The batch path above still
/// calls it, so the existing part-format tests are what prove the two agree.
///
/// Public for `benches/bloom.rs` alone, and it has to be: this function is
/// **63% of the flush pass** at the ceiling, and the bench that claimed to
/// measure it held its own copy of the trigram loop instead — so the shipped
/// code could change without the bench moving, which is a regression gate
/// guarding nothing. A bench measures the function or it measures a fiction.
pub fn encode_group_blooms(
    rows: &[Row],
    parsed_rows: &[Option<BTreeMap<String, String>>],
) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut unique_trigrams = crate::bloom::TrigramSet::new();
    // One pass. Sizing the filters needs the token counts and filling them
    // needs the tokens, and this used to be two passes that each ran the JSON
    // and logfmt parsers over every line — the whole parse, twice, for a
    // count. The tokens are collected once instead, into their windows; the
    // scratch lives for one row group.
    let window_count = rows.len().div_ceil(crate::part::BLOOM_WINDOW_ROWS);
    let mut window_tokens: Vec<Vec<Vec<u8>>> = vec![Vec::new(); window_count];
    for (index, (row, parsed)) in rows.iter().zip(parsed_rows).enumerate() {
        let window = &mut window_tokens[index / crate::part::BLOOM_WINDOW_ROWS];
        unique_trigrams.add_line(&row.line);
        for (name, value) in &row.structured_metadata {
            for value in crate::logql::canonical_index_values(value) {
                window.push(encode_exact_field_token(name, &value)?);
            }
        }
        // The `| json` half of the parser-visible fields comes off the parse
        // the `_pf:` columns already paid for — the bloom used to run the same
        // `extract_json` a second time per row behind a first-byte gate, and
        // the gate also silently kept top-level-array extractions out of the
        // filter, which a bloom prune would then have false-negatived on.
        if let Some(parsed) = parsed {
            for (name, value) in parsed {
                for value in crate::logql::canonical_index_values(value) {
                    window.push(encode_exact_field_token(name, &value)?);
                }
            }
        }
        for (name, values) in crate::logql::indexed_logfmt_fields(&row.line) {
            for value in values {
                for value in crate::logql::canonical_index_values(&value) {
                    window.push(encode_exact_field_token(&name, &value)?);
                }
            }
        }
    }
    let bytes = unique_trigrams.finish().encode();
    buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    buf.extend_from_slice(&bytes);
    // No token to index means no filters at all — a zero window count. A
    // filter built for zero items still costs the `optimal_bits` floor, and
    // an all-zero filter and an absent one prune identically.
    if window_tokens.iter().all(Vec::is_empty) {
        buf.extend_from_slice(&0u32.to_le_bytes());
        return Ok(buf);
    }
    buf.extend_from_slice(&(window_count as u32).to_le_bytes());
    for tokens in &window_tokens {
        if tokens.is_empty() {
            buf.extend_from_slice(&0u32.to_le_bytes());
            continue;
        }
        // A filter is sized by its token count, and the token count is
        // attacker-shaped: a wide-JSON row contributes a token per field per
        // canonical variant, so an adversarial tenant could make `index.bin`
        // outweigh `data.parquet`. Past the cap the window is written as
        // *saturated* — admit everything, prune nothing — which is the
        // conservative direction; absence (the zero length above) keeps
        // meaning the opposite.
        if tokens.len() > crate::part::MAX_EXACT_TOKENS_PER_WINDOW {
            buf.extend_from_slice(&crate::part::SATURATED_WINDOW_SENTINEL.to_le_bytes());
            continue;
        }
        let mut filter =
            BloomFilter::with_capacity(tokens.len(), crate::part::EXACT_FIELD_WINDOW_FPP);
        for token in tokens {
            filter.insert(token);
        }
        let bytes = filter.encode();
        buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(&bytes);
    }
    Ok(buf)
}
