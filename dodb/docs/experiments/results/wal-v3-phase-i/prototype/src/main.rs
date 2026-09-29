use std::alloc::{GlobalAlloc, Layout};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use dodb_core::{
    DocumentKey, PrimaryKey, Result, Revision, RevisionState, SortKey, TransactionCondition,
    TransactionMutation, TransactionRequest,
};
use dodb_storage::blink::BlinkReadHandle;
use dodb_storage::{BlinkStore, DatabaseConfig, DurableFile};

struct AllocationCounter;

static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static ALLOCATION_BYTES: AtomicU64 = AtomicU64::new(0);
static SOURCE_COMMIT: OnceLock<String> = OnceLock::new();

fn source_commit() -> &'static str {
    SOURCE_COMMIT
        .get_or_init(|| {
            let output = std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .output()
                .expect("git should be available in the benchmark checkout");
            assert!(output.status.success());
            String::from_utf8(output.stdout)
                .expect("git commit should be UTF-8")
                .trim()
                .to_owned()
        })
        .as_str()
}

unsafe impl GlobalAlloc for AllocationCounter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOCATION_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { mimalloc::MiMalloc.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOCATION_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { mimalloc::MiMalloc.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { mimalloc::MiMalloc.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOCATION_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        unsafe { mimalloc::MiMalloc.realloc(pointer, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: AllocationCounter = AllocationCounter;

struct VolatileFile {
    file: File,
}

impl VolatileFile {
    fn create(path: &Path) -> Result<Self> {
        Ok(Self {
            file: OpenOptions::new()
                .create(true)
                .truncate(true)
                .read(true)
                .write(true)
                .open(path)?,
        })
    }
}

impl DurableFile for VolatileFile {
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.file.seek(SeekFrom::Start(offset))?;
        Ok(self.file.read(buffer)?)
    }

    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<usize> {
        self.file.seek(SeekFrom::Start(offset))?;
        Ok(self.file.write(bytes)?)
    }

    fn len(&self) -> Result<u64> {
        Ok(self.file.metadata()?.len())
    }

    fn set_len(&mut self, length: u64) -> Result<()> {
        self.file.set_len(length)?;
        Ok(())
    }

    fn sync_data(&mut self) -> Result<()> {
        Ok(())
    }

    fn sync_all(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
struct VersionedValue {
    revision: Revision,
    value: Option<Arc<[u8]>>,
}

struct OverlaySegment {
    entries: Vec<(DocumentKey, VersionedValue)>,
}

struct PublishedView {
    base: BlinkReadHandle,
    segments: Arc<[Arc<OverlaySegment>]>,
}

impl PublishedView {
    fn lookup(&self, key: &DocumentKey) -> Result<(VersionedValue, usize)> {
        for (segments_consulted, segment) in self.segments.iter().rev().enumerate() {
            if let Ok(entry_index) = segment.entries.binary_search_by(|entry| entry.0.cmp(key)) {
                return Ok((
                    segment.entries[entry_index].1.clone(),
                    segments_consulted + 1,
                ));
            }
        }
        match self.base.get(key)? {
            RevisionState::Present { value, revision } => Ok((
                VersionedValue {
                    revision,
                    value: Some(Arc::from(value)),
                },
                self.segments.len(),
            )),
            RevisionState::Missing { revision } => Ok((
                VersionedValue {
                    revision,
                    value: None,
                },
                self.segments.len(),
            )),
        }
    }

    fn get(&self, key: &DocumentKey) -> Result<RevisionState> {
        let (value, _) = self.lookup(key)?;
        Ok(match value.value {
            Some(bytes) => RevisionState::present(bytes.as_ref().to_vec(), value.revision),
            None => RevisionState::missing(value.revision),
        })
    }

    fn matching_entries<'segment>(
        &'segment self,
        segment: &'segment OverlaySegment,
        after: Option<&DocumentKey>,
        limit: usize,
        primary_key: Option<&PrimaryKey>,
    ) -> impl Iterator<Item = &'segment (DocumentKey, VersionedValue)> {
        let start = segment.entries.partition_point(|entry| {
            after.is_some_and(|after_key| entry.0 <= *after_key)
                || primary_key.is_some_and(|pk| entry.0.pk < *pk)
        });
        segment.entries[start..]
            .iter()
            .take_while(move |entry| primary_key.is_none_or(|pk| entry.0.pk == *pk))
            .take(limit)
    }

    fn query(
        &self,
        primary_key: &PrimaryKey,
        exclusive_after_sort_key: Option<&SortKey>,
        limit: usize,
    ) -> Result<Vec<dodb_storage::Document>> {
        let overlay_entry_count = self
            .segments
            .iter()
            .map(|segment| {
                self.matching_entries(segment, None, usize::MAX, Some(primary_key))
                    .count()
            })
            .sum::<usize>();
        let base_limit = limit.saturating_add(overlay_entry_count);
        let base_rows = self
            .base
            .query(primary_key, exclusive_after_sort_key, base_limit)?;
        let mut merged = BTreeMap::new();
        for row in base_rows {
            if exclusive_after_sort_key.is_none_or(|sort_key| row.key.sk > *sort_key) {
                merged.insert(
                    row.key.clone(),
                    VersionedValue {
                        revision: row.revision,
                        value: Some(Arc::from(row.value)),
                    },
                );
            }
        }
        for segment in self.segments.iter() {
            for (key, value) in self.matching_entries(segment, None, usize::MAX, Some(primary_key))
            {
                if exclusive_after_sort_key.is_some_and(|sort_key| key.sk <= *sort_key) {
                    continue;
                }
                match &value.value {
                    Some(_) => {
                        merged.insert(key.clone(), value.clone());
                    }
                    None => {
                        merged.remove(key);
                    }
                }
            }
        }
        Ok(merged
            .into_iter()
            .take(limit)
            .map(|(key, value)| dodb_storage::Document {
                key,
                value: value.value.unwrap().as_ref().to_vec(),
                revision: value.revision,
            })
            .collect())
    }

    fn scan(
        &self,
        cursor: Option<&DocumentKey>,
        limit: usize,
    ) -> Result<Vec<dodb_storage::Document>> {
        let overlay_entry_count = self
            .segments
            .iter()
            .map(|segment| {
                self.matching_entries(segment, cursor, usize::MAX, None)
                    .count()
            })
            .sum::<usize>();
        let base_limit = limit.saturating_add(overlay_entry_count);
        let base_rows = self.base.scan(cursor, base_limit)?;
        let mut merged = BTreeMap::new();
        for row in base_rows {
            merged.insert(
                row.key.clone(),
                VersionedValue {
                    revision: row.revision,
                    value: Some(Arc::from(row.value)),
                },
            );
        }
        for segment in self.segments.iter() {
            for (key, value) in self.matching_entries(segment, cursor, usize::MAX, None) {
                match &value.value {
                    Some(_) => {
                        merged.insert(key.clone(), value.clone());
                    }
                    None => {
                        merged.remove(key);
                    }
                }
            }
        }
        Ok(merged
            .into_iter()
            .take(limit)
            .map(|(key, value)| dodb_storage::Document {
                key,
                value: value.value.unwrap().as_ref().to_vec(),
                revision: value.revision,
            })
            .collect())
    }
}

#[derive(Default)]
struct GroupTiming {
    admission_nanos: u64,
    overlay_mutation_nanos: u64,
    freeze_sort_nanos: u64,
    publication_nanos: u64,
}

struct OverlayPrototype {
    published: Arc<PublishedView>,
    next_sequence: u64,
}

impl OverlayPrototype {
    fn apply_group(&mut self, requests: &[TransactionRequest]) -> Result<(Vec<bool>, GroupTiming)> {
        let mut group_overlay = BTreeMap::<DocumentKey, VersionedValue>::new();
        let mut results = Vec::with_capacity(requests.len());
        let mut successful_count = 0usize;
        let mut timing = GroupTiming::default();
        for request in requests {
            let admission_started = Instant::now();
            if request.validate().is_err() {
                timing.admission_nanos = timing
                    .admission_nanos
                    .saturating_add(admission_started.elapsed().as_nanos() as u64);
                results.push(false);
                continue;
            }
            let mut conditions_hold = true;
            for condition in &request.conditions {
                let state = match group_overlay.get(condition.key()) {
                    Some(value) => state_from_value(value),
                    None => self.published.get(condition.key())?,
                };
                conditions_hold &= match condition {
                    TransactionCondition::RevisionEquals {
                        expected_revision, ..
                    } => state.revision() == *expected_revision,
                    TransactionCondition::Exists { .. } => !state.is_missing(),
                    TransactionCondition::NotExists { .. } => state.is_missing(),
                };
            }
            if !conditions_hold {
                timing.admission_nanos = timing
                    .admission_nanos
                    .saturating_add(admission_started.elapsed().as_nanos() as u64);
                results.push(false);
                continue;
            }
            timing.admission_nanos = timing
                .admission_nanos
                .saturating_add(admission_started.elapsed().as_nanos() as u64);
            let revision = Revision::new(self.next_sequence);
            self.next_sequence = self.next_sequence.saturating_add(1);
            let mutation_started = Instant::now();
            for mutation in &request.mutations {
                let value = match mutation {
                    TransactionMutation::Put { value, .. } => Some(Arc::from(value.clone())),
                    TransactionMutation::Delete { .. } => None,
                };
                group_overlay.insert(mutation.key().clone(), VersionedValue { revision, value });
            }
            timing.overlay_mutation_nanos = timing
                .overlay_mutation_nanos
                .saturating_add(mutation_started.elapsed().as_nanos() as u64);
            successful_count += 1;
            results.push(true);
        }
        if successful_count == 0 {
            return Ok((results, timing));
        }
        let freeze_started = Instant::now();
        let segment = Arc::new(OverlaySegment {
            entries: group_overlay.into_iter().collect(),
        });
        timing.freeze_sort_nanos = freeze_started.elapsed().as_nanos() as u64;
        let publication_started = Instant::now();
        let mut segments = self.published.segments.iter().cloned().collect::<Vec<_>>();
        segments.push(segment);
        self.published = Arc::new(PublishedView {
            base: self.published.base.clone(),
            segments: segments.into(),
        });
        timing.publication_nanos = publication_started.elapsed().as_nanos() as u64;
        Ok((results, timing))
    }
}

fn state_from_value(value: &VersionedValue) -> RevisionState {
    match &value.value {
        Some(bytes) => RevisionState::present(bytes.as_ref().to_vec(), value.revision),
        None => RevisionState::missing(value.revision),
    }
}

fn key_for(index: u64) -> DocumentKey {
    DocumentKey::from_parts(
        PrimaryKey::new((index / 100).to_be_bytes().to_vec()),
        SortKey::new(index.to_be_bytes().to_vec()),
    )
}

fn request_group(seed: u64, transaction_count: usize, width: usize) -> Vec<TransactionRequest> {
    let mut random = seed;
    (0..transaction_count)
        .map(|transaction_index| {
            let mut selected = BTreeSet::new();
            let mut mutations = Vec::with_capacity(width);
            while mutations.len() < width {
                random = random
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let key_index = random % 100_000;
                if selected.insert(key_index) {
                    mutations.push(TransactionMutation::Put {
                        key: key_for(key_index),
                        value: vec![(transaction_index % 251) as u8; 64],
                    });
                }
            }
            TransactionRequest::new(Vec::new(), mutations)
        })
        .collect()
}

fn seed_base(
    directory: &Path,
) -> Result<(BlinkStore<VolatileFile, VolatileFile>, PathBuf, PathBuf)> {
    let data_path = directory.join(format!("phase-i-overlay-{}.db", std::process::id()));
    let wal_path = data_path.with_extension("wal");
    let mut store = BlinkStore::open_with_wal(
        VolatileFile::create(&data_path)?,
        VolatileFile::create(&wal_path)?,
        DatabaseConfig::default(),
    )?;
    store.enable_planned_execution();
    for offset in (0..100_000u64).step_by(64) {
        let end = (offset + 64).min(100_000);
        let requests = (offset..end)
            .map(|key_index| {
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: key_for(key_index),
                        value: vec![(key_index % 251) as u8; 64],
                    }],
                )
            })
            .collect::<Vec<_>>();
        store.apply_transaction_group(&requests)?;
    }
    Ok((store, data_path, wal_path))
}

fn segment_for_range(start: u64, count: usize, revision: u64) -> Arc<OverlaySegment> {
    let entries = (0..count)
        .map(|offset| {
            let key_index = start + offset as u64;
            (
                key_for(key_index),
                VersionedValue {
                    revision: Revision::new(revision),
                    value: (offset % 11 != 0).then(|| Arc::from(vec![(offset % 251) as u8; 64])),
                },
            )
        })
        .collect();
    Arc::new(OverlaySegment { entries })
}

fn ticks() -> u64 {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let result = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut time) };
    assert_eq!(result, 0);
    (time.tv_sec as u64)
        .saturating_mul(1_000_000_000)
        .saturating_add(time.tv_nsec as u64)
}

fn reset_allocation_counters() -> (u64, u64) {
    (
        ALLOCATION_COUNT.swap(0, Ordering::Relaxed),
        ALLOCATION_BYTES.swap(0, Ordering::Relaxed),
    )
}

fn emit_write_measurements(
    base: BlinkReadHandle,
    requests: &[TransactionRequest],
    segment_count: usize,
) -> Result<()> {
    let seed_segments = (0..segment_count)
        .map(|segment_index| {
            segment_for_range(
                10_000 + segment_index as u64 * 1_000,
                704,
                segment_index as u64 + 1,
            )
        })
        .collect::<Vec<_>>();
    let published = Arc::new(PublishedView {
        base,
        segments: seed_segments.into(),
    });
    let mut prototype = OverlayPrototype {
        published: Arc::clone(&published),
        next_sequence: 2_000_000,
    };
    let iteration_count = 200usize;
    let transactions = iteration_count * requests.len();
    reset_allocation_counters();
    let cpu_started = ticks();
    let wall_started = Instant::now();
    let mut totals = GroupTiming::default();
    for _ in 0..iteration_count {
        let (results, timing) = prototype.apply_group(requests)?;
        assert!(results.iter().all(|success| *success));
        totals.admission_nanos += timing.admission_nanos;
        totals.overlay_mutation_nanos += timing.overlay_mutation_nanos;
        totals.freeze_sort_nanos += timing.freeze_sort_nanos;
        totals.publication_nanos += timing.publication_nanos;
        prototype.published = Arc::clone(&published);
    }
    let wall_nanos = wall_started.elapsed().as_nanos() as u64;
    let cpu_nanos = ticks().saturating_sub(cpu_started);
    let (allocations, allocated_bytes) = reset_allocation_counters();
    println!(
        "{{\"record_type\":\"write_cpu\",\"git_commit\":{},\"segments_before_commit\":{},\"iterations\":{},\"successful_transactions\":{},\"logical_mutations\":{},\"cpu_ns_per_tx\":{},\"wall_ns_per_tx\":{},\"allocations_per_tx\":{:.4},\"allocated_bytes_per_tx\":{:.2},\"admission_ns_per_tx\":{:.2},\"overlay_mutation_ns_per_tx\":{:.2},\"freeze_sort_ns_per_tx\":{:.2},\"publication_ns_per_tx\":{:.2}}}",
        json_string(source_commit()),
        segment_count,
        iteration_count,
        transactions,
        transactions * requests[0].mutations.len(),
        cpu_nanos / transactions as u64,
        wall_nanos / transactions as u64,
        allocations as f64 / transactions as f64,
        allocated_bytes as f64 / transactions as f64,
        totals.admission_nanos as f64 / transactions as f64,
        totals.overlay_mutation_nanos as f64 / transactions as f64,
        totals.freeze_sort_nanos as f64 / transactions as f64,
        totals.publication_nanos as f64 / transactions as f64,
    );
    Ok(())
}

fn measure_reads(view: &PublishedView, segment_count: usize) -> Result<()> {
    let base_hit = key_for(99_999);
    let base_miss = key_for(200_000);
    let oldest_hit = key_for(1);
    let newest_hit = key_for(segment_count.saturating_sub(1) as u64 * 704 + 1);
    let cases = [
        ("newest", newest_hit),
        ("oldest", oldest_hit),
        ("base_hit", base_hit),
        ("miss", base_miss),
    ];
    for (case_name, key) in cases {
        let operation_count = 10_000u64;
        let started = Instant::now();
        let mut segments_consulted = 0usize;
        for _ in 0..operation_count {
            let (value, consulted) = view.lookup(&key)?;
            segments_consulted += consulted;
            std::hint::black_box(value);
        }
        let elapsed = started.elapsed().as_nanos() as u64;
        let base_started = Instant::now();
        for _ in 0..operation_count {
            std::hint::black_box(view.base.get(&key)?);
        }
        let base_elapsed = base_started.elapsed().as_nanos() as u64;
        println!(
            "{{\"record_type\":\"get\",\"git_commit\":{},\"segments\":{},\"case\":{},\"operations\":{},\"ns_per_op\":{:.2},\"ops_per_second\":{:.2},\"segments_consulted_per_op\":{:.2},\"h1_base_ns_per_op\":{:.2},\"prototype_over_base\":{:.3}}}",
            json_string(source_commit()),
            segment_count,
            json_string(case_name),
            operation_count,
            elapsed as f64 / operation_count as f64,
            operation_count as f64 * 1_000_000_000.0 / elapsed.max(1) as f64,
            segments_consulted as f64 / operation_count as f64,
            base_elapsed as f64 / operation_count as f64,
            elapsed as f64 / base_elapsed.max(1) as f64,
        );
    }
    let query_primary_key = key_for(12_345).pk;
    for operation in ["query", "scan"] {
        let started = Instant::now();
        let operation_count = 20u64;
        let mut prototype_rows_read = 0usize;
        for _ in 0..operation_count {
            prototype_rows_read += if operation == "query" {
                view.query(&query_primary_key, None, 100)?.len()
            } else {
                view.scan(None, 100)?.len()
            };
        }
        let elapsed = started.elapsed().as_nanos() as u64;
        let base_started = Instant::now();
        let mut base_rows_read = 0usize;
        for _ in 0..operation_count {
            base_rows_read += if operation == "query" {
                view.base.query(&query_primary_key, None, 100)?.len()
            } else {
                view.base.scan(None, 100)?.len()
            };
        }
        let base_elapsed = base_started.elapsed().as_nanos() as u64;
        println!(
            "{{\"record_type\":\"range_read\",\"git_commit\":{},\"segments\":{},\"operation\":{},\"operations\":{},\"rows_per_operation\":{:.2},\"h1_base_rows_per_operation\":{:.2},\"ns_per_op\":{:.2},\"ops_per_second\":{:.2},\"h1_base_ns_per_op\":{:.2},\"prototype_over_base\":{:.3}}}",
            json_string(source_commit()),
            segment_count,
            json_string(operation),
            operation_count,
            prototype_rows_read as f64 / operation_count as f64,
            base_rows_read as f64 / operation_count as f64,
            elapsed as f64 / operation_count as f64,
            operation_count as f64 * 1_000_000_000.0 / elapsed.max(1) as f64,
            base_elapsed as f64 / operation_count as f64,
            elapsed as f64 / base_elapsed.max(1) as f64,
        );
    }
    Ok(())
}

fn correctness_check(base: BlinkReadHandle) -> Result<()> {
    let key_a = DocumentKey::new(b"check".to_vec(), b"a".to_vec());
    let key_b = DocumentKey::new(b"check".to_vec(), b"b".to_vec());
    let key_c = DocumentKey::new(b"check".to_vec(), b"c".to_vec());
    let initial = Arc::new(PublishedView {
        base,
        segments: Arc::from([]),
    });
    let mut prototype = OverlayPrototype {
        published: Arc::clone(&initial),
        next_sequence: 900_000,
    };
    let old_view = Arc::clone(&prototype.published);
    let first_revision = Revision::new(900_000);
    let requests = vec![
        TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Put {
                key: key_a.clone(),
                value: b"a".to_vec(),
            }],
        ),
        TransactionRequest::new(
            vec![TransactionCondition::Exists { key: key_a.clone() }],
            vec![TransactionMutation::Put {
                key: key_b.clone(),
                value: b"b".to_vec(),
            }],
        ),
        TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: key_b.clone(),
                expected_revision: Revision::new(900_001),
            }],
            vec![TransactionMutation::Put {
                key: key_c.clone(),
                value: b"c".to_vec(),
            }],
        ),
        TransactionRequest::new(
            vec![TransactionCondition::NotExists { key: key_a.clone() }],
            vec![TransactionMutation::Put {
                key: DocumentKey::new(b"check".to_vec(), b"failed".to_vec()),
                value: b"failed".to_vec(),
            }],
        ),
    ];
    let (results, _) = prototype.apply_group(&requests)?;
    assert_eq!(results, [true, true, true, false]);
    assert!(matches!(
        old_view.get(&key_a)?,
        RevisionState::Missing { .. }
    ));
    assert_eq!(
        prototype.published.get(&key_a)?.value(),
        Some(b"a".as_slice())
    );
    assert_eq!(
        prototype.published.get(&key_b)?.value(),
        Some(b"b".as_slice())
    );
    assert_eq!(
        prototype.published.get(&key_c)?.value(),
        Some(b"c".as_slice())
    );
    assert_eq!(prototype.published.get(&key_a)?.revision(), first_revision);
    let delete_and_read = vec![
        TransactionRequest::new(
            vec![TransactionCondition::Exists { key: key_a.clone() }],
            vec![TransactionMutation::Delete { key: key_a.clone() }],
        ),
        TransactionRequest::new(
            vec![TransactionCondition::NotExists { key: key_a.clone() }],
            vec![TransactionMutation::Put {
                key: DocumentKey::new(b"check".to_vec(), b"after-delete".to_vec()),
                value: b"visible".to_vec(),
            }],
        ),
    ];
    let (delete_results, _) = prototype.apply_group(&delete_and_read)?;
    assert_eq!(delete_results, [true, true]);
    assert!(prototype.published.get(&key_a)?.is_missing());
    assert!(matches!(
        old_view.get(&key_a)?,
        RevisionState::Missing { .. }
    ));
    let query_rows = prototype
        .published
        .query(&key_a.pk, None, 10)?
        .into_iter()
        .map(|row| row.key)
        .collect::<BTreeSet<_>>();
    let scan_rows = prototype
        .published
        .scan(None, 10)?
        .into_iter()
        .map(|row| row.key)
        .collect::<BTreeSet<_>>();
    assert!(!query_rows.contains(&key_a));
    assert!(!scan_rows.contains(&key_a));
    println!(
        "{{\"record_type\":\"correctness\",\"git_commit\":{},\"ordered_conditions\":true,\"failed_transaction_isolation\":true,\"delete_tombstone\":true,\"pinned_view_immutability\":true,\"query_overlay_merge\":true,\"scan_overlay_merge\":true}}",
        json_string(source_commit())
    );
    Ok(())
}

fn json_string(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() + 2);
    encoded.push('"');
    for character in value.chars() {
        match character {
            '"' => encoded.push_str("\\\""),
            '\\' => encoded.push_str("\\\\"),
            '\n' => encoded.push_str("\\n"),
            '\r' => encoded.push_str("\\r"),
            '\t' => encoded.push_str("\\t"),
            character => encoded.push(character),
        }
    }
    encoded.push('"');
    encoded
}

fn run() -> Result<()> {
    let directory = std::env::var_os("TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&directory)?;
    let (store, data_path, wal_path) = seed_base(&directory)?;
    let base = store.versioned_read_handle();
    correctness_check(base.clone())?;
    let requests = request_group(1_100_000, 44, 16);
    for segment_count in [0, 1, 2, 4, 8, 16, 32] {
        emit_write_measurements(base.clone(), &requests, segment_count)?;
    }
    for segment_count in [1, 2, 4, 8, 16, 32] {
        let segments = (0..segment_count)
            .map(|segment_index| {
                segment_for_range(segment_index as u64 * 704, 704, segment_index as u64 + 1)
            })
            .collect::<Vec<_>>();
        let view = PublishedView {
            base: base.clone(),
            segments: segments.into(),
        };
        measure_reads(&view, segment_count)?;
    }
    drop(store);
    std::fs::remove_file(data_path)?;
    std::fs::remove_file(wal_path)?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("phase-i overlay prototype failed: {error}");
        std::process::exit(1);
    }
}
