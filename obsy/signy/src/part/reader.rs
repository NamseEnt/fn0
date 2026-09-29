#[derive(Clone)]
pub struct PreadReader {
    file: Arc<fs::File>,
    len: u64,
}

impl PreadReader {
    pub fn new(file: fs::File) -> io::Result<Self> {
        let len = file.metadata()?.len();
        Ok(Self {
            file: Arc::new(file),
            len,
        })
    }
}

impl Length for PreadReader {
    fn len(&self) -> u64 {
        self.len
    }
}

impl ChunkReader for PreadReader {
    type T = PreadCursor;

    fn get_read(&self, start: u64) -> ParquetResult<Self::T> {
        Ok(PreadCursor {
            file: self.file.clone(),
            pos: start,
            len: self.len,
        })
    }

    fn get_bytes(&self, start: u64, length: usize) -> ParquetResult<Bytes> {
        let mut buf = vec![0u8; length];
        self.file.read_exact_at(&mut buf, start).map_err(|e| {
            parquet::errors::ParquetError::from(std::io::Error::new(e.kind(), e.to_string()))
        })?;
        Ok(Bytes::from(buf))
    }
}

pub struct PreadCursor {
    file: Arc<fs::File>,
    pos: u64,
    len: u64,
}

impl Read for PreadCursor {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.len.saturating_sub(self.pos) as usize;
        if remaining == 0 {
            return Ok(0);
        }
        let to_read = buf.len().min(remaining);
        let n = self.file.read_at(&mut buf[..to_read], self.pos)?;
        self.pos += n as u64;
        Ok(n)
    }
}

pub struct PartReader {
    part: Part,
    /// The line and exact-field window blooms, resident while the global
    /// bloom-cache budget admits them and re-decoded from `index.bin` when a
    /// pruning query needs them back. The 24-hour soak measured the
    /// always-resident form at ~2 MiB per part, growing with the retention
    /// window; see the bloom cache (`bloom_cache.rs`).
    blooms: Arc<BloomSlot>,
    /// The keys stored as `_sm:` columns, in schema (sorted) order.
    metadata_keys: Vec<String>,
    /// The `| json`-extracted fields stored as `_pf:` columns, sorted.
    parsed_keys: Vec<String>,
    /// Whole row groups decoded by earlier scans, kept under the process-wide
    /// budget; dies with this reader.
    group_cache: GroupCache,
    /// The projection every cached batch was decoded with — `ColumnSet::all`
    /// over this part's keys — built once so cache hits from narrower callers
    /// read the right column positions.
    cache_projection: ScanProjection,
    /// The parsed Parquet footer plus the timestamp column's page index,
    /// cached for the reader's lifetime after the first scan loads it.
    ///
    /// The *parse* is cached, never the file handle: a part's `data.parquet`
    /// is evictable, and a held descriptor would keep the inode's bytes alive
    /// past the eviction that exists to reclaim them. The metadata is safe to
    /// keep because a part is immutable and a restore fetches the same
    /// object — and it is what the rare shapes' latency floor turned out to
    /// be: a matcherless query re-read and re-parsed every admitted part's
    /// footer and page index on every scan.
    data_metadata: std::sync::OnceLock<ArrowReaderMetadata>,
}

/// One window's exact-field state, decoded.
pub(crate) enum WindowBloom {
    /// The window indexed no token: no exact-field predicate can match here.
    Absent,
    /// The window held more tokens than a filter is allowed to be sized for;
    /// everything is admitted, nothing is pruned.
    Saturated,
    /// Where the window's filter sits in the mapping, and the two header
    /// words a lookup needs. The bits stay in the file.
    Filter(BloomLocation),
}

/// A stored filter's position in `index.bin`, and its header.
///
/// Sixteen bytes instead of the bitset they point at. `num_bits` and `k` are
/// kept rather than re-read on every lookup because they are two words and
/// the alternative is a bounds check and three loads per probe.
#[derive(Clone, Copy)]
pub(crate) struct BloomLocation {
    offset: u32,
    len: u32,
    num_bits: u32,
    k: u32,
}

/// The evictable half of a part's sidecar: what the bloom cache
/// (`bloom_cache.rs`) holds under its budget.
///
/// It used to be the decoded filters themselves — every bitset copied out of
/// `index.bin` into an owned `Vec`. Decoding barely shrinks a bloom, so that
/// copy was very nearly the file: the soak measured 4418 KiB of sidecar
/// becoming 4120 KiB of cache, a factor of 1.01. A 122 MiB budget therefore
/// held about thirty parts against a working set of sixty to two hundred, and
/// the cache thrashed — mean residency 2.03 s, mean time to the same part
/// being decoded again 0.144 s, two thirds of installs a part coming back.
/// Each of those re-decodes allocated four megabytes and freed it seconds
/// later, at two hundred a second.
///
/// So this holds the mapping and the offsets into it. The bits are read
/// through [`BloomIndex::bloom`] at the moment of a lookup and never copied.
/// The mapping costs address space rather than resident memory — its pages
/// fault in on use and the kernel can reclaim them, unlike the anonymous
/// bytes a decode produced — and what the cache now bounds is an index whose
/// size is a function of the row-group and window count, not of the data.
pub(crate) struct BloomIndex {
    map: Arc<memmap2::Mmap>,
    /// One per row group: the line bloom, then one entry per
    /// [`BLOOM_WINDOW_ROWS`]-row window. An empty window vector means the
    /// group indexed no exact-field token at all.
    line: Vec<BloomLocation>,
    exact_fields: Vec<Vec<WindowBloom>>,
}

impl BloomIndex {
    fn bloom(&self, at: BloomLocation) -> crate::bloom::BloomRef<'_> {
        let start = at.offset as usize;
        let end = start + at.len as usize;
        // Bounds were checked once, against this same mapping, when the index
        // was built; a part is write-once, so they cannot have moved.
        crate::bloom::BloomRef::from_parts(
            &self.map[start..end],
            at.num_bits as usize,
            at.k,
        )
    }

    /// What the cache charges for this entry: the index, not the blooms.
    fn index_bytes(&self) -> u64 {
        let windows: usize = self.exact_fields.iter().map(Vec::len).sum();
        (std::mem::size_of::<Self>()
            + self.line.len() * std::mem::size_of::<BloomLocation>()
            + self.exact_fields.len() * std::mem::size_of::<Vec<WindowBloom>>()
            + windows * std::mem::size_of::<WindowBloom>()) as u64
    }
}


fn validate_sidecar_files(part: &Part) -> Result<(), String> {
    let expected = part.meta.integrity.index_crc32;
    let actual = file_crc32(&part.index_path()).map_err(|error| {
        format!(
            "failed to checksum {INDEX_FILE} for part {}: {error}",
            part.meta.id
        )
    })?;
    if actual != expected {
        return Err(format!(
            "{INDEX_FILE} checksum mismatch for part {}: expected {expected}, got {actual}",
            part.meta.id
        ));
    }
    Ok(())
}

/// The rows of one group whose *pages* can hold a row in the window, from the
/// timestamp column's page index — or `None` when the file carries no page
/// index to ask, which selects everything.
fn time_page_selection(
    part_metadata: &ArrowReaderMetadata,
    rgu: usize,
    time_range: QueryTimeRange,
) -> Option<RowSelection> {
    let column_index = part_metadata.metadata().column_index()?.get(rgu)?;
    let offset_index = part_metadata.metadata().offset_index()?.get(rgu)?;
    // `timestamp_ns` is column 1 in every part schema.
    let parquet::file::page_index::column_index::ColumnIndexMetaData::INT64(ts_index) =
        column_index.get(1)?
    else {
        return None;
    };
    let pages = offset_index.get(1)?.page_locations();
    let group_rows = part_metadata.metadata().row_group(rgu).num_rows().max(0) as usize;
    let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
    for (page, location) in pages.iter().enumerate() {
        let start = location.first_row_index.max(0) as usize;
        let end = pages
            .get(page + 1)
            .map(|next| next.first_row_index.max(0) as usize)
            .unwrap_or(group_rows);
        let keep = match (ts_index.min_value(page), ts_index.max_value(page)) {
            (Some(min), Some(max)) => time_range.overlaps(*min, *max),
            // A page without recorded bounds cannot be excluded.
            _ => true,
        };
        if keep {
            match ranges.last_mut() {
                Some(range) if range.end == start => range.end = end,
                _ => ranges.push(start..end),
            }
        }
    }
    Some(RowSelection::from_consecutive_ranges(
        ranges.into_iter(),
        group_rows,
    ))
}

/// Zero-copy slices of a cached group covering the selection's kept ranges,
/// in group order. No selection means the batches themselves.
fn slice_cached_group(
    cached: &CachedGroupRead,
    selection: Option<&RowSelection>,
) -> Vec<RecordBatch> {
    let Some(selection) = selection else {
        return cached.batches.iter().cloned().collect();
    };
    let mut ranges = Vec::new();
    let mut at = 0usize;
    for selector in selection.iter() {
        if !selector.skip && selector.row_count > 0 {
            ranges.push(at..at + selector.row_count);
        }
        at += selector.row_count;
    }
    slice_entry_ranges(cached, &ranges)
}

/// Slices for row ranges given in the entry's own row space; a range may
/// span cached batch boundaries, emitting one slice per overlapped batch.
fn slice_entry_ranges(
    cached: &CachedGroupRead,
    ranges: &[std::ops::Range<usize>],
) -> Vec<RecordBatch> {
    let mut slices = Vec::new();
    for range in ranges {
        let mut start = range.start;
        let end = range.end;
        for (batch, offset) in cached.batches.iter().zip(&cached.offsets) {
            let batch_end = offset + batch.num_rows();
            if start >= end {
                break;
            }
            if start >= batch_end || end <= *offset {
                continue;
            }
            let slice_start = start - offset;
            let slice_len = end.min(batch_end) - start;
            slices.push(batch.slice(slice_start, slice_len));
            start += slice_len;
        }
    }
    slices
}

/// The subset's rows sliced out of an entry holding a superset: every kept
/// range is translated from group-absolute rows into the entry's row space
/// through the entry's selection key. `None` when the subset needs a row
/// the entry does not hold — the caller decodes instead. A narrow-pass
/// selection is always a subset of the base selection the pass examined,
/// so for that caller this only misses when the base entry itself is gone.
fn slice_cached_subset(
    cached: &CachedGroupRead,
    entry_key: &crate::part::SelectionKey,
    subset: &RowSelection,
) -> Option<Vec<RecordBatch>> {
    // The entry's select runs as (group-space start, entry-space start, len).
    let mut runs: Vec<(usize, usize, usize)> = Vec::new();
    let (mut group_at, mut entry_at) = (0usize, 0usize);
    for &(skip, rows) in entry_key.iter() {
        let rows = rows as usize;
        if !skip {
            runs.push((group_at, entry_at, rows));
            entry_at += rows;
        }
        group_at += rows;
    }
    let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
    let mut at = 0usize;
    for selector in subset.iter() {
        if !selector.skip && selector.row_count > 0 {
            let (start, end) = (at, at + selector.row_count);
            let index = runs.partition_point(|&(gs, _, len)| gs + len <= start);
            let &(gs, es, len) = runs.get(index)?;
            if start < gs || end > gs + len {
                return None;
            }
            ranges.push(es + (start - gs)..es + (end - gs));
        }
        at += selector.row_count;
    }
    Some(slice_entry_ranges(cached, &ranges))
}

/// The rows of one group inside the windows an exact-field mask admitted.
///
/// Padded to `group_rows` by `from_consecutive_ranges`, and that padding is
/// load-bearing: `RowSelection::intersection` passes the longer operand's
/// tail through when the other is exhausted, so every selection combined for
/// one group must span the group's full row count.
fn window_row_selection(mask: u64, group_rows: usize) -> RowSelection {
    let windows = group_rows.div_ceil(crate::part::BLOOM_WINDOW_ROWS);
    RowSelection::from_consecutive_ranges(
        (0..windows).filter(|window| mask & (1u64 << window) != 0).map(|window| {
            let start = window * crate::part::BLOOM_WINDOW_ROWS;
            start..((start + crate::part::BLOOM_WINDOW_ROWS).min(group_rows))
        }),
        group_rows,
    )
}

// Counts what a scan spends on opening the body, so the cost can be asserted
// rather than assumed. Every call here is a `File::open` plus a full Parquet
// footer parse.
//
// Per thread rather than global: a scan is synchronous and runs on its caller's
// thread, and the test harness runs tests in parallel in one process, so a
// shared counter would report other tests' scans.
#[cfg(test)]
thread_local! {
    pub(crate) static PART_DATA_OPENS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn open_part_data(
    part: &Part,
    validate_checksum: bool,
    page_index: bool,
) -> Result<(PreadReader, ArrowReaderMetadata), String> {
    #[cfg(test)]
    PART_DATA_OPENS.with(|opens| opens.set(opens.get() + 1));
    if validate_checksum {
        let actual = file_crc32(&part.data_path()).map_err(|error| {
            format!(
                "failed to checksum {DATA_FILE} for part {}: {error}",
                part.meta.id
            )
        })?;
        if actual != part.meta.integrity.data_crc32 {
            return Err(format!(
                "{DATA_FILE} checksum mismatch for part {}: expected {}, got {actual}",
                part.meta.id, part.meta.integrity.data_crc32
            ));
        }
    }

    let data_file =
        PreadReader::new(fs::File::open(part.data_path()).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    let policy = if page_index {
        parquet::file::metadata::PageIndexPolicy::Optional
    } else {
        parquet::file::metadata::PageIndexPolicy::Skip
    };
    let options =
        parquet::arrow::arrow_reader::ArrowReaderOptions::new().with_page_index_policy(policy);
    let arrow_reader_metadata =
        ArrowReaderMetadata::load(&data_file, options).map_err(|e| e.to_string())?;

    let parquet_rg_count = arrow_reader_metadata.metadata().num_row_groups();
    if parquet_rg_count != part.meta.row_group_count as usize {
        return Err(format!(
            "row group count mismatch for part {}: parquet footer says {}, meta says {}",
            part.meta.id, parquet_rg_count, part.meta.row_group_count
        ));
    }
    let parquet_row_count = arrow_reader_metadata.metadata().file_metadata().num_rows();
    if parquet_row_count != part.meta.row_count as i64 {
        return Err(format!(
            "row count mismatch for part {}: parquet footer says {}, meta says {}",
            part.meta.id, parquet_row_count, part.meta.row_count
        ));
    }
    let metadata_keys: Vec<String> = part
        .meta
        .metadata_columns
        .iter()
        .map(|(key, _)| key.clone())
        .collect();
    let parsed_keys: Vec<String> = part
        .meta
        .parsed_columns
        .iter()
        .map(|(key, _)| key.clone())
        .collect();
    let expected_schema = part_schema(&metadata_keys, &parsed_keys);
    if arrow_reader_metadata.schema().fields() != expected_schema.fields() {
        return Err(format!(
            "parquet schema does not match metadata for part {}: expected {:?}, got {:?}",
            part.meta.id,
            expected_schema.fields(),
            arrow_reader_metadata.schema().fields()
        ));
    }
    Ok((data_file, arrow_reader_metadata))
}

impl PartReader {
    pub fn open(part: Part) -> Result<Self, String> {
        Self::open_internal(part, true)
    }

    /// Opens the metadata and indexes for an object-store cached part. The
    /// Parquet body may have been evicted and is opened only while a query is
    /// actively reading it.
    pub fn open_cached(part: Part) -> Result<Self, String> {
        Self::open_internal(part, false)
    }

    fn open_internal(part: Part, require_data: bool) -> Result<Self, String> {
        // Innermost wins, so the blooms a query faults in are charged here
        // rather than to the query: they outlive it.
        let _arena = crate::memprof::enter(crate::memprof::Arena::Sidecar);
        validate_sidecar_files(&part)?;
        if part.meta.row_group_count == 0
            || part.meta.row_group_min_ts.len() != part.meta.row_group_count as usize
            || part.meta.row_group_max_ts.len() != part.meta.row_group_count as usize
        {
            return Err(format!(
                "row group metadata length mismatch for part {}",
                part.meta.id
            ));
        }
        let index_map = Arc::new(map_index_file(&part.index_path())?);
        let decoded_blooms = Arc::new(build_bloom_index(
            index_map,
            &part.meta.row_group_rows,
        )?);
        let metadata_keys: Vec<String> = part
            .meta
            .metadata_columns
            .iter()
            .map(|(key, _)| key.clone())
            .collect();
        let parsed_keys: Vec<String> = part
            .meta
            .parsed_columns
            .iter()
            .map(|(key, _)| key.clone())
            .collect();
        if require_data || part.data_path().exists() {
            open_part_data(&part, true, false)?;
        }
        let cache_projection =
            ScanProjection::build(&metadata_keys, &parsed_keys, &ColumnSet::all());
        let blooms = BloomSlot::new();
        let index_bytes = decoded_blooms.index_bytes();
        blooms.install(decoded_blooms.clone(), index_bytes);
        Ok(Self {
            part,
            blooms,
            metadata_keys,
            parsed_keys,
            group_cache: GroupCache::from_global(),
            cache_projection,
            data_metadata: std::sync::OnceLock::new(),
        })
    }

    /// The blooms, resident or re-read: a cache hit touches the LRU, a miss
    /// re-reads `index.bin` — the same bytes `open` validated by checksum —
    /// and reinstalls them under the global budget, evicting other parts'
    /// least-recently-used blooms if the total is over it.
    fn decoded_blooms(&self) -> Result<Arc<BloomIndex>, String> {
        if let Some(blooms) = self.blooms.get() {
            crate::part::record_bloom_cache_hit();
            return Ok(blooms);
        }
        // Charged to the sidecar arena, not to the faulting query: the
        // blooms outlive it.
        let _arena = crate::memprof::enter(crate::memprof::Arena::Sidecar);
        let index_map = map_index_file(&self.part.index_path()).map_err(|error| {
            format!(
                "failed to re-map {INDEX_FILE} for part {}: {error}",
                self.part.meta.id
            )
        })?;
        crate::part::record_bloom_cache_miss(index_map.len() as u64);
        let index_map = Arc::new(index_map);
        let decoded = Arc::new(build_bloom_index(
            index_map,
            &self.part.meta.row_group_rows,
        )?);
        let bytes = decoded.index_bytes();
        self.blooms.install(decoded.clone(), bytes);
        Ok(decoded)
    }

    pub fn part(&self) -> &Part {
        &self.part
    }

    pub fn meta(&self) -> &PartMeta {
        &self.part.meta
    }

    /// The row groups a tenant may address, or `None` when the part holds no
    /// rows for it. Every read path funnels through this, so a tenant cannot
    /// reach another tenant's rows even if a matcher or filter would select
    /// them.
    fn tenant_row_groups(&self, tenant: &TenantId) -> Option<std::ops::Range<u32>> {
        self.part
            .meta
            .tenant_segment(tenant)
            .map(|segment| segment.row_group_start..segment.row_group_end)
    }

    pub fn query(
        &self,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        range: QueryTimeRange,
        limit: usize,
        forward: bool,
    ) -> Result<Vec<StreamResult>, String> {
        Ok(self
            .query_with_exact_field_pruning_and_scan_limit(
                tenant,
                ExactFieldPruning::new(line_filters, &[]),
                range,
                limit,
                forward,
                None,
                None,
            )?
            .results)
    }

    /// Uses exact-field predicates only for row-group pruning. Bloom filters
    /// can return false positives, so the caller remains responsible for
    /// evaluating the predicates against each returned entry.
    pub fn query_with_exact_field_pruning(
        &self,
        tenant: &TenantId,
        pruning: ExactFieldPruning<'_>,
        range: QueryTimeRange,
        limit: usize,
        forward: bool,
    ) -> Result<Vec<StreamResult>, String> {
        Ok(self
            .query_with_exact_field_pruning_and_scan_limit(
                tenant, pruning, range, limit, forward, None, None,
            )?
            .results)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn query_with_exact_field_pruning_and_scan_limit(
        &self,
        tenant: &TenantId,
        pruning: ExactFieldPruning<'_>,
        range: QueryTimeRange,
        limit: usize,
        forward: bool,
        scan_limit: Option<usize>,
        cancellation: Option<&AtomicBool>,
    ) -> Result<QueryResult, String> {
        self.query_with_exact_field_pruning_and_scan_limits(
            tenant,
            pruning,
            range,
            limit,
            forward,
            scan_limit,
            None,
            cancellation,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn query_with_exact_field_pruning_and_scan_limits(
        &self,
        tenant: &TenantId,
        pruning: ExactFieldPruning<'_>,
        range: QueryTimeRange,
        limit: usize,
        forward: bool,
        scan_limit: Option<usize>,
        scan_bytes_limit: Option<u64>,
        cancellation: Option<&AtomicBool>,
    ) -> Result<QueryResult, String> {
        self.query_internal(
            tenant,
            pruning.line_filters,
            pruning.exact_fields,
            range,
            limit,
            forward,
            scan_limit,
            scan_bytes_limit,
            cancellation,
            None,
        )
    }

    /// Every row in the part, tenant by tenant. Merge is the only caller: it
    /// rewrites a part and therefore has to see all tenants, so this returns
    /// `Row`s that carry their own tenant instead of `StreamResult`s that do
    /// not.
    pub fn read_all_rows(&self, scan_bytes_limit: Option<u64>) -> Result<Vec<Row>, String> {
        self.read_rows_in_row_groups(0..self.part.meta.row_group_count, scan_bytes_limit)
    }

    /// The same, restricted to a window of row groups.
    ///
    /// Row groups are the part's own bounded unit — `row_group_size` caps how
    /// many rows one holds — so a window is what lets a rewrite of a part too
    /// large to materialize proceed in pieces instead of failing forever.
    pub fn read_rows_in_row_groups(
        &self,
        row_groups: std::ops::Range<u32>,
        scan_bytes_limit: Option<u64>,
    ) -> Result<Vec<Row>, String> {
        let mut rows = Vec::new();
        for segment in &self.part.meta.tenants {
            if segment.row_group_start >= row_groups.end || segment.row_group_end <= row_groups.start
            {
                continue;
            }
            // Straight into `Row`s. This used to build `StreamResult`s grouped
            // by label set so that the caller could immediately flatten them
            // again, which is the reader's share of the triple materialize.
            let mut collector = RowCollector::new(&segment.tenant);
            self.scan_into(
                &segment.tenant,
                &[],
                &[],
                QueryTimeRange::unbounded(),
                true,
                None,
                scan_bytes_limit,
                None,
                Some(row_groups.clone()),
                &ColumnSet::all(),
                &mut collector,
            )?;
            rows.append(&mut collector.into_rows());
        }
        Ok(rows)
    }

    pub fn row_group_count(&self) -> u32 {
        self.part.meta.row_group_count
    }

    fn select_row_groups(
        &self,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        time_range: QueryTimeRange,
    ) -> Result<Vec<u32>, String> {
        self.select_row_groups_with_exact_fields(tenant, line_filters, &[], time_range)
    }

    fn select_row_groups_with_exact_fields(
        &self,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        exact_fields: &[ExactFieldPredicate],
        time_range: QueryTimeRange,
    ) -> Result<Vec<u32>, String> {
        let Some(groups) = self.tenant_row_groups(tenant) else {
            return Ok(Vec::new());
        };
        // Once per scan, not per row group: a `|=` needle is a literal as it
        // stands, and a `|~` contributes the literals every match must
        // contain, extracted from its parsed form. `|~ "error.*timeout"` was
        // never pruned before this — both its literals were indexable and the
        // bloom was only asked about `Contains`.
        let prune_literals: Vec<String> = line_filters
            .iter()
            .flat_map(|filter| match filter {
                LineFilter::Contains(literal) => vec![literal.clone()],
                LineFilter::Regex(regex) => {
                    crate::logql::required_regex_literals(regex.as_str())
                }
                LineFilter::NotContains(_) | LineFilter::NotRegex(_) => Vec::new(),
            })
            .collect();
        // Fetched only when a filter can actually consult them, so a
        // matchers-only query never faults evicted blooms back in — it never
        // read them before eviction existed either.
        let blooms = if prune_literals.is_empty() && exact_fields.is_empty() {
            None
        } else {
            Some(self.decoded_blooms()?)
        };
        let mut selected = Vec::with_capacity(groups.len());
        for rg in groups {
            let rgu = rg as usize;
            if !time_range.overlaps(
                self.part.meta.row_group_min_ts[rgu],
                self.part.meta.row_group_max_ts[rgu],
            ) {
                continue;
            }
            if let Some(blooms) = &blooms {
                if !self.bloom_prune(blooms, rgu, &prune_literals) {
                    continue;
                }
                if !self.exact_field_bloom_prune(blooms, rgu, exact_fields) {
                    continue;
                }
            }
            selected.push(rg);
        }
        Ok(selected)
    }

    /// Returns whether any row group can satisfy the catalog-visible portion
    /// of a query. This does not open `data.parquet`, so object-store callers
    /// can use it before deciding which evicted bodies to restore.
    pub fn may_match_exact_fields(
        &self,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        exact_fields: &[ExactFieldPredicate],
        range: QueryTimeRange,
    ) -> bool {
        // A bloom re-read that fails must answer "may match": skipping a
        // part on an I/O error would silently drop rows, where admitting it
        // surfaces the error on the scan that follows.
        self.select_row_groups_with_exact_fields(tenant, line_filters, exact_fields, range)
            .map(|groups| !groups.is_empty())
            .unwrap_or(true)
    }

    #[allow(clippy::too_many_arguments)]
    fn query_internal(
        &self,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        exact_fields: &[ExactFieldPredicate],
        time_range: QueryTimeRange,
        limit: usize,
        forward: bool,
        scan_limit: Option<usize>,
        scan_bytes_limit: Option<u64>,
        cancellation: Option<&AtomicBool>,
        row_group_window: Option<std::ops::Range<u32>>,
    ) -> Result<QueryResult, String> {
        let mut rows = TopKRows::new(limit, forward);
        let stats = self.scan_into(
            tenant,
            line_filters,
            exact_fields,
            time_range,
            forward,
            scan_limit,
            scan_bytes_limit,
            cancellation,
            row_group_window,
            &ColumnSet::all(),
            &mut rows,
        )?;
        Ok(QueryResult {
            results: rows.into_stream_results(),
            scanned_rows: stats.scanned_rows,
            scanned_bytes: stats.scanned_bytes,
        })
    }

    /// One part's rows, offered to `sink` in the query's direction.
    ///
    /// The scan is bounded by whatever the sink can still take rather than by a
    /// `limit` argument, which is the point: the caller that knows whether a row
    /// survives the pipeline is the caller that owns the sink, so it is the sink
    /// that says when to stop.
    ///
    /// **This relies on rows being ordered within a tenant** — `Row::sort_key`
    /// is `(tenant, timestamp_ns, …)` and every writer sorts, which is also what
    /// makes `row_group_min_ts`/`max_ts` selective. So the first row on the far
    /// side of the sink's frontier ends the part: every row after it in this
    /// direction is on the far side too.
    /// `part::tests::a_parts_rows_come_back_in_timestamp_order_within_a_tenant`
    /// is what holds it.
    #[allow(clippy::too_many_arguments)]
    pub fn scan_into(
        &self,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        exact_fields: &[ExactFieldPredicate],
        time_range: QueryTimeRange,
        forward: bool,
        scan_limit: Option<usize>,
        scan_bytes_limit: Option<u64>,
        cancellation: Option<&AtomicBool>,
        row_group_window: Option<std::ops::Range<u32>>,
        columns: &ColumnSet,
        sink: &mut dyn RowSink,
    ) -> Result<ScanStats, String> {
        let mut stats = ScanStats::default();
        if self.part.meta.row_group_count == 0 || sink.is_closed() {
            return Ok(stats);
        }
        debug_assert!(
            columns.line || line_filters.is_empty(),
            "a scan that skips the line column cannot evaluate line filters"
        );
        let projection = ScanProjection::build(&self.metadata_keys, &self.parsed_keys, columns);
        // This scan's fields re-addressed into the cache's batch layout.
        // `Some` for every projection the cache can serve — the full log
        // projection (identical layout) and the metric path's named columns
        // alike; a metric scan then reads a cached decode some broad query
        // paid for, materializing only the columns it names.
        let cache_view = self
            .group_cache
            .enabled()
            .then(|| projection.view_in(&self.cache_projection))
            .flatten();
        // Lookups are for everyone; *fills* are for queries only. A rewrite
        // reads every group of the parts it is about to delete, so what it
        // decodes is worth nothing to a later scan — the readers die with the
        // merge that opened them. Filling from here spent the whole cache
        // budget on batches no query would ask for, evicting the entries that
        // were serving one, and — because memprof attributes at allocation and
        // the cache only retains — parked those bytes under the *merge* tag
        // (`docs/VISION.md` records the same mis-binding for `query`, where the
        // gauge is the separator). A merge still reads from the cache: a group
        // some query already decoded is free to reuse.
        let fill_cache = row_group_window.is_none();
        // The predicates the part's own columns answer *exactly*: a string
        // equality on a field, read from the `_sm:` column (pushed metadata)
        // and — when the only parser in the pipeline is `| json` over the
        // stored line — falling back to the `_pf:` column exactly as the
        // pipeline falls back from metadata to extraction. For such a
        // predicate the columns *are* the field the filter would see, so rows
        // are selected on the narrow columns before the wide decode — the
        // row-level precision the row-group bloom cannot give.
        //
        // Absence is decidable when a key list is under its cap: every present
        // key then has a column, so a key that is not listed is on no row. A
        // predicate whose field is provably absent on both sides matches
        // nothing — the whole part is skipped, which is what turns a bloom
        // false positive from a group decode into no read at all.
        let mut definitive: Vec<DefinitiveColumn> = Vec::new();
        for predicate in exact_fields {
            if predicate.canonical || predicate.value.is_empty() {
                continue;
            }
            let sm_key = self.metadata_keys.binary_search(&predicate.name).ok();
            let sm_decidable =
                sm_key.is_some() || self.metadata_keys.len() < MAX_METADATA_COLUMNS;
            if !predicate.may_be_extracted {
                match sm_key {
                    Some(key) => definitive.push(DefinitiveColumn {
                        value: predicate.value.clone(),
                        sm_key: Some(key),
                        pf_key: None,
                        extracted: false,
                    }),
                    None if sm_decidable => return Ok(stats),
                    None => {}
                }
                continue;
            }
            if !predicate.json_only_extraction {
                continue;
            }
            let pf_key = self.parsed_keys.binary_search(&predicate.name).ok();
            let pf_decidable = pf_key.is_some() || self.parsed_keys.len() < MAX_METADATA_COLUMNS;
            if !(sm_decidable && pf_decidable) {
                continue;
            }
            if sm_key.is_none() && pf_key.is_none() {
                return Ok(stats);
            }
            definitive.push(DefinitiveColumn {
                value: predicate.value.clone(),
                sm_key,
                pf_key,
                extracted: true,
            });
        }

        let selected = if exact_fields.is_empty() {
            self.select_row_groups(tenant, line_filters, time_range)?
        } else {
            self.select_row_groups_with_exact_fields(tenant, line_filters, exact_fields, time_range)?
        };
        let mut sorted_selected = selected;
        if let Some(window) = &row_group_window {
            sorted_selected.retain(|row_group| window.contains(row_group));
        }
        if sorted_selected.is_empty() {
            return Ok(stats);
        }
        // Query scans only: a rewrite passes a window and reads what it was
        // told to, so its selectivity is its own and not a query's.
        if row_group_window.is_none() {
            let tenant_groups = self
                .tenant_row_groups(tenant)
                .map(|groups| groups.end - groups.start)
                .unwrap_or(0);
            crate::restore_meter::global().note_query_scan(
                &self.part.dir,
                tenant,
                self.part.meta.row_group_count,
                tenant_groups,
                &sorted_selected,
            );
        }
        // By the end the scan reaches first, so the sink's frontier tightens
        // as early as possible. A windowed read is the exception: it is a
        // rewrite reading the part in layout order, and `MergedRows`
        // k-way-merges what it returns on the promise that each page arrives
        // in `Row::sort_key` order.
        if row_group_window.is_none() {
            sorted_selected.sort_unstable_by_key(|&row_group| {
                let rgu = row_group as usize;
                if forward {
                    (self.part.meta.row_group_min_ts[rgu], row_group)
                } else {
                    (-self.part.meta.row_group_max_ts[rgu], row_group)
                }
            });
        }

        // Outside the row-group loop: a stream spans row groups, so a cache per
        // group would rebuild every label set once per group.
        // One footer parse for the whole scan. This used to re-open the file and
        // re-run `ArrowReaderMetadata::load` inside the loop, so two hundred
        // selected row groups were two hundred footer parses
        // (`docs/VISION.md` III). Both handles are cheap to clone — a `File`
        // behind an `Arc` and an `Arc<ParquetMetaData>` — which is what makes a
        // reader per row group, and per window inside one, affordable.
        let (data_file, part_metadata) = self.scan_part_data()?;
        // Fetched once for the whole scan when the window masks below will
        // consult them; the selection above already faulted them in, so this
        // is an LRU touch, not a read.
        let scan_blooms = if exact_fields.is_empty() {
            None
        } else {
            Some(self.decoded_blooms()?)
        };
        // The sink plumbing still carries a shared label map per row; with no
        // stream concept it is one empty map for the whole scan.
        let empty_labels: SharedLabels = SharedLabels::default();

        'row_groups: for &row_group in &sorted_selected {
            if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
                break;
            }
            let rgu = row_group as usize;
            // Re-read per group, because the frontier tightens as the sink
            // fills: a group whose whole span is behind it holds nothing that
            // can enter the result, and is rejected from `meta.json` without the
            // Parquet body being touched at all.
            if span_beyond_frontier(
                sink.frontier_ns(),
                forward,
                self.part.meta.row_group_min_ts[rgu],
                self.part.meta.row_group_max_ts[rgu],
            ) {
                continue;
            }
            // Pass one, when a definitive predicate exists: read only the
            // timestamp and the predicate columns, keep the row indices that
            // satisfy them, and hand the wide decode a `RowSelection` instead
            // of the whole group. The group the bloom admitted wrongly costs
            // one narrow read; the group holding four of a rare value decodes
            // four rows wide instead of eight thousand.
            // Sub-group time pruning off the page index: `timestamp_ns` is the
            // one column that keeps page-level bounds (`part_writer_properties`),
            // and a page whose whole span misses the window is skipped before
            // it is decoded — by the narrow first pass too, which is what the
            // `trace_window` measurement forced: without it the first pass
            // examined every row of every admitted group whatever the window
            // said, and a two-second window cost what the whole range cost.
            let time_selection = time_page_selection(&part_metadata, rgu, time_range);
            if let Some(time_selection) = &time_selection
                && !time_selection.selects_any()
            {
                continue;
            }
            // Sub-group exact-field pruning: the per-window blooms restrict a
            // match to the windows whose filters admit every predicate's
            // token, and the selection makes both passes decode only those
            // windows. Both operands are padded to the group's full row count
            // (`from_consecutive_ranges` pads the tail), which `intersection`
            // requires — a shorter operand's missing tail would pass the
            // longer one's rows through unfiltered.
            let group_rows = part_metadata.metadata().row_group(rgu).num_rows() as usize;
            let window_selection = scan_blooms
                .as_ref()
                .and_then(|blooms| self.exact_field_window_mask(blooms, rgu, exact_fields))
                .map(|mask| window_row_selection(mask, group_rows));
            let base_selection = match (time_selection, window_selection) {
                (Some(time), Some(window)) => Some(time.intersection(&window)),
                (time, window) => time.or(window),
            };
            if let Some(base_selection) = &base_selection
                && !base_selection.selects_any()
            {
                continue;
            }
            // A group decoded whole by an earlier scan serves this one from
            // memory: the per-group constant — reader build, dictionary
            // pages, page decompression for every projected column — is what
            // the cache exists to not pay twice. Selections are applied by
            // zero-copy slicing, and the batches carry the full projection,
            // so a narrower caller reads them through `cache_projection`.
            let cached = cache_view
                .is_some()
                .then(|| self.group_cache.get_full(row_group, group_rows))
                .flatten();
            // The narrow pass answers from the cache when every definitive
            // column lives in it (`_sm:`; the full projection carries no
            // `_pf:` columns, so extraction-backed predicates keep the
            // builder path, whose two-column read is cheap).
            let cache_covers_definitive =
                !definitive.is_empty() && definitive.iter().all(|def| def.pf_key.is_none());
            // The narrow pass is deterministic in (group, window, base
            // selection, predicates), so its outcome — a selection or a
            // rejection — is remembered too: a repeated rare query then
            // pays two map lookups instead of a builder per group.
            let narrow_query = (self.group_cache.enabled() && !definitive.is_empty())
                .then(|| crate::part::NarrowQuery {
                    rgu: row_group,
                    time: time_range.cache_identity(),
                    base: crate::part::selection_key(base_selection.as_ref(), group_rows),
                    defs: definitive
                        .iter()
                        .map(|def| (def.sm_key, def.pf_key, def.extracted, def.value.clone()))
                        .collect(),
                });
            let remembered_narrow = narrow_query
                .as_ref()
                .and_then(|query| self.group_cache.get_narrow(query));
            let mut selection = if definitive.is_empty() {
                None
            } else if let Some(outcome) = remembered_narrow {
                match outcome {
                    Some(key) => Some(crate::part::selection_of(&key, group_rows)),
                    None => continue,
                }
            } else {
                let outcome = if let (Some(cached), true) = (&cached, cache_covers_definitive) {
                    self.select_rows_in_cached_group(
                        cached,
                        &definitive,
                        time_range,
                        base_selection.as_ref(),
                        &mut stats,
                    )?
                } else {
                    self.select_rows_in_group(
                        &data_file,
                        &part_metadata,
                        rgu,
                        &definitive,
                        time_range,
                        base_selection.as_ref(),
                        &mut stats,
                    )?
                };
                if let Some(query) = narrow_query {
                    self.group_cache.insert_narrow(
                        query,
                        outcome
                            .as_ref()
                            .map(|selection| crate::part::selection_key(Some(selection), group_rows)),
                    );
                }
                match outcome {
                    Some(selection) => Some(selection),
                    None => continue,
                }
            };
            let count_scanned_rows = selection.is_none();
            // The base's identity, taken before the fold consumes it: it is
            // both the effective key when no narrow pass ran, and the key a
            // broad query's fill sits under when one did — which is what
            // the subset serve below slices from.
            let base_key = cache_view
                .is_some()
                .then(|| crate::part::selection_key(base_selection.as_ref(), group_rows));
            if selection.is_none()
                && let Some(base_selection) = base_selection
            {
                selection = Some(base_selection);
            }
            // The effective selection is this decode's identity in the
            // cache. A repeated window resolves to the same page selection,
            // and a different predicate resolving to the same rows —
            // `metadata_rare` after `json_field_rare` — to the same narrow
            // result; both replay the decode below without touching Parquet.
            // Any viewable scan looks up; only a decode in the cache's own
            // layout fills, because a narrower decode cannot serve later
            // callers.
            let selection_key = if count_scanned_rows {
                base_key.clone()
            } else {
                cache_view
                    .is_some()
                    .then(|| crate::part::selection_key(selection.as_ref(), group_rows))
            };
            let fill_layout = projection.leaves == self.cache_projection.leaves;
            if let Some(cached) = cached {
                // Serve the group from memory: slice the selected ranges
                // (zero-copy) and walk them through the same scan_batch the
                // decode path uses, under the cache's own full projection.
                let mut slices = slice_cached_group(&cached, selection.as_ref());
                let selected_rows: usize = slices.iter().map(RecordBatch::num_rows).sum();
                if !forward {
                    slices.reverse();
                }
                // Proportional, not measured: a slice reports its underlying
                // buffers' full capacity, and the decode path would have
                // charged only the selected rows' batches. Approximate what a
                // miss would have reported rather than the cache's residency.
                stats.scanned_bytes = stats.scanned_bytes.saturating_add(
                    (cached.bytes as u128 * selected_rows as u128
                        / cached.total_rows.max(1) as u128) as u64,
                );
                if scan_bytes_limit.is_some_and(|limit| stats.scanned_bytes > limit) {
                    return Err(format!(
                        "query exceeds the maximum of {} scanned bytes",
                        scan_bytes_limit.unwrap_or_default()
                    ));
                }
                for batch in &slices {
                    match self.scan_batch(
                        batch,
                        tenant,
                        line_filters,
                        time_range,
                        forward,
                        row_group,
                        scan_limit,
                        // Bytes were charged proportionally above; None keeps
                        // scan_batch from re-measuring slice capacities.
                        None,
                        cancellation,
                        cache_view.as_ref().expect("a cached serve implies a view"),
                        false,
                        count_scanned_rows,
                        &empty_labels,
                        &mut stats,
                        sink,
                    ) {
                        Ok(ScanStep::Continue) => {}
                        Ok(ScanStep::Stop) => break 'row_groups,
                        Err(error) => return Err(error),
                    }
                }
                continue;
            }
            // An exact match on the effective selection replays the decode:
            // the batches are the very ones a miss would produce, so they go
            // through the same scan_batch with the same accounting.
            if let Some(key) = &selection_key
                && let Some(replay) = self.group_cache.get(row_group, key)
            {
                let mut batches: Vec<&RecordBatch> = replay.batches.iter().collect();
                if !forward {
                    batches.reverse();
                }
                for batch in batches {
                    match self.scan_batch(
                        batch,
                        tenant,
                        line_filters,
                        time_range,
                        forward,
                        row_group,
                        scan_limit,
                        scan_bytes_limit,
                        cancellation,
                        cache_view.as_ref().expect("a cached serve implies a view"),
                        true,
                        count_scanned_rows,
                        &empty_labels,
                        &mut stats,
                        sink,
                    ) {
                        Ok(ScanStep::Continue) => {}
                        Ok(ScanStep::Stop) => break 'row_groups,
                        Err(error) => return Err(error),
                    }
                }
                continue;
            }
            // A narrowed selection is a subset of the base selection the
            // pass examined, and a broad query cached the base's decode
            // under exactly that key — `json_field` after `label_only` in
            // the bed. Slicing the needed rows out of that entry serves
            // the wide pass without a builder, and the parse still runs
            // only on the narrow survivors, which is what the narrow pass
            // is for.
            if !count_scanned_rows
                && let (Some(view), Some(base_key)) = (cache_view.as_ref(), &base_key)
                && let Some(entry) = self.group_cache.get(row_group, base_key)
                && let Some(mut slices) = slice_cached_subset(
                    &entry,
                    base_key,
                    selection.as_ref().expect("a narrow pass produced a selection"),
                )
            {
                #[cfg(test)]
                self.group_cache
                    .subset_serves
                    .fetch_add(1, Ordering::AcqRel);
                let selected_rows: usize = slices.iter().map(RecordBatch::num_rows).sum();
                if !forward {
                    slices.reverse();
                }
                stats.scanned_bytes = stats.scanned_bytes.saturating_add(
                    (entry.bytes as u128 * selected_rows as u128
                        / entry.total_rows.max(1) as u128) as u64,
                );
                if scan_bytes_limit.is_some_and(|limit| stats.scanned_bytes > limit) {
                    return Err(format!(
                        "query exceeds the maximum of {} scanned bytes",
                        scan_bytes_limit.unwrap_or_default()
                    ));
                }
                for batch in &slices {
                    match self.scan_batch(
                        batch,
                        tenant,
                        line_filters,
                        time_range,
                        forward,
                        row_group,
                        scan_limit,
                        None,
                        cancellation,
                        view,
                        false,
                        count_scanned_rows,
                        &empty_labels,
                        &mut stats,
                        sink,
                    ) {
                        Ok(ScanStep::Continue) => {}
                        Ok(ScanStep::Stop) => break 'row_groups,
                        Err(error) => return Err(error),
                    }
                }
                continue;
            }
            let projection = &projection;
            if forward {
                // Forwards, Parquet's own order is the query's order, so one
                // reader streams the group and the sink's frontier ends it.
                let mut builder = ParquetRecordBatchReaderBuilder::new_with_metadata(
                    data_file.clone(),
                    part_metadata.clone(),
                )
                .with_projection(ProjectionMask::leaves(
                    part_metadata.metadata().file_metadata().schema_descr(),
                    projection.leaves.iter().copied(),
                ))
                .with_batch_size(window_rows(scan_limit, stats.scanned_rows, &*sink))
                .with_row_groups(vec![rgu]);
                if let Some(selection) = selection.clone() {
                    builder = builder.with_row_selection(selection);
                }
                let reader = builder.build().map_err(|e| e.to_string())?;
                let mut fill: Option<Vec<RecordBatch>> =
                    (fill_cache && selection_key.is_some() && fill_layout).then(Vec::new);
                for batch in reader {
                    let batch = batch.map_err(|e| e.to_string())?;
                    if let Some(fill) = &mut fill {
                        fill.push(batch.clone());
                    }
                    match self.scan_batch(
                        &batch,
                        tenant,
                        line_filters,
                        time_range,
                        forward,
                        row_group,
                        scan_limit,
                        scan_bytes_limit,
                        cancellation,
                        projection,
                        true,
                        count_scanned_rows,
                        &empty_labels,
                        &mut stats,
                        sink,
                    ) {
                        Ok(ScanStep::Continue) => {}
                        // A stop exits the whole scan, which also skips the
                        // fill below: a stopped group may be partially
                        // decoded, and only a completed decode is cacheable.
                        Ok(ScanStep::Stop) => break 'row_groups,
                        Err(error) => return Err(error),
                    }
                }
                if let (Some(fill), Some(key)) = (fill, selection_key)
                    && fill.iter().map(RecordBatch::num_rows).sum::<usize>()
                        == crate::part::selected_rows_of(&key)
                {
                    self.group_cache.insert(row_group, key, fill);
                }
                continue;
            }
            // Backwards, Parquet still only reads forwards, so the group is
            // decoded once in row order and the batches offered newest-first.
            //
            // This *was* a doubling-window walk from the group's end
            // (`with_offset`), which was exact when a group was one
            // timestamp-ordered run. A group now holds several whole streams,
            // so its end is the last *stream's* rows, not the newest — the
            // walk fed the sink one stream's tail, the frontier tightened on
            // it, and the scan stopped before the other streams' newer rows,
            // which is the wrong-answer the comparison bed caught. Reading the
            // group whole is the correct baseline; reading *less* of a group
            // needs to know the group is time-ordered, which the format does
            // not record yet — when it does, the windowed walk can return for
            // exactly the groups it was correct on.
            let mut builder = ParquetRecordBatchReaderBuilder::new_with_metadata(
                data_file.clone(),
                part_metadata.clone(),
            )
            .with_projection(ProjectionMask::leaves(
                part_metadata.metadata().file_metadata().schema_descr(),
                projection.leaves.iter().copied(),
            ))
            .with_batch_size(1024)
            .with_row_groups(vec![rgu]);
            if let Some(selection) = selection {
                builder = builder.with_row_selection(selection);
            }
            let reader = builder.build().map_err(|e| e.to_string())?;
            let mut batches: Vec<_> = reader
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            // Backwards decodes its selection whole before scanning, so the
            // batches are cacheable right here — `RecordBatch` clones are
            // refcounts, not copies. Which is also why the shrink happens
            // before the clone rather than inside `insert`: once the scan below
            // and the entry both hold a buffer, Arrow declines to shrink it.
            if let Some(key) = selection_key
                && fill_cache
                && fill_layout
                && batches.iter().map(RecordBatch::num_rows).sum::<usize>()
                    == crate::part::selected_rows_of(&key)
            {
                batches = crate::part::shrink_batches(batches);
                self.group_cache.insert(row_group, key, batches.clone());
            }
            batches.reverse();
            for batch in &batches {
                match self.scan_batch(
                    batch,
                    tenant,
                    line_filters,
                    time_range,
                    forward,
                    row_group,
                    scan_limit,
                    scan_bytes_limit,
                    cancellation,
                    projection,
                    true,
                    count_scanned_rows,
                    &empty_labels,
                    &mut stats,
                    sink,
                ) {
                    Ok(ScanStep::Continue) => {}
                    Ok(ScanStep::Stop) => break 'row_groups,
                    Err(error) => return Err(error),
                }
            }
        }

        Ok(stats)
    }


    /// The scan path's handle pair: a fresh descriptor (so eviction can still
    /// reclaim the bytes once no scan holds one) and the cached footer.
    fn scan_part_data(&self) -> Result<(PreadReader, ArrowReaderMetadata), String> {
        if let Some(part_metadata) = self.data_metadata.get() {
            let data_file = PreadReader::new(
                fs::File::open(self.part.data_path()).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            return Ok((data_file, part_metadata.clone()));
        }
        let (data_file, part_metadata) = open_part_data(&self.part, false, true)?;
        let part_metadata = self
            .data_metadata
            .get_or_init(|| part_metadata.clone())
            .clone();
        Ok((data_file, part_metadata))
    }

    #[allow(clippy::too_many_arguments)]
    /// The narrow pass served from a cached group: the same keep logic as
    /// [`select_rows_in_group`](Self::select_rows_in_group), evaluated over
    /// the in-memory batches instead of a second reader build. Only called
    /// when every definitive column is `_sm:`-backed — the cached full
    /// projection carries no `_pf:` columns.
    fn select_rows_in_cached_group(
        &self,
        cached: &CachedGroupRead,
        definitive: &[DefinitiveColumn],
        time_range: QueryTimeRange,
        base_selection: Option<&RowSelection>,
        stats: &mut ScanStats,
    ) -> Result<Option<RowSelection>, String> {
        let sm_position = |key: usize| -> Result<usize, String> {
            self.cache_projection
                .metadata
                .iter()
                .find(|&&(key_index, _)| key_index == key)
                .map(|&(_, position)| position)
                .ok_or_else(|| "cached projection is missing a metadata column".to_string())
        };
        let columns: Vec<usize> = definitive
            .iter()
            .map(|def| sm_position(def.sm_key.expect("caller checked sm-only definitive")))
            .collect::<Result<_, _>>()?;

        let mut kept: Vec<std::ops::Range<usize>> = Vec::new();
        let mut examined = 0usize;
        let mut visit = |start: usize, end: usize| {
            let mut row = start;
            for (batch, offset) in cached.batches.iter().zip(&cached.offsets) {
                let batch_end = offset + batch.num_rows();
                if row >= end {
                    break;
                }
                if row >= batch_end || end <= *offset {
                    continue;
                }
                let ts = batch.column(1).as_primitive::<Int64Type>();
                let sm: Vec<&StringArray> = columns
                    .iter()
                    .map(|&position| batch.column(position).as_string::<i32>())
                    .collect();
                while row < end.min(batch_end) {
                    let i = row - offset;
                    examined += 1;
                    let keep = time_range.contains(ts.value(i))
                        && definitive.iter().zip(&sm).all(|(def, column)| {
                            !column.is_null(i) && column.value(i) == def.value
                        });
                    if keep {
                        match kept.last_mut() {
                            Some(range) if range.end == row => range.end = row + 1,
                            _ => kept.push(row..row + 1),
                        }
                    }
                    row += 1;
                }
            }
        };
        match base_selection {
            Some(selection) => {
                let mut at = 0usize;
                for selector in selection.iter() {
                    if !selector.skip {
                        visit(at, at + selector.row_count);
                    }
                    at += selector.row_count;
                }
            }
            None => visit(0, cached.total_rows),
        }
        stats.scanned_rows = stats.scanned_rows.saturating_add(examined);
        stats.scanned_bytes = stats.scanned_bytes.saturating_add(
            (cached.bytes as u128 * examined as u128 / cached.total_rows.max(1) as u128) as u64,
        );
        if kept.is_empty() {
            return Ok(None);
        }
        Ok(Some(RowSelection::from_consecutive_ranges(
            kept.into_iter(),
            cached.total_rows,
        )))
    }

    #[allow(clippy::too_many_arguments)]
    fn select_rows_in_group(
        &self,
        data_file: &PreadReader,
        part_metadata: &ArrowReaderMetadata,
        rgu: usize,
        definitive: &[DefinitiveColumn],
        time_range: QueryTimeRange,
        base_selection: Option<&RowSelection>,
        stats: &mut ScanStats,
    ) -> Result<Option<RowSelection>, String> {
        let sm_leaf = |key: usize| 3 + key;
        let pf_leaf = |key: usize| 3 + self.metadata_keys.len() + key;
        let mut leaves = vec![1usize];
        for def in definitive {
            leaves.extend(def.sm_key.map(sm_leaf));
            leaves.extend(def.pf_key.map(pf_leaf));
        }
        leaves.sort_unstable();
        leaves.dedup();
        let position = |leaf: usize| leaves.binary_search(&leaf).expect("projected leaf");
        let reader = ParquetRecordBatchReaderBuilder::new_with_metadata(
            data_file.clone(),
            part_metadata.clone(),
        )
        .with_projection(ProjectionMask::leaves(
            part_metadata.metadata().file_metadata().schema_descr(),
            leaves.iter().copied(),
        ))
        .with_batch_size(8192)
        .with_row_groups(vec![rgu]);
        let reader = match base_selection {
            Some(base_selection) => reader.with_row_selection(base_selection.clone()),
            None => reader,
        }
        .build()
        .map_err(|e| e.to_string())?;
        // Under a row selection the reader yields only the selected rows, so
        // the running index must walk the selection to stay group-absolute —
        // the returned `RowSelection` addresses the group, not the pass.
        let group_rows = part_metadata.metadata().row_group(rgu).num_rows().max(0) as usize;
        let mut absolute: Box<dyn Iterator<Item = usize>> = match base_selection {
            Some(base_selection) => {
                let mut at = 0usize;
                let mut indices = Vec::new();
                for selector in base_selection.iter() {
                    if !selector.skip {
                        indices.extend(at..at + selector.row_count);
                    }
                    at += selector.row_count;
                }
                Box::new(indices.into_iter())
            }
            None => Box::new(0..group_rows),
        };

        let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
        let mut last_row = 0usize;
        for batch in reader {
            let batch = batch.map_err(|e| e.to_string())?;
            stats.scanned_bytes = stats
                .scanned_bytes
                .saturating_add(batch.get_array_memory_size() as u64);
            let ts = batch
                .column(position(1))
                .as_primitive::<Int64Type>();
            let columns: Vec<(Option<&StringArray>, Option<&StringArray>)> = definitive
                .iter()
                .map(|def| {
                    (
                        def.sm_key
                            .map(|key| batch.column(position(sm_leaf(key))).as_string::<i32>()),
                        def.pf_key
                            .map(|key| batch.column(position(pf_leaf(key))).as_string::<i32>()),
                    )
                })
                .collect();
            for i in 0..batch.num_rows() {
                let row = absolute
                    .next()
                    .ok_or_else(|| "row selection shorter than the batch".to_string())?;
                last_row = last_row.max(row + 1);
                stats.scanned_rows = stats.scanned_rows.saturating_add(1);
                let keep = time_range.contains(ts.value(i))
                    && definitive.iter().zip(&columns).all(|(def, (sm, pf))| {
                        // Metadata wins over extraction, exactly as the
                        // pipeline's shadowing rule: when the row carries the
                        // key as metadata, extraction never reaches the field.
                        let value = match sm {
                            Some(sm) if !sm.is_null(i) => Some(sm.value(i)),
                            _ if def.extracted => match pf {
                                Some(pf) if !pf.is_null(i) => Some(pf.value(i)),
                                _ => None,
                            },
                            _ => None,
                        };
                        value == Some(def.value.as_str())
                    });
                if keep {
                    match ranges.last_mut() {
                        Some(range) if range.end == row => range.end = row + 1,
                        _ => ranges.push(row..row + 1),
                    }
                }
            }
        }
        if ranges.is_empty() {
            return Ok(None);
        }
        Ok(Some(RowSelection::from_consecutive_ranges(
            ranges.into_iter(),
            last_row.max(group_rows),
        )))
    }

    /// One decoded batch, offered row by row.
    #[allow(clippy::too_many_arguments)]
    fn scan_batch(
        &self,
        batch: &RecordBatch,
        tenant: &TenantId,
        line_filters: &[LineFilter],
        time_range: QueryTimeRange,
        forward: bool,
        row_group: u32,
        scan_limit: Option<usize>,
        scan_bytes_limit: Option<u64>,
        cancellation: Option<&AtomicBool>,
        projection: &ScanProjection,
        // A cache-served slice reports its underlying buffers' full capacity,
        // so the hit path charges bytes proportionally itself and turns this
        // off.
        charge_batch_bytes: bool,
        count_scanned_rows: bool,
        empty_labels: &SharedLabels,
        stats: &mut ScanStats,
        sink: &mut dyn RowSink,
    ) -> Result<ScanStep, String> {
        if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
            return Ok(ScanStep::Stop);
        }
        if charge_batch_bytes {
            stats.scanned_bytes = stats
                .scanned_bytes
                .saturating_add(batch.get_array_memory_size() as u64);
        }
        if scan_bytes_limit.is_some_and(|limit| stats.scanned_bytes > limit) {
            return Err(format!(
                "query exceeds the maximum of {} scanned bytes",
                scan_bytes_limit.unwrap_or_default()
            ));
        }
        let row_tenant = batch.column(0).as_string::<i32>();
        let ts = batch.column(1).as_primitive::<Int64Type>();
        let msg = projection
            .msg
            .map(|index| batch.column(index).as_string::<i32>());
        let metadata_cols: Vec<(usize, &StringArray)> = projection
            .metadata
            .iter()
            .map(|&(key_index, batch_index)| {
                (key_index, batch.column(batch_index).as_string::<i32>())
            })
            .collect();
        let parsed_cols: Vec<(usize, &StringArray)> = projection
            .parsed
            .iter()
            .map(|&(key_index, batch_index)| {
                (key_index, batch.column(batch_index).as_string::<i32>())
            })
            .collect();
        let sm = projection
            .residual
            .map(|index| batch.column(index).as_string::<i32>());

        let row_indices: Box<dyn Iterator<Item = usize>> = if forward {
            Box::new(0..batch.num_rows())
        } else {
            Box::new((0..batch.num_rows()).rev())
        };
        for i in row_indices {
            if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
                return Ok(ScanStep::Stop);
            }
            // Row groups are tenant-aligned, so this never rejects a row in a
            // well-formed part. It is kept so isolation does not depend on
            // `meta.json` alone: a metadata bug becomes an empty result rather
            // than a cross-tenant read.
            if row_tenant.value(i) != tenant.as_str() {
                return Err(format!(
                    "part {} row group {row_group} contains rows outside tenant {tenant}",
                    self.part.meta.id
                ));
            }
            let ts_val = ts.value(i);
            if beyond_frontier(sink.frontier_ns(), forward, ts_val) {
                continue;
            }
            if scan_limit.is_some_and(|limit| stats.scanned_rows >= limit) {
                return Ok(ScanStep::Stop);
            }
            // Counted per row examined rather than per batch decoded. The batch
            // is a read granularity the client never asked for, and charging a
            // whole one to `totalLinesProcessed` reported a query that stopped
            // after two rows as having processed a thousand. Counted once: a
            // two-pass scan already charged these rows when the narrow pass
            // examined them.
            if count_scanned_rows {
                stats.scanned_rows = stats.scanned_rows.saturating_add(1);
            }
            if !time_range.contains(ts_val) {
                continue;
            }
            // Filters test the borrowed value; the `String` is built only for
            // a row that survives them. Allocating first charged every
            // rejected row for a line nothing would read. When the line is not
            // projected the filters are guaranteed empty (asserted at scan
            // entry) and the entry carries an empty line nothing will read.
            if let Some(msg) = msg
                && !line_filters.iter().all(|f| f.matches(msg.value(i)))
            {
                continue;
            }
            let line = msg.map(|msg| msg.value(i).to_string()).unwrap_or_default();
            // Rebuilt from the `_sm:` columns plus the residual blob — a merge
            // of two key-sorted lists, so the canonical order survives without
            // a sort. The residual is null for any row whose keys all made
            // columns, which is every row of the intended consumer, so the
            // common path runs no serde at all. The per-row
            // `serde_json::from_str` this replaces was measured as the 1.8x
            // per-line term of `metadata_rare`'s loss.
            let residual: Vec<(String, String)> = match sm {
                Some(sm) if !sm.is_null(i) => {
                    serde_json::from_str(sm.value(i)).map_err(|error| {
                        format!(
                            "invalid structured metadata in part {} at timestamp {ts_val}: {error}",
                            self.part.meta.id
                        )
                    })?
                }
                _ => Vec::new(),
            };
            let mut structured_metadata: Vec<(String, String)> =
                Vec::with_capacity(residual.len() + metadata_cols.len());
            let mut residual = residual.into_iter().peekable();
            for &(key_index, column) in &metadata_cols {
                let key = &self.metadata_keys[key_index];
                while residual.peek().is_some_and(|(name, _)| name < key) {
                    structured_metadata.extend(residual.next());
                }
                if !column.is_null(i) {
                    structured_metadata.push((key.clone(), column.value(i).to_string()));
                }
            }
            structured_metadata.extend(residual);
            // The row's `| json` extraction, straight off the `_pf:` columns.
            // Non-empty means the write-side parse succeeded and every key is
            // here (the projection refuses capped lists); empty is ambiguous
            // between "no scalar fields" and "not JSON", so the sink's
            // pipeline falls back to parsing the line and setting `__error__`
            // itself.
            let extracted_json = (!parsed_cols.is_empty())
                .then(|| {
                    let mut extracted = std::collections::BTreeMap::new();
                    for &(key_index, column) in &parsed_cols {
                        if !column.is_null(i) {
                            extracted.insert(
                                self.parsed_keys[key_index].clone(),
                                column.value(i).to_string(),
                            );
                        }
                    }
                    extracted
                })
                .filter(|extracted| !extracted.is_empty());
            sink.accept_extracted(
                empty_labels,
                LogEntry {
                    timestamp_ns: ts_val,
                    line,
                    structured_metadata,
                },
                extracted_json,
            )?;
        }
        Ok(ScanStep::Continue)
    }
}

/// One exact-field predicate the part's columns can answer per row.
struct DefinitiveColumn {
    value: String,
    /// Index into the part's `_sm:` key list, when the key is columnized.
    sm_key: Option<usize>,
    /// Index into the part's `_pf:` key list, for json-only extraction.
    pf_key: Option<usize>,
    /// Whether `| json` extraction may supply the field when metadata does
    /// not. `false` means the predicate reads pushed metadata alone.
    extracted: bool,
}

/// Where each logical column landed in a projected batch.
///
/// A `ProjectionMask` keeps the projected columns in schema order, so the
/// positions are computable up front; computing them per batch would re-derive
/// the same map thousands of times per scan.
struct ScanProjection {
    /// Parquet leaf ordinals to project. Every column here is a primitive, so
    /// leaf ordinals and top-level ordinals coincide.
    leaves: Vec<usize>,
    /// Projected index of `_msg`, when the line is read at all.
    msg: Option<usize>,
    /// `(index into metadata_keys, projected index)` per projected `_sm:`.
    metadata: Vec<(usize, usize)>,
    /// `(index into parsed_keys, projected index)` per projected `_pf:` —
    /// non-empty only when the caller asked to feed the pipeline's `| json`
    /// from the columns, and only on a part whose parsed-key list is under
    /// its cap (a capped list cannot promise a complete extraction).
    parsed: Vec<(usize, usize)>,
    /// Projected index of the residual blob, when it is read at all.
    residual: Option<usize>,
}

impl ScanProjection {
    fn build(
        metadata_keys: &[String],
        parsed_keys: &[String],
        columns: &ColumnSet,
    ) -> Self {
        let residual_leaf = 3 + metadata_keys.len() + parsed_keys.len();
        let mut leaves = vec![0usize, 1];
        if columns.line {
            leaves.push(2);
        }
        let mut metadata_leaves: Vec<(usize, usize)> = Vec::new();
        let include_residual = match &columns.metadata {
            MetadataProjection::All => {
                for key in 0..metadata_keys.len() {
                    metadata_leaves.push((key, 3 + key));
                }
                true
            }
            MetadataProjection::Named(names) => {
                for (key_index, key) in metadata_keys.iter().enumerate() {
                    if names.contains(key) {
                        metadata_leaves.push((key_index, 3 + key_index));
                    }
                }
                // The residual is read only when a named field might live in
                // it — a name that is not one of this part's columns. The
                // columnization invariant makes this exact: a columnized key
                // never also appears in a row's residual.
                names
                    .iter()
                    .any(|name| metadata_keys.binary_search(name).is_err())
            }
        };
        let mut parsed_leaves: Vec<(usize, usize)> = Vec::new();
        // A capped parsed-key list cannot promise a complete extraction, and
        // an incomplete one silently changes what `| json` produces — so the
        // shortcut is offered only when every extracted key has a column.
        if columns.parsed_fields && parsed_keys.len() < MAX_METADATA_COLUMNS {
            for key in 0..parsed_keys.len() {
                parsed_leaves.push((key, 3 + metadata_keys.len() + key));
            }
        }
        leaves.extend(metadata_leaves.iter().map(|&(_, leaf)| leaf));
        leaves.extend(parsed_leaves.iter().map(|&(_, leaf)| leaf));
        if include_residual {
            leaves.push(residual_leaf);
        }
        leaves.sort_unstable();
        leaves.dedup();
        // The projected batch follows schema order, so every position is the
        // leaf's rank in the sorted mask — derived, never counted, which is
        // what made an interleaving mistake possible when it was counted.
        let rank = |leaf: usize| -> usize {
            leaves
                .binary_search(&leaf)
                .expect("every projected leaf is in the mask")
        };
        Self {
            msg: columns.line.then(|| rank(2)),
            metadata: metadata_leaves
                .iter()
                .map(|&(key, leaf)| (key, rank(leaf)))
                .collect(),
            parsed: parsed_leaves
                .iter()
                .map(|&(key, leaf)| (key, rank(leaf)))
                .collect(),
            residual: include_residual.then(|| rank(residual_leaf)),
            leaves,
        }
    }

    /// This projection's fields re-addressed into another decode's batch
    /// layout — how a narrower scan reads batches the cache decoded under
    /// the full projection. `None` when the layout is missing a leaf this
    /// projection needs (the cache's `all()` never carries `_pf:` columns).
    /// The per-row work is the narrow scan's own: a field the view does not
    /// name is never materialized, however many columns the batch holds.
    fn view_in(&self, layout: &ScanProjection) -> Option<ScanProjection> {
        let rank_of = |position_in_self: usize| -> Option<usize> {
            let leaf = self.leaves[position_in_self];
            layout.leaves.binary_search(&leaf).ok()
        };
        Some(ScanProjection {
            msg: match self.msg {
                Some(position) => Some(rank_of(position)?),
                None => None,
            },
            metadata: self
                .metadata
                .iter()
                .map(|&(key, position)| Some((key, rank_of(position)?)))
                .collect::<Option<_>>()?,
            parsed: self
                .parsed
                .iter()
                .map(|&(key, position)| Some((key, rank_of(position)?)))
                .collect::<Option<_>>()?,
            residual: match self.residual {
                Some(position) => Some(rank_of(position)?),
                None => None,
            },
            leaves: layout.leaves.clone(),
        })
    }
}

#[derive(PartialEq, Eq)]
enum ScanStep {
    Continue,
    /// Nothing further in the part can enter the result: a limit was reached or
    /// the query was cancelled, neither of which depends on order.
    Stop,
}

/// How many rows one read of a row group should decode.
///
/// A read larger than the sink can still take is mostly decoded for nothing; a
/// tiny one pays Arrow's per-batch cost per row. The floor is what keeps a
/// `limit=1` from reading one row at a time through a whole window, and the
/// ceiling is the batch size this reader always used.
fn window_rows(scan_limit: Option<usize>, scanned_rows: usize, sink: &dyn RowSink) -> usize {
    scan_limit
        .map(|limit| limit.saturating_sub(scanned_rows))
        .into_iter()
        .chain(sink.remaining().map(|remaining| remaining.max(256)))
        .min()
        .unwrap_or(1024)
        .clamp(1, 1024)
}

impl PartReader {
    fn bloom_prune(&self, blooms: &BloomIndex, rg: usize, literals: &[String]) -> bool {
        let line = blooms.bloom(blooms.line[rg]);
        literals
            .iter()
            .all(|literal| line.might_contain_substr(literal))
    }

    fn exact_field_bloom_prune(
        &self,
        blooms: &BloomIndex,
        rg: usize,
        exact_fields: &[ExactFieldPredicate],
    ) -> bool {
        self.exact_field_window_mask(blooms, rg, exact_fields) != Some(0)
    }

    /// Which of the row group's [`BLOOM_WINDOW_ROWS`]-row windows may hold a
    /// row matching *every* predicate.
    ///
    /// `None` means the blooms constrain nothing — every predicate was in a
    /// class the filters cannot answer (stream label already proven by the
    /// index, empty value, unrepresentable token). `Some(mask)` restricts a
    /// match to the set windows; `Some(0)` prunes the group. Masks AND across
    /// predicates, which is sound because a row matching all of them carries
    /// all of their tokens in its *own* window — strictly stronger than the
    /// old any-window-per-predicate admission and just as free of false
    /// negatives.
    fn exact_field_window_mask(
        &self,
        blooms: &BloomIndex,
        rg: usize,
        exact_fields: &[ExactFieldPredicate],
    ) -> Option<u64> {
        let mut combined: Option<u64> = None;
        for predicate in exact_fields {
            let Some(mask) = self.predicate_window_mask(blooms, rg, predicate) else {
                continue;
            };
            let mask = match combined {
                Some(existing) => existing & mask,
                None => mask,
            };
            if mask == 0 {
                return Some(0);
            }
            combined = Some(mask);
        }
        combined
    }

    fn predicate_window_mask(
        &self,
        blooms: &BloomIndex,
        rg: usize,
        predicate: &ExactFieldPredicate,
    ) -> Option<u64> {
        // Field-filter execution may treat an absent field as an empty
        // string. Absence is not represented in the bloom, so an empty
        // equality cannot safely reject anything.
        if predicate.value.is_empty() {
            return None;
        }
        // No windows at all means the row group indexed no exact-field
        // token, so this predicate cannot match. That is the same answer
        // the all-zero filter this used to store would have given.
        let windows = &blooms.exact_fields[rg];
        if windows.is_empty() {
            return Some(0);
        }
        let Ok(token) = encode_exact_field_token(&predicate.name, &predicate.value) else {
            // An unrepresentable predicate must conservatively scan.
            return None;
        };
        let mut mask = 0u64;
        for (window, bloom) in windows.iter().enumerate() {
            let admitted = match bloom {
                WindowBloom::Absent => false,
                WindowBloom::Saturated => true,
                WindowBloom::Filter(at) => blooms.bloom(*at).contains(&token),
            };
            if admitted {
                mask |= 1u64 << window;
            }
        }
        Some(mask)
    }
}

impl Drop for PartReader {
    fn drop(&mut self) {
        // A merged-away or retention-deleted part gives its bloom bytes back
        // now rather than waiting to be chosen as an eviction victim.
        self.blooms.remove();
    }
}

/// The bloom sidecar, mapped rather than read.
///
/// `fs::read` copied the whole of `index.bin` — 3 MiB on the soak rig — into
/// an owned buffer, decoded it, and dropped the buffer. That is a large
/// allocate-and-free per cache miss, and the soak measured what it costs: once
/// the bloom working set outgrew `sidecar_cache_max_bytes`, misses ran at
/// ~200/s and re-read **379 MiB/s**, while jemalloc's unreturned dirty pages
/// went 297 -> 953 MiB with live memory flat. The bytes were already in the
/// page cache; copying them into the heap produced a second, anonymous
/// representation whose only role was to be freed.
///
/// Mapping removes that copy. The bytes stay file-backed, so the kernel can
/// reclaim them under pressure and read them again from a file that has not
/// changed — where an allocator's freed pages are anonymous and, with swap
/// off, cannot be reclaimed at all. This does not take them out of the
/// cgroup's accounting; it takes them out of the half of it that an OOM kill
/// is decided on.
///
/// Safety rests on the same invariant `series_part::map_part_file` documents:
/// a committed part is write-once. `index.bin` is written and fsynced before
/// the part is published and is never modified in place; retirement and cache
/// eviction `remove_file` it, and an unlinked file keeps its inode while a
/// mapping holds it. Nothing truncates a mapped part, which is the case that
/// would fault a reader.
fn map_index_file(path: &std::path::Path) -> Result<memmap2::Mmap, String> {
    let file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    // SAFETY: parts are write-once. See the note above.
    unsafe { memmap2::Mmap::map(&file) }.map_err(|error| format!("{}: {error}", path.display()))
}

/// Byte offset of the bloom section inside `index.bin`: the magic, then the
/// four-byte length `split_index` reads.
fn index_bloom_offset(buf: &[u8]) -> Result<usize, String> {
    let base = crate::part::INDEX_MAGIC.len() + 4;
    if buf.len() < base {
        return Err("part index file is truncated".to_string());
    }
    Ok(base)
}

/// Walks `index.bin` once and records where each stored bloom sits.
///
/// The same validation the decoder did — magic, row-group count, window count
/// and its ceiling, no trailing bytes — with the bitsets left in the mapping
/// instead of copied out. Every location is bounds-checked here so a lookup
/// can slice without one.
fn build_bloom_index(
    map: Arc<memmap2::Mmap>,
    row_group_rows: &[u32],
) -> Result<BloomIndex, String> {
    // `index.bin` is the index header followed by the bloom section;
    // `split_index` proves the header and hands back that section. The
    // locations recorded are absolute in the mapping, so `base` carries into
    // every one.
    let base = index_bloom_offset(&map)?;
    let (line, exact_fields) = locate_blooms(split_index(&map)?, base, row_group_rows)?;
    Ok(BloomIndex {
        map,
        line,
        exact_fields,
    })
}

/// Walks the bloom section once and records where each stored bloom sits.
///
/// The same validation the decoder did — magic, row-group count, window count
/// and its ceiling, no trailing bytes — with the bitsets left where they are
/// instead of copied out. Every location is bounds-checked here so a lookup
/// can slice without one. Split from the mapping so the framing rules can be
/// tested on bytes rather than on a file.
#[allow(clippy::type_complexity)]
fn locate_blooms(
    buf: &[u8],
    base: usize,
    row_group_rows: &[u32],
) -> Result<(Vec<BloomLocation>, Vec<Vec<WindowBloom>>), String> {
    let expected_count = row_group_rows.len();
    if buf.len() < 8 {
        return Err("bloom file too short".to_string());
    }
    if &buf[0..4] != BLOOM_MAGIC {
        return Err("bloom magic mismatch".to_string());
    }
    let count = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
    if count != expected_count {
        return Err(format!(
            "row group count mismatch: bloom says {count}, metadata says {expected_count}"
        ));
    }
    let mut pos = 8;
    let mut line = Vec::with_capacity(count);
    let mut exact_fields = Vec::with_capacity(count);
    for (group, rows) in row_group_rows.iter().enumerate() {
        line.push(locate_length_prefixed_bloom(buf, base, &mut pos)?);
        let window_count = {
            let end = pos
                .checked_add(4)
                .ok_or_else(|| "bloom window count overflow".to_string())?;
            let bytes: [u8; 4] = buf
                .get(pos..end)
                .ok_or_else(|| "bloom window count truncated".to_string())?
                .try_into()
                .expect("length checked");
            pos = end;
            u32::from_le_bytes(bytes) as usize
        };
        let rows = *rows as usize;
        let expected_windows = rows.div_ceil(crate::part::BLOOM_WINDOW_ROWS);
        if window_count != 0 && window_count != expected_windows {
            return Err(format!(
                "bloom window count mismatch: group {group} has {rows} rows, \
expected {expected_windows} windows, found {window_count}"
            ));
        }
        if window_count > 64 {
            return Err(format!(
                "bloom window count {window_count} exceeds the 64-window limit"
            ));
        }
        let mut windows = Vec::with_capacity(window_count);
        for _ in 0..window_count {
            windows.push(locate_window_bloom(buf, base, &mut pos)?);
        }
        exact_fields.push(windows);
    }
    if pos != buf.len() {
        return Err("bloom file has trailing bytes".to_string());
    }
    Ok((line, exact_fields))
}

/// One window slot: a zero length says the window indexed no exact-field
/// token — a fact the reader prunes on; the saturation sentinel says the
/// window held more tokens than a filter may be sized for — nothing is
/// pruned; any other length decodes as an ordinary filter.
fn locate_window_bloom(buf: &[u8], base: usize, pos: &mut usize) -> Result<WindowBloom, String> {
    match buf.get(*pos..*pos + 4) {
        Some(&[0, 0, 0, 0]) => {
            *pos += 4;
            Ok(WindowBloom::Absent)
        }
        Some(bytes)
            if bytes == crate::part::SATURATED_WINDOW_SENTINEL.to_le_bytes() =>
        {
            *pos += 4;
            Ok(WindowBloom::Saturated)
        }
        _ => locate_length_prefixed_bloom(buf, base, pos).map(WindowBloom::Filter),
    }
}

fn locate_length_prefixed_bloom(
    buf: &[u8],
    base: usize,
    pos: &mut usize,
) -> Result<BloomLocation, String> {
    let length_end = pos
        .checked_add(4)
        .ok_or_else(|| "bloom length overflow".to_string())?;
    let length_bytes: [u8; 4] = buf
        .get(*pos..length_end)
        .ok_or_else(|| "bloom length truncated".to_string())?
        .try_into()
        .map_err(|_| "bloom length truncated".to_string())?;
    let len = u32::from_le_bytes(length_bytes) as usize;
    *pos = length_end;
    let payload_end = pos
        .checked_add(len)
        .ok_or_else(|| "bloom payload length overflow".to_string())?;
    let payload = buf
        .get(*pos..payload_end)
        .ok_or_else(|| "bloom payload truncated".to_string())?;
    // Validates the header on the same grounds the owning decoder used, and
    // proves the slice below is in range so a lookup needs no check.
    let (num_bits, k) = crate::bloom::decode_bloom_header(payload)?;
    let bits_start = *pos + crate::bloom::BLOOM_HEADER_BYTES;
    *pos = payload_end;
    Ok(BloomLocation {
        offset: u32::try_from(base + bits_start)
            .map_err(|_| "bloom offset exceeds 4 GiB".to_string())?,
        len: u32::try_from(payload_end - bits_start)
            .map_err(|_| "bloom payload exceeds 4 GiB".to_string())?,
        num_bits: u32::try_from(num_bits).map_err(|_| "bloom num_bits overflow".to_string())?,
        k,
    })
}

pub fn group_by_labels(collected: Vec<(SharedLabels, LogEntry)>) -> Vec<StreamResult> {
    let mut grouped: BTreeMap<SharedLabels, Vec<LogEntry>> = BTreeMap::new();
    for (labels, entry) in collected {
        grouped.entry(labels).or_default().push(entry);
    }
    grouped
        .into_iter()
        .map(|(labels, entries)| StreamResult { labels, entries })
        .collect()
}
