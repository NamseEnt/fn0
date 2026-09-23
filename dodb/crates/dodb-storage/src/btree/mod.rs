//! Phase 1 single-file mutable B+Tree.
//!
//! The engine owns one data [`DurableFile`] and optionally a separate redo-only
//! WAL. The single-file constructor remains available for Phase 1 format tests;
//! production path opening uses the WAL-backed constructor.

mod cache;
mod checker;
mod coordinator;
mod format;

pub use coordinator::{AsyncShard, CoordinatorConfig, CoordinatorMetrics};

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::Path;
use std::time::Instant;

use dodb_core::{
    DocumentKey, Error, Lsn, ObservedState, PageId, PrimaryKey, Result, Revision, RevisionState,
    ShardEpoch, ShardId, SortKey, TenantId, TransactionCondition, TransactionConflict,
    TransactionMutation, TransactionRequest, TransactionResult,
};

use self::cache::PageCache;
use self::format::{
    INLINE_VALUE_LIMIT, MAX_OVERFLOW_PAGES, MAX_VALUE_SIZE, PageData, ValueRef, internal_fits,
    leaf_fits,
};
use crate::PAGE_SIZE;
use crate::fault::FaultInjector;
use crate::wal::{CommittedWalBatch, WalCommit, WalIdentity, WalLog, WalMetrics, WalPageImage};
use crate::{
    DurableFile, ProductionFile, Superblock, SuperblockSlot, choose_superblock, decode_page_at,
    decode_superblock, encode_superblock,
};

const FIRST_DATA_PAGE: u64 = 2;
/// Maximum canonical key length supported by a leaf record, in bytes.
pub const MAX_ENCODED_KEY_SIZE: usize = 3992;
const SUPERBLOCK_A_OFFSET: u64 = 0;
const SUPERBLOCK_B_OFFSET: u64 = PAGE_SIZE as u64;

/// Marker file type used by the compatibility constructor that has no
/// separate WAL file.
#[derive(Debug, Default)]
pub struct NoWal;

impl DurableFile for NoWal {
    fn read_at(&mut self, _offset: u64, _buffer: &mut [u8]) -> Result<usize> {
        Ok(0)
    }

    fn write_at(&mut self, _offset: u64, _bytes: &[u8]) -> Result<usize> {
        Err(Error::invariant("NoWal cannot receive WAL writes"))
    }

    fn len(&self) -> Result<u64> {
        Ok(0)
    }

    fn set_len(&mut self, _length: u64) -> Result<()> {
        Ok(())
    }

    fn sync_data(&mut self) -> Result<()> {
        Ok(())
    }

    fn sync_all(&mut self) -> Result<()> {
        Ok(())
    }
}

/// The supported value and page-layout limits for Phase 1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageLimits {
    /// Values at or below this size are stored inline when the leaf record fits.
    pub inline_value_limit: usize,
    /// Larger values are stored in overflow pages up to this size.
    pub max_value_size: usize,
}

impl Default for StorageLimits {
    fn default() -> Self {
        Self {
            inline_value_limit: INLINE_VALUE_LIMIT,
            max_value_size: MAX_VALUE_SIZE,
        }
    }
}

/// Configuration used only when creating a new file and for cache sizing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatabaseConfig {
    pub database_uuid: [u8; 16],
    pub tenant_id: TenantId,
    pub shard_id: ShardId,
    pub shard_epoch: ShardEpoch,
    pub cache_capacity: usize,
    pub limits: StorageLimits,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_uuid: [0; 16],
            tenant_id: TenantId::ZERO,
            shard_id: ShardId::ZERO,
            shard_epoch: ShardEpoch::ZERO,
            cache_capacity: 128,
            limits: StorageLimits::default(),
        }
    }
}

impl DatabaseConfig {
    pub fn with_cache_capacity(mut self, capacity: usize) -> Self {
        self.cache_capacity = capacity;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Document {
    pub key: DocumentKey,
    pub value: Vec<u8>,
    pub revision: Revision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Mutation {
    Put { key: DocumentKey, value: Vec<u8> },
    Delete { key: DocumentKey },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchRequest {
    Get {
        key: DocumentKey,
    },
    Put {
        key: DocumentKey,
        value: Vec<u8>,
    },
    Delete {
        key: DocumentKey,
    },
    Query {
        pk: PrimaryKey,
        exclusive_after_sk: Option<SortKey>,
        limit: usize,
    },
    Scan {
        exclusive_after_key: Option<DocumentKey>,
        limit: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchResponse {
    Get(RevisionState),
    Put(Revision),
    Delete(Revision),
    Query(Vec<Document>),
    Scan(Vec<Document>),
}

pub type EngineRequest = BatchRequest;
pub type EngineResponse = BatchResponse;

/// A complete page-oriented result of preparing one request batch.
///
/// The changed pages are full encoded page images.  Phase 2 can consume this
/// map as WAL after-images before calling [`BTreeStore::publish_prepared`].
#[derive(Debug)]
pub struct PreparedBatch {
    responses: Vec<BatchResponse>,
    changed_pages: BTreeMap<PageId, [u8; PAGE_SIZE]>,
    changed_decoded: BTreeMap<PageId, PageData>,
    read_pages: BTreeMap<PageId, PageData>,
    new_superblock: Superblock,
    new_slot: SuperblockSlot,
    base_generation: u64,
    next_revision: Revision,
    root_page_id: PageId,
    free_list_head: Option<PageId>,
    high_water_page_id: PageId,
    commit_lsn: Option<Lsn>,
    batch_id: u64,
}

impl PreparedBatch {
    pub fn responses(&self) -> &[BatchResponse] {
        &self.responses
    }

    pub fn changed_pages(&self) -> &BTreeMap<PageId, [u8; PAGE_SIZE]> {
        &self.changed_pages
    }

    pub fn superblock(&self) -> &Superblock {
        &self.new_superblock
    }

    pub fn superblock_image(&self) -> Result<[u8; PAGE_SIZE]> {
        encode_superblock(&self.new_superblock)
    }

    pub fn superblock_slot(&self) -> SuperblockSlot {
        self.new_slot
    }

    pub fn read_page_ids(&self) -> impl Iterator<Item = PageId> + '_ {
        self.read_pages.keys().copied()
    }

    pub fn commit_lsn(&self) -> Option<Lsn> {
        self.commit_lsn
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvariantReport {
    pub reachable_pages: usize,
    pub free_pages: usize,
    pub leaked_pages: Vec<PageId>,
    pub max_revision: Revision,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StorageMetrics {
    pub validation_nanos: u64,
    pub btree_preparation_nanos: u64,
    pub publication_nanos: u64,
}

const RESPONSE_WIRE_OVERHEAD: usize = 17;
const RESPONSE_MEMORY_OVERHEAD: usize = 64 * 1024;

struct ResponseBudget {
    maximum_wire: usize,
    maximum_memory: usize,
    used_wire: usize,
    used_memory: usize,
}

impl ResponseBudget {
    fn new(maximum: usize) -> Result<Self> {
        if maximum < RESPONSE_WIRE_OVERHEAD {
            return Err(Error::response_too_large(format!(
                "response envelope exceeds {maximum} bytes"
            )));
        }
        Ok(Self {
            maximum_wire: maximum,
            maximum_memory: maximum.saturating_add(RESPONSE_MEMORY_OVERHEAD),
            used_wire: RESPONSE_WIRE_OVERHEAD,
            used_memory: 0,
        })
    }

    fn reserve(&mut self, wire: usize, memory: usize) -> Result<()> {
        let next_wire = self
            .used_wire
            .checked_add(wire)
            .ok_or_else(|| Error::response_too_large("encoded response size overflows"))?;
        let next_memory = self
            .used_memory
            .checked_add(memory)
            .ok_or_else(|| Error::response_too_large("response materialization size overflows"))?;
        if next_wire > self.maximum_wire || next_memory > self.maximum_memory {
            return Err(Error::response_too_large(format!(
                "response materialization exceeds {} bytes",
                self.maximum_wire
            )));
        }
        self.used_wire = next_wire;
        self.used_memory = next_memory;
        Ok(())
    }

    fn reserve_state(&mut self, value_len: usize) -> Result<()> {
        let wire = 1 + 4 + value_len + 8;
        self.reserve(wire, value_len.saturating_add(32))
    }

    fn reserve_document(&mut self, key: &DocumentKey, value_len: usize) -> Result<()> {
        let materialized = key
            .pk
            .as_bytes()
            .len()
            .saturating_add(key.sk.as_bytes().len())
            .saturating_add(value_len)
            .saturating_add(64);
        let encoded = key
            .pk
            .as_bytes()
            .len()
            .saturating_add(key.sk.as_bytes().len())
            .saturating_add(value_len)
            .saturating_add(20);
        let wire = key
            .pk
            .as_bytes()
            .len()
            .saturating_add(key.sk.as_bytes().len())
            .saturating_add(value_len)
            .saturating_add(20);
        self.reserve(wire, materialized.max(encoded))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointReport {
    pub checkpoint_lsn: Lsn,
    pub pages_flushed: usize,
    pub bytes_written: u64,
    pub wal_bytes_reclaimed: u64,
    pub duration_nanos: u64,
}

/// A mutable, single-file B+Tree over canonical encoded document keys.
pub struct BTreeStore<F: DurableFile, W: DurableFile = NoWal> {
    file: F,
    wal: Option<WalLog<W>>,
    current_superblock: Superblock,
    active_slot: SuperblockSlot,
    root_page_id: PageId,
    free_list_head: Option<PageId>,
    high_water_page_id: PageId,
    next_revision: Revision,
    next_lsn: Lsn,
    next_batch_id: u64,
    cache: PageCache,
    config: DatabaseConfig,
    dirty_pages: BTreeMap<PageId, [u8; PAGE_SIZE]>,
    dirty_superblock: Option<[u8; PAGE_SIZE]>,
    track_published_pages: bool,
    published_page_updates: BTreeMap<PageId, PageData>,
    storage_metrics: StorageMetrics,
    broken: Option<String>,
    fault_injector: Option<Box<dyn FaultInjector + Send>>,
}

impl<F: DurableFile, W: DurableFile> BTreeStore<F, W> {
    /// Opens an existing database or initializes a truly empty file.
    ///
    /// A non-empty file is never recreated.  It must contain two complete
    /// superblock pages and a valid selected tree.
    fn open_no_wal(mut file: F, config: DatabaseConfig) -> Result<BTreeStore<F, NoWal>> {
        let length = file.len()?;
        if length == 0 {
            return BTreeStore::<F, NoWal>::initialize(file, config);
        }
        if length < (FIRST_DATA_PAGE * PAGE_SIZE as u64) || !length.is_multiple_of(PAGE_SIZE as u64)
        {
            return Err(Error::corruption(
                "database file length is not page aligned",
            ));
        }

        let slot_a = read_exact_at(&mut file, SUPERBLOCK_A_OFFSET, PAGE_SIZE)?;
        let slot_b = read_exact_at(&mut file, SUPERBLOCK_B_OFFSET, PAGE_SIZE)?;
        let selected = choose_superblock(&slot_a, &slot_b)?;
        let (root_page_id, free_list_head, high_water_page_id) =
            metadata_from_superblock(&selected.superblock, length)?;
        let mut store = BTreeStore::<F, NoWal> {
            file,
            wal: None,
            current_superblock: selected.superblock,
            active_slot: selected.slot,
            root_page_id,
            free_list_head,
            high_water_page_id,
            next_revision: Revision::new(1),
            next_lsn: Lsn::new(1),
            next_batch_id: 1,
            cache: PageCache::new(config.cache_capacity),
            config,
            dirty_pages: BTreeMap::new(),
            dirty_superblock: None,
            track_published_pages: false,
            published_page_updates: BTreeMap::new(),
            storage_metrics: StorageMetrics::default(),
            broken: None,
            fault_injector: None,
        };
        let report = store.check_invariants()?;
        store.next_revision = Revision::new(
            report
                .max_revision
                .get()
                .checked_add(1)
                .ok_or_else(|| Error::invariant("storage revision exhausted"))?,
        );
        Ok(store)
    }

    pub fn open_path(
        path: impl AsRef<Path>,
        config: DatabaseConfig,
    ) -> Result<BTreeStore<ProductionFile, ProductionFile>> {
        let path = path.as_ref();
        let wal_path = path.with_extension("wal");
        BTreeStore::<ProductionFile, ProductionFile>::open_with_wal(
            ProductionFile::open(path)?,
            ProductionFile::open(wal_path)?,
            config,
        )
    }

    pub fn open_with_wal(file: F, wal_file: W, config: DatabaseConfig) -> Result<BTreeStore<F, W>> {
        Self::open_with_wal_internal(file, wal_file, config, None)
    }

    pub fn open_with_wal_and_fault_injector<I>(
        file: F,
        wal_file: W,
        config: DatabaseConfig,
        injector: I,
    ) -> Result<BTreeStore<F, W>>
    where
        I: FaultInjector + Send + 'static,
    {
        Self::open_with_wal_internal(file, wal_file, config, Some(Box::new(injector)))
    }

    fn open_with_wal_internal(
        mut file: F,
        wal_file: W,
        config: DatabaseConfig,
        mut fault_injector: Option<Box<dyn FaultInjector + Send>>,
    ) -> Result<BTreeStore<F, W>> {
        let identity = WalIdentity::new(
            config.database_uuid,
            config.tenant_id,
            config.shard_id,
            config.shard_epoch,
        );
        let wal_length = wal_file.len()?;
        let checkpoint_hint =
            validate_existing_identity_before_wal(&mut file, &identity, wal_length)?
                .unwrap_or(Lsn::ZERO);
        let wal = WalLog::open_with_fault_injector_and_start_lsn(
            wal_file,
            identity.clone(),
            checkpoint_hint,
            fault_injector.as_deref_mut(),
        )?;
        if file.is_empty()? && wal.committed_batches().is_empty() {
            let mut store = BTreeStore::<F, W>::initialize(file, config)?;
            store.wal = Some(wal);
            store.next_lsn = store.wal.as_ref().unwrap().next_lsn();
            store.next_batch_id = store.wal.as_ref().unwrap().next_batch_id();
            store.fault_injector = fault_injector;
            return Ok(store);
        }

        recover_data_file(
            &mut file,
            wal.committed_batches(),
            checkpoint_hint,
            fault_injector.as_deref_mut(),
        )?;
        let length = file.len()?;
        if length < (FIRST_DATA_PAGE * PAGE_SIZE as u64) || !length.is_multiple_of(PAGE_SIZE as u64)
        {
            return Err(Error::recovery(
                "recovered database file is not page aligned",
            ));
        }
        let slot_a = read_exact_at(&mut file, SUPERBLOCK_A_OFFSET, PAGE_SIZE)?;
        let slot_b = read_exact_at(&mut file, SUPERBLOCK_B_OFFSET, PAGE_SIZE)?;
        let selected = choose_superblock(&slot_a, &slot_b)?;
        validate_superblock_identity(&selected.superblock, &identity)?;
        let (root_page_id, free_list_head, high_water_page_id) =
            metadata_from_superblock(&selected.superblock, length)?;
        let mut wal = wal;
        wal.resume_after(selected.superblock.checkpoint_lsn)?;
        let mut store = BTreeStore::<F, W> {
            file,
            wal: Some(wal),
            current_superblock: selected.superblock,
            active_slot: selected.slot,
            root_page_id,
            free_list_head,
            high_water_page_id,
            next_revision: Revision::new(1),
            next_lsn: Lsn::ZERO,
            next_batch_id: 1,
            cache: PageCache::new(config.cache_capacity),
            config,
            dirty_pages: BTreeMap::new(),
            dirty_superblock: None,
            track_published_pages: false,
            published_page_updates: BTreeMap::new(),
            storage_metrics: StorageMetrics::default(),
            broken: None,
            fault_injector,
        };
        let report = store.check_invariants()?;
        let max_commit_lsn = store
            .wal
            .as_ref()
            .and_then(|wal| wal.committed_batches().last().map(|batch| batch.commit_lsn))
            .unwrap_or(Lsn::ZERO);
        store.next_revision = Revision::new(
            report
                .max_revision
                .get()
                .max(max_commit_lsn.get())
                .checked_add(1)
                .ok_or_else(|| Error::invariant("storage revision exhausted"))?,
        );
        store.next_lsn = store.wal.as_ref().unwrap().next_lsn();
        store.next_batch_id = store.wal.as_ref().unwrap().next_batch_id();
        Ok(store)
    }

    pub fn current_superblock(&self) -> &Superblock {
        &self.current_superblock
    }

    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }

    pub(crate) fn enable_published_page_tracking(&mut self) {
        self.track_published_pages = true;
    }

    pub(crate) fn take_published_page_updates(&mut self) -> BTreeMap<PageId, PageData> {
        std::mem::take(&mut self.published_page_updates)
    }

    pub(crate) fn degraded_reason(&self) -> Option<&str> {
        self.broken.as_deref()
    }

    pub(crate) fn add_validation_time(&mut self, nanos: u64) {
        self.storage_metrics.validation_nanos =
            self.storage_metrics.validation_nanos.saturating_add(nanos);
    }

    pub fn storage_metrics(&self) -> StorageMetrics {
        self.storage_metrics.clone()
    }

    pub fn wal_metrics(&self) -> Result<Option<WalMetrics>> {
        self.wal.as_ref().map(WalLog::metrics).transpose()
    }

    pub fn set_fault_injector<I>(&mut self, injector: I)
    where
        I: FaultInjector + Send + 'static,
    {
        self.fault_injector = Some(Box::new(injector));
    }

    pub fn set_boxed_fault_injector(&mut self, injector: Box<dyn FaultInjector + Send>) {
        self.fault_injector = Some(injector);
    }

    pub fn get(&mut self, key: &DocumentKey) -> Result<RevisionState> {
        let request = BatchRequest::Get { key: key.clone() };
        match self.apply_batch(std::slice::from_ref(&request))?.remove(0) {
            BatchResponse::Get(state) => Ok(state),
            _ => Err(Error::invariant(
                "get produced an unexpected batch response",
            )),
        }
    }

    pub fn put(&mut self, key: DocumentKey, value: impl Into<Vec<u8>>) -> Result<Revision> {
        let request = BatchRequest::Put {
            key,
            value: value.into(),
        };
        match self.apply_batch(std::slice::from_ref(&request))?.remove(0) {
            BatchResponse::Put(revision) => Ok(revision),
            _ => Err(Error::invariant(
                "put produced an unexpected batch response",
            )),
        }
    }

    pub fn delete(&mut self, key: DocumentKey) -> Result<Revision> {
        let request = BatchRequest::Delete { key };
        match self.apply_batch(std::slice::from_ref(&request))?.remove(0) {
            BatchResponse::Delete(revision) => Ok(revision),
            _ => Err(Error::invariant(
                "delete produced an unexpected batch response",
            )),
        }
    }

    pub fn query(
        &mut self,
        pk: &PrimaryKey,
        exclusive_after_sk: Option<&SortKey>,
        limit: usize,
    ) -> Result<Vec<Document>> {
        let request = BatchRequest::Query {
            pk: pk.clone(),
            exclusive_after_sk: exclusive_after_sk.cloned(),
            limit,
        };
        match self.apply_batch(std::slice::from_ref(&request))?.remove(0) {
            BatchResponse::Query(rows) => Ok(rows),
            _ => Err(Error::invariant(
                "query produced an unexpected batch response",
            )),
        }
    }

    pub fn scan(
        &mut self,
        exclusive_after_key: Option<&DocumentKey>,
        limit: usize,
    ) -> Result<Vec<Document>> {
        let request = BatchRequest::Scan {
            exclusive_after_key: exclusive_after_key.cloned(),
            limit,
        };
        match self.apply_batch(std::slice::from_ref(&request))?.remove(0) {
            BatchResponse::Scan(rows) => Ok(rows),
            _ => Err(Error::invariant(
                "scan produced an unexpected batch response",
            )),
        }
    }

    pub fn apply_batch(&mut self, requests: &[BatchRequest]) -> Result<Vec<BatchResponse>> {
        let prepared = self.prepare_batch(requests)?;
        self.publish_prepared(prepared)
    }

    pub fn apply_batch_with_response_budget(
        &mut self,
        requests: &[BatchRequest],
        max_response_bytes: usize,
    ) -> Result<Vec<BatchResponse>> {
        let prepared = self.prepare_batch_with_response_budget(requests, max_response_bytes)?;
        self.publish_prepared(prepared)
    }

    /// Commits one optimistic point-key transaction as one logical commit.
    pub fn transact(&mut self, request: TransactionRequest) -> Result<TransactionResult> {
        let mut results = self.apply_transaction_group(std::slice::from_ref(&request))?;
        results
            .pop()
            .ok_or_else(|| Error::invariant("transaction group returned no result"))?
    }

    /// Commits candidates in coordinator order. Accepted candidates are
    /// staged serially, so later validation sees earlier accepted writes, but
    /// no staged state is published until the WAL group is durable.
    pub fn apply_transaction_group(
        &mut self,
        requests: &[TransactionRequest],
    ) -> Result<Vec<Result<TransactionResult>>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let mut overlay = Overlay::new(self);
        let mut prepared = Vec::new();
        let mut results = Vec::with_capacity(requests.len());
        for request in requests {
            let preparation_started = Instant::now();
            match overlay.prepare_transaction(request) {
                Ok(Some(candidate)) => {
                    let commit_lsn = candidate.commit_lsn.ok_or_else(|| {
                        Error::invariant("mutation transaction has no commit LSN")
                    })?;
                    prepared.push(candidate);
                    results.push(Ok(TransactionResult {
                        commit_lsn: Some(commit_lsn),
                    }));
                }
                Ok(None) => {
                    results.push(Ok(TransactionResult { commit_lsn: None }));
                }
                Err(error @ Error::Conflict(_))
                | Err(error @ Error::InvalidRequest(_))
                | Err(error @ Error::InvalidInput(_)) => {
                    results.push(Err(error));
                }
                Err(error) => return Err(error),
            }
            overlay.store.storage_metrics.btree_preparation_nanos = overlay
                .store
                .storage_metrics
                .btree_preparation_nanos
                .saturating_add(elapsed_nanos(preparation_started));
        }

        if prepared.is_empty() {
            return Ok(results);
        }
        self.publish_prepared_group(prepared)?;
        Ok(results)
    }

    /// Reads a set of point keys from one committed coordinator state. The
    /// output preserves the caller's input order.
    pub fn transact_get(&mut self, keys: &[DocumentKey]) -> Result<Vec<RevisionState>> {
        self.transact_get_with_response_budget(keys, usize::MAX)
    }

    pub fn transact_get_with_response_budget(
        &mut self,
        keys: &[DocumentKey],
        max_response_bytes: usize,
    ) -> Result<Vec<RevisionState>> {
        if let Some(message) = &self.broken {
            return Err(Error::durability(format!(
                "storage shard is not serving after an uncertain persistence failure: {message}"
            )));
        }
        let mut overlay = Overlay::new(self);
        let mut budget = ResponseBudget::new(max_response_bytes)?;
        keys.iter()
            .map(|key| overlay.get_state_with_budget(key, &mut budget))
            .collect()
    }

    /// Observes point-key presence and revisions without materializing values.
    pub fn observe(&mut self, keys: &[DocumentKey]) -> Result<Vec<ObservedState>> {
        if let Some(message) = &self.broken {
            return Err(Error::durability(format!(
                "storage shard is not serving after an uncertain persistence failure: {message}"
            )));
        }
        let mut overlay = Overlay::new(self);
        keys.iter()
            .map(|key| overlay.get_observed_state(key))
            .collect()
    }

    pub fn prepare_batch(&mut self, requests: &[BatchRequest]) -> Result<PreparedBatch> {
        self.prepare_batch_with_response_budget(requests, usize::MAX)
    }

    pub fn prepare_batch_with_response_budget(
        &mut self,
        requests: &[BatchRequest],
        max_response_bytes: usize,
    ) -> Result<PreparedBatch> {
        let mut overlay = Overlay::new(self);
        let mut budget = ResponseBudget::new(max_response_bytes)?;
        let mut responses = Vec::with_capacity(requests.len());
        for request in requests {
            responses.push(overlay.execute_with_budget(request, &mut budget)?);
        }
        overlay.finish(responses)
    }

    /// Publishes a prepared batch. WAL-backed stores append and sync the full
    /// after-image commit before changing the committed view. Data pages are
    /// retained as dirty committed images until [`Self::flush`] is called.
    pub fn publish_prepared(&mut self, prepared: PreparedBatch) -> Result<Vec<BatchResponse>> {
        let responses = prepared.responses.clone();
        self.publish_prepared_group(vec![prepared])?;
        Ok(responses)
    }

    fn publish_prepared_group(&mut self, prepared: Vec<PreparedBatch>) -> Result<()> {
        if let Some(message) = &self.broken {
            return Err(Error::durability(format!(
                "storage shard is not serving after an uncertain persistence failure: {message}"
            )));
        }

        let mut expected_generation = self.current_superblock.generation;
        for candidate in &prepared {
            if candidate.base_generation != expected_generation {
                return Err(Error::invariant(
                    "prepared transaction group has a non-contiguous superblock generation",
                ));
            }
            expected_generation = candidate.new_superblock.generation;
        }

        if prepared
            .iter()
            .all(|candidate| candidate.changed_pages.is_empty())
        {
            return Ok(());
        }

        let mut encoded_superblocks = Vec::with_capacity(prepared.len());
        let mut wal_commits = Vec::with_capacity(prepared.len());
        for candidate in &prepared {
            let superblock_bytes = encode_superblock(&candidate.new_superblock)?;
            if self.wal.is_some() {
                let commit_lsn = candidate
                    .commit_lsn
                    .ok_or_else(|| Error::invariant("WAL transaction has no commit LSN"))?;
                let mut images = candidate
                    .changed_pages
                    .iter()
                    .map(|(page_id, image)| WalPageImage {
                        page_id: *page_id,
                        image: *image,
                    })
                    .collect::<Vec<_>>();
                images.push(WalPageImage {
                    page_id: match candidate.new_slot {
                        SuperblockSlot::A => PageId::ZERO,
                        SuperblockSlot::B => PageId::new(1),
                    },
                    image: superblock_bytes,
                });
                wal_commits.push(WalCommit {
                    batch_id: candidate.batch_id,
                    commit_lsn,
                    pages: images,
                });
            }
            encoded_superblocks.push(superblock_bytes);
        }

        if let Some(wal) = self.wal.as_mut() {
            if let Err(error) = wal.append_group(&wal_commits, self.fault_injector.as_deref_mut()) {
                self.broken = Some(error.to_string());
                return Err(error);
            }
        } else {
            for (candidate, superblock_bytes) in prepared.iter().zip(&encoded_superblocks) {
                let target_length = candidate
                    .high_water_page_id
                    .get()
                    .checked_add(1)
                    .ok_or_else(|| Error::invalid_input("database page id is exhausted"))?
                    .checked_mul(PAGE_SIZE as u64)
                    .ok_or_else(|| Error::invalid_input("database file length overflows"))?;
                if self.file.len()? < target_length {
                    self.file.set_len(target_length)?;
                }
                for (page_id, bytes) in &candidate.changed_pages {
                    write_all_at(&mut self.file, page_id.get() * PAGE_SIZE as u64, bytes)?;
                }
                let superblock_offset = match candidate.new_slot {
                    SuperblockSlot::A => SUPERBLOCK_A_OFFSET,
                    SuperblockSlot::B => SUPERBLOCK_B_OFFSET,
                };
                write_all_at(&mut self.file, superblock_offset, superblock_bytes)?;
                self.file.sync_data()?;
            }
        }

        if let Some(injector) = self.fault_injector.as_deref_mut()
            && let Err(error) = injector.hit("before_publish")
        {
            self.broken = Some(error.to_string());
            return Err(error);
        }

        let publication_started = Instant::now();
        for (candidate, superblock_bytes) in prepared.into_iter().zip(encoded_superblocks) {
            self.current_superblock = crate::decode_superblock(&superblock_bytes)?;
            self.active_slot = candidate.new_slot;
            self.root_page_id = candidate.root_page_id;
            self.free_list_head = candidate.free_list_head;
            self.high_water_page_id = candidate.high_water_page_id;
            self.next_revision = candidate.next_revision;
            if let Some(commit_lsn) = candidate.commit_lsn {
                self.next_lsn = Lsn::new(
                    commit_lsn
                        .get()
                        .checked_add(1)
                        .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
                );
                self.next_batch_id = candidate
                    .batch_id
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("WAL batch ID exhausted"))?;
                self.dirty_pages.extend(candidate.changed_pages);
                self.dirty_superblock = Some(superblock_bytes);
            }
            self.cache.insert_many(candidate.read_pages);
            if self.track_published_pages {
                self.published_page_updates.extend(
                    candidate
                        .changed_decoded
                        .iter()
                        .map(|(page_id, page)| (*page_id, page.clone())),
                );
            }
            self.cache.insert_many(candidate.changed_decoded);
        }
        self.storage_metrics.publication_nanos = self
            .storage_metrics
            .publication_nanos
            .saturating_add(elapsed_nanos(publication_started));
        if let Some(injector) = self.fault_injector.as_deref_mut() {
            injector.hit("after_publish")?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        if self.wal.is_none() {
            return self.file.sync_data();
        }
        self.flush_dirty_pages()
    }

    pub fn checkpoint(&mut self) -> Result<CheckpointReport> {
        let started = Instant::now();
        if let Some(message) = &self.broken {
            return Err(Error::checkpoint(format!(
                "storage shard is degraded: {message}"
            )));
        }
        self.hit_fault("before_checkpoint_gate")?;
        let Some(wal) = self.wal.as_ref() else {
            return Err(Error::checkpoint(
                "formal checkpoint requires a WAL-backed store",
            ));
        };
        let checkpoint_lsn = wal
            .committed_batches()
            .last()
            .map(|batch| batch.commit_lsn)
            .unwrap_or(self.current_superblock.checkpoint_lsn);
        if checkpoint_lsn < self.current_superblock.checkpoint_lsn {
            return Err(Error::invariant("checkpoint LSN would move backwards"));
        }
        let metadata_changed = checkpoint_lsn > self.current_superblock.checkpoint_lsn;
        let dirty_page_count = self.dirty_pages.len();
        let dirty_superblock_bytes = if self.dirty_superblock.is_some() {
            PAGE_SIZE as u64
        } else {
            0
        };
        let bytes_written = u64::try_from(dirty_page_count)
            .ok()
            .and_then(|count| count.checked_mul(PAGE_SIZE as u64))
            .and_then(|bytes| {
                bytes.checked_add(dirty_superblock_bytes).and_then(|bytes| {
                    bytes.checked_add(if metadata_changed {
                        PAGE_SIZE as u64
                    } else {
                        0
                    })
                })
            })
            .ok_or_else(|| Error::checkpoint("checkpoint byte count overflows"))?;
        let wal_bytes_before = wal.metrics()?.wal_bytes;

        self.hit_fault("before_checkpoint_data_flush")?;
        self.flush_dirty_pages().map_err(|error| {
            Error::checkpoint(format!("checkpoint data-file flush failed: {error}"))
        })?;
        self.hit_fault("after_checkpoint_data_sync")?;

        if metadata_changed {
            let checkpoint_superblock = decode_superblock(&encode_superblock(&Superblock {
                generation: self
                    .current_superblock
                    .generation
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("superblock generation exhausted"))?,
                checkpoint_lsn,
                ..self.current_superblock.clone()
            })?)?;
            let checkpoint_bytes = encode_superblock(&checkpoint_superblock)?;
            let checkpoint_slot = match self.active_slot {
                SuperblockSlot::A => SuperblockSlot::B,
                SuperblockSlot::B => SuperblockSlot::A,
            };
            self.hit_fault("before_checkpoint_superblock_write")?;
            let offset = match checkpoint_slot {
                SuperblockSlot::A => SUPERBLOCK_A_OFFSET,
                SuperblockSlot::B => SUPERBLOCK_B_OFFSET,
            };
            if let Err(error) = write_all_at(&mut self.file, offset, &checkpoint_bytes) {
                self.broken = Some(error.to_string());
                return Err(Error::checkpoint(format!(
                    "checkpoint superblock write failed: {error}"
                )));
            }
            self.hit_fault("after_checkpoint_superblock_write")?;
            self.hit_fault("before_checkpoint_metadata_sync")?;
            self.hit_fault("during_checkpoint_metadata_sync")?;
            if let Err(error) = self.file.sync_data() {
                self.broken = Some(error.to_string());
                return Err(Error::checkpoint(format!(
                    "checkpoint metadata sync failed: {error}"
                )));
            }
            if let Err(error) = self.hit_fault("after_checkpoint_metadata_sync") {
                self.broken = Some(error.to_string());
                return Err(Error::checkpoint(format!(
                    "checkpoint metadata completion was uncertain: {error}"
                )));
            }
            self.current_superblock = checkpoint_superblock;
            self.active_slot = checkpoint_slot;
        }

        if let Err(error) = self.check_invariants() {
            self.broken = Some(error.to_string());
            return Err(Error::checkpoint(format!(
                "checkpoint invariant check failed: {error}"
            )));
        }

        let should_reset_wal = self.wal.as_ref().is_some_and(|wal| {
            !wal.committed_batches().is_empty() || wal.history_start_lsn() < checkpoint_lsn
        });
        if should_reset_wal {
            let mut wal = self
                .wal
                .take()
                .ok_or_else(|| Error::invariant("WAL disappeared during checkpoint"))?;
            let reset_result = wal.reset(checkpoint_lsn, self.fault_injector.as_deref_mut());
            if let Err(error) = reset_result {
                self.wal = Some(wal);
                self.broken = Some(error.to_string());
                return Err(Error::checkpoint(format!("WAL reset failed: {error}")));
            }
            self.next_lsn = wal.next_lsn();
            self.next_batch_id = wal.next_batch_id();
            self.wal = Some(wal);
        }
        if let Err(error) = self.hit_fault("before_checkpoint_complete") {
            self.broken = Some(error.to_string());
            return Err(Error::checkpoint(format!(
                "checkpoint completion was uncertain: {error}"
            )));
        }
        let wal_bytes_after = self
            .wal
            .as_ref()
            .map(WalLog::metrics)
            .transpose()?
            .map_or(0, |metrics| metrics.wal_bytes);
        let wal_bytes_reclaimed = wal_bytes_before.saturating_sub(wal_bytes_after);
        Ok(CheckpointReport {
            checkpoint_lsn,
            pages_flushed: dirty_page_count,
            bytes_written,
            wal_bytes_reclaimed,
            duration_nanos: elapsed_nanos(started),
        })
    }

    pub fn check_invariants(&mut self) -> Result<InvariantReport> {
        if self.wal.is_some() {
            self.flush_dirty_pages()?;
        }
        let length = self.file.len()?;
        if length < (FIRST_DATA_PAGE * PAGE_SIZE as u64) || !length.is_multiple_of(PAGE_SIZE as u64)
        {
            return Err(Error::corruption(
                "database file length is not page aligned",
            ));
        }
        let slot_a = read_exact_at(&mut self.file, SUPERBLOCK_A_OFFSET, PAGE_SIZE)?;
        let slot_b = read_exact_at(&mut self.file, SUPERBLOCK_B_OFFSET, PAGE_SIZE)?;
        let selected = choose_superblock(&slot_a, &slot_b)?;
        if selected.slot != self.active_slot || selected.superblock != self.current_superblock {
            return Err(Error::corruption(
                "in-memory superblock is not the selected file copy",
            ));
        }
        let (root, free_head, high_water) = metadata_from_superblock(&selected.superblock, length)?;
        if root != self.root_page_id
            || free_head != self.free_list_head
            || high_water != self.high_water_page_id
        {
            return Err(Error::corruption(
                "in-memory allocator metadata does not match superblock",
            ));
        }
        if let Some(wal) = self.wal.as_ref() {
            let latest_known_lsn = wal
                .committed_batches()
                .last()
                .map(|batch| batch.commit_lsn)
                .unwrap_or(self.current_superblock.checkpoint_lsn);
            if self.current_superblock.checkpoint_lsn > latest_known_lsn {
                return Err(Error::corruption(
                    "checkpoint LSN is newer than the known durable WAL state",
                ));
            }
        }
        checker::check(&mut self.file, root, free_head, high_water)
    }

    pub fn into_file(self) -> F {
        self.file
    }

    pub fn into_files(self) -> Result<(F, W)> {
        let wal = self
            .wal
            .ok_or_else(|| Error::invalid_input("store does not have a WAL file"))?;
        Ok((self.file, wal.into_file()))
    }

    fn flush_dirty_pages(&mut self) -> Result<()> {
        if self.wal.is_none() {
            return self.file.sync_data();
        }
        if self.dirty_pages.is_empty() && self.dirty_superblock.is_none() {
            return Ok(());
        }
        let flush_result = (|| {
            let target_length = self
                .high_water_page_id
                .get()
                .checked_add(1)
                .ok_or_else(|| Error::invalid_input("database page id is exhausted"))?
                .checked_mul(PAGE_SIZE as u64)
                .ok_or_else(|| Error::invalid_input("database file length overflows"))?;
            if self.file.len()? < target_length {
                self.file.set_len(target_length)?;
            }
            if let Some(injector) = self.fault_injector.as_deref_mut() {
                injector.hit("before_data_page_write")?;
            }
            for (page_id, bytes) in &self.dirty_pages {
                if let Some(injector) = self.fault_injector.as_deref_mut() {
                    injector.hit("during_data_page_write")?;
                }
                write_all_at(&mut self.file, page_id.get() * PAGE_SIZE as u64, bytes)?;
            }
            if let Some(injector) = self.fault_injector.as_deref_mut() {
                injector.hit("after_data_page_write")?;
            }
            if let Some(superblock_bytes) = self.dirty_superblock {
                let offset = match self.active_slot {
                    SuperblockSlot::A => SUPERBLOCK_A_OFFSET,
                    SuperblockSlot::B => SUPERBLOCK_B_OFFSET,
                };
                write_all_at(&mut self.file, offset, &superblock_bytes)?;
            }
            Ok::<(), Error>(())
        })();
        if let Err(error) = flush_result {
            self.broken = Some(error.to_string());
            return Err(Error::durability(format!(
                "database data-file write failed after WAL commit: {error}"
            )));
        }
        if let Some(injector) = self.fault_injector.as_deref_mut() {
            if let Err(error) = injector.hit("before_data_file_sync") {
                self.broken = Some(error.to_string());
                return Err(error);
            }
            if let Err(error) = injector.hit("during_data_file_sync") {
                self.broken = Some(error.to_string());
                return Err(error);
            }
        }
        if let Err(error) = self.file.sync_data() {
            self.broken = Some(error.to_string());
            return Err(Error::durability(format!(
                "database data-file sync failed after WAL commit: {error}"
            )));
        }
        self.dirty_pages.clear();
        self.dirty_superblock = None;
        if let Some(injector) = self.fault_injector.as_deref_mut() {
            injector.hit("after_data_file_sync")?;
        }
        Ok(())
    }

    fn initialize(mut file: F, config: DatabaseConfig) -> Result<Self> {
        let root_page_id = PageId::new(FIRST_DATA_PAGE);
        let superblock = Superblock {
            root_page_id: Some(root_page_id),
            high_water_page_id: Some(root_page_id),
            ..Superblock::new(
                1,
                config.database_uuid,
                config.tenant_id,
                config.shard_id,
                config.shard_epoch,
            )
        };
        let root = PageData::Leaf {
            lsn: Lsn::ZERO,
            next_leaf: None,
            entries: Vec::new(),
        };
        let root_bytes = root.encode(root_page_id)?;
        let superblock_bytes = encode_superblock(&superblock)?;
        file.set_len(3 * PAGE_SIZE as u64)?;
        write_all_at(&mut file, SUPERBLOCK_A_OFFSET, &superblock_bytes)?;
        write_all_at(&mut file, SUPERBLOCK_B_OFFSET, &superblock_bytes)?;
        write_all_at(
            &mut file,
            root_page_id.get() * PAGE_SIZE as u64,
            &root_bytes,
        )?;
        file.sync_all()?;
        Ok(Self {
            file,
            wal: None,
            current_superblock: superblock,
            active_slot: SuperblockSlot::A,
            root_page_id,
            free_list_head: None,
            high_water_page_id: root_page_id,
            next_revision: Revision::new(1),
            next_lsn: Lsn::new(1),
            next_batch_id: 1,
            cache: PageCache::new(config.cache_capacity),
            config,
            dirty_pages: BTreeMap::new(),
            dirty_superblock: None,
            track_published_pages: false,
            published_page_updates: BTreeMap::new(),
            storage_metrics: StorageMetrics::default(),
            broken: None,
            fault_injector: None,
        })
    }

    fn read_page_from_file(&mut self, page_id: PageId) -> Result<PageData> {
        if let Some(bytes) = self.dirty_pages.get(&page_id) {
            let decoded = decode_page_at(bytes, Some(page_id))?;
            return PageData::decode(decoded);
        }
        let bytes = read_exact_at(&mut self.file, page_id.get() * PAGE_SIZE as u64, PAGE_SIZE)?;
        let decoded = decode_page_at(&bytes, Some(page_id))?;
        PageData::decode(decoded)
    }

    fn hit_fault(&mut self, point: &str) -> Result<()> {
        if let Some(injector) = self.fault_injector.as_deref_mut() {
            injector.hit(point)?;
        }
        Ok(())
    }
}

impl<F: DurableFile> BTreeStore<F, NoWal> {
    /// Opens the legacy single-file form without a separate WAL. This remains
    /// useful for format-level tests; crash-safe deployments use
    /// [`BTreeStore::open_with_wal`] or [`BTreeStore::open_path`].
    pub fn open(file: F, config: DatabaseConfig) -> Result<Self> {
        Self::open_no_wal(file, config)
    }
}

struct Overlay<'a, F: DurableFile, W: DurableFile> {
    store: &'a mut BTreeStore<F, W>,
    pages: BTreeMap<PageId, PageData>,
    dirty: BTreeSet<PageId>,
    read_pages: BTreeMap<PageId, PageData>,
    allocated: HashSet<PageId>,
    root_page_id: PageId,
    free_list_head: Option<PageId>,
    high_water_page_id: PageId,
    next_revision: Revision,
    last_lsn: Option<Lsn>,
    next_lsn: Lsn,
    next_batch_id: u64,
    current_superblock: Superblock,
    active_slot: SuperblockSlot,
}

impl<'a, F: DurableFile, W: DurableFile> Overlay<'a, F, W> {
    fn new(store: &'a mut BTreeStore<F, W>) -> Self {
        let root_page_id = store.root_page_id;
        let free_list_head = store.free_list_head;
        let high_water_page_id = store.high_water_page_id;
        let next_revision = store.next_revision;
        let next_lsn = store.next_lsn;
        let next_batch_id = store.next_batch_id;
        let current_superblock = store.current_superblock.clone();
        let active_slot = store.active_slot;
        Self {
            root_page_id,
            free_list_head,
            high_water_page_id,
            next_revision,
            store,
            pages: BTreeMap::new(),
            dirty: BTreeSet::new(),
            read_pages: BTreeMap::new(),
            allocated: HashSet::new(),
            last_lsn: None,
            next_lsn,
            next_batch_id,
            current_superblock,
            active_slot,
        }
    }

    fn execute_with_budget(
        &mut self,
        request: &BatchRequest,
        budget: &mut ResponseBudget,
    ) -> Result<BatchResponse> {
        match request {
            BatchRequest::Get { key } => {
                Ok(BatchResponse::Get(self.get_state_with_budget(key, budget)?))
            }
            BatchRequest::Put { key, value } => Ok(BatchResponse::Put(self.put(key, value)?)),
            BatchRequest::Delete { key } => Ok(BatchResponse::Delete(self.delete(key)?)),
            BatchRequest::Query {
                pk,
                exclusive_after_sk,
                limit,
            } => Ok(BatchResponse::Query(self.query_with_budget(
                pk,
                exclusive_after_sk.as_ref(),
                *limit,
                budget,
            )?)),
            BatchRequest::Scan {
                exclusive_after_key,
                limit,
            } => Ok(BatchResponse::Scan(self.scan_with_budget(
                exclusive_after_key.as_ref(),
                *limit,
                budget,
            )?)),
        }
    }

    fn validate_transaction_mutation(&self, mutation: &TransactionMutation) -> Result<()> {
        match mutation {
            TransactionMutation::Put { key, value } => {
                self.validate_value(value)?;
                validate_encoded_key(&key.encode())?;
            }
            TransactionMutation::Delete { key } => {
                validate_encoded_key(&key.encode())?;
            }
        }
        Ok(())
    }

    fn prepare_transaction(
        &mut self,
        request: &TransactionRequest,
    ) -> Result<Option<PreparedBatch>> {
        let validation_started = Instant::now();
        request.validate()?;
        for mutation in &request.mutations {
            self.validate_transaction_mutation(mutation)?;
        }
        for condition in &request.conditions {
            let actual = self.get_observed_state(condition.key())?;
            let satisfied = match condition {
                TransactionCondition::RevisionEquals {
                    expected_revision, ..
                } => actual.revision() == *expected_revision,
                TransactionCondition::Exists { .. } => !actual.is_missing(),
                TransactionCondition::NotExists { .. } => actual.is_missing(),
            };
            if !satisfied {
                self.store
                    .add_validation_time(elapsed_nanos(validation_started));
                return Err(Error::conflict(TransactionConflict {
                    key: condition.key().clone(),
                    expected: condition.expectation(),
                    actual,
                }));
            }
        }
        self.store
            .add_validation_time(elapsed_nanos(validation_started));

        if request.mutations.is_empty() {
            return Ok(None);
        }

        for mutation in &request.mutations {
            match mutation {
                TransactionMutation::Put { key, value } => {
                    self.put(key, value)?;
                }
                TransactionMutation::Delete { key } => {
                    self.delete(key)?;
                }
            }
        }
        self.finish_transaction().map(Some)
    }

    fn finish_transaction(&mut self) -> Result<PreparedBatch> {
        let provisional_lsn = self
            .last_lsn
            .ok_or_else(|| Error::invariant("transaction has no provisional revision"))?;
        let base_generation = self.current_superblock.generation;
        let commit_lsn = if self.store.wal.is_some() {
            let image_count = self
                .dirty
                .len()
                .checked_add(1)
                .ok_or_else(|| Error::invariant("WAL image count overflow"))?;
            Lsn::new(
                self.next_lsn
                    .get()
                    .checked_add(image_count as u64)
                    .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
            )
        } else {
            provisional_lsn
        };

        for page_id in &self.dirty {
            self.pages
                .get_mut(page_id)
                .ok_or_else(|| Error::invariant("dirty page missing from overlay"))?
                .restamp(Revision::from(provisional_lsn), commit_lsn);
        }

        let new_superblock = {
            let candidate = Superblock {
                generation: base_generation
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("superblock generation exhausted"))?,
                root_page_id: Some(self.root_page_id),
                free_list_head: self.free_list_head,
                high_water_page_id: Some(self.high_water_page_id),
                checkpoint_lsn: self.current_superblock.checkpoint_lsn,
                ..self.current_superblock.clone()
            };
            decode_superblock(&encode_superblock(&candidate)?)?
        };
        let new_slot = match self.active_slot {
            SuperblockSlot::A => SuperblockSlot::B,
            SuperblockSlot::B => SuperblockSlot::A,
        };
        let mut changed_pages = BTreeMap::new();
        let mut changed_decoded = BTreeMap::new();
        for page_id in &self.dirty {
            let page = self
                .pages
                .get(page_id)
                .ok_or_else(|| Error::invariant("dirty page missing from overlay"))?;
            changed_pages.insert(*page_id, page.encode(*page_id)?);
            changed_decoded.insert(*page_id, page.clone());
        }
        let read_pages = std::mem::take(&mut self.read_pages);
        let next_revision = Revision::new(
            commit_lsn
                .get()
                .checked_add(1)
                .ok_or_else(|| Error::invariant("storage revision exhausted"))?,
        );
        let prepared = PreparedBatch {
            responses: Vec::new(),
            changed_pages,
            changed_decoded,
            read_pages,
            new_superblock: new_superblock.clone(),
            new_slot,
            base_generation,
            next_revision,
            root_page_id: self.root_page_id,
            free_list_head: self.free_list_head,
            high_water_page_id: self.high_water_page_id,
            commit_lsn: Some(commit_lsn),
            batch_id: self.next_batch_id,
        };

        self.current_superblock = new_superblock;
        self.active_slot = new_slot;
        self.next_revision = next_revision;
        self.next_lsn = Lsn::new(
            commit_lsn
                .get()
                .checked_add(1)
                .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
        );
        self.next_batch_id = self
            .next_batch_id
            .checked_add(1)
            .ok_or_else(|| Error::invariant("WAL batch ID exhausted"))?;
        self.dirty.clear();
        self.last_lsn = None;
        Ok(prepared)
    }

    fn finish(self, responses: Vec<BatchResponse>) -> Result<PreparedBatch> {
        let Overlay {
            store,
            mut pages,
            dirty,
            read_pages,
            root_page_id,
            free_list_head,
            high_water_page_id,
            next_revision: _,
            last_lsn,
            ..
        } = self;
        let provisional_lsn = last_lsn;
        let commit_lsn = if provisional_lsn.is_some() {
            let image_count = dirty
                .len()
                .checked_add(1)
                .ok_or_else(|| Error::invariant("WAL image count overflow"))?;
            if store.wal.is_some() {
                Some(Lsn::new(
                    store
                        .next_lsn
                        .get()
                        .checked_add(image_count as u64)
                        .ok_or_else(|| Error::invariant("WAL LSN exhausted"))?,
                ))
            } else {
                provisional_lsn
            }
        } else {
            None
        };
        if let (Some(provisional), Some(committed)) = (provisional_lsn, commit_lsn) {
            for page_id in &dirty {
                pages
                    .get_mut(page_id)
                    .ok_or_else(|| Error::invariant("dirty page missing from overlay"))?
                    .restamp(Revision::from(provisional), committed);
            }
        }
        let new_superblock = if commit_lsn.is_some() {
            let candidate = Superblock {
                generation: store
                    .current_superblock
                    .generation
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("superblock generation exhausted"))?,
                root_page_id: Some(root_page_id),
                free_list_head,
                high_water_page_id: Some(high_water_page_id),
                checkpoint_lsn: store.current_superblock.checkpoint_lsn,
                ..store.current_superblock.clone()
            };
            decode_superblock(&encode_superblock(&candidate)?)?
        } else {
            store.current_superblock.clone()
        };
        let new_slot = match store.active_slot {
            SuperblockSlot::A => SuperblockSlot::B,
            SuperblockSlot::B => SuperblockSlot::A,
        };
        let mut changed_pages = BTreeMap::new();
        let mut changed_decoded = BTreeMap::new();
        for page_id in dirty {
            let page = pages
                .get(&page_id)
                .ok_or_else(|| Error::invariant("dirty page missing from overlay"))?;
            changed_pages.insert(page_id, page.encode(page_id)?);
            changed_decoded.insert(page_id, page.clone());
        }
        let responses = if let (Some(provisional), Some(committed)) = (provisional_lsn, commit_lsn)
        {
            restamp_responses(
                responses,
                Revision::from(provisional),
                Revision::from(committed),
            )
        } else {
            responses
        };
        let next_revision = match commit_lsn {
            Some(lsn) => Revision::new(
                lsn.get()
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("storage revision exhausted"))?,
            ),
            None => store.next_revision,
        };
        Ok(PreparedBatch {
            responses,
            changed_pages,
            changed_decoded,
            read_pages,
            new_superblock,
            new_slot,
            base_generation: store.current_superblock.generation,
            next_revision,
            root_page_id,
            free_list_head,
            high_water_page_id,
            commit_lsn,
            batch_id: store.next_batch_id,
        })
    }

    fn get_state_with_budget(
        &mut self,
        key: &DocumentKey,
        budget: &mut ResponseBudget,
    ) -> Result<RevisionState> {
        let encoded = key.encode();
        validate_encoded_key(&encoded)?;
        let leaf_id = self.find_leaf(&encoded)?.0;
        let leaf = self.leaf(leaf_id)?;
        let Some(entry) = leaf.entries.iter().find(|entry| entry.key == encoded) else {
            budget.reserve_state(0)?;
            return Ok(RevisionState::missing(Revision::ZERO));
        };
        match &entry.value {
            Some(value_ref) => {
                budget.reserve_state(value_length(value_ref)?)?;
                Ok(RevisionState::present(
                    self.read_value(value_ref)?,
                    entry.revision,
                ))
            }
            None => {
                budget.reserve_state(0)?;
                Ok(RevisionState::missing(entry.revision))
            }
        }
    }

    fn get_observed_state(&mut self, key: &DocumentKey) -> Result<ObservedState> {
        let encoded = key.encode();
        validate_encoded_key(&encoded)?;
        let leaf_id = self.find_leaf(&encoded)?.0;
        let leaf = self.leaf(leaf_id)?;
        let Some(entry) = leaf.entries.iter().find(|entry| entry.key == encoded) else {
            return Ok(ObservedState::missing(Revision::ZERO));
        };
        if entry.value.is_some() {
            Ok(ObservedState::present(entry.revision))
        } else {
            Ok(ObservedState::missing(entry.revision))
        }
    }

    fn put(&mut self, key: &DocumentKey, value: &[u8]) -> Result<Revision> {
        self.validate_value(value)?;
        let encoded = key.encode();
        validate_encoded_key(&encoded)?;
        let revision = self.allocate_revision()?;
        let (leaf_id, route) = self.find_leaf(&encoded)?;
        let mut leaf = self.leaf(leaf_id)?;
        let index = leaf
            .entries
            .binary_search_by(|entry| entry.key.cmp(&encoded));
        let value_ref = self.make_value(value, revision)?;
        match index {
            Ok(index) => {
                let old_value = leaf.entries[index].value.clone();
                let LeafPage {
                    next_leaf,
                    mut entries,
                } = leaf;
                entries[index] = format::LeafEntry {
                    key: encoded,
                    revision,
                    value: Some(value_ref),
                };
                self.insert_leaf_entries(
                    leaf_id,
                    route,
                    LeafPage { next_leaf, entries },
                    revision,
                )?;
                self.free_value(old_value, revision)?;
            }
            Err(index) => {
                leaf.entries.insert(
                    index,
                    format::LeafEntry {
                        key: encoded,
                        revision,
                        value: Some(value_ref),
                    },
                );
                self.insert_leaf_entries(leaf_id, route, leaf, revision)?;
            }
        }
        Ok(revision)
    }

    fn delete(&mut self, key: &DocumentKey) -> Result<Revision> {
        let encoded = key.encode();
        validate_encoded_key(&encoded)?;
        let revision = self.allocate_revision()?;
        let (leaf_id, route) = self.find_leaf(&encoded)?;
        let mut leaf = self.leaf(leaf_id)?;
        let index = leaf
            .entries
            .binary_search_by(|entry| entry.key.cmp(&encoded));
        match index {
            Ok(index) => {
                let old_value = leaf.entries[index].value.clone();
                let LeafPage {
                    next_leaf,
                    mut entries,
                } = leaf;
                entries[index] = format::LeafEntry {
                    key: encoded,
                    revision,
                    value: None,
                };
                self.replace_page(
                    leaf_id,
                    PageData::Leaf {
                        lsn: Lsn::new(revision.get()),
                        next_leaf,
                        entries,
                    },
                );
                self.free_value(old_value, revision)?;
            }
            Err(index) => {
                leaf.entries.insert(
                    index,
                    format::LeafEntry {
                        key: encoded,
                        revision,
                        value: None,
                    },
                );
                self.insert_leaf_entries(leaf_id, route, leaf, revision)?;
            }
        }
        Ok(revision)
    }

    fn query_with_budget(
        &mut self,
        pk: &PrimaryKey,
        exclusive_after_sk: Option<&SortKey>,
        limit: usize,
        budget: &mut ResponseBudget,
    ) -> Result<Vec<Document>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let start_key = DocumentKey::new(pk.as_bytes().to_vec(), Vec::new()).encode();
        validate_encoded_key(&start_key)?;
        let (mut leaf_id, _) = self.find_leaf(&start_key)?;
        let cursor = exclusive_after_sk
            .map(|sk| DocumentKey::new(pk.as_bytes().to_vec(), sk.as_bytes().to_vec()).encode());
        if let Some(cursor) = &cursor {
            validate_encoded_key(cursor)?;
        }
        let mut first = true;
        let mut visited = HashSet::new();
        let mut rows = Vec::new();
        while rows.len() < limit {
            if !visited.insert(leaf_id) {
                return Err(Error::corruption(
                    "leaf chain contains a cycle during query",
                ));
            }
            let leaf = self.leaf(leaf_id)?;
            for entry in &leaf.entries {
                if first && cursor.as_ref().is_some_and(|cursor| entry.key <= *cursor) {
                    continue;
                }
                first = false;
                let document_key = DocumentKey::decode(&entry.key).map_err(|error| {
                    Error::corruption(format!("leaf key decode failed: {error}"))
                })?;
                if document_key.pk != *pk {
                    if document_key.pk > *pk {
                        return Ok(rows);
                    }
                    continue;
                }
                if let Some(cursor) = exclusive_after_sk
                    && document_key.sk <= *cursor
                {
                    continue;
                }
                if let Some(value) = &entry.value {
                    budget.reserve_document(&document_key, value_length(value)?)?;
                    rows.push(Document {
                        key: document_key,
                        value: self.read_value(value)?,
                        revision: entry.revision,
                    });
                    if rows.len() == limit {
                        return Ok(rows);
                    }
                }
            }
            let next = leaf.next_leaf;
            let Some(next) = next else {
                break;
            };
            leaf_id = next;
            first = false;
        }
        Ok(rows)
    }

    fn scan_with_budget(
        &mut self,
        cursor: Option<&DocumentKey>,
        limit: usize,
        budget: &mut ResponseBudget,
    ) -> Result<Vec<Document>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let (mut leaf_id, _) = match cursor {
            Some(key) => {
                let encoded = key.encode();
                validate_encoded_key(&encoded)?;
                self.find_leaf(&encoded)?
            }
            None => (self.leftmost_leaf()?, Vec::new()),
        };
        let cursor = cursor.map(DocumentKey::encode);
        let mut rows = Vec::new();
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(leaf_id) {
                return Err(Error::corruption("leaf chain contains a cycle during scan"));
            }
            let leaf = self.leaf(leaf_id)?;
            for entry in &leaf.entries {
                if cursor.as_ref().is_some_and(|cursor| entry.key <= *cursor) {
                    continue;
                }
                let document_key = DocumentKey::decode(&entry.key).map_err(|error| {
                    Error::corruption(format!("leaf key decode failed: {error}"))
                })?;
                if let Some(value) = &entry.value {
                    budget.reserve_document(&document_key, value_length(value)?)?;
                    rows.push(Document {
                        key: document_key,
                        value: self.read_value(value)?,
                        revision: entry.revision,
                    });
                    if rows.len() == limit {
                        return Ok(rows);
                    }
                }
            }
            let Some(next) = leaf.next_leaf else {
                break;
            };
            leaf_id = next;
        }
        Ok(rows)
    }

    fn insert_leaf_entries(
        &mut self,
        leaf_id: PageId,
        route: Vec<PageId>,
        leaf: LeafPage,
        lsn: Revision,
    ) -> Result<()> {
        let LeafPage { next_leaf, entries } = leaf;
        if leaf_fits(&entries) {
            self.replace_page(
                leaf_id,
                PageData::Leaf {
                    lsn: Lsn::new(lsn.get()),
                    next_leaf,
                    entries,
                },
            );
            return Ok(());
        }
        let split = choose_leaf_split(&entries)?;
        let right_id = self.allocate_page()?;
        let left_entries = entries[..split].to_vec();
        let right_entries = entries[split..].to_vec();
        let separator = right_entries
            .first()
            .map(|entry| entry.key.clone())
            .ok_or_else(|| Error::invariant("leaf split produced an empty right page"))?;
        self.replace_page(
            leaf_id,
            PageData::Leaf {
                lsn: Lsn::new(lsn.get()),
                next_leaf: Some(right_id),
                entries: left_entries,
            },
        );
        self.replace_page(
            right_id,
            PageData::Leaf {
                lsn: Lsn::new(lsn.get()),
                next_leaf,
                entries: right_entries,
            },
        );
        self.insert_parent_separator(route, separator, right_id, lsn)
    }

    fn insert_parent_separator(
        &mut self,
        mut route: Vec<PageId>,
        mut separator: Vec<u8>,
        mut right_child: PageId,
        lsn: Revision,
    ) -> Result<()> {
        route.pop();
        while let Some(parent_id) = route.pop() {
            let parent = self.internal(parent_id)?;
            let mut entries = parent.entries;
            let index = entries.partition_point(|entry| entry.key < separator);
            entries.insert(
                index,
                format::InternalEntry {
                    key: separator,
                    right_child,
                },
            );
            if internal_fits(parent.leftmost_child, &entries) {
                self.replace_page(
                    parent_id,
                    PageData::Internal {
                        lsn: Lsn::new(lsn.get()),
                        leftmost_child: parent.leftmost_child,
                        entries,
                    },
                );
                return Ok(());
            }
            let split = choose_internal_split(parent.leftmost_child, &entries)?;
            let promoted = entries[split].key.clone();
            let right_leftmost = entries[split].right_child;
            let right_id = self.allocate_page()?;
            let left_entries = entries[..split].to_vec();
            let right_entries = entries[split + 1..].to_vec();
            if !internal_fits(parent.leftmost_child, &left_entries)
                || !internal_fits(right_leftmost, &right_entries)
            {
                return Err(Error::invalid_input(
                    "internal split produced an oversized page",
                ));
            }
            self.replace_page(
                parent_id,
                PageData::Internal {
                    lsn: Lsn::new(lsn.get()),
                    leftmost_child: parent.leftmost_child,
                    entries: left_entries,
                },
            );
            self.replace_page(
                right_id,
                PageData::Internal {
                    lsn: Lsn::new(lsn.get()),
                    leftmost_child: right_leftmost,
                    entries: right_entries,
                },
            );
            separator = promoted;
            right_child = right_id;
        }

        let new_root_id = self.allocate_page()?;
        self.replace_page(
            new_root_id,
            PageData::Internal {
                lsn: Lsn::new(lsn.get()),
                leftmost_child: self.root_page_id,
                entries: vec![format::InternalEntry {
                    key: separator,
                    right_child,
                }],
            },
        );
        self.root_page_id = new_root_id;
        Ok(())
    }

    fn find_leaf(&mut self, key: &[u8]) -> Result<(PageId, Vec<PageId>)> {
        let mut page_id = self.root_page_id;
        let mut route = Vec::new();
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(page_id) {
                return Err(Error::corruption("tree traversal contains a cycle"));
            }
            route.push(page_id);
            match self.page(page_id)? {
                PageData::Leaf { .. } => return Ok((page_id, route)),
                PageData::Internal {
                    leftmost_child,
                    entries,
                    ..
                } => {
                    let index = entries.partition_point(|entry| entry.key.as_slice() <= key);
                    page_id = if index == 0 {
                        leftmost_child
                    } else {
                        entries[index - 1].right_child
                    };
                }
                _ => return Err(Error::corruption("tree child has an impossible page type")),
            }
        }
    }

    fn leftmost_leaf(&mut self) -> Result<PageId> {
        let mut page_id = self.root_page_id;
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(page_id) {
                return Err(Error::corruption("leftmost traversal contains a cycle"));
            }
            match self.page(page_id)? {
                PageData::Leaf { .. } => return Ok(page_id),
                PageData::Internal { leftmost_child, .. } => page_id = leftmost_child,
                _ => return Err(Error::corruption("tree child has an impossible page type")),
            }
        }
    }

    fn leaf(&mut self, page_id: PageId) -> Result<LeafPage> {
        match self.page(page_id)? {
            PageData::Leaf {
                next_leaf, entries, ..
            } => Ok(LeafPage { next_leaf, entries }),
            _ => Err(Error::corruption("expected a leaf page")),
        }
    }

    fn internal(&mut self, page_id: PageId) -> Result<InternalPage> {
        match self.page(page_id)? {
            PageData::Internal {
                leftmost_child,
                entries,
                ..
            } => Ok(InternalPage {
                leftmost_child,
                entries,
            }),
            _ => Err(Error::corruption("expected an internal page")),
        }
    }

    fn page(&mut self, page_id: PageId) -> Result<PageData> {
        self.check_page_id(page_id)?;
        if let Some(page) = self.pages.get(&page_id) {
            return Ok(page.clone());
        }
        if let Some(page) = self.store.cache.get(page_id) {
            self.pages.insert(page_id, page.clone());
            return Ok(page);
        }
        let page = self.store.read_page_from_file(page_id)?;
        self.read_pages.insert(page_id, page.clone());
        self.pages.insert(page_id, page.clone());
        Ok(page)
    }

    fn replace_page(&mut self, page_id: PageId, page: PageData) {
        self.pages.insert(page_id, page);
        self.dirty.insert(page_id);
    }

    fn check_page_id(&self, page_id: PageId) -> Result<()> {
        if page_id.get() < FIRST_DATA_PAGE || page_id > self.high_water_page_id {
            return Err(Error::corruption(format!(
                "page id {} is outside the allocated range",
                page_id.get()
            )));
        }
        Ok(())
    }

    fn allocate_page(&mut self) -> Result<PageId> {
        if let Some(page_id) = self.free_list_head {
            let page = self.page(page_id)?;
            let PageData::Free { next, .. } = page else {
                return Err(Error::corruption("free-list head is not a free page"));
            };
            self.free_list_head = next;
            self.allocated.insert(page_id);
            return Ok(page_id);
        }
        let page_id = match self.high_water_page_id.get().checked_add(1) {
            Some(value) => PageId::new(value),
            None => return Err(Error::invariant("page id exhausted")),
        };
        self.high_water_page_id = page_id;
        self.allocated.insert(page_id);
        Ok(page_id)
    }

    fn allocate_revision(&mut self) -> Result<Revision> {
        let revision = self
            .last_lsn
            .map(Revision::from)
            .unwrap_or(self.next_revision);
        self.last_lsn = Some(Lsn::new(revision.get()));
        Ok(revision)
    }

    fn validate_value(&self, value: &[u8]) -> Result<()> {
        if value.len() > self.store.config.limits.max_value_size || value.len() > MAX_VALUE_SIZE {
            return Err(Error::invalid_input(format!(
                "value is {} bytes, maximum is {}",
                value.len(),
                self.store.config.limits.max_value_size.min(MAX_VALUE_SIZE)
            )));
        }
        Ok(())
    }

    fn make_value(&mut self, value: &[u8], lsn: Revision) -> Result<ValueRef> {
        if value.len() <= self.store.config.limits.inline_value_limit
            && value.len() <= INLINE_VALUE_LIMIT
        {
            return Ok(ValueRef::Inline(value.to_vec()));
        }
        let ids = value.len().div_ceil(format::OVERFLOW_DATA_SIZE);
        if ids > MAX_OVERFLOW_PAGES {
            return Err(Error::invalid_input(
                "value requires too many overflow pages",
            ));
        }
        let mut page_ids = Vec::with_capacity(ids);
        for _ in 0..ids {
            page_ids.push(self.allocate_page()?);
        }
        for (index, chunk) in value.chunks(format::OVERFLOW_DATA_SIZE).enumerate() {
            self.replace_page(
                page_ids[index],
                PageData::Overflow {
                    lsn: Lsn::new(lsn.get()),
                    next: page_ids.get(index + 1).copied(),
                    total_length: value.len() as u64,
                    chunk: chunk.to_vec(),
                },
            );
        }
        Ok(ValueRef::Overflow {
            head: page_ids[0],
            length: value.len() as u64,
        })
    }

    fn free_value(&mut self, value: Option<ValueRef>, lsn: Revision) -> Result<()> {
        let Some(ValueRef::Overflow { head, length }) = value else {
            return Ok(());
        };
        let mut page_id = Some(head);
        let mut visited = HashSet::new();
        let mut total_length = 0u64;
        let mut count = 0usize;
        while let Some(current) = page_id {
            if !visited.insert(current) || count >= MAX_OVERFLOW_PAGES {
                return Err(Error::corruption("overflow chain is cyclic or too long"));
            }
            let page = self.page(current)?;
            let PageData::Overflow {
                next,
                total_length: page_total,
                chunk,
                ..
            } = page
            else {
                return Err(Error::corruption(
                    "value reference points to a non-overflow page",
                ));
            };
            if page_total != length {
                return Err(Error::corruption(
                    "overflow length metadata disagrees with leaf",
                ));
            }
            total_length = total_length
                .checked_add(chunk.len() as u64)
                .ok_or_else(|| Error::corruption("overflow length overflows"))?;
            page_id = next;
            count += 1;
            self.free_page(current, lsn)?;
        }
        if total_length != length || length == 0 {
            return Err(Error::corruption(
                "overflow chain content length is invalid",
            ));
        }
        Ok(())
    }

    fn read_value(&mut self, value: &ValueRef) -> Result<Vec<u8>> {
        match value {
            ValueRef::Inline(value) => Ok(value.clone()),
            ValueRef::Overflow { head, length } => {
                let mut output = Vec::new();
                let mut page_id = Some(*head);
                let mut visited = HashSet::new();
                let mut count = 0usize;
                while let Some(current) = page_id {
                    if !visited.insert(current) || count >= MAX_OVERFLOW_PAGES {
                        return Err(Error::corruption("overflow chain is cyclic or too long"));
                    }
                    let page = self.page(current)?;
                    let PageData::Overflow {
                        next,
                        total_length,
                        chunk,
                        ..
                    } = page
                    else {
                        return Err(Error::corruption(
                            "value reference points to a non-overflow page",
                        ));
                    };
                    if total_length != *length {
                        return Err(Error::corruption(
                            "overflow length metadata disagrees with leaf",
                        ));
                    }
                    output.extend_from_slice(&chunk);
                    page_id = next;
                    count += 1;
                }
                if output.len() as u64 != *length {
                    return Err(Error::corruption("overflow content length is invalid"));
                }
                Ok(output)
            }
        }
    }

    fn free_page(&mut self, page_id: PageId, lsn: Revision) -> Result<()> {
        self.allocated.remove(&page_id);
        self.replace_page(
            page_id,
            PageData::Free {
                lsn: Lsn::new(lsn.get()),
                next: self.free_list_head,
            },
        );
        self.free_list_head = Some(page_id);
        Ok(())
    }
}

struct InternalPage {
    leftmost_child: PageId,
    entries: Vec<format::InternalEntry>,
}

struct LeafPage {
    next_leaf: Option<PageId>,
    entries: Vec<format::LeafEntry>,
}

fn choose_leaf_split(entries: &[format::LeafEntry]) -> Result<usize> {
    for candidate in split_candidates(entries.len()) {
        if leaf_fits(&entries[..candidate]) && leaf_fits(&entries[candidate..]) {
            return Ok(candidate);
        }
    }
    Err(Error::invalid_input(
        "a single leaf entry exceeds page capacity",
    ))
}

fn choose_internal_split(leftmost: PageId, entries: &[format::InternalEntry]) -> Result<usize> {
    if entries.len() < 3 {
        return Err(Error::invalid_input("internal page cannot be split safely"));
    }
    for candidate in split_candidates(entries.len()) {
        let right_leftmost = entries[candidate].right_child;
        if internal_fits(leftmost, &entries[..candidate])
            && internal_fits(right_leftmost, &entries[candidate + 1..])
        {
            return Ok(candidate);
        }
    }
    Err(Error::invalid_input(
        "internal separators exceed page capacity",
    ))
}

fn split_candidates(length: usize) -> impl Iterator<Item = usize> {
    let middle = length / 2;
    let mut candidates: Vec<_> = (1..length).collect();
    candidates.sort_by_key(move |candidate| candidate.abs_diff(middle));
    candidates.into_iter()
}

fn validate_superblock_identity(superblock: &Superblock, identity: &WalIdentity) -> Result<()> {
    if superblock.database_uuid != identity.database_uuid
        || superblock.tenant_id != identity.tenant_id
        || superblock.shard_id != identity.shard_id
        || superblock.shard_epoch != identity.shard_epoch
    {
        return Err(Error::corruption(
            "database superblock identity does not match the requested shard",
        ));
    }
    Ok(())
}

fn recover_data_file<F: DurableFile>(
    file: &mut F,
    committed_batches: &[CommittedWalBatch],
    checkpoint_lsn: Lsn,
    mut injector: Option<&mut (dyn FaultInjector + Send + '_)>,
) -> Result<()> {
    let committed_batches = committed_batches
        .iter()
        .filter(|batch| batch.commit_lsn > checkpoint_lsn)
        .collect::<Vec<_>>();
    if committed_batches.is_empty() {
        return Ok(());
    }
    let mut highest_data_page = FIRST_DATA_PAGE;
    for batch in &committed_batches {
        for page in &batch.pages {
            if page.page_id.get() >= FIRST_DATA_PAGE {
                highest_data_page = highest_data_page.max(page.page_id.get());
            }
        }
    }
    let target_length = highest_data_page
        .checked_add(1)
        .ok_or_else(|| Error::recovery("recovery page range overflows"))?
        .checked_mul(PAGE_SIZE as u64)
        .ok_or_else(|| Error::recovery("recovery file length overflows"))?;
    let current_length = file.len()?;
    if current_length == 0 || current_length < target_length {
        file.set_len(target_length)?;
    } else if !current_length.is_multiple_of(PAGE_SIZE as u64) {
        return Err(Error::recovery(
            "database file has a non-page-aligned length before recovery",
        ));
    }

    let mut changed = current_length < target_length;
    let readable_length = file.len()?;
    for batch in &committed_batches {
        for page in &batch.pages {
            let offset = page
                .page_id
                .get()
                .checked_mul(PAGE_SIZE as u64)
                .ok_or_else(|| Error::recovery("recovery page offset overflows"))?;
            let should_write = if page.page_id == PageId::ZERO || page.page_id == PageId::new(1) {
                true
            } else {
                let current = if offset
                    .checked_add(PAGE_SIZE as u64)
                    .is_some_and(|end| end <= readable_length)
                {
                    let bytes = read_exact_at(file, offset, PAGE_SIZE)?;
                    decode_page_at(&bytes, Some(page.page_id)).ok()
                } else {
                    None
                };
                match current {
                    Some(current) => current.header.page_lsn < batch.commit_lsn,
                    None => true,
                }
            };
            if should_write {
                hit_fault(&mut injector, "during_recovery_page_write")?;
                write_all_at(file, offset, &page.image)?;
                changed = true;
            }
        }
    }
    if changed {
        hit_fault(&mut injector, "before_recovery_sync")?;
        hit_fault(&mut injector, "during_recovery_sync")?;
        file.sync_data().map_err(|error| {
            Error::durability(format!("database sync during WAL recovery failed: {error}"))
        })?;
        hit_fault(&mut injector, "after_recovery_sync")?;
    }
    Ok(())
}

fn validate_existing_identity_before_wal<F: DurableFile>(
    file: &mut F,
    identity: &WalIdentity,
    wal_length: u64,
) -> Result<Option<Lsn>> {
    let length = file.len()?;
    if length == 0 {
        return Ok(None);
    }
    if length < (FIRST_DATA_PAGE * PAGE_SIZE as u64) || !length.is_multiple_of(PAGE_SIZE as u64) {
        if wal_length == 0 {
            return Err(Error::corruption(
                "database file is not a complete existing database",
            ));
        }
        return Ok(None);
    }
    let slot_a = read_exact_at(file, SUPERBLOCK_A_OFFSET, PAGE_SIZE)?;
    let slot_b = read_exact_at(file, SUPERBLOCK_B_OFFSET, PAGE_SIZE)?;
    let selected = match choose_superblock(&slot_a, &slot_b) {
        Ok(selected) => selected,
        Err(_error) if wal_length > 0 => return Ok(None),
        Err(error) => return Err(error),
    };
    validate_superblock_identity(&selected.superblock, identity)?;
    Ok(Some(selected.superblock.checkpoint_lsn))
}

fn hit_fault(
    injector: &mut Option<&mut (dyn FaultInjector + Send + '_)>,
    point: &str,
) -> Result<()> {
    if let Some(injector) = injector.as_deref_mut() {
        injector.hit(point)?;
    }
    Ok(())
}

fn metadata_from_superblock(
    superblock: &Superblock,
    file_length: u64,
) -> Result<(PageId, Option<PageId>, PageId)> {
    if superblock.page_size as usize != PAGE_SIZE {
        return Err(Error::unsupported_format(
            "superblock page size is unsupported",
        ));
    }
    let page_count = file_length / PAGE_SIZE as u64;
    let root = superblock
        .root_page_id
        .ok_or_else(|| Error::corruption("selected superblock has no root page"))?;
    let high_water = superblock
        .high_water_page_id
        .ok_or_else(|| Error::corruption("selected superblock has no high-water page"))?;
    let expected_page_count = high_water
        .get()
        .checked_add(1)
        .ok_or_else(|| Error::corruption("superblock high-water page overflows"))?;
    if high_water.get() < FIRST_DATA_PAGE || expected_page_count != page_count {
        return Err(Error::corruption(
            "superblock high-water page is inconsistent with file length",
        ));
    }
    if root.get() < FIRST_DATA_PAGE || root > high_water {
        return Err(Error::corruption(
            "superblock root page is outside the file",
        ));
    }
    if let Some(free) = superblock.free_list_head
        && (free.get() < FIRST_DATA_PAGE || free > high_water)
    {
        return Err(Error::corruption(
            "superblock free-list head is outside the file",
        ));
    }
    Ok((root, superblock.free_list_head, high_water))
}

fn read_exact_at<F: DurableFile>(file: &mut F, offset: u64, length: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0; length];
    let mut position = 0;
    while position < length {
        let count = file.read_at(offset + position as u64, &mut bytes[position..])?;
        if count == 0 {
            return Err(Error::corruption("unexpected end of database file"));
        }
        position += count;
    }
    Ok(bytes)
}

fn write_all_at<F: DurableFile>(file: &mut F, offset: u64, bytes: &[u8]) -> Result<()> {
    let mut position = 0;
    while position < bytes.len() {
        let count = file.write_at(offset + position as u64, &bytes[position..])?;
        if count == 0 {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "database file returned a zero-byte write",
            )));
        }
        position += count;
    }
    Ok(())
}

fn elapsed_nanos(started: Instant) -> u64 {
    started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX)
}

fn validate_encoded_key(key: &[u8]) -> Result<()> {
    if key.len() > MAX_ENCODED_KEY_SIZE {
        return Err(Error::invalid_input(format!(
            "encoded document key is {} bytes, maximum is {MAX_ENCODED_KEY_SIZE}",
            key.len()
        )));
    }
    Ok(())
}

fn value_length(value: &ValueRef) -> Result<usize> {
    match value {
        ValueRef::Inline(bytes) => Ok(bytes.len()),
        ValueRef::Overflow { length, .. } => usize::try_from(*length)
            .map_err(|_| Error::corruption("overflow value length does not fit usize")),
    }
}

fn restamp_responses(
    mut responses: Vec<BatchResponse>,
    provisional: Revision,
    committed: Revision,
) -> Vec<BatchResponse> {
    for response in &mut responses {
        match response {
            BatchResponse::Put(revision) | BatchResponse::Delete(revision) => {
                if *revision == provisional {
                    *revision = committed;
                }
            }
            BatchResponse::Get(state) => restamp_state(state, provisional, committed),
            BatchResponse::Query(rows) | BatchResponse::Scan(rows) => {
                for row in rows {
                    if row.revision == provisional {
                        row.revision = committed;
                    }
                }
            }
        }
    }
    responses
}

fn restamp_state(state: &mut RevisionState, provisional: Revision, committed: Revision) {
    match state {
        RevisionState::Present { revision, .. } | RevisionState::Missing { revision }
            if *revision == provisional =>
        {
            *revision = committed
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;
