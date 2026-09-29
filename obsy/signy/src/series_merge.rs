//! The metric compactor (M14, issue #8): size-tiered, per partition, metric-
//! specific. Traces set no merge precedent and the log `merge/` machinery is
//! Parquet-shaped, so this uses the metric part writer's streaming re-flush
//! path — a compaction rewrites its inputs without materialising their full
//! sample set.
//!
//! Tiers are constants, not knobs, until a load run says otherwise
//! (`todo.md`, M14): L0 under 16 MiB of chunk bytes, L1 under 256 MiB, L2 the
//! rest. A partition's tier compacts when it holds at least
//! [`COMPACT_MIN_PARTS`] parts.
//!
//! **Crash safety is a commit record**, `metrics_root/.compact/<id>.json`,
//! written durably before the replacement part is created and before any input
//! is removed. Local mode replays it in `startup::recover_with_signals`; remote
//! mode replays it in `reconcile_metric_local_cache`, where the idempotent
//! manifest replacement (`publish_metric_parts(added, removed)`) makes every
//! crash window converge on the same end state — which is the same shape the
//! log merge's tombstone replay has, carried in a sidecar directory because a
//! metric part has no row-group tombstone to ride in.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use tokio::time::interval;

use crate::compaction_tier::{TierCandidate, TierPolicy, select_tier};
use crate::config::Config;
use crate::object_storage::{MetricManifestPart, RemoteCache, is_inputs_changed_error};
use crate::series_part::{self, SeriesPartReader};
use crate::series_registry::SeriesRegistry;
use crate::shutdown::wait_for_drain;

pub const COMPACT_MIN_PARTS: usize = 8;
/// Metric parts hold chunked samples rather than rows, so a pass reads more
/// per byte than the other signals do; the part count and the input bytes are
/// what bound what one compaction materializes.
const TIER_POLICY: TierPolicy = TierPolicy {
    max_part_bytes: 256 * 1024 * 1024,
    min_parts: COMPACT_MIN_PARTS,
    max_parts: 16,
    max_input_bytes: 256 * 1024 * 1024,
};
const COMPACT_DIR: &str = ".compact";

fn crash_if_requested(point: &str) {
    #[cfg(test)]
    if std::env::var("SIGNY_TEST_COMPACTION_CRASH_POINT")
        .ok()
        .as_deref()
        == Some(point)
    {
        std::process::abort();
    }
    #[cfg(not(test))]
    let _ = point;
}

/// Durable intent: the replacement is in `new`, the inputs it supersedes in
/// `inputs`, both as `partition/id` relative to the metrics root.
#[derive(Serialize, Deserialize)]
pub(crate) struct CompactRecord {
    pub new: Vec<String>,
    pub inputs: Vec<String>,
}

fn chunk_bytes(meta: &crate::series_part::SeriesPartMeta) -> u64 {
    meta.tenants.iter().map(|segment| segment.bytes.len()).sum()
}

/// The inclusive timestamp span of a merged series, which the snapshot the
/// writer takes carries so nothing downstream has to decode to learn it.
#[cfg(test)]
fn sample_bounds(samples: &[(i64, f64)]) -> Option<(i64, i64)> {
    let first = samples.first()?.0;
    let last = samples.last()?.0;
    Some((first.min(last), first.max(last)))
}

/// The parts one pass rewrites, chosen by [`crate::compaction_tier`].
pub(crate) fn select_inputs(
    readers: &[Arc<SeriesPartReader>],
    now_ns: i64,
) -> Option<Vec<Arc<SeriesPartReader>>> {
    let candidates: Vec<TierCandidate> = readers
        .iter()
        .map(|reader| {
            let meta = &reader.part().meta;
            TierCandidate {
                partition: meta.partition.clone(),
                bytes: chunk_bytes(meta),
                max_ts_ns: meta.max_ts_ns,
            }
        })
        .collect();
    let selected = select_tier(&candidates, &TIER_POLICY, now_ns)?;
    Some(
        selected
            .into_iter()
            .map(|index| readers[index].clone())
            .collect(),
    )
}

pub(crate) fn compact_dir(metrics_root: &Path) -> PathBuf {
    metrics_root.join(COMPACT_DIR)
}

fn relative_dir(metrics_root: &Path, dir: &Path) -> Result<String, String> {
    let relative = dir
        .strip_prefix(metrics_root)
        .map_err(|_| format!("part directory escapes the metrics root: {}", dir.display()))?;
    relative
        .to_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("part directory is not UTF-8: {}", dir.display()))
}

fn write_record(metrics_root: &Path, id: &str, record: &CompactRecord) -> Result<PathBuf, String> {
    let dir = compact_dir(metrics_root);
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let path = dir.join(format!("{id}.json"));
    let temporary = dir.join(format!(".{id}.{}.tmp", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec_pretty(record).map_err(|error| error.to_string())?;
    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        crash_if_requested("before_temp_fsync");
        file.sync_all()?;
        crash_if_requested("after_temp_fsync");
        std::fs::rename(&temporary, &path)?;
        crash_if_requested("after_rename");
        crash_if_requested("before_directory_fsync");
        std::fs::File::open(&dir)?.sync_all()?;
        crash_if_requested("after_directory_fsync");
        Ok(())
    })();
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    Ok(path)
}

fn remove_record(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

/// Every pending commit record, with its path. Malformed records are an
/// error, not a skip: silently dropping one could leave inputs alive beside
/// their replacement.
pub(crate) fn read_records(metrics_root: &Path) -> Result<Vec<(PathBuf, CompactRecord)>, String> {
    let dir = compact_dir(metrics_root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
        let record: CompactRecord = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid compaction record {}: {error}", path.display()))?;
        records.push((path, record));
    }
    Ok(records)
}

/// A record's relative dir joined back under the root, refusing traversal.
pub(crate) fn record_dir(metrics_root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    let mut components = path.components();
    let safe = matches!(
        (components.next(), components.next(), components.next()),
        (
            Some(std::path::Component::Normal(_)),
            Some(std::path::Component::Normal(_)),
            None
        )
    );
    if !safe {
        return Err(format!("unsafe compaction record path {relative:?}"));
    }
    Ok(metrics_root.join(path))
}

/// The local half of crash recovery, run before the registry loads: a record
/// whose replacement exists wins (surviving inputs are removed), a record
/// whose replacement never became durable loses (the inputs stay and only the
/// record goes).
pub fn recover_local_compactions(metrics_root: &Path) -> Result<(), String> {
    remove_temporary_records(metrics_root)?;
    for (path, record) in read_records(metrics_root)? {
        crash_if_requested("recovery_record_processing");
        let replacement_durable = record.new.iter().all(|relative| {
            record_dir(metrics_root, relative)
                .map(|dir| series_part::load_series_part(&dir).is_ok())
                .unwrap_or(false)
        });
        if replacement_durable {
            let dirs = record
                .inputs
                .iter()
                .map(|relative| record_dir(metrics_root, relative))
                .collect::<Result<Vec<_>, _>>()?;
            crate::part::remove_part_dirs(&dirs)?;
        } else {
            let dirs = record
                .new
                .iter()
                .map(|relative| record_dir(metrics_root, relative))
                .collect::<Result<Vec<_>, _>>()?;
            crate::part::remove_part_dirs(&dirs)?;
        }
        remove_record(&path)?;
    }
    Ok(())
}

fn remove_temporary_records(metrics_root: &Path) -> Result<(), String> {
    let dir = compact_dir(metrics_root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("tmp") {
            std::fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

/// One compaction pass. Returns whether anything was compacted.
pub async fn compact_once(
    registry: &SeriesRegistry,
    metrics_root: &Path,
    remote: Option<&RemoteCache>,
) -> Result<bool, String> {
    let Some(inputs) = select_inputs(&registry.snapshot(), crate::clock::Clock::system().now_ns())
    else {
        return Ok(false);
    };

    // Everything from here to the record write is synchronous, so the arena
    // guard is safe to hold across it and is dropped before the first await.
    let arena = crate::memprof::enter(crate::memprof::Arena::Merge);

    // Input catalogs and sample chunks are merged one head at a time. The
    // streaming writer preserves the same tenant/labels ordering and stable
    // duplicate-timestamp order as the old snapshot materialisation path.
    let input_descriptors: Vec<MetricManifestPart> = inputs
        .iter()
        .map(|reader| MetricManifestPart {
            id: reader.part().meta.id.clone(),
            partition: reader.part().meta.partition.clone(),
        })
        .collect();
    let input_dirs: Vec<PathBuf> = inputs
        .iter()
        .map(|reader| reader.part().dir.clone())
        .collect();
    let partition = inputs[0].part().meta.partition.clone();
    let output_id = format!("{}-{}", partition.replace('-', ""), uuid::Uuid::new_v4());
    let record = CompactRecord {
        new: vec![format!("{partition}/{output_id}")],
        inputs: input_dirs
            .iter()
            .map(|dir| relative_dir(metrics_root, dir))
            .collect::<Result<_, _>>()?,
    };
    // The intent is durable before the replacement can become visible. A
    // restart can therefore safely discard a partial output and retain all
    // inputs, including a crash between record creation and the first write.
    let record_path = write_record(metrics_root, &output_id, &record)?;
    crash_if_requested("after_durable_intent");
    let new_parts =
        match series_part::compact_series_parts_with_id(&inputs, metrics_root, &output_id) {
            Ok(parts) => parts,
            Err(error) => {
                remove_record(&record_path)?;
                return Err(format!(
                    "compaction failed to write its replacement part: {error}"
                ));
            }
        };
    if new_parts.len() != 1 || new_parts[0].meta.id != output_id {
        let _ = crate::part::remove_part_dirs(
            &new_parts
                .iter()
                .map(|part| part.dir.clone())
                .collect::<Vec<_>>(),
        );
        remove_record(&record_path)?;
        return Err("metric compaction wrote an unexpected replacement id".to_string());
    }
    drop(arena);

    if let Some(cache) = remote {
        // One CAS replaces the inputs with the output; a conflict means
        // another writer (retention) touched an input first, and the pass
        // steps aside rather than reapplying a replacement over it.
        match cache
            .storage
            .publish_metric_parts(&new_parts, &input_descriptors)
            .await
        {
            Ok(_) => cache.record_remote_success(),
            Err(error) if is_inputs_changed_error(&error) => {
                let new_dirs: Vec<PathBuf> =
                    new_parts.iter().map(|part| part.dir.clone()).collect();
                crate::part::remove_part_dirs(&new_dirs)?;
                remove_record(&record_path)?;
                tracing::info!(%error, "metric compaction skipped: inputs changed under it");
                return Ok(false);
            }
            Err(error) => {
                cache.record_remote_failure();
                return Err(error);
            }
        }
    }

    // The visibility transition, atomic against queries exactly as flush's is.
    let opened = SeriesRegistry::open_parts(new_parts.clone())?;
    let input_ids: Vec<String> = input_descriptors
        .iter()
        .map(|part| part.id.clone())
        .collect();
    {
        let _guard =
            crate::part_registry::PartRegistry::write_without_convoy(registry.operation_lock())
                .await;
        registry.register_opened(opened);
        registry.unregister(&input_ids);
        crate::part::remove_part_dirs(&input_dirs)?;
    }
    remove_record(&record_path)?;

    if let Some(cache) = remote {
        // Object deletion last: descriptors already left the manifest, so a
        // crash here leaves orphans for the grace-period collector, never a
        // manifest entry without objects.
        cache
            .storage
            .delete_metric_part_objects(&input_descriptors)
            .await?;
    }

    tracing::info!(
        inputs = inputs.len(),
        outputs = new_parts.len(),
        samples = new_parts
            .iter()
            .map(|part| part.meta.sample_count)
            .sum::<u64>(),
        "metric compaction replaced a tier"
    );
    Ok(true)
}

/// The compaction worker, paced by the same interval the log merge uses —
/// both answer "how often does background rewriting look for debt".
pub async fn compact_loop(
    registry: Arc<SeriesRegistry>,
    remote_cache: Option<Arc<RemoteCache>>,
    config: Arc<Config>,
    healthy: Arc<AtomicBool>,
    mut drain_rx: watch::Receiver<bool>,
) {
    let metrics_root = config.data_dir.join("metrics");
    let mut ticker = interval(config.merge_interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = wait_for_drain(&mut drain_rx) => return,
        }
        // Drain the debt this tick found; each pass re-selects, so a burst of
        // small flushes converges instead of compacting once per interval.
        loop {
            let Some(_memory_charge) = config
                .memory_account
                .try_admit(config.merge_max_memory_bytes)
            else {
                // Queries and flush own the account right now. Leave the parts
                // in place and retry the compaction on the next tick rather
                // than materialising work that cannot be admitted safely.
                tracing::debug!("metric compaction waiting for memory account room");
                break;
            };
            match compact_once(&registry, &metrics_root, remote_cache.as_deref()).await {
                Ok(true) => {
                    healthy.store(true, Ordering::Release);
                }
                Ok(false) => {
                    healthy.store(true, Ordering::Release);
                    break;
                }
                Err(error) => {
                    healthy.store(false, Ordering::Release);
                    tracing::error!(%error, "metric compaction failed");
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    /// The fixtures' own wall clock. Selection reads it to tell a partition
    /// still being written from one nothing has touched for an hour, and the
    /// fixtures date their rows rather than using the real clock.
    const FIXTURE_NOW_NS: i64 = 1_772_000_000_000_000_000;

    use super::*;
    use crate::series::{
        METRIC_NAME_LABEL, MetricSample, MetricValue, SampleKind, SeriesLabels, SeriesMemTable,
        SeriesSnapshot, SnapshotSeries,
    };
    use crate::tenant::test_tenant;

    fn labels(name: &str, instance: &str) -> SeriesLabels {
        SeriesLabels::from_pairs(vec![
            (METRIC_NAME_LABEL.to_string(), name.to_string()),
            ("instance".to_string(), instance.to_string()),
        ])
    }

    fn sample(series: &SeriesLabels, ts: i64, value: f64) -> MetricSample {
        MetricSample {
            tenant: test_tenant(),
            labels: series.clone(),
            ts_ns: ts,
            value: MetricValue::Scalar(value),
            kind: SampleKind::Gauge,
            datapoint_index: 0,
        }
    }

    fn temp_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("signy-series-merge-{tag}-{}", uuid::Uuid::new_v4()))
    }

    /// One flushed part per call, all in one partition.
    fn flush_one_part(root: &Path, series: &SeriesLabels, base_ts: i64, count: usize) {
        let memtable = SeriesMemTable::new();
        let samples: Vec<MetricSample> = (0..count)
            .map(|index| sample(series, base_ts + index as i64 * 1_000_000_000, index as f64))
            .collect();
        memtable.insert(samples);
        let snapshot = memtable.begin_flush();
        series_part::flush_series_snapshot(&snapshot, root).unwrap();
        memtable.commit_flush();
    }

    fn registry_over(root: &Path) -> SeriesRegistry {
        SeriesRegistry::load_from_disk(root, Arc::new(tokio::sync::RwLock::new(()))).unwrap()
    }

    fn all_samples(
        registry: &SeriesRegistry,
    ) -> std::collections::BTreeMap<SeriesLabels, Vec<(i64, f64)>> {
        let mut merged: std::collections::BTreeMap<SeriesLabels, Vec<(i64, f64)>> =
            std::collections::BTreeMap::new();
        for reader in registry.snapshot() {
            for entry in reader.tenant_catalog(&test_tenant()).iter() {
                merged
                    .entry(SeriesLabels::from_canonical(entry.labels.to_vec()))
                    .or_default()
                    .extend(reader.read_series(entry.chunk).unwrap());
            }
        }
        for samples in merged.values_mut() {
            samples.sort_by_key(|(ts, _)| *ts);
        }
        merged
    }

    #[test]
    fn the_trigger_needs_a_full_tier_in_one_partition() {
        let root = temp_root("trigger");
        let series = labels("queue_depth", "a");
        for index in 0..COMPACT_MIN_PARTS - 1 {
            flush_one_part(&root, &series, 1_772_000_000_000_000_000 + index as i64, 4);
        }
        let registry = registry_over(&root);
        assert!(
            select_inputs(&registry.snapshot(), FIXTURE_NOW_NS).is_none(),
            "{} parts are one short of the tier",
            COMPACT_MIN_PARTS - 1
        );
        flush_one_part(
            &root,
            &series,
            1_772_000_000_000_000_000 + COMPACT_MIN_PARTS as i64,
            4,
        );
        let registry = registry_over(&root);
        assert_eq!(
            select_inputs(&registry.snapshot(), FIXTURE_NOW_NS).map(|inputs| inputs.len()),
            Some(COMPACT_MIN_PARTS)
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn a_compaction_replaces_the_tier_and_answers_do_not_change() {
        let root = temp_root("equivalence");
        let series_a = labels("queue_depth", "a");
        let series_b = labels("queue_depth", "b");
        for index in 0..COMPACT_MIN_PARTS {
            let base = 1_772_000_000_000_000_000 + index as i64 * 60_000_000_000;
            flush_one_part(&root, &series_a, base, 5);
            flush_one_part(&root, &series_b, base + 1, 3);
        }
        let registry = registry_over(&root);
        let before = all_samples(&registry);
        let before_count = registry.part_count();
        let before_bytes = registry.tenant_stored_bytes(&test_tenant());
        assert!(before_bytes > 0);

        assert!(compact_once(&registry, &root, None).await.unwrap());

        assert!(
            registry.part_count() < before_count,
            "{} parts did not shrink from {before_count}",
            registry.part_count()
        );
        assert_eq!(
            all_samples(&registry),
            before,
            "a compaction changes nothing a query reads"
        );
        assert!(
            registry.tenant_stored_bytes(&test_tenant()) > 0,
            "the census follows the replacement"
        );
        // The inputs are gone from disk and the commit record is cleared.
        assert!(read_records(&root).unwrap().is_empty());
        let discovered = series_part::discover_series_parts(&root).unwrap();
        assert_eq!(discovered.len(), registry.part_count());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn streaming_compaction_merges_overlapping_series_stably() {
        let root = temp_root("streaming-overlap");
        let series = labels("queue_depth", "a");
        // Every part contains the same timestamps. The old snapshot merge's
        // stable sort retained input order for equal timestamps; the sample
        // heap must do the same while holding only one decoder per input.
        for _ in 0..COMPACT_MIN_PARTS {
            flush_one_part(&root, &series, 1_772_000_000_000_000_000, 4);
        }
        let registry = registry_over(&root);
        let inputs =
            select_inputs(&registry.snapshot(), FIXTURE_NOW_NS).expect("the tier is complete");
        let mut expected = Vec::new();
        for reader in &inputs {
            expected.extend(
                reader
                    .read_series(reader.tenant_catalog(&test_tenant()).get(0).unwrap().chunk)
                    .unwrap(),
            );
        }
        expected.sort_by_key(|(ts, _)| *ts);

        let parts = series_part::compact_series_parts(&inputs, &root).unwrap();
        assert_eq!(parts.len(), 1);
        let reader = SeriesPartReader::open(parts.into_iter().next().unwrap()).unwrap();
        assert_eq!(
            reader
                .read_series(reader.tenant_catalog(&test_tenant()).get(0).unwrap().chunk)
                .unwrap(),
            expected,
            "overlapping samples retain stable timestamp ordering"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn a_crash_between_commit_record_and_input_removal_recovers_without_duplicates() {
        let root = temp_root("crash");
        let series = labels("queue_depth", "a");
        for index in 0..COMPACT_MIN_PARTS {
            flush_one_part(
                &root,
                &series,
                1_772_000_000_000_000_000 + index as i64 * 60_000_000_000,
                5,
            );
        }
        let registry = registry_over(&root);
        let before = all_samples(&registry);
        let inputs = select_inputs(&registry.snapshot(), FIXTURE_NOW_NS).unwrap();

        // The crash window, simulated by hand: the replacement is durable and
        // the record exists, but no input was removed.
        let mut tenants: std::collections::BTreeMap<SeriesLabels, Vec<(i64, f64)>> =
            std::collections::BTreeMap::new();
        for reader in &inputs {
            for entry in reader.tenant_catalog(&test_tenant()).iter() {
                tenants
                    .entry(SeriesLabels::from_canonical(entry.labels.to_vec()))
                    .or_default()
                    .extend(reader.read_series(entry.chunk).unwrap());
            }
        }
        let snapshot = SeriesSnapshot {
            tenants: [(
                test_tenant(),
                tenants
                    .into_iter()
                    .map(|(labels, spill)| SnapshotSeries {
                        labels,
                        bounds: sample_bounds(&spill),
                        chunks: Vec::new(),
                        spill,
                        points: Vec::new(),
                    })
                    .collect(),
            )]
            .into_iter()
            .collect(),
        };
        let new_parts = series_part::flush_series_snapshot(&snapshot, &root).unwrap();
        let record = CompactRecord {
            new: new_parts
                .iter()
                .map(|part| relative_dir(&root, &part.dir).unwrap())
                .collect(),
            inputs: inputs
                .iter()
                .map(|reader| relative_dir(&root, &reader.part().dir).unwrap())
                .collect(),
        };
        write_record(&root, &new_parts[0].meta.id, &record).unwrap();

        recover_local_compactions(&root).unwrap();

        let recovered = registry_over(&root);
        assert_eq!(
            all_samples(&recovered),
            before,
            "recovery keeps exactly one copy of every sample"
        );
        assert!(read_records(&root).unwrap().is_empty());
        std::fs::remove_dir_all(&root).ok();
    }

    /// Found in production: a record left from a pass whose inputs and
    /// replacement the manifest no longer names. The replay has to drop both
    /// rather than wedge startup or republish either.
    #[tokio::test]
    async fn a_remote_replay_drops_a_compaction_the_manifest_already_retired() {
        let data_dir = temp_root("remote-retired");
        let root = data_dir.join("metrics");
        let series = labels("queue_depth", "a");
        for index in 0..COMPACT_MIN_PARTS {
            flush_one_part(
                &root,
                &series,
                1_772_000_000_000_000_000 + index as i64 * 60_000_000_000,
                5,
            );
        }
        let registry = registry_over(&root);
        let inputs = select_inputs(&registry.snapshot(), FIXTURE_NOW_NS).unwrap();
        let new_parts = series_part::compact_series_parts(&inputs, &root).unwrap();
        let record = CompactRecord {
            new: new_parts
                .iter()
                .map(|part| relative_dir(&root, &part.dir).unwrap())
                .collect(),
            inputs: inputs
                .iter()
                .map(|reader| relative_dir(&root, &reader.part().dir).unwrap())
                .collect(),
        };
        write_record(&root, &new_parts[0].meta.id, &record).unwrap();
        let storage = crate::object_storage::ObjectStorage::in_memory();

        let manifest = storage.reconcile_metric_local_cache(&root).await.unwrap();

        assert!(manifest.parts.is_empty());
        assert!(read_records(&root).unwrap().is_empty());
        assert!(
            series_part::discover_series_parts(&root)
                .unwrap()
                .is_empty()
        );
        std::fs::remove_dir_all(&data_dir).ok();
    }

    #[test]
    fn a_record_whose_replacement_never_became_durable_keeps_the_inputs() {
        let root = temp_root("undone");
        let series = labels("queue_depth", "a");
        flush_one_part(&root, &series, 1_772_000_000_000_000_000, 5);
        let registry = registry_over(&root);
        let input_dir = registry.snapshot()[0].part().dir.clone();
        let record = CompactRecord {
            new: vec!["2026-02-25/never-written".to_string()],
            inputs: vec![relative_dir(&root, &input_dir).unwrap()],
        };
        write_record(&root, "never-written", &record).unwrap();

        recover_local_compactions(&root).unwrap();

        assert!(input_dir.exists(), "the inputs survive an unfinished pass");
        assert!(read_records(&root).unwrap().is_empty());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_partial_replacement_is_removed_before_registry_restart() {
        let root = temp_root("partial-replacement");
        let series = labels("queue_depth", "a");
        flush_one_part(&root, &series, 1_772_000_000_000_000_000, 5);
        let registry = registry_over(&root);
        let input_dir = registry.snapshot()[0].part().dir.clone();
        let replacement = root.join("2026-02-25/partial-output");
        std::fs::create_dir_all(&replacement).unwrap();
        std::fs::write(replacement.join("truncated.json"), b"partial").unwrap();
        let record = CompactRecord {
            new: vec![relative_dir(&root, &replacement).unwrap()],
            inputs: vec![relative_dir(&root, &input_dir).unwrap()],
        };
        write_record(&root, "partial-output", &record).unwrap();

        recover_local_compactions(&root).unwrap();

        assert!(input_dir.exists(), "the inputs survive a partial output");
        assert!(!replacement.exists(), "the partial output is rolled back");
        assert!(read_records(&root).unwrap().is_empty());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn process_crashes_at_each_metric_compaction_boundary_recover_without_loss() {
        let points = [
            "before_temp_fsync",
            "after_temp_fsync",
            "after_rename",
            "before_directory_fsync",
            "after_directory_fsync",
            "after_durable_intent",
            "recovery_record_processing",
        ];
        for point in points {
            let root = temp_root("process-fault");
            let series = labels("queue_depth", "a");
            for index in 0..COMPACT_MIN_PARTS {
                flush_one_part(
                    &root,
                    &series,
                    1_772_000_000_000_000_000 + index as i64 * 60_000_000_000,
                    5,
                );
            }
            let registry = registry_over(&root);
            let before = all_samples(&registry);
            if point == "recovery_record_processing" {
                let inputs = select_inputs(&registry.snapshot(), FIXTURE_NOW_NS).unwrap();
                let partition = inputs[0].part().meta.partition.clone();
                let record = CompactRecord {
                    new: vec![format!("{partition}/recovery-output")],
                    inputs: inputs
                        .iter()
                        .map(|reader| relative_dir(&root, &reader.part().dir).unwrap())
                        .collect(),
                };
                write_record(&root, "recovery-output", &record).unwrap();
            }

            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "series_merge::tests::metric_compaction_fault_helper",
                    "--nocapture",
                ])
                .env("SIGNY_TEST_COMPACTION_CRASH_POINT", point)
                .env("SIGNY_TEST_COMPACTION_ROOT", &root)
                .status()
                .unwrap();
            assert!(
                !status.success(),
                "fault point {point} did not terminate the child"
            );

            for _ in 0..3 {
                recover_local_compactions(&root).unwrap();
                let restarted = registry_over(&root);
                assert_eq!(all_samples(&restarted), before, "fault point {point}");
            }
            assert!(read_records(&root).unwrap().is_empty());
            assert!(std::fs::read_dir(compact_dir(&root)).unwrap().all(|entry| {
                entry
                    .unwrap()
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    != Some("tmp")
            }));
            std::fs::remove_dir_all(&root).ok();
        }
    }

    #[tokio::test]
    async fn metric_compaction_fault_helper() {
        let Some(root) = std::env::var_os("SIGNY_TEST_COMPACTION_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        if std::env::var("SIGNY_TEST_COMPACTION_CRASH_POINT").as_deref()
            == Ok("recovery_record_processing")
        {
            recover_local_compactions(&root).unwrap();
        } else {
            let registry = registry_over(&root);
            compact_once(&registry, &root, None).await.unwrap();
        }
    }
}
