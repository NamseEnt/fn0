/// Number of parts currently eligible for but not yet merged: the point-in-time
/// merge backlog. Runs the scheduler's own `select_groups`, so the count covers
/// both kinds of pending work — small parts accumulating into an ordinary group,
/// and parts a retention rewrite has made eligible on their own. Counting only
/// the first left retention-driven backlog invisible to operators, with
/// `retention_rewrite_skipped` reporting only the rewrites that had already
/// failed.
pub fn merge_debt_part_count(
    registry: &PartRegistry,
    config: &Config,
    cutoffs: Option<&Cutoffs>,
    deletes: &crate::delete_requests::DeleteMasks,
    now_ns: i64,
) -> usize {
    let readers = registry.snapshot();
    if readers.is_empty() {
        return 0;
    }
    let mut by_partition: HashMap<String, Vec<Arc<PartReader>>> = HashMap::new();
    for reader in readers {
        by_partition
            .entry(reader.meta().partition.clone())
            .or_default()
            .push(reader);
    }
    let mut debt = 0usize;
    for (_partition, mut parts) in by_partition {
        parts.sort_by_key(|reader| reader.meta().row_count);
        for group in select_groups(&parts, config, cutoffs, deletes, now_ns) {
            debt += group.parts.len();
        }
    }
    debt
}

/// One group of merge inputs and the reason it was selected.
struct MergeGroup {
    parts: Vec<Arc<PartReader>>,
    /// The group is below `merge_min_part_count`, so nothing but retention
    /// would have picked it. Its only product is reclaimed bytes, which makes
    /// a failure to process it a missed optimization rather than a merge that
    /// did not happen.
    retention_only: bool,
}

/// Merge groups for one partition, including the ones retention needs.
///
/// An ordinary group must reach `merge_min_part_count` to be worth the write.
/// A part whose expired share reaches `retention_rewrite_threshold` is worth
/// it on its own, so it is admitted as a group of one — even when it is too
/// large for `group_for_merge` to consider at all.
///
/// A tenant at zero retention is a deleted tenant, and its rows are reclaimed
/// regardless of the threshold: otherwise "deletion" would leave rows in a
/// large part indefinitely, which is not a thing the word can mean.
///
/// A deletion request is the same sentence with a narrower subject, so it is
/// admitted on the same terms. Without this, a part large enough that no
/// ordinary group would take it keeps the rows somebody asked to have removed
/// until retention happens to delete the whole part — the rows are already
/// unreadable, but "deleted" would be describing a mask rather than a removal.
fn select_groups(
    parts: &[Arc<PartReader>],
    config: &Config,
    cutoffs: Option<&Cutoffs>,
    deletes: &crate::delete_requests::DeleteMasks,
    now_ns: i64,
) -> Vec<MergeGroup> {
    let min_part_count = config.merge_min_part_count.max(2);
    // A partition nothing has written to for an hour will not fill a tier by
    // itself, and leaving its handful of parts unmerged costs every query that
    // opens them.
    let idle_before_ns = now_ns.saturating_sub(IDLE_PARTITION_AGE.as_nanos() as i64);
    let partition_is_idle = parts
        .iter()
        .all(|reader| reader.meta().max_ts_ns < idle_before_ns);
    let needs_rewrite = |reader: &Arc<PartReader>| {
        if deletes.may_cover_part(reader.meta()) {
            return true;
        }
        cutoffs.is_some_and(|cutoffs| {
            cutoffs.holds_zero_retention_rows(reader.meta())
                || cutoffs.expired_log_row_fraction(reader.meta())
                    >= config.retention_rewrite_threshold
        })
    };

    let mut selected: Vec<MergeGroup> = Vec::new();
    let mut grouped: std::collections::HashSet<String> = std::collections::HashSet::new();
    for group in group_for_merge(parts, config) {
        let age_admits = partition_is_idle && group.len() >= 2;
        if group.len() < min_part_count && !age_admits && !group.iter().any(needs_rewrite) {
            continue;
        }
        for reader in &group {
            grouped.insert(reader.meta().id.clone());
        }
        selected.push(MergeGroup {
            retention_only: group.len() < min_part_count && !age_admits,
            parts: group,
        });
    }
    for reader in parts {
        if needs_rewrite(reader) && !grouped.contains(&reader.meta().id) {
            selected.push(MergeGroup {
                parts: vec![reader.clone()],
                retention_only: true,
            });
        }
    }
    selected
}

/// Groups within one size tier, never across them.
///
/// Grouping by row count alone put the part a partition has been accumulating
/// into a group with each flush's new parts, so every pass rewrote the whole
/// of it to absorb a few rows. See [`crate::compaction_tier`].
fn group_for_merge(parts: &[Arc<PartReader>], config: &Config) -> Vec<Vec<Arc<PartReader>>> {
    let mut by_tier: std::collections::BTreeMap<u32, Vec<Arc<PartReader>>> =
        std::collections::BTreeMap::new();
    for reader in parts {
        if reader.meta().row_count >= config.merge_max_part_rows {
            // Do not add an already-large part to a merge group.
            continue;
        }
        by_tier
            .entry(crate::compaction_tier::tier_of(estimated_part_bytes(reader)))
            .or_default()
            .push(reader.clone());
    }
    by_tier
        .into_values()
        .flat_map(|tier| group_within_tier(&tier, config))
        .collect()
}

fn group_within_tier(parts: &[Arc<PartReader>], config: &Config) -> Vec<Vec<Arc<PartReader>>> {
    let mut groups: Vec<Vec<Arc<PartReader>>> = Vec::new();
    let mut current: Vec<Arc<PartReader>> = Vec::new();
    let mut current_rows: u64 = 0;
    let mut current_bytes: u64 = 0;
    // A merge must always make progress. Treat the target row count as a soft
    // limit until enough parts have accumulated; otherwise, for example, four
    // 300k-row parts with a 1M target are split into groups of 3 and 1 and are
    // skipped forever by merge_min_part_count=4.
    let min_part_count = config.merge_min_part_count.max(2);
    // Paging costs at least MIN_STREAM_PAGE_BYTES per input part, so a group
    // larger than the paging budget can pay for is a group whose rewrite is
    // over budget before it reads a row. A chunked flush produces several
    // small parts per flush, which is exactly how groups grow past it.
    let max_group_parts = (merge_paging_budget(config.merge_max_memory_bytes)
        / MIN_STREAM_PAGE_BYTES)
        .max(min_part_count as u64) as usize;
    for r in parts {
        let next_rows = current_rows.saturating_add(r.meta().row_count);
        let next_bytes = current_bytes.saturating_add(estimated_part_bytes(r));
        if current.len() >= min_part_count && next_bytes > config.merge_max_input_bytes {
            groups.push(std::mem::take(&mut current));
            current_rows = 0;
            current_bytes = 0;
        }
        if !current.is_empty() && next_rows > config.merge_max_part_rows {
            // The target is soft, but the maximum is a hard output bound.
            // Finalize the current group before adding a part that would make
            // the replacement oversized. merge_once will skip it if it does
            // not contain enough inputs to make progress.
            groups.push(std::mem::take(&mut current));
            current_rows = 0;
            current_bytes = 0;
        }

        current_rows = current_rows.saturating_add(r.meta().row_count);
        current_bytes = current_bytes.saturating_add(estimated_part_bytes(r));
        current.push(r.clone());
        if current.len() >= max_group_parts
            || (current.len() >= min_part_count && current_rows >= config.merge_target_part_rows)
        {
            groups.push(std::mem::take(&mut current));
            current_rows = 0;
            current_bytes = 0;
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

/// What reading this part will materialize, straight from `meta.json`.
///
/// Replaces a `stat` of the compressed file: that number was both the wrong
/// unit for the budgets it was compared against and a syscall per part on
/// every `/metrics` scrape.
fn estimated_part_bytes(reader: &PartReader) -> u64 {
    reader.meta().materialized_bytes
}

/// Most bytes of one part's rows a stream holds while paging.
///
/// A page is per **input part**, so a group's paging liveness is this times
/// its part count — which is why [`rewrite_group`] shrinks the page below
/// this ceiling when `merge_max_memory_bytes / 2` divided by the group is
/// smaller, and why [`group_for_merge`] caps the group's part count at what
/// that budget can page at [`MIN_STREAM_PAGE_BYTES`]. Before either bound
/// existed, a 55-part group paged 8 MiB each and the rewrite's liveness was
/// whatever the backlog happened to be.
const STREAM_PAGE_BYTES: u64 = 8 * 1024 * 1024;

/// Floor under the adaptive page: below this a page is smaller than one row
/// group's worth of rows for typical lines, and the stream re-reads the same
/// group over and over.
const MIN_STREAM_PAGE_BYTES: u64 = 2 * 1024 * 1024;

/// Half of `merge_max_memory_bytes`: the input-paging share of the budget,
/// the other half being the writer's row group, index and bloom state.
fn merge_paging_budget(merge_max_memory_bytes: u64) -> u64 {
    (merge_max_memory_bytes / 2).max(MIN_STREAM_PAGE_BYTES)
}

#[cfg(test)]
fn read_all_rows(readers: &[Arc<PartReader>]) -> Result<Vec<part::Row>, String> {
    let mut merged = part::MergedRows::new(readers, STREAM_PAGE_BYTES);
    let mut rows = Vec::new();
    while let Some(row) = merged.next_row()? {
        rows.push(row);
    }
    Ok(rows)
}

/// What one group's rewrite produced.
pub struct GroupRewrite {
    pub new_parts: Vec<part::Part>,
    pub dropped_rows: usize,
    pub kept_rows: usize,
}

/// Read a group, drop what has expired, and write the survivors.
///
/// Reading and writing are interleaved rather than staged, because the whole
/// point of splitting an oversized group is to keep peak memory bounded:
/// holding every batch until the end would cost exactly what materializing the
/// group at once costs.
///
/// However many output parts this produces, they all carry the same merge
/// tombstone naming `old_dirs`, so the commit that follows still replaces the
/// inputs in one transaction.
pub fn rewrite_group(
    readers: &[Arc<PartReader>],
    cutoffs: Option<&Cutoffs>,
    deletes: &crate::delete_requests::DeleteMasks,
    parts_root: &Path,
    row_group_size: usize,
    merge_max_memory_bytes: u64,
    old_dirs: &[PathBuf],
) -> Result<GroupRewrite, String> {
    if readers.is_empty() {
        return Ok(GroupRewrite {
            new_parts: Vec::new(),
            dropped_rows: 0,
            kept_rows: 0,
        });
    }
    // Both come from the inputs' `meta.json` rather than from their rows,
    // because the output schema names a column per stream label and has to
    // exist before the first row group is written. Reading them here is what
    // makes streaming the rewrite possible at all.
    let partition = readers[0].meta().partition.clone();
    if let Some(other) = readers
        .iter()
        .find(|reader| reader.meta().partition != partition)
    {
        // Selection groups within one partition. If that ever stops being true
        // the rows would be filed under the wrong day, which no read path could
        // detect, so it is checked rather than assumed.
        return Err(format!(
            "merge group spans partitions {} and {}",
            partition,
            other.meta().partition
        ));
    }
    // The metadata columns the same way: sum the inputs' recorded per-key row
    // counts and take the same top-N the batch writer takes. Deterministic
    // without reading a row — and it is why the counts are in `meta.json` at
    // all. The writer re-counts during its own pass, so rows that retention or
    // a delete drops below do not inflate the next merge's choice.
    let mut metadata_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    for reader in readers {
        for (key, count) in &reader.meta().metadata_columns {
            *metadata_counts.entry(key.clone()).or_default() += count;
        }
    }
    let metadata_keys: Vec<String> = part::select_metadata_columns(metadata_counts)
        .into_iter()
        .map(|(key, _)| key)
        .collect();
    let mut parsed_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    for reader in readers {
        for (key, count) in &reader.meta().parsed_columns {
            *parsed_counts.entry(key.clone()).or_default() += count;
        }
    }
    let parsed_keys: Vec<String> = part::select_metadata_columns(parsed_counts)
        .into_iter()
        .map(|(key, _)| key)
        .collect();

    let page_bytes = (merge_paging_budget(merge_max_memory_bytes) / readers.len() as u64)
        .clamp(MIN_STREAM_PAGE_BYTES, STREAM_PAGE_BYTES);
    let mut merged = part::MergedRows::new(readers, page_bytes);
    let mut keep = |row: &part::Row| {
        if let Some(cutoffs) = cutoffs
            && cutoffs.is_expired(&row.tenant, crate::tenant_policy::Signal::Logs, row.timestamp_ns)
        {
            return false;
        }
        // The same predicate the scan hides with, applied where the bytes
        // actually leave. A part that no rewrite ever touches keeps them, which
        // is why the request stays `received` until one has.
        if !deletes.is_empty()
            && deletes.hides_row(&row.tenant, row.timestamp_ns, &row.line, &row.structured_metadata)
        {
            return false;
        }
        true
    };
    let (new_parts, dropped_rows, kept_rows) = part::flush_row_stream_with_merge_tombstone(
        &mut merged,
        &mut keep,
        parts_root,
        &partition,
        metadata_keys,
        parsed_keys,
        row_group_size,
        old_dirs,
    )
    .map_err(|error| error.to_string())?;
    Ok(GroupRewrite {
        new_parts,
        dropped_rows: dropped_rows as usize,
        kept_rows: kept_rows as usize,
    })
}

