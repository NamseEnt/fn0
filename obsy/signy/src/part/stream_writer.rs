/// Writes a part without ever holding it.
///
/// The batch path materializes the whole part as a `Vec<Row>` before a byte is
/// written, and [`docs/MEMORY_ATTRIBUTION.md`] measured what that costs merge:
/// **829 of 847 live megabytes** at the settle peak, across **7.2 million live
/// allocations**, which is why the budget gate is red at 2 GiB.
///
/// Nothing about the format required that. Every file a part is made of is
/// produced a row group at a time — Parquet flushes per row group already, the
/// blooms are per row group, and `meta.json`'s extremes, tenant segments and
/// counts all accumulate. The one thing that genuinely has to be known before
/// the first row group is the **schema**, because it names a column per
/// metadata key — and a merge can read that from its inputs' `meta.json`
/// without touching a row.
///
/// So this holds one row group and a set of counters. The output is the same
/// bytes the batch path writes, and it is the same functions that write them:
/// `encode_group_blooms` and `row_group_batch` are shared, so the two cannot
/// drift into producing different parts.
///
/// **Rows must arrive sorted by `(tenant, timestamp_ns, …)` and deduplicated.**
/// [`MergedRows`] is what guarantees that. A row group is cut when the tenant
/// changes or `row_group_size` is reached, which is exactly what
/// `row_group_bounds` computes for the batch path.
pub struct StreamingPartWriter {
    schema: Arc<Schema>,
    writer: ArrowWriter<fs::File>,
    data_path: PathBuf,
    /// The keys this part stores as `_sm:` columns, chosen before the first
    /// row: a merge sums its inputs' recorded per-key row counts and takes the
    /// same top-N `select_metadata_columns` takes — deterministic without
    /// reading a row. The counts are
    /// re-counted during `push`, so keys whose rows retention or a delete
    /// dropped do not inflate the next merge's choice.
    metadata_keys: Vec<String>,
    metadata_counts: BTreeMap<String, u64>,
    parsed_keys: Vec<String>,
    parsed_counts: BTreeMap<String, u64>,
    row_group_size: usize,
    group: Vec<Row>,
    /// `parsed_json_fields` of each `group` row, computed once in `push` and
    /// consumed by both the counts and the column fill.
    group_parsed: Vec<Option<BTreeMap<String, String>>>,

    bloom_sections: Vec<Vec<u8>>,

    row_count: u64,
    min_ts_ns: i64,
    max_ts_ns: i64,
    materialized_bytes: u64,
    row_group_min_ts: Vec<i64>,
    row_group_max_ts: Vec<i64>,
    row_group_rows: Vec<u32>,
    row_group_ts_monotonic: Vec<bool>,
    tenants: Vec<TenantSegment>,
}

impl StreamingPartWriter {
    pub fn create(
        dir: &Path,
        metadata_keys: Vec<String>,
        parsed_keys: Vec<String>,
        row_group_size: usize,
    ) -> io::Result<Self> {
        let schema = part_schema(&metadata_keys, &parsed_keys);
        let data_path = dir.join(DATA_FILE);
        let file = fs::File::create(&data_path)?;
        let props = part_writer_properties(row_group_size);
        let writer =
            ArrowWriter::try_new(file, schema.clone(), Some(props)).map_err(io::Error::other)?;
        Ok(Self {
            schema,
            writer,
            data_path,
            metadata_keys,
            metadata_counts: BTreeMap::new(),
            parsed_keys,
            parsed_counts: BTreeMap::new(),
            row_group_size,
            group: Vec::new(),
            group_parsed: Vec::new(),
            bloom_sections: Vec::new(),
            row_count: 0,
            min_ts_ns: i64::MAX,
            max_ts_ns: i64::MIN,
            materialized_bytes: 0,
            row_group_min_ts: Vec::new(),
            row_group_max_ts: Vec::new(),
            row_group_rows: Vec::new(),
            row_group_ts_monotonic: Vec::new(),
            tenants: Vec::new(),
        })
    }

    pub fn push(&mut self, row: Row) -> io::Result<()> {
        // The same rule `row_group_bounds` applies to a slice: one tenant per
        // group, cut on size.
        let cut = self
            .group
            .first()
            .is_some_and(|first| first.tenant != row.tenant)
            || self.group.len() >= self.row_group_size;
        if cut {
            self.flush_group()?;
        }
        self.row_count += 1;
        self.min_ts_ns = self.min_ts_ns.min(row.timestamp_ns);
        self.max_ts_ns = self.max_ts_ns.max(row.timestamp_ns);
        self.materialized_bytes = self
            .materialized_bytes
            .saturating_add(row.materialized_bytes());
        for (name, _) in &row.structured_metadata {
            if self.metadata_keys.binary_search(name).is_ok() {
                *self.metadata_counts.entry(name.clone()).or_default() += 1;
            }
        }
        let parsed = if self.parsed_keys.is_empty() {
            None
        } else {
            crate::logql::parsed_json_fields(&row.line)
        };
        if let Some(fields) = &parsed {
            for name in fields.keys() {
                if self.parsed_keys.binary_search(name).is_ok() {
                    *self.parsed_counts.entry(name.clone()).or_default() += 1;
                }
            }
        }
        self.group_parsed.push(parsed);
        self.group.push(row);
        Ok(())
    }

    fn flush_group(&mut self) -> io::Result<()> {
        if self.group.is_empty() {
            return Ok(());
        }
        let ordinal = self.row_group_min_ts.len() as u32;
        let batch = row_group_batch(
            &self.schema,
            &self.group,
            &self.group_parsed,
            &self.metadata_keys,
            &self.parsed_keys,
        )?;
        self.writer.write(&batch).map_err(io::Error::other)?;
        // The sidecars address row groups by ordinal, so a flush per batch pins
        // the boundary rather than letting the writer pick one that straddles a
        // tenant. Same reason the batch path flushes here.
        self.writer.flush().map_err(io::Error::other)?;
        self.bloom_sections
            .push(encode_group_blooms(&self.group, &self.group_parsed)?);

        let first = self.group.first().expect("checked non-empty");
        let min_ts_ns = self
            .group
            .iter()
            .map(|row| row.timestamp_ns)
            .min()
            .unwrap_or_default();
        let max_ts_ns = self
            .group
            .iter()
            .map(|row| row.timestamp_ns)
            .max()
            .unwrap_or_default();
        self.row_group_min_ts.push(min_ts_ns);
        self.row_group_max_ts.push(max_ts_ns);
        self.row_group_rows.push(self.group.len() as u32);
        self.row_group_ts_monotonic.push(
            self.group
                .windows(2)
                .all(|pair| pair[0].timestamp_ns <= pair[1].timestamp_ns),
        );
        match self.tenants.last_mut() {
            Some(segment) if segment.tenant == first.tenant => {
                segment.row_group_end = ordinal + 1;
                segment.row_count += self.group.len() as u64;
                segment.min_ts_ns = segment.min_ts_ns.min(min_ts_ns);
                segment.max_ts_ns = segment.max_ts_ns.max(max_ts_ns);
            }
            // `bytes` and `crc32` are filled in by `finish`: the writer only
            // knows where a row group landed once it has closed the file.
            _ => self.tenants.push(TenantSegment {
                tenant: first.tenant.clone(),
                row_group_start: ordinal,
                row_group_end: ordinal + 1,
                row_count: self.group.len() as u64,
                min_ts_ns,
                max_ts_ns,
                bytes: ByteRange::default(),
                crc32: 0,
            }),
        }

        self.group.clear();
        self.group_parsed.clear();
        Ok(())
    }

    /// Whether any row was ever pushed. An empty stream must not leave a part.
    pub fn is_empty(&self) -> bool {
        self.row_count == 0
    }

    pub fn min_ts_ns(&self) -> i64 {
        self.min_ts_ns
    }

    pub fn finish(mut self, dir: &Path, id: &str, partition: &str) -> io::Result<()> {
        self.flush_group()?;
        let parquet_metadata = self.writer.close().map_err(io::Error::other)?;
        sync_file(&self.data_path)?;
        // Synced, therefore clean — and dropped from the page cache on
        // purpose, so the write stream cannot ride `memory.current` into
        // `memory.max`'s reclaim stall (see `crate::page_cache`). A query
        // that wants this part re-reads it once.
        crate::page_cache::drop_cache(&self.data_path);

        let mut blooms = Vec::new();
        blooms.extend_from_slice(BLOOM_MAGIC);
        blooms.extend_from_slice(&(self.bloom_sections.len() as u32).to_le_bytes());
        for section in &self.bloom_sections {
            blooms.extend_from_slice(section);
        }
        write_index_sections(&dir.join(INDEX_FILE), &blooms)?;

        // A tenant's extent is the span of its row groups, which the writer
        // only learns from the metadata it just wrote. The segments were
        // accumulated as the groups were cut, so filling the ranges in is a
        // walk over them rather than a second pass over the rows.
        let row_group_ranges = row_group_byte_ranges(&parquet_metadata);
        if row_group_ranges.len() != self.row_group_rows.len() {
            return Err(io::Error::other(format!(
                "parquet wrote {} row groups for {} cut groups",
                row_group_ranges.len(),
                self.row_group_rows.len()
            )));
        }
        let mut tenants = self.tenants;
        for segment in &mut tenants {
            let mut start = u64::MAX;
            let mut end = 0u64;
            for range in &row_group_ranges
                [segment.row_group_start as usize..segment.row_group_end as usize]
            {
                start = start.min(range.start);
                end = end.max(range.end);
            }
            segment.bytes = ByteRange { start, end };
        }
        // One pass over the body for both checksums, the same as the batch
        // path: the segments are contiguous, ordered and non-overlapping.
        let data_crc32 = checksum_data_and_segments(&self.data_path, &mut tenants)?;
        let integrity = PartIntegrity {
            data_crc32,
            index_crc32: file_crc32(&dir.join(INDEX_FILE))?,
            metadata_crc32: 0,
        };
        let mut meta = MetaFile {
            id: id.to_string(),
            partition: partition.to_string(),
            min_ts_ns: if self.row_count == 0 {
                0
            } else {
                self.min_ts_ns
            },
            max_ts_ns: if self.row_count == 0 {
                0
            } else {
                self.max_ts_ns
            },
            row_count: self.row_count,
            row_group_count: self.row_group_min_ts.len() as u32,
            row_group_min_ts: self.row_group_min_ts,
            row_group_max_ts: self.row_group_max_ts,
            row_group_rows: self.row_group_rows,
            row_group_ts_monotonic: self.row_group_ts_monotonic,
            tenants,
            materialized_bytes: self.materialized_bytes,
            // Every declared key stays listed, even at a count of zero:
            // the Parquet file carries its (all-null) column either way, and
            // `meta.json`'s key list is what the schema check rebuilds.
            metadata_columns: self
                .metadata_keys
                .iter()
                .map(|key| {
                    (
                        key.clone(),
                        self.metadata_counts.get(key).copied().unwrap_or(0),
                    )
                })
                .collect(),
            parsed_columns: self
                .parsed_keys
                .iter()
                .map(|key| (key.clone(), self.parsed_counts.get(key).copied().unwrap_or(0)))
                .collect(),
            integrity,
        };
        meta.integrity.metadata_crc32 = metadata_crc32(&meta).map_err(io::Error::other)?;
        let encoded = serde_json::to_string(&meta).map_err(io::Error::other)?;
        fs::write(dir.join(META_FILE), encoded)?;
        sync_file(&dir.join(META_FILE))?;
        Ok(())
    }
}
