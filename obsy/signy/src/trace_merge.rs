//! The trace compactor: size-tiered, per partition, on the metric
//! compactor's crash-safety shape.
//!
//! Every flush writes one trace part per partition, so a steady trickle of
//! spans becomes one part per flush interval, each a Parquet file, a bloom
//! file and a metadata file locally and in the object store. The compactor
//! rewrites a full tier into one part, which is what keeps the part count, and
//! with it the object count, the catalog size and the per-query open cost,
//! growing with the data rather than with the number of flushes.
//!
//! A pass holds the deletion lock's read half from input selection to the
//! visibility commit, as the log merge does, so neither retention nor cache
//! eviction can remove an input body it is reading. Missing bodies are
//! restored first, because eviction may already have dropped them.
//!
//! **Crash safety is a commit record**, `traces_root/.compact/<id>.json`,
//! written durably before the replacement is created and before any input is
//! removed. Local mode resolves it before the registry loads; remote mode replays it in
//! `reconcile_trace_local_cache`, where the manifest replacement is
//! idempotent. Input objects are left to the orphan collector rather than
//! deleted here, so a query that planned against an input before the
//! replacement landed can still restore it for the length of the grace period.

use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use tokio::time::interval;

use crate::compaction_tier::{TierCandidate, TierPolicy, select_tier};
use crate::config::Config;
use crate::object_storage::{RemoteCache, TraceManifestPart, is_inputs_changed_error};
use crate::shutdown::wait_for_drain;
use crate::trace_part::{self, TracePartReader};
use crate::trace_registry::TraceRegistry;

pub const COMPACT_MIN_PARTS: usize = 8;
/// Trace parts are small and a pass reads them whole, so the size tiers are
/// the shared ones and only the ceilings are trace-specific. `max_input_bytes`
/// is stored bytes: a decoded span is several times its stored size, which is
/// what bounds the pass's memory rather than the part count.
const TIER_POLICY: TierPolicy = TierPolicy {
    max_part_bytes: 16 * 1024 * 1024,
    min_parts: COMPACT_MIN_PARTS,
    max_parts: 32,
    max_input_bytes: 16 * 1024 * 1024,
};
const COMPACT_DIR: &str = ".compact";
const SPAN_DECODE_EXPANSION: u64 = 8;

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
/// `inputs`, both as `partition/id` relative to the traces root.
#[derive(Serialize, Deserialize)]
pub(crate) struct CompactRecord {
    pub new: Vec<String>,
    pub inputs: Vec<String>,
}

fn stored_bytes(reader: &TracePartReader) -> u64 {
    reader
        .part()
        .meta
        .tenants
        .iter()
        .map(|segment| segment.bytes.len())
        .sum()
}

/// The parts one pass rewrites, chosen by [`crate::compaction_tier`].
pub(crate) fn select_inputs(
    readers: &[Arc<TracePartReader>],
    now_ns: i64,
) -> Option<Vec<Arc<TracePartReader>>> {
    let candidates: Vec<TierCandidate> = readers
        .iter()
        .map(|reader| TierCandidate {
            partition: reader.part().meta.partition.clone(),
            bytes: stored_bytes(reader),
            max_ts_ns: reader.part().meta.max_ts_ns,
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

pub(crate) fn compact_dir(traces_root: &Path) -> PathBuf {
    traces_root.join(COMPACT_DIR)
}

fn relative_dir(traces_root: &Path, dir: &Path) -> Result<String, String> {
    let relative = dir
        .strip_prefix(traces_root)
        .map_err(|_| format!("part directory escapes the traces root: {}", dir.display()))?;
    relative
        .to_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("part directory is not UTF-8: {}", dir.display()))
}

fn write_record(traces_root: &Path, id: &str, record: &CompactRecord) -> Result<PathBuf, String> {
    let dir = compact_dir(traces_root);
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

pub(crate) fn remove_record(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

/// Every pending commit record, with its path. A malformed record is an error
/// rather than a skip: dropping one could leave inputs alive beside their
/// replacement, and every span in them would answer twice.
pub(crate) fn read_records(traces_root: &Path) -> Result<Vec<(PathBuf, CompactRecord)>, String> {
    let dir = compact_dir(traces_root);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
        let record: CompactRecord = serde_json::from_slice(&bytes).map_err(|error| {
            format!(
                "invalid trace compaction record {}: {error}",
                path.display()
            )
        })?;
        records.push((path, record));
    }
    Ok(records)
}

/// A record's relative dir joined back under the root, refusing traversal.
pub(crate) fn record_dir(traces_root: &Path, relative: &str) -> Result<PathBuf, String> {
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
        return Err(format!("unsafe trace compaction record path {relative:?}"));
    }
    Ok(traces_root.join(path))
}

/// The input of a record as a manifest descriptor.
pub(crate) fn record_input_descriptor(relative: &str) -> Result<TraceManifestPart, String> {
    let (partition, id) = relative
        .split_once('/')
        .ok_or_else(|| format!("malformed trace compaction input {relative:?}"))?;
    Ok(TraceManifestPart {
        id: id.to_string(),
        partition: partition.to_string(),
    })
}

/// Local-mode crash recovery, run before the registry loads: a record whose
/// replacement is durable wins and its surviving inputs are removed; a record
/// whose replacement never became durable loses and only the record goes.
///
/// Remote mode must not run this. There the manifest may still name the
/// inputs, and removing them locally first would leave the reconcile scan
/// publishing the replacement beside inputs it then restores again.
pub fn recover_local_compactions(traces_root: &Path) -> Result<(), String> {
    remove_temporary_records(traces_root)?;
    for (path, record) in read_records(traces_root)? {
        crash_if_requested("recovery_record_processing");
        let replacement_durable = record.new.iter().all(|relative| {
            record_dir(traces_root, relative)
                .and_then(|dir| trace_part::load_trace_part(&dir))
                .and_then(TracePartReader::open)
                .is_ok()
        });
        if replacement_durable {
            let dirs = record
                .inputs
                .iter()
                .map(|relative| record_dir(traces_root, relative))
                .collect::<Result<Vec<_>, _>>()?;
            crate::part::remove_part_dirs(&dirs)?;
        } else {
            let mut written_dirs = Vec::new();
            for relative in &record.new {
                let dir = record_dir(traces_root, relative)?;
                if dir.exists() {
                    written_dirs.push(dir);
                }
            }
            crate::part::remove_part_dirs(&written_dirs)?;
        }
        remove_record(&path)?;
    }
    Ok(())
}

fn remove_temporary_records(traces_root: &Path) -> Result<(), String> {
    let dir = compact_dir(traces_root);
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

async fn restore_missing_bodies(
    registry: &TraceRegistry,
    remote: &RemoteCache,
    inputs: &[Arc<TracePartReader>],
    config: &Config,
) -> Result<(), String> {
    let ids: HashSet<String> = inputs
        .iter()
        .map(|reader| reader.part().meta.id.clone())
        .collect();
    let missing = registry.missing_data_ids(&ids);
    if missing.is_empty() {
        return Ok(());
    }
    match tokio::time::timeout(
        config.max_query_runtime,
        remote
            .storage
            .restore_trace_parts(&remote.trace_parts_root(), &missing),
    )
    .await
    {
        Ok(Ok(())) => {
            remote.record_remote_success();
            Ok(())
        }
        Ok(Err(error)) => {
            remote.record_remote_failure();
            Err(error)
        }
        Err(_) => {
            remote.record_remote_failure();
            Err("trace compaction restore timed out".to_string())
        }
    }
}

fn read_all_spans(inputs: &[Arc<TracePartReader>]) -> Result<Vec<crate::trace::TraceSpan>, String> {
    let mut spans = Vec::new();
    for reader in inputs {
        for segment in &reader.part().meta.tenants {
            spans.extend(reader.query_all_limited(&segment.tenant, usize::MAX, None)?);
        }
    }
    Ok(spans)
}

fn inputs_still_registered(registry: &TraceRegistry, inputs: &[Arc<TracePartReader>]) -> bool {
    let registered: std::collections::HashMap<String, PathBuf> = registry
        .snapshot()
        .into_iter()
        .map(|reader| (reader.part().meta.id.clone(), reader.part().dir.clone()))
        .collect();
    inputs
        .iter()
        .all(|reader| registered.get(&reader.part().meta.id) == Some(&reader.part().dir))
}

/// One compaction pass. Returns whether anything was compacted.
pub async fn compact_once(
    registry: &TraceRegistry,
    deletion_lock: Arc<tokio::sync::RwLock<()>>,
    traces_root: &Path,
    remote: Option<&RemoteCache>,
    config: &Config,
) -> Result<bool, String> {
    let _deletion_guard = deletion_lock.read_owned().await;
    let Some(inputs) = select_inputs(&registry.snapshot(), crate::clock::Clock::system().now_ns())
    else {
        return Ok(false);
    };
    if !inputs_still_registered(registry, &inputs) {
        return Ok(false);
    }
    let Some(_memory_charge) = config.memory_account.try_admit(
        inputs
            .iter()
            .map(|reader| stored_bytes(reader))
            .sum::<u64>()
            .saturating_mul(SPAN_DECODE_EXPANSION),
    ) else {
        tracing::debug!("trace compaction waiting for memory account room");
        return Ok(false);
    };
    if let Some(cache) = remote {
        restore_missing_bodies(registry, cache, &inputs, config).await?;
    }

    let arena = crate::memprof::enter(crate::memprof::Arena::Merge);
    let spans = read_all_spans(&inputs)?;
    let span_count = spans.len();

    let input_descriptors: Vec<TraceManifestPart> = inputs
        .iter()
        .map(|reader| TraceManifestPart {
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
            .map(|dir| relative_dir(traces_root, dir))
            .collect::<Result<_, _>>()?,
    };
    // The intent is durable before the replacement can become visible. A
    // restart can therefore discard a partial output and retain all inputs,
    // including a crash between record creation and the first write.
    let record_path = write_record(traces_root, &output_id, &record)?;
    crash_if_requested("after_durable_intent");
    let new_parts = match trace_part::flush_trace_spans_with_id(
        &spans,
        traces_root,
        config.row_group_size,
        Some(&output_id),
    ) {
        Ok(parts) => parts,
        Err(error) => {
            remove_record(&record_path)?;
            return Err(format!(
                "trace compaction failed to write its replacement: {error}"
            ));
        }
    };
    drop(spans);
    if new_parts.len() != 1 || new_parts[0].meta.id != output_id {
        let _ = crate::part::remove_part_dirs(
            &new_parts
                .iter()
                .map(|part| part.dir.clone())
                .collect::<Vec<_>>(),
        );
        remove_record(&record_path)?;
        return Err("trace compaction wrote an unexpected replacement id".to_string());
    }
    drop(arena);

    if let Some(cache) = remote {
        match cache
            .storage
            .replace_trace_parts(&new_parts, &input_descriptors)
            .await
        {
            Ok(_) => cache.record_remote_success(),
            Err(error) if is_inputs_changed_error(&error) => {
                let new_dirs: Vec<PathBuf> =
                    new_parts.iter().map(|part| part.dir.clone()).collect();
                crate::part::remove_part_dirs(&new_dirs)?;
                remove_record(&record_path)?;
                tracing::info!(%error, "trace compaction skipped: inputs changed under it");
                return Ok(false);
            }
            Err(error) => {
                cache.record_remote_failure();
                return Err(error);
            }
        }
    }

    let opened = TraceRegistry::open_parts(new_parts.clone())?;
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

    tracing::info!(
        inputs = inputs.len(),
        outputs = new_parts.len(),
        spans = span_count,
        "trace compaction replaced a tier"
    );
    Ok(true)
}

/// The compaction worker, on the merge cadence like the other two.
pub async fn compact_loop(
    registry: Arc<TraceRegistry>,
    deletion_lock: Arc<tokio::sync::RwLock<()>>,
    remote_cache: Option<Arc<RemoteCache>>,
    config: Arc<Config>,
    healthy: Arc<AtomicBool>,
    mut drain_rx: watch::Receiver<bool>,
) {
    let traces_root = config.data_dir.join("traces");
    let mut ticker = interval(config.merge_interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = wait_for_drain(&mut drain_rx) => return,
        }
        loop {
            match compact_once(
                &registry,
                deletion_lock.clone(),
                &traces_root,
                remote_cache.as_deref(),
                &config,
            )
            .await
            {
                Ok(true) => {
                    healthy.store(true, Ordering::Release);
                }
                Ok(false) => {
                    healthy.store(true, Ordering::Release);
                    break;
                }
                Err(error) => {
                    healthy.store(false, Ordering::Release);
                    tracing::error!(%error, "trace compaction failed");
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
    use crate::tenant::test_tenant;
    use crate::trace::{TraceSpan, normalize_request};
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};

    fn temp_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("signy-trace-merge-{tag}-{}", uuid::Uuid::new_v4()))
    }

    fn flush_one_part(root: &Path, part_index: usize) {
        let base_ns = 1_772_000_000_000_000_000u64 + part_index as u64 * 1_000_000;
        let spans = (0..3u64)
            .map(|span_index| Span {
                trace_id: vec![part_index as u8 + 1; 16],
                span_id: vec![span_index as u8 + 1; 8],
                start_time_unix_nano: base_ns + span_index,
                end_time_unix_nano: base_ns + span_index + 1_000,
                ..Default::default()
            })
            .collect();
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                scope_spans: vec![ScopeSpans {
                    spans,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let spans: Vec<TraceSpan> = normalize_request(&test_tenant(), request).unwrap();
        trace_part::flush_trace_spans(&spans, root, 16).unwrap();
    }

    fn all_spans(registry: &TraceRegistry) -> Vec<(String, String, i64)> {
        let mut spans: Vec<(String, String, i64)> = registry
            .snapshot()
            .iter()
            .flat_map(|reader| reader.query_all(&test_tenant()).unwrap())
            .map(|span| (span.trace_id, span.span_id, span.start_time_ns))
            .collect();
        spans.sort();
        spans
    }

    fn test_config(root: &Path) -> Config {
        Config {
            data_dir: root.parent().unwrap().to_path_buf(),
            row_group_size: 16,
            ..Config::default()
        }
    }

    #[test]
    fn the_trigger_needs_a_full_tier_in_one_partition() {
        let root = temp_root("trigger");
        for part_index in 0..COMPACT_MIN_PARTS - 1 {
            flush_one_part(&root, part_index);
        }
        let registry =
            TraceRegistry::load_from_disk(&root, Arc::new(tokio::sync::RwLock::new(()))).unwrap();
        assert!(select_inputs(&registry.snapshot(), FIXTURE_NOW_NS).is_none());

        flush_one_part(&root, COMPACT_MIN_PARTS);
        let registry =
            TraceRegistry::load_from_disk(&root, Arc::new(tokio::sync::RwLock::new(()))).unwrap();
        assert_eq!(
            select_inputs(&registry.snapshot(), FIXTURE_NOW_NS).map(|inputs| inputs.len()),
            Some(COMPACT_MIN_PARTS)
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn a_compaction_replaces_the_tier_and_answers_do_not_change() {
        let root = temp_root("equivalence");
        for part_index in 0..COMPACT_MIN_PARTS {
            flush_one_part(&root, part_index);
        }
        let registry =
            TraceRegistry::load_from_disk(&root, Arc::new(tokio::sync::RwLock::new(()))).unwrap();
        let before = all_spans(&registry);
        let config = test_config(&root);

        let compacted = compact_once(
            &registry,
            Arc::new(tokio::sync::RwLock::new(())),
            &root,
            None,
            &config,
        )
        .await
        .unwrap();

        assert!(compacted);
        assert_eq!(registry.snapshot().len(), 1);
        assert_eq!(
            all_spans(&registry),
            before,
            "a compaction changes nothing a query reads"
        );
        assert!(read_records(&root).unwrap().is_empty());
        assert_eq!(trace_part::discover_trace_parts(&root).unwrap().len(), 1);
        std::fs::remove_dir_all(&root).ok();
    }

    fn descriptor_of(reader: &TracePartReader) -> TraceManifestPart {
        TraceManifestPart {
            id: reader.part().meta.id.clone(),
            partition: reader.part().meta.partition.clone(),
        }
    }

    #[tokio::test]
    async fn a_remote_compaction_swaps_the_manifest_in_one_commit_and_repeats_as_a_no_op() {
        let data_dir = temp_root("remote");
        let traces_root = data_dir.join("traces");
        for part_index in 0..COMPACT_MIN_PARTS {
            flush_one_part(&traces_root, part_index);
        }
        let storage = Arc::new(crate::object_storage::ObjectStorage::in_memory());
        storage
            .publish_trace_parts(&trace_part::discover_trace_parts(&traces_root).unwrap())
            .await
            .unwrap();
        let cache = RemoteCache::new(storage.clone(), data_dir.join("parts"));
        let registry =
            TraceRegistry::load_from_disk(&traces_root, Arc::new(tokio::sync::RwLock::new(())))
                .unwrap();
        let before = all_spans(&registry);
        let input_descriptors: Vec<TraceManifestPart> = registry
            .snapshot()
            .iter()
            .map(|reader| descriptor_of(reader))
            .collect();
        let generation_before = storage.load_trace_manifest().await.unwrap().generation;
        let config = Config {
            data_dir: data_dir.clone(),
            row_group_size: 16,
            ..Config::default()
        };

        assert!(
            compact_once(
                &registry,
                Arc::new(tokio::sync::RwLock::new(())),
                &traces_root,
                Some(&cache),
                &config,
            )
            .await
            .unwrap()
        );

        let manifest = storage.load_trace_manifest().await.unwrap();
        assert_eq!(manifest.generation, generation_before + 1);
        let replacement = registry.snapshot()[0].part().clone();
        assert_eq!(manifest.parts, vec![descriptor_of(&registry.snapshot()[0])]);
        assert_eq!(all_spans(&registry), before);

        let repeated = storage
            .replace_trace_parts(&[replacement], &input_descriptors)
            .await
            .unwrap();
        assert_eq!(repeated.generation, manifest.generation);
        std::fs::remove_dir_all(&data_dir).ok();
    }

    #[tokio::test]
    async fn a_remote_restart_replays_a_compaction_that_crashed_before_the_manifest_swap() {
        let data_dir = temp_root("remote-replay");
        let traces_root = data_dir.join("traces");
        for part_index in 0..COMPACT_MIN_PARTS {
            flush_one_part(&traces_root, part_index);
        }
        let storage = crate::object_storage::ObjectStorage::in_memory();
        storage
            .publish_trace_parts(&trace_part::discover_trace_parts(&traces_root).unwrap())
            .await
            .unwrap();
        let registry =
            TraceRegistry::load_from_disk(&traces_root, Arc::new(tokio::sync::RwLock::new(())))
                .unwrap();
        let before = all_spans(&registry);
        let inputs = select_inputs(&registry.snapshot(), FIXTURE_NOW_NS).unwrap();
        let spans = read_all_spans(&inputs).unwrap();
        let new_parts = trace_part::flush_trace_spans(&spans, &traces_root, 16).unwrap();
        let record = CompactRecord {
            new: new_parts
                .iter()
                .map(|part| relative_dir(&traces_root, &part.dir).unwrap())
                .collect(),
            inputs: inputs
                .iter()
                .map(|reader| relative_dir(&traces_root, &reader.part().dir).unwrap())
                .collect(),
        };
        write_record(&traces_root, &new_parts[0].meta.id, &record).unwrap();

        let manifest = storage
            .reconcile_trace_local_cache(&traces_root)
            .await
            .unwrap();

        assert_eq!(
            manifest
                .parts
                .iter()
                .map(|part| part.id.clone())
                .collect::<Vec<_>>(),
            vec![new_parts[0].meta.id.clone()],
            "the replacement stands alone in the manifest, not beside its inputs"
        );
        assert!(read_records(&traces_root).unwrap().is_empty());
        let restarted = TraceRegistry::load_from_manifest(
            &traces_root,
            &manifest,
            Arc::new(tokio::sync::RwLock::new(())),
        )
        .unwrap();
        assert_eq!(all_spans(&restarted), before);
        std::fs::remove_dir_all(&data_dir).ok();
    }

    #[test]
    fn recovery_keeps_one_copy_when_the_replacement_is_durable() {
        let root = temp_root("recover-durable");
        for part_index in 0..COMPACT_MIN_PARTS {
            flush_one_part(&root, part_index);
        }
        let registry =
            TraceRegistry::load_from_disk(&root, Arc::new(tokio::sync::RwLock::new(()))).unwrap();
        let before = all_spans(&registry);
        let inputs = select_inputs(&registry.snapshot(), FIXTURE_NOW_NS).unwrap();
        let spans = read_all_spans(&inputs).unwrap();
        let new_parts = trace_part::flush_trace_spans(&spans, &root, 16).unwrap();
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

        let recovered =
            TraceRegistry::load_from_disk(&root, Arc::new(tokio::sync::RwLock::new(()))).unwrap();
        assert_eq!(all_spans(&recovered), before);
        assert!(read_records(&root).unwrap().is_empty());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn recovery_keeps_the_inputs_when_the_replacement_never_became_durable() {
        let root = temp_root("recover-undone");
        flush_one_part(&root, 0);
        let registry =
            TraceRegistry::load_from_disk(&root, Arc::new(tokio::sync::RwLock::new(()))).unwrap();
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
    fn recovery_removes_a_partial_replacement_before_registry_restart() {
        let root = temp_root("recover-partial");
        flush_one_part(&root, 0);
        let registry =
            TraceRegistry::load_from_disk(&root, Arc::new(tokio::sync::RwLock::new(()))).unwrap();
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
    fn process_crashes_at_each_trace_compaction_boundary_recover_without_loss() {
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
            for part_index in 0..COMPACT_MIN_PARTS {
                flush_one_part(&root, part_index);
            }
            let registry =
                TraceRegistry::load_from_disk(&root, Arc::new(tokio::sync::RwLock::new(())))
                    .unwrap();
            let before = all_spans(&registry);
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
                    "trace_merge::tests::trace_compaction_fault_helper",
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
                let restarted =
                    TraceRegistry::load_from_disk(&root, Arc::new(tokio::sync::RwLock::new(())))
                        .unwrap();
                assert_eq!(all_spans(&restarted), before, "fault point {point}");
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
    async fn trace_compaction_fault_helper() {
        let Some(root) = std::env::var_os("SIGNY_TEST_COMPACTION_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        if std::env::var("SIGNY_TEST_COMPACTION_CRASH_POINT").as_deref()
            == Ok("recovery_record_processing")
        {
            recover_local_compactions(&root).unwrap();
        } else {
            let registry =
                TraceRegistry::load_from_disk(&root, Arc::new(tokio::sync::RwLock::new(())))
                    .unwrap();
            let config = Config {
                row_group_size: 16,
                ..Config::default()
            };
            compact_once(
                &registry,
                Arc::new(tokio::sync::RwLock::new(())),
                &root,
                None,
                &config,
            )
            .await
            .unwrap();
        }
    }
}
