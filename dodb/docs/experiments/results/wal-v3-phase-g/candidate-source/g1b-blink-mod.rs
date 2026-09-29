//! Experimental serial B-link tree.
//!
//! This module is deliberately independent from [`crate::btree`].  It is the
//! Phase 1 control implementation: one caller mutates the tree at a time, but
//! pages already carry the fences and sibling links that later phases will
//! use for optimistic reads and parallel execution.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::ErrorKind;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle, ThreadId};
use std::time::Instant;

use bumpalo::{Bump, collections::Vec as BumpVec};
use dodb_core::{
    DocumentKey, Error, Lsn, ObservedState, PageId, PrimaryKey, Result, Revision, RevisionState,
    ShardEpoch, ShardId, SortKey, TenantId, TransactionCondition, TransactionConflict,
    TransactionMutation, TransactionRequest, TransactionResult,
};

use crate::btree::{
    BatchRequest, BatchResponse, DatabaseConfig, Document, InvariantReport, StorageLimits,
    StorageMetrics,
};
mod leaf;

use crate::churn::{self, ChurnCounter, ChurnSite};
use crate::durable_file::{DurableFile, ProductionFile};
use crate::fault::FaultInjector;
#[cfg(test)]
use crate::page::encode_page;
use crate::page::{
    PAGE_HEADER_SIZE, PAGE_SIZE, PageHeader, PageType, decode_page_at, finalize_encoded_page,
};
use crate::wal::{
    PAGE_IMAGE_PAYLOAD_SIZE, PreparedWalCommit, PreparedWalRecord, PreparedWalRedo,
    RecoveredWalPage, WalCommit, WalDeltaRequest, WalIdentity, WalLog, WalMetrics, WalPageImage,
    WalPageImageFormat, decode_page_delta, encode_page_delta, page_delta_rebuilds,
};
use leaf::{BlinkValueRef, LeafEntries, LeafEntryRef, LeafRange, StoredValue};

const FIRST_DATA_PAGE: u64 = 2;
const PAGE_CATALOG_CHUNK_SIZE: usize = 64;
const NULL_PAGE_ID: u64 = u64::MAX;
const BLINK_SUPERBLOCK_MAGIC: [u8; 4] = *b"DBLK";
const BLINK_SUPERBLOCK_VERSION: u16 = 2;
const BLINK_ENGINE_TAG: [u8; 4] = *b"BLNK";
const SUPERBLOCK_CHECKSUM_OFFSET: usize = 100;
const SUPERBLOCK_ENGINE_OFFSET: usize = 96;
const BODY_SIZE: usize = PAGE_SIZE - PAGE_HEADER_SIZE;
const BODY_VERSION: u16 = 2;
const LEAF_MAGIC: [u8; 4] = *b"BLKL";
const INTERNAL_MAGIC: [u8; 4] = *b"BLKI";
const OVERFLOW_MAGIC: [u8; 4] = *b"BLKO";
const FREE_MAGIC: [u8; 4] = *b"BLKF";
const LEAF_HEADER_SIZE: usize = 48;
const INTERNAL_HEADER_SIZE: usize = 56;
const SLOT_SIZE: usize = 8;
const LEAF_RECORD_HEADER_SIZE: usize = 32;
const INTERNAL_RECORD_HEADER_SIZE: usize = 16;
const OVERFLOW_HEADER_SIZE: usize = 32;
const INLINE_VALUE_LIMIT: usize = 512;
const MAX_VALUE_SIZE: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct BlinkSplitMetrics {
    pub leaf_splits: u64,
    pub internal_splits: u64,
    pub root_splits: u64,
    pub right_link_corrections: u64,
    pub pages_touched: u64,
    pub page_images: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvisionalRevisionToken {
    pub transaction_position: usize,
    pub ordinal: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyKind {
    SameKey,
    ConditionKey,
    SameTargetPage,
    SameTransaction,
    StructuralRoute,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyEdge {
    pub predecessor: usize,
    pub successor: usize,
    pub kind: DependencyKind,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DependencyMetadata {
    pub same_key_predecessors: Vec<usize>,
    pub condition_key_predecessors: Vec<usize>,
    pub same_target_page_predecessors: Vec<usize>,
    pub same_transaction_positions: Vec<usize>,
    pub structural_route_predecessors: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteHint {
    pub leaf_id: PageId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlannedWrite {
    Put(Arc<[u8]>),
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedMutation {
    pub write: PlannedWrite,
    pub encoded_key: Vec<u8>,
    pub route_hint: RouteHint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalTransactionPlan {
    pub fifo_position: usize,
    pub provisional_revision: ProvisionalRevisionToken,
    pub mutations: Vec<PlannedMutation>,
    pub mutated_key_set: BTreeSet<Vec<u8>>,
    pub dependency_metadata: DependencyMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeafGroupPlan {
    pub leaf_hint: PageId,
    pub mutations: Vec<(usize, usize)>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BatchPlan {
    pub transactions: Vec<PhysicalTransactionPlan>,
    pub leaf_groups: Vec<LeafGroupPlan>,
    pub dependencies: Vec<DependencyEdge>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlinkBatchMetrics {
    pub logical_groups: u64,
    pub logical_transactions: u64,
    pub admitted_transactions: u64,
    pub conflicted_transactions: u64,
    pub rejected_transactions: u64,
    pub logical_admission_nanos: u64,
    pub planning_nanos: u64,
    pub planner_route_nanos: u64,
    pub planner_route_calls: u64,
    pub planner_route_page_visits: u64,
    pub planner_route_right_link_hops: u64,
    pub state_clone_nanos: u64,
    pub physical_execution_nanos: u64,
    pub physical_mutation_nanos: u64,
    pub leaf_load_clone_nanos: u64,
    pub leaf_entries_clone_nanos: u64,
    pub leaf_install_clone_nanos: u64,
    pub physical_restamp_nanos: u64,
    pub physical_cached_refresh_nanos: u64,
    pub physical_page_encode_nanos: u64,
    pub physical_superblock_encode_nanos: u64,
    pub superblock_images_emitted: u64,
    pub superblock_images_elided: u64,
    pub leaf_load_clones: u64,
    pub leaf_entries_clones: u64,
    pub leaf_install_clones: u64,
    pub cached_refresh_clones: u64,
    pub dirty_union_nanos: u64,
    pub full_state_clones: u64,
    pub mutations_planned: u64,
    pub routes_calculated: u64,
    pub route_reuses: u64,
    pub route_invalidations: u64,
    pub reroutes: u64,
    pub leaf_groups: u64,
    pub same_leaf_groups: u64,
    pub mutations_per_leaf_group: u64,
    pub coalesced_mutations: u64,
    pub independent_leaf_groups: u64,
    pub dependency_edges: u64,
    pub leaf_loads: u64,
    pub leaf_encodes: u64,
    pub structural_fallbacks: u64,
    pub split_triggered_reroutes: u64,
    pub coalescing_interruptions: u64,
    pub page_images: u64,
    pub wal_bytes: u64,
    pub catalog_construction_nanos: u64,
    pub catalog_map_clone_nanos: u64,
    pub catalog_directory_clone_nanos: u64,
    pub catalog_chunk_clone_nanos: u64,
    pub catalog_chunk_clones: u64,
    pub catalog_state_scan_nanos: u64,
    pub wal_assembly_nanos: u64,
    pub state_install_nanos: u64,
    pub generation_publication_nanos: u64,
    pub publication_swap_nanos: u64,
    pub retired_generation_drop_nanos: u64,
    pub dirty_tracking_nanos: u64,
    pub parallel_groups: u64,
    pub parallel_leaf_jobs: u64,
    pub parallel_transactions: u64,
    pub parallel_mutations: u64,
    pub parallel_worker_dispatches: u64,
    pub parallel_worker_nanos: u64,
    pub parallel_join_nanos: u64,
    pub parallel_job_operations: u64,
    pub parallel_dispatch_nanos: u64,
    pub parallel_collect_nanos: u64,
    pub parallel_worker_slot_nanos: u64,
    pub parallel_coordinator_lane_nanos: u64,
    pub parallel_worker_base_nanos: u64,
    pub parallel_worker_mutation_nanos: u64,
    pub parallel_worker_encode_nanos: u64,
    pub parallel_worker_delta_nanos: u64,
    pub parallel_fallback_groups: u64,
    pub parallel_fallback_after_dispatch: u64,
    pub parallel_fallback_no_delta_wal: u64,
    pub parallel_fallback_route: u64,
    pub parallel_fallback_overflow: u64,
    pub parallel_fallback_structural: u64,
    pub parallel_skipped_single_leaf: u64,
    pub parallel_skipped_small_group: u64,
    pub structural_transactions: u64,
    pub transaction_mutation_histogram: Vec<u64>,
    pub transaction_dirty_page_histogram: Vec<u64>,
    pub transaction_leaf_page_histogram: Vec<u64>,
}

pub const BLINK_LOCALITY_HISTOGRAM_BUCKETS: usize = 65;

fn record_histogram(histogram: &mut Vec<u64>, value: usize) {
    if histogram.len() < BLINK_LOCALITY_HISTOGRAM_BUCKETS {
        histogram.resize(BLINK_LOCALITY_HISTOGRAM_BUCKETS, 0);
    }
    histogram[value.min(BLINK_LOCALITY_HISTOGRAM_BUCKETS - 1)] += 1;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlinkCheckpointReport {
    pub checkpoint_lsn: Lsn,
    pub pages_flushed: usize,
    pub bytes_written: u64,
    pub wal_bytes_reclaimed: u64,
    pub duration_nanos: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BlinkSuperblock {
    generation: u64,
    database_uuid: [u8; 16],
    tenant_id: TenantId,
    shard_id: ShardId,
    shard_epoch: ShardEpoch,
    root_page_id: PageId,
    free_list_head: Option<PageId>,
    high_water_page_id: PageId,
    checkpoint_lsn: Lsn,
}

impl BlinkSuperblock {
    fn new(config: &DatabaseConfig, root_page_id: PageId) -> Self {
        Self {
            generation: 1,
            database_uuid: config.database_uuid,
            tenant_id: config.tenant_id,
            shard_id: config.shard_id,
            shard_epoch: config.shard_epoch,
            root_page_id,
            free_list_head: None,
            high_water_page_id: root_page_id,
            checkpoint_lsn: Lsn::ZERO,
        }
    }
}

pub(crate) fn decode_blink_superblock_image(bytes: &[u8]) -> Result<BlinkSuperblock> {
    if bytes.len() != PAGE_SIZE {
        return Err(Error::corruption(
            "experimental superblock has invalid length",
        ));
    }
    if bytes[0..4] != BLINK_SUPERBLOCK_MAGIC {
        return Err(Error::corruption("experimental superblock magic mismatch"));
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
    if version != BLINK_SUPERBLOCK_VERSION {
        return Err(Error::unsupported_format(format!(
            "experimental superblock version {version}, supported {BLINK_SUPERBLOCK_VERSION}"
        )));
    }
    if bytes[6..8].iter().any(|byte| *byte != 0)
        || bytes[SUPERBLOCK_ENGINE_OFFSET..SUPERBLOCK_ENGINE_OFFSET + 4] != BLINK_ENGINE_TAG
        || bytes[104..].iter().any(|byte| *byte != 0)
    {
        return Err(Error::corruption(
            "experimental superblock reserved bytes or engine tag are invalid",
        ));
    }
    let stored = u32::from_le_bytes(bytes[SUPERBLOCK_CHECKSUM_OFFSET..104].try_into().unwrap());
    let mut checksum_input = [0u8; PAGE_SIZE];
    checksum_input.copy_from_slice(bytes);
    checksum_input[SUPERBLOCK_CHECKSUM_OFFSET..104].fill(0);
    if stored != crc32c::crc32c(&checksum_input) {
        return Err(Error::corruption(
            "experimental superblock checksum mismatch",
        ));
    }
    let page_size = u32::from_le_bytes(bytes[56..60].try_into().unwrap());
    if page_size as usize != PAGE_SIZE {
        return Err(Error::unsupported_format(
            "experimental superblock page size mismatch",
        ));
    }
    let root = PageId::new(u64::from_le_bytes(bytes[60..68].try_into().unwrap()));
    if root.get() < FIRST_DATA_PAGE {
        return Err(Error::corruption("experimental root page id is invalid"));
    }
    let high = PageId::new(u64::from_le_bytes(bytes[76..84].try_into().unwrap()));
    if high < root {
        return Err(Error::corruption(
            "experimental high-water mark precedes root",
        ));
    }
    Ok(BlinkSuperblock {
        generation: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
        database_uuid: bytes[16..32].try_into().unwrap(),
        tenant_id: TenantId::new(u64::from_le_bytes(bytes[32..40].try_into().unwrap())),
        shard_id: ShardId::new(u64::from_le_bytes(bytes[40..48].try_into().unwrap())),
        shard_epoch: ShardEpoch::new(u64::from_le_bytes(bytes[48..56].try_into().unwrap())),
        root_page_id: root,
        free_list_head: decode_page_id(u64::from_le_bytes(bytes[68..76].try_into().unwrap())),
        high_water_page_id: high,
        checkpoint_lsn: Lsn::new(u64::from_le_bytes(bytes[84..92].try_into().unwrap())),
    })
}

fn encode_blink_superblock(sb: &BlinkSuperblock) -> Result<[u8; PAGE_SIZE]> {
    if sb.root_page_id.get() < FIRST_DATA_PAGE || sb.high_water_page_id < sb.root_page_id {
        return Err(Error::invalid_input(
            "experimental superblock page metadata is invalid",
        ));
    }
    let mut bytes = [0u8; PAGE_SIZE];
    bytes[0..4].copy_from_slice(&BLINK_SUPERBLOCK_MAGIC);
    bytes[4..6].copy_from_slice(&BLINK_SUPERBLOCK_VERSION.to_le_bytes());
    bytes[8..16].copy_from_slice(&sb.generation.to_le_bytes());
    bytes[16..32].copy_from_slice(&sb.database_uuid);
    bytes[32..40].copy_from_slice(&sb.tenant_id.get().to_le_bytes());
    bytes[40..48].copy_from_slice(&sb.shard_id.get().to_le_bytes());
    bytes[48..56].copy_from_slice(&sb.shard_epoch.get().to_le_bytes());
    bytes[56..60].copy_from_slice(&(PAGE_SIZE as u32).to_le_bytes());
    bytes[60..68].copy_from_slice(&sb.root_page_id.get().to_le_bytes());
    bytes[68..76].copy_from_slice(&encode_page_id(sb.free_list_head).to_le_bytes());
    bytes[76..84].copy_from_slice(&sb.high_water_page_id.get().to_le_bytes());
    bytes[84..92].copy_from_slice(&sb.checkpoint_lsn.get().to_le_bytes());
    bytes[SUPERBLOCK_ENGINE_OFFSET..SUPERBLOCK_ENGINE_OFFSET + 4]
        .copy_from_slice(&BLINK_ENGINE_TAG);
    let checksum = crc32c::crc32c(&bytes);
    bytes[SUPERBLOCK_CHECKSUM_OFFSET..104].copy_from_slice(&checksum.to_le_bytes());
    Ok(bytes)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InternalEntry {
    key: Vec<u8>,
    right_child: PageId,
}

#[derive(Debug, Eq, PartialEq)]
enum BlinkPage {
    Leaf {
        lsn: Lsn,
        high_key: Option<Vec<u8>>,
        right_sibling: Option<PageId>,
        entries: LeafEntries,
    },
    Internal {
        lsn: Lsn,
        level: u16,
        high_key: Option<Vec<u8>>,
        right_sibling: Option<PageId>,
        leftmost_child: PageId,
        entries: Vec<InternalEntry>,
    },
    Overflow {
        lsn: Lsn,
        next: Option<PageId>,
        total_length: u64,
        chunk: Vec<u8>,
    },
    Free {
        lsn: Lsn,
        next: Option<PageId>,
    },
}

impl Clone for BlinkPage {
    fn clone(&self) -> Self {
        match self {
            Self::Leaf {
                lsn,
                high_key,
                right_sibling,
                entries,
            } => {
                churn::add(ChurnCounter::LeafPageClones, 1);
                Self::Leaf {
                    lsn: *lsn,
                    high_key: high_key.clone(),
                    right_sibling: *right_sibling,
                    entries: entries.clone(),
                }
            }
            Self::Internal {
                lsn,
                level,
                high_key,
                right_sibling,
                leftmost_child,
                entries,
            } => {
                churn::add(ChurnCounter::InternalPageClones, 1);
                Self::Internal {
                    lsn: *lsn,
                    level: *level,
                    high_key: high_key.clone(),
                    right_sibling: *right_sibling,
                    leftmost_child: *leftmost_child,
                    entries: entries.clone(),
                }
            }
            Self::Overflow {
                lsn,
                next,
                total_length,
                chunk,
            } => Self::Overflow {
                lsn: *lsn,
                next: *next,
                total_length: *total_length,
                chunk: chunk.clone(),
            },
            Self::Free { lsn, next } => Self::Free {
                lsn: *lsn,
                next: *next,
            },
        }
    }
}

impl BlinkPage {
    fn page_type(&self) -> PageType {
        match self {
            Self::Leaf { .. } => PageType::Leaf,
            Self::Internal { .. } => PageType::Internal,
            Self::Overflow { .. } => PageType::Overflow,
            Self::Free { .. } => PageType::Free,
        }
    }

    fn lsn(&self) -> Lsn {
        match self {
            Self::Leaf { lsn, .. }
            | Self::Internal { lsn, .. }
            | Self::Overflow { lsn, .. }
            | Self::Free { lsn, .. } => *lsn,
        }
    }

    fn restamp(&mut self, provisional: Revision, committed: Lsn, mutated_keys: &BTreeSet<Vec<u8>>) {
        match self {
            Self::Leaf { lsn, entries, .. } => {
                *lsn = committed;
                for index in 0..entries.len() {
                    let entry = entries.get(index);
                    if entry.revision == provisional && mutated_keys.contains(entry.key) {
                        entries.set_revision(index, Revision::from(committed));
                    }
                }
            }
            Self::Internal { lsn, .. } | Self::Overflow { lsn, .. } | Self::Free { lsn, .. } => {
                *lsn = committed
            }
        }
    }
}

#[derive(Clone, Debug)]
struct BlinkState {
    pages: BTreeMap<PageId, Arc<BlinkPage>>,
    root_page_id: PageId,
    free_list_head: Option<PageId>,
    high_water_page_id: PageId,
    allow_page_reuse: bool,
}

trait BlinkMutationState {
    fn root_page_id(&self) -> PageId;
    fn set_root_page_id(&mut self, value: PageId);
    fn free_list_head(&self) -> Option<PageId>;
    fn set_free_list_head(&mut self, value: Option<PageId>);
    fn high_water_page_id(&self) -> PageId;
    fn set_high_water_page_id(&mut self, value: PageId);
    fn allow_page_reuse(&self) -> bool;
    fn page(&self, page_id: PageId) -> Option<&BlinkPage>;
    fn insert_page(&mut self, page_id: PageId, page: BlinkPage);
}

impl BlinkMutationState for BlinkState {
    fn root_page_id(&self) -> PageId {
        self.root_page_id
    }
    fn set_root_page_id(&mut self, value: PageId) {
        self.root_page_id = value;
    }
    fn free_list_head(&self) -> Option<PageId> {
        self.free_list_head
    }
    fn set_free_list_head(&mut self, value: Option<PageId>) {
        self.free_list_head = value;
    }
    fn high_water_page_id(&self) -> PageId {
        self.high_water_page_id
    }
    fn set_high_water_page_id(&mut self, value: PageId) {
        self.high_water_page_id = value;
    }
    fn allow_page_reuse(&self) -> bool {
        self.allow_page_reuse
    }
    fn page(&self, page_id: PageId) -> Option<&BlinkPage> {
        self.pages.get(&page_id).map(|page| &**page)
    }
    fn insert_page(&mut self, page_id: PageId, page: BlinkPage) {
        self.pages.insert(page_id, Arc::new(page));
    }
}

impl BlinkState {
    fn page_ref(&self, page_id: PageId) -> Option<&BlinkPage> {
        self.pages.get(&page_id).map(|page| &**page)
    }
}

struct WorkingBlinkState<'a> {
    base: &'a BlinkState,
    pages: BTreeMap<PageId, BlinkPage>,
    root_page_id: PageId,
    free_list_head: Option<PageId>,
    high_water_page_id: PageId,
    allow_page_reuse: bool,
}

impl<'a> WorkingBlinkState<'a> {
    fn new(base: &'a BlinkState, allow_page_reuse: bool) -> Self {
        Self {
            base,
            pages: BTreeMap::new(),
            root_page_id: base.root_page_id,
            free_list_head: base.free_list_head,
            high_water_page_id: base.high_water_page_id,
            allow_page_reuse,
        }
    }

    fn overlay_page_mut(&mut self, page_id: PageId) -> Option<&mut BlinkPage> {
        self.pages.get_mut(&page_id)
    }

    fn ensure_overlay_page(&mut self, page_id: PageId) -> Result<bool> {
        if self.pages.contains_key(&page_id) {
            return Ok(false);
        }
        let page = self
            .base
            .page_ref(page_id)
            .cloned()
            .ok_or_else(|| Error::corruption("planned Blink page is missing"))?;
        self.pages.insert(page_id, page);
        Ok(true)
    }

    fn into_delta(self) -> BlinkStateDelta {
        BlinkStateDelta {
            pages: self
                .pages
                .into_iter()
                .map(|(page_id, page)| (page_id, Arc::new(page)))
                .collect(),
            root_page_id: self.root_page_id,
            free_list_head: self.free_list_head,
            high_water_page_id: self.high_water_page_id,
            allow_page_reuse: self.allow_page_reuse,
        }
    }
}

impl BlinkMutationState for WorkingBlinkState<'_> {
    fn root_page_id(&self) -> PageId {
        self.root_page_id
    }
    fn set_root_page_id(&mut self, value: PageId) {
        self.root_page_id = value;
    }
    fn free_list_head(&self) -> Option<PageId> {
        self.free_list_head
    }
    fn set_free_list_head(&mut self, value: Option<PageId>) {
        self.free_list_head = value;
    }
    fn high_water_page_id(&self) -> PageId {
        self.high_water_page_id
    }
    fn set_high_water_page_id(&mut self, value: PageId) {
        self.high_water_page_id = value;
    }
    fn allow_page_reuse(&self) -> bool {
        self.allow_page_reuse
    }
    fn page(&self, page_id: PageId) -> Option<&BlinkPage> {
        self.pages
            .get(&page_id)
            .or_else(|| self.base.page_ref(page_id))
    }
    fn insert_page(&mut self, page_id: PageId, page: BlinkPage) {
        self.pages.insert(page_id, page);
    }
}

struct BlinkStateDelta {
    pages: BTreeMap<PageId, Arc<BlinkPage>>,
    root_page_id: PageId,
    free_list_head: Option<PageId>,
    high_water_page_id: PageId,
    allow_page_reuse: bool,
}

#[derive(Clone, Debug)]
enum LogicalRevision {
    Provisional(ProvisionalRevisionToken),
}

#[derive(Clone, Debug)]
struct LogicalEntry {
    present: bool,
    revision: LogicalRevision,
    originating_transaction_position: usize,
}

struct LogicalOverlay<'a> {
    committed: &'a BlinkState,
    entries: BTreeMap<Vec<u8>, LogicalEntry>,
}

struct AdmittedTransaction<'a> {
    fifo_position: usize,
    request: &'a TransactionRequest,
    encoded_mutation_keys: Vec<Vec<u8>>,
    provisional_revision: ProvisionalRevisionToken,
}

struct CachedLeaf {
    leaf_id: PageId,
}

struct PhysicalExecutionState {
    cached_leaf: Option<CachedLeaf>,
}

#[derive(Clone, Debug)]
struct PageVersion {
    epoch: u64,
    page: Arc<BlinkPage>,
}

/// A generation-scoped immutable page cell. A later generation may replace the
/// cell for the same logical PageId, but a pinned older generation retains this
/// cell and therefore the old page object until its last reader releases it.
#[derive(Debug)]
struct PageCell {
    version: PageVersion,
    metrics: Arc<PublicationMetrics>,
}

impl PageCell {
    fn page_at(&self, epoch: u64) -> Result<Arc<BlinkPage>> {
        if self.version.epoch > epoch {
            return Err(Error::corruption(
                "published page version is newer than its generation",
            ));
        }
        Ok(Arc::clone(&self.version.page))
    }
}

impl Drop for PageCell {
    fn drop(&mut self) {
        self.metrics
            .versions_reclaimed
            .fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Clone, Debug)]
struct PageCatalogChunk {
    entries: [Option<Arc<PageCell>>; PAGE_CATALOG_CHUNK_SIZE],
}

impl PageCatalogChunk {
    fn empty() -> Self {
        Self {
            entries: std::array::from_fn(|_| None),
        }
    }
}

#[derive(Debug)]
struct PageCatalog {
    chunks: Vec<Arc<PageCatalogChunk>>,
}

impl PageCatalog {
    fn get(&self, page_id: PageId) -> Option<&Arc<PageCell>> {
        if page_id.get() < FIRST_DATA_PAGE {
            return None;
        }
        let chunk_index = usize::try_from(page_id.get() / PAGE_CATALOG_CHUNK_SIZE as u64).ok()?;
        let slot_index = usize::try_from(page_id.get() % PAGE_CATALOG_CHUNK_SIZE as u64).ok()?;
        self.chunks.get(chunk_index)?.entries[slot_index].as_ref()
    }
}

#[derive(Debug)]
struct PublishedGeneration {
    epoch: u64,
    root_page_id: PageId,
    high_water_page_id: PageId,
    catalog: Arc<PageCatalog>,
}

#[derive(Debug, Default)]
struct PublicationMetrics {
    generation_pins: AtomicU64,
    active_generation_pins: AtomicU64,
    max_concurrent_pins: AtomicU64,
    page_version_installs: AtomicU64,
    versions_reclaimed: AtomicU64,
    published_generations: AtomicU64,
    right_link_corrections: AtomicU64,
    read_operations: AtomicU64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlinkVersionedReadMetrics {
    pub published_generations: u64,
    pub active_generation_pins: u64,
    pub max_concurrent_pins: u64,
    pub generation_pins: u64,
    pub page_version_installs: u64,
    pub versions_retained: u64,
    pub versions_reclaimed: u64,
    pub retired_page_ids: u64,
    pub reusable_page_ids: u64,
    pub read_operations: u64,
    pub right_link_corrections: u64,
}

impl PublicationMetrics {
    fn snapshot(&self, retired_page_ids: u64, reusable_page_ids: u64) -> BlinkVersionedReadMetrics {
        let installed = self.page_version_installs.load(Ordering::Relaxed);
        let reclaimed = self.versions_reclaimed.load(Ordering::Relaxed);
        BlinkVersionedReadMetrics {
            published_generations: self.published_generations.load(Ordering::Relaxed),
            active_generation_pins: self.active_generation_pins.load(Ordering::Relaxed),
            max_concurrent_pins: self.max_concurrent_pins.load(Ordering::Relaxed),
            generation_pins: self.generation_pins.load(Ordering::Relaxed),
            page_version_installs: installed,
            versions_retained: installed.saturating_sub(reclaimed),
            versions_reclaimed: reclaimed,
            retired_page_ids,
            reusable_page_ids,
            read_operations: self.read_operations.load(Ordering::Relaxed),
            right_link_corrections: self.right_link_corrections.load(Ordering::Relaxed),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PublicationPrepareTiming {
    catalog_map_clone_nanos: u64,
    catalog_directory_clone_nanos: u64,
    catalog_chunk_clone_nanos: u64,
    catalog_chunk_clones: u64,
    catalog_state_scan_nanos: u64,
}

#[derive(Clone, Copy, Debug, Default)]
struct PublicationPublishTiming {
    swap_nanos: u64,
    retired_generation_drop_nanos: u64,
}

#[derive(Debug)]
struct GenerationPublisher {
    current: RwLock<Arc<PublishedGeneration>>,
    metrics: Arc<PublicationMetrics>,
}

impl GenerationPublisher {
    fn new(state: &BlinkState, epoch: u64) -> Result<Arc<Self>> {
        let metrics = Arc::new(PublicationMetrics::default());
        let chunk_count = Self::chunk_count(state.high_water_page_id.get())
            .ok_or_else(|| Error::invariant("Blink catalog chunk count overflows usize"))?;
        let mut chunks = (0..chunk_count)
            .map(|_| PageCatalogChunk::empty())
            .collect::<Vec<_>>();
        for (page_id, page) in &state.pages {
            if let Some((chunk_index, slot_index)) = Self::indices(*page_id) {
                chunks[chunk_index].entries[slot_index] = Some(Arc::new(PageCell {
                    version: PageVersion {
                        epoch,
                        page: Arc::clone(page),
                    },
                    metrics: Arc::clone(&metrics),
                }));
            }
            metrics
                .page_version_installs
                .fetch_add(1, Ordering::Relaxed);
        }
        let chunks = chunks.into_iter().map(Arc::new).collect();
        let generation = Arc::new(PublishedGeneration {
            epoch,
            root_page_id: state.root_page_id,
            high_water_page_id: state.high_water_page_id,
            catalog: Arc::new(PageCatalog { chunks }),
        });
        Ok(Arc::new(Self {
            current: RwLock::new(generation),
            metrics,
        }))
    }

    fn pin(&self) -> GenerationPin {
        let generation = Arc::clone(&self.current.read().expect("generation lock poisoned"));
        let active = self
            .metrics
            .active_generation_pins
            .fetch_add(1, Ordering::Relaxed)
            + 1;
        self.metrics.generation_pins.fetch_add(1, Ordering::Relaxed);
        let mut observed = self.metrics.max_concurrent_pins.load(Ordering::Relaxed);
        while active > observed {
            match self.metrics.max_concurrent_pins.compare_exchange_weak(
                observed,
                active,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(next) => observed = next,
            }
        }
        GenerationPin {
            generation,
            metrics: Arc::clone(&self.metrics),
        }
    }

    fn prepare(
        &self,
        state: &BlinkState,
        superblock: &BlinkSuperblock,
        dirty: &BTreeSet<PageId>,
    ) -> Result<(Arc<PublishedGeneration>, PublicationPrepareTiming)> {
        let base = self.pin();
        let mut dirty_or_missing = dirty.clone();
        for page_id in state.pages.keys() {
            if base.generation.catalog.get(*page_id).is_none() {
                dirty_or_missing.insert(*page_id);
            }
        }
        let (catalog, timing) = self.prepare_catalog_delta(
            &base.generation.catalog,
            base.generation.high_water_page_id,
            superblock.high_water_page_id,
            superblock.generation,
            &dirty_or_missing,
            |page_id| state.pages.get(&page_id).map(Arc::clone),
            false,
        )?;
        Ok((
            Arc::new(PublishedGeneration {
                epoch: superblock.generation,
                root_page_id: state.root_page_id,
                high_water_page_id: state.high_water_page_id,
                catalog: Arc::new(catalog),
            }),
            timing,
        ))
    }

    #[cfg(test)]
    fn prepare_delta<S: BlinkMutationState>(
        &self,
        state: &S,
        superblock: &BlinkSuperblock,
        dirty: &BTreeSet<PageId>,
    ) -> Result<(Arc<PublishedGeneration>, PublicationPrepareTiming)> {
        let base = self.pin();
        let (catalog, timing) = self.prepare_catalog_delta(
            &base.generation.catalog,
            base.generation.high_water_page_id,
            state.high_water_page_id(),
            superblock.generation,
            dirty,
            |page_id| state.page(page_id).cloned().map(Arc::new),
            true,
        )?;
        Ok((
            Arc::new(PublishedGeneration {
                epoch: superblock.generation,
                root_page_id: state.root_page_id(),
                high_water_page_id: state.high_water_page_id(),
                catalog: Arc::new(catalog),
            }),
            timing,
        ))
    }

    fn prepare_shared_delta(
        &self,
        delta: &BlinkStateDelta,
        base: &BlinkState,
        superblock: &BlinkSuperblock,
        dirty: &BTreeSet<PageId>,
    ) -> Result<(Arc<PublishedGeneration>, PublicationPrepareTiming)> {
        let published = self.pin();
        let (catalog, timing) = self.prepare_catalog_delta(
            &published.generation.catalog,
            published.generation.high_water_page_id,
            delta.high_water_page_id,
            superblock.generation,
            dirty,
            |page_id| {
                delta
                    .pages
                    .get(&page_id)
                    .or_else(|| base.pages.get(&page_id))
                    .map(Arc::clone)
            },
            true,
        )?;
        Ok((
            Arc::new(PublishedGeneration {
                epoch: superblock.generation,
                root_page_id: delta.root_page_id,
                high_water_page_id: delta.high_water_page_id,
                catalog: Arc::new(catalog),
            }),
            timing,
        ))
    }

    fn prepare_catalog_delta<F>(
        &self,
        base: &PageCatalog,
        base_high_water_page_id: PageId,
        high_water_page_id: PageId,
        epoch: u64,
        dirty: &BTreeSet<PageId>,
        mut page_for: F,
        validate_extension: bool,
    ) -> Result<(PageCatalog, PublicationPrepareTiming)>
    where
        F: FnMut(PageId) -> Option<Arc<BlinkPage>>,
    {
        let directory_clone_started = Instant::now();
        let mut chunks = base.chunks.clone();
        let catalog_directory_clone_nanos = elapsed_nanos(directory_clone_started);
        let target_chunk_count = Self::chunk_count(high_water_page_id.get())
            .ok_or_else(|| Error::invariant("Blink catalog chunk count overflows usize"))?;
        while chunks.len() < target_chunk_count {
            chunks.push(Arc::new(PageCatalogChunk::empty()));
        }

        if validate_extension && high_water_page_id > base_high_water_page_id {
            for raw_page_id in
                base_high_water_page_id.get().saturating_add(1)..=high_water_page_id.get()
            {
                let page_id = PageId::new(raw_page_id);
                if !dirty.contains(&page_id) || page_for(page_id).is_none() {
                    return Err(Error::invariant(
                        "new high-water Blink page is missing from dirty state",
                    ));
                }
            }
        }

        let catalog_state_scan_started = Instant::now();
        let mut dirty_by_chunk = BTreeMap::<usize, Vec<PageId>>::new();
        for page_id in dirty {
            if let Some((chunk_index, _)) = Self::indices(*page_id) {
                dirty_by_chunk
                    .entry(chunk_index)
                    .or_default()
                    .push(*page_id);
            }
        }
        let mut catalog_chunk_clone_nanos = 0u64;
        let mut catalog_chunk_clones = 0u64;
        for (chunk_index, page_ids) in dirty_by_chunk {
            let chunk_clone_started = Instant::now();
            let mut next_chunk = chunks
                .get(chunk_index)
                .ok_or_else(|| Error::invariant("dirty Blink page exceeds catalog directory"))?
                .as_ref()
                .clone();
            catalog_chunk_clone_nanos =
                catalog_chunk_clone_nanos.saturating_add(elapsed_nanos(chunk_clone_started));
            catalog_chunk_clones = catalog_chunk_clones.saturating_add(1);
            for page_id in page_ids {
                let (_, slot_index) = Self::indices(page_id)
                    .ok_or_else(|| Error::invariant("invalid Blink catalog page id"))?;
                let page = page_for(page_id)
                    .ok_or_else(|| Error::invariant("dirty planned page is missing"))?;
                next_chunk.entries[slot_index] = Some(Arc::new(PageCell {
                    version: PageVersion { epoch, page },
                    metrics: Arc::clone(&self.metrics),
                }));
                self.metrics
                    .page_version_installs
                    .fetch_add(1, Ordering::Relaxed);
            }
            chunks[chunk_index] = Arc::new(next_chunk);
        }
        let catalog_map_clone_nanos =
            catalog_directory_clone_nanos.saturating_add(catalog_chunk_clone_nanos);
        let timing = PublicationPrepareTiming {
            catalog_map_clone_nanos,
            catalog_directory_clone_nanos,
            catalog_chunk_clone_nanos,
            catalog_chunk_clones,
            catalog_state_scan_nanos: elapsed_nanos(catalog_state_scan_started),
        };
        Ok((PageCatalog { chunks }, timing))
    }

    fn chunk_count(high_water_page_id: u64) -> Option<usize> {
        let last_chunk = high_water_page_id / PAGE_CATALOG_CHUNK_SIZE as u64;
        usize::try_from(last_chunk.checked_add(1)?).ok()
    }

    fn indices(page_id: PageId) -> Option<(usize, usize)> {
        if page_id.get() < FIRST_DATA_PAGE {
            return None;
        }
        Some((
            usize::try_from(page_id.get() / PAGE_CATALOG_CHUNK_SIZE as u64).ok()?,
            usize::try_from(page_id.get() % PAGE_CATALOG_CHUNK_SIZE as u64).ok()?,
        ))
    }

    fn publish(&self, generation: Arc<PublishedGeneration>) -> PublicationPublishTiming {
        let mut current = self.current.write().expect("generation lock poisoned");
        let swap_started = Instant::now();
        let retired_generation = std::mem::replace(&mut *current, generation);
        let swap_nanos = elapsed_nanos(swap_started);
        let retired_generation_drop_started = Instant::now();
        drop(retired_generation);
        let retired_generation_drop_nanos = elapsed_nanos(retired_generation_drop_started);
        drop(current);
        self.metrics
            .published_generations
            .fetch_add(1, Ordering::Relaxed);
        PublicationPublishTiming {
            swap_nanos,
            retired_generation_drop_nanos,
        }
    }

    fn can_reuse_pages(&self) -> bool {
        self.metrics.active_generation_pins.load(Ordering::Acquire) == 0
    }

    fn metrics(&self, retired_page_ids: u64, reusable_page_ids: u64) -> BlinkVersionedReadMetrics {
        self.metrics.snapshot(retired_page_ids, reusable_page_ids)
    }
}

#[derive(Debug)]
struct GenerationPin {
    generation: Arc<PublishedGeneration>,
    metrics: Arc<PublicationMetrics>,
}

impl Drop for GenerationPin {
    fn drop(&mut self) {
        self.metrics
            .active_generation_pins
            .fetch_sub(1, Ordering::Release);
    }
}

#[derive(Clone, Debug)]
pub struct BlinkReadHandle {
    publisher: Arc<GenerationPublisher>,
}

impl BlinkReadHandle {
    pub fn get(&self, key: &DocumentKey) -> Result<RevisionState> {
        let pin = self.publisher.pin();
        self.publisher
            .metrics
            .read_operations
            .fetch_add(1, Ordering::Relaxed);
        let mut corrections = 0;
        let result = read_state(&pin, key, &mut corrections);
        self.publisher
            .metrics
            .right_link_corrections
            .fetch_add(corrections, Ordering::Relaxed);
        result
    }

    pub fn query(
        &self,
        pk: &PrimaryKey,
        exclusive_after_sk: Option<&SortKey>,
        limit: usize,
    ) -> Result<Vec<Document>> {
        let pin = self.publisher.pin();
        self.publisher
            .metrics
            .read_operations
            .fetch_add(1, Ordering::Relaxed);
        let mut corrections = 0;
        let result = query_state(&pin, pk, exclusive_after_sk, limit, &mut corrections);
        self.publisher
            .metrics
            .right_link_corrections
            .fetch_add(corrections, Ordering::Relaxed);
        result
    }

    pub fn scan(&self, cursor: Option<&DocumentKey>, limit: usize) -> Result<Vec<Document>> {
        let pin = self.publisher.pin();
        self.publisher
            .metrics
            .read_operations
            .fetch_add(1, Ordering::Relaxed);
        let mut corrections = 0;
        let result = scan_state(&pin, cursor, limit, &mut corrections);
        self.publisher
            .metrics
            .right_link_corrections
            .fetch_add(corrections, Ordering::Relaxed);
        result
    }

    pub fn metrics(&self) -> BlinkVersionedReadMetrics {
        self.publisher.metrics(0, 0)
    }
}

#[derive(Clone, Debug)]
struct Candidate {
    state: BlinkState,
    dirty: BTreeSet<PageId>,
    result: TransactionResult,
    superblock: BlinkSuperblock,
    slot: SuperblockSlot,
    next_revision: Revision,
    next_lsn: Lsn,
    next_batch_id: u64,
}

struct ExecutedPlanTransaction {
    batch_id: u64,
    result: TransactionResult,
    dirty: BTreeSet<PageId>,
    images: Vec<WalPageImage>,
    superblock: BlinkSuperblock,
    slot: SuperblockSlot,
    superblock_image_emitted: bool,
    next_revision: Revision,
    next_lsn: Lsn,
    next_batch_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParallelFallbackReason {
    NoPageDeltaWal,
    RouteMismatch,
    OverflowOrAllocator,
    Structural,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParallelWorkerFault {
    Error { leaf_group_index: usize },
    Panic { leaf_group_index: usize },
}

/// All physical work of one leaf in one WAL group: the mutations of every
/// transaction that routes to this leaf, in transaction FIFO order.
struct LeafChainJob {
    leaf_id: PageId,
    initial_page: Arc<BlinkPage>,
    base_image: Option<Arc<[u8; PAGE_SIZE]>>,
    chain_entry: Option<(Lsn, u32)>,
    steps: Vec<(u32, u32)>,
    plan: Arc<BatchPlan>,
    commit_lsns: Arc<[Lsn]>,
    #[cfg(test)]
    fault: Option<ParallelWorkerFault>,
}

enum LeafChainRedo {
    Image(Arc<[u8; PAGE_SIZE]>),
    Delta {
        payload: Vec<u8>,
        base_lsn: Lsn,
        base_crc: u32,
        spans: u64,
        changed_bytes: u64,
    },
}

struct LeafChainBoundary {
    transaction_index: usize,
    commit_lsn: Lsn,
    image_crc: u32,
    redo: LeafChainRedo,
}

#[derive(Clone, Copy, Debug, Default)]
struct LeafChainTiming {
    base_nanos: u64,
    mutation_nanos: u64,
    encode_nanos: u64,
    delta_nanos: u64,
}

struct LeafChainResult {
    leaf_id: PageId,
    boundaries: Vec<LeafChainBoundary>,
    final_page: BlinkPage,
    final_image: Arc<[u8; PAGE_SIZE]>,
    timing: LeafChainTiming,
}

enum LeafChainOutcome {
    Prepared(LeafChainResult),
    Fallback { reason: ParallelFallbackReason },
}

/// Worker output kept by the coordinator until the WAL append. Each
/// transaction lists its records as (result index, boundary index) in page
/// order, the same order the serial executor writes its dirty pages.
struct LeafParallelRedo {
    results: Vec<LeafChainResult>,
    transaction_records: Vec<Vec<(u32, u32)>>,
}

struct PlannedExecutionPreparation<'a> {
    working: WorkingBlinkState<'a>,
    executed: Vec<ExecutedPlanTransaction>,
    parallel_redo: Option<LeafParallelRedo>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SuperblockSlot {
    A,
    B,
}

/// A serial, durable B-link tree. It is intentionally not a drop-in alias for
/// `BTreeStore`: opening and decoding are format-specific and explicit.
pub struct BlinkStore<F: DurableFile, W: DurableFile = crate::btree::NoWal> {
    file: F,
    wal: Option<WalLog<W>>,
    state: BlinkState,
    publisher: Arc<GenerationPublisher>,
    current_superblock: BlinkSuperblock,
    active_slot: SuperblockSlot,
    next_revision: Revision,
    next_lsn: Lsn,
    next_batch_id: u64,
    config: DatabaseConfig,
    dirty_pages: BTreeMap<PageId, Arc<[u8; PAGE_SIZE]>>,
    group_scratch: Bump,
    dirty_superblock: Option<[u8; PAGE_SIZE]>,
    storage_metrics: StorageMetrics,
    split_metrics: BlinkSplitMetrics,
    batch_metrics: BlinkBatchMetrics,
    planned_execution: bool,
    parallel_workers: usize,
    parallel_min_group_mutations: usize,
    parallel_worker_pool: Option<ParallelWorkerPool>,
    #[cfg(test)]
    parallel_worker_fault: Option<ParallelWorkerFault>,
    broken: Option<String>,
    fault_injector: Option<Box<dyn FaultInjector + Send>>,
}

impl<F: DurableFile, W: DurableFile> BlinkStore<F, W> {
    pub fn open_with_wal(file: F, wal_file: W, config: DatabaseConfig) -> Result<Self> {
        Self::open_internal(file, wal_file, config, None)
    }

    pub fn open_with_wal_and_fault_injector<I>(
        file: F,
        wal_file: W,
        config: DatabaseConfig,
        injector: I,
    ) -> Result<Self>
    where
        I: FaultInjector + Send + 'static,
    {
        Self::open_internal(file, wal_file, config, Some(Box::new(injector)))
    }

    pub fn open_path(
        path: impl AsRef<Path>,
        config: DatabaseConfig,
    ) -> Result<BlinkStore<ProductionFile, ProductionFile>> {
        let path = path.as_ref();
        BlinkStore::<ProductionFile, ProductionFile>::open_with_wal(
            ProductionFile::open(path)?,
            ProductionFile::open(path.with_extension("wal"))?,
            config,
        )
    }

    fn open_internal(
        mut file: F,
        wal_file: W,
        config: DatabaseConfig,
        mut fault_injector: Option<Box<dyn FaultInjector + Send>>,
    ) -> Result<Self> {
        let identity = WalIdentity::new(
            config.database_uuid,
            config.tenant_id,
            config.shard_id,
            config.shard_epoch,
        );
        let checkpoint_hint = existing_checkpoint(&mut file, &identity, wal_file.len()?)?;
        let mut wal = WalLog::open_with_page_image_format_and_fault_injector_and_start_lsn(
            wal_file,
            identity,
            WalPageImageFormat::ExperimentalBlink,
            checkpoint_hint,
            fault_injector.as_deref_mut(),
        )?;
        let recovery_pages = wal.take_recovery_pages();
        if file.is_empty()? && recovery_pages.is_empty() {
            let mut store = Self::initialize(file, config)?;
            store.wal = Some(wal);
            store.next_lsn = store.wal.as_ref().unwrap().next_lsn();
            store.next_batch_id = store.wal.as_ref().unwrap().next_batch_id();
            store.fault_injector = fault_injector;
            return Ok(store);
        }
        recover_data_file(&mut file, &recovery_pages, checkpoint_hint)?;
        drop(recovery_pages);
        let (mut store, selected) = Self::load_file(file, config)?;
        wal.resume_after(selected.checkpoint_lsn)?;
        store.wal = Some(wal);
        store.next_lsn = store.wal.as_ref().unwrap().next_lsn();
        store.next_batch_id = store.wal.as_ref().unwrap().next_batch_id();
        store.next_revision = Revision::new(
            store
                .check_invariants()?
                .max_revision
                .get()
                .max(store.next_lsn.get())
                .checked_add(1)
                .ok_or_else(|| Error::invariant("experimental revision exhausted"))?,
        );
        store.fault_injector = fault_injector;
        Ok(store)
    }

    fn initialize(mut file: F, config: DatabaseConfig) -> Result<Self> {
        let root = PageId::new(FIRST_DATA_PAGE);
        let state = BlinkState {
            pages: BTreeMap::from([(
                root,
                Arc::new(BlinkPage::Leaf {
                    lsn: Lsn::ZERO,
                    high_key: None,
                    right_sibling: None,
                    entries: LeafEntries::default(),
                }),
            )]),
            root_page_id: root,
            free_list_head: None,
            high_water_page_id: root,
            allow_page_reuse: true,
        };
        let sb = BlinkSuperblock::new(&config, root);
        let sb_bytes = encode_blink_superblock(&sb)?;
        let root_bytes = encode_blink_page(root, state.page_ref(root).unwrap())?;
        file.set_len((FIRST_DATA_PAGE + 1) * PAGE_SIZE as u64)?;
        write_all_at(&mut file, 0, &sb_bytes)?;
        write_all_at(&mut file, PAGE_SIZE as u64, &sb_bytes)?;
        write_all_at(&mut file, root.get() * PAGE_SIZE as u64, &root_bytes)?;
        file.sync_all()?;
        let publisher = GenerationPublisher::new(&state, sb.generation)?;
        Ok(Self {
            file,
            wal: None,
            state,
            publisher,
            current_superblock: sb,
            active_slot: SuperblockSlot::B,
            next_revision: Revision::new(1),
            next_lsn: Lsn::new(1),
            next_batch_id: 1,
            config,
            dirty_pages: BTreeMap::new(),
            group_scratch: Bump::new(),
            dirty_superblock: None,
            storage_metrics: StorageMetrics::default(),
            split_metrics: BlinkSplitMetrics::default(),
            batch_metrics: BlinkBatchMetrics::default(),
            planned_execution: false,
            parallel_workers: 1,
            parallel_min_group_mutations: 0,
            parallel_worker_pool: None,
            #[cfg(test)]
            parallel_worker_fault: None,
            broken: None,
            fault_injector: None,
        })
    }

    fn load_file(mut file: F, config: DatabaseConfig) -> Result<(Self, BlinkSuperblock)> {
        let length = file.len()?;
        if length < FIRST_DATA_PAGE * PAGE_SIZE as u64 || !length.is_multiple_of(PAGE_SIZE as u64) {
            return Err(Error::corruption(
                "experimental file length is not page aligned",
            ));
        }
        let a = read_exact_at(&mut file, 0, PAGE_SIZE)?;
        let b = read_exact_at(&mut file, PAGE_SIZE as u64, PAGE_SIZE)?;
        let sb_a = decode_blink_superblock_image(&a);
        let sb_b = decode_blink_superblock_image(&b);
        let (active_slot, sb) = match (sb_a, sb_b) {
            (Ok(a), Ok(b)) if a.generation >= b.generation => (SuperblockSlot::A, a),
            (Ok(_), Ok(b)) => (SuperblockSlot::B, b),
            (Ok(a), Err(_)) => (SuperblockSlot::A, a),
            (Err(_), Ok(b)) => (SuperblockSlot::B, b),
            (Err(a), Err(b)) => {
                return Err(Error::corruption(format!(
                    "no valid experimental superblock: {a}; {b}"
                )));
            }
        };
        if sb.database_uuid != config.database_uuid
            || sb.tenant_id != config.tenant_id
            || sb.shard_id != config.shard_id
            || sb.shard_epoch != config.shard_epoch
        {
            return Err(Error::corruption(
                "experimental superblock identity mismatch",
            ));
        }
        let mut pages = BTreeMap::new();
        for id in FIRST_DATA_PAGE..=sb.high_water_page_id.get() {
            let page_id = PageId::new(id);
            let bytes = read_exact_at(&mut file, id * PAGE_SIZE as u64, PAGE_SIZE)?;
            pages.insert(page_id, Arc::new(decode_blink_page(&bytes, page_id)?));
        }
        let state = BlinkState {
            pages,
            root_page_id: sb.root_page_id,
            free_list_head: sb.free_list_head,
            high_water_page_id: sb.high_water_page_id,
            allow_page_reuse: true,
        };
        let publisher = GenerationPublisher::new(&state, sb.generation)?;
        let store = Self {
            file,
            wal: None,
            state,
            publisher,
            current_superblock: sb.clone(),
            active_slot,
            next_revision: Revision::new(1),
            next_lsn: Lsn::new(1),
            next_batch_id: 1,
            config,
            dirty_pages: BTreeMap::new(),
            group_scratch: Bump::new(),
            dirty_superblock: None,
            storage_metrics: StorageMetrics::default(),
            split_metrics: BlinkSplitMetrics::default(),
            batch_metrics: BlinkBatchMetrics::default(),
            planned_execution: false,
            parallel_workers: 1,
            parallel_min_group_mutations: 0,
            parallel_worker_pool: None,
            #[cfg(test)]
            parallel_worker_fault: None,
            broken: None,
            fault_injector: None,
        };
        store.check_invariants()?;
        Ok((store, sb))
    }

    pub fn storage_metrics(&self) -> StorageMetrics {
        self.storage_metrics.clone()
    }

    pub fn wal_metrics(&self) -> Result<Option<WalMetrics>> {
        self.wal.as_ref().map(WalLog::metrics).transpose()
    }

    pub fn split_metrics(&self) -> BlinkSplitMetrics {
        self.split_metrics.clone()
    }

    pub fn batch_metrics(&self) -> BlinkBatchMetrics {
        self.batch_metrics.clone()
    }

    pub fn dirty_page_count(&self) -> usize {
        self.dirty_pages.len()
    }

    pub fn enable_planned_execution(&mut self) {
        self.planned_execution = true;
    }

    /// Groups with fewer planned mutations than `mutations` run on the serial
    /// executor even when the parallel executor is enabled. 0 keeps every
    /// multi-leaf group eligible.
    pub fn set_parallel_min_group_mutations(&mut self, mutations: usize) {
        self.parallel_min_group_mutations = mutations;
    }

    pub fn enable_parallel_execution(&mut self, workers: usize) -> Result<()> {
        let worker_count = workers.max(1);
        if self
            .parallel_worker_pool
            .as_ref()
            .is_some_and(|pool| pool.workers.len() + 1 == worker_count)
        {
            self.planned_execution = true;
            self.parallel_workers = worker_count;
            return Ok(());
        }
        let worker_pool = ParallelWorkerPool::new(worker_count - 1)?;
        self.planned_execution = true;
        self.parallel_workers = worker_count;
        self.parallel_worker_pool = Some(worker_pool);
        Ok(())
    }

    pub fn current_superblock_generation(&self) -> u64 {
        self.current_superblock.generation
    }

    /// Returns a read-only handle whose operations pin one immutable committed
    /// generation for their full duration. The handle does not borrow or lock
    /// the serial writer store while traversing the tree.
    pub fn versioned_read_handle(&self) -> BlinkReadHandle {
        BlinkReadHandle {
            publisher: Arc::clone(&self.publisher),
        }
    }

    pub fn versioned_read_metrics(&self) -> BlinkVersionedReadMetrics {
        let retired_page_ids = count_free_pages(&self.state).unwrap_or(0);
        let reusable_page_ids = if self.publisher.can_reuse_pages() {
            retired_page_ids
        } else {
            0
        };
        self.publisher.metrics(retired_page_ids, reusable_page_ids)
    }

    pub fn set_fault_injector<I>(&mut self, injector: I)
    where
        I: FaultInjector + Send + 'static,
    {
        self.fault_injector = Some(Box::new(injector));
    }

    pub fn get(&mut self, key: &DocumentKey) -> Result<RevisionState> {
        match self
            .apply_batch(&[BatchRequest::Get { key: key.clone() }])?
            .remove(0)
        {
            BatchResponse::Get(value) => Ok(value),
            _ => Err(Error::invariant("Blink get response mismatch")),
        }
    }

    pub fn query(
        &mut self,
        pk: &PrimaryKey,
        exclusive_after_sk: Option<&SortKey>,
        limit: usize,
    ) -> Result<Vec<Document>> {
        match self
            .apply_batch(&[BatchRequest::Query {
                pk: pk.clone(),
                exclusive_after_sk: exclusive_after_sk.cloned(),
                limit,
            }])?
            .remove(0)
        {
            BatchResponse::Query(rows) => Ok(rows),
            _ => Err(Error::invariant("Blink query response mismatch")),
        }
    }

    pub fn scan(&mut self, cursor: Option<&DocumentKey>, limit: usize) -> Result<Vec<Document>> {
        match self
            .apply_batch(&[BatchRequest::Scan {
                exclusive_after_key: cursor.cloned(),
                limit,
            }])?
            .remove(0)
        {
            BatchResponse::Scan(rows) => Ok(rows),
            _ => Err(Error::invariant("Blink scan response mismatch")),
        }
    }

    pub fn put(&mut self, key: DocumentKey, value: impl Into<Vec<u8>>) -> Result<Revision> {
        match self
            .apply_batch(&[BatchRequest::Put {
                key,
                value: value.into(),
            }])?
            .remove(0)
        {
            BatchResponse::Put(revision) => Ok(revision),
            _ => Err(Error::invariant("Blink put response mismatch")),
        }
    }

    pub fn delete(&mut self, key: DocumentKey) -> Result<Revision> {
        match self.apply_batch(&[BatchRequest::Delete { key }])?.remove(0) {
            BatchResponse::Delete(revision) => Ok(revision),
            _ => Err(Error::invariant("Blink delete response mismatch")),
        }
    }

    pub fn apply_batch(&mut self, requests: &[BatchRequest]) -> Result<Vec<BatchResponse>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        let mut responses = Vec::with_capacity(requests.len());
        let mut right_link_corrections = 0;
        let started = Instant::now();
        for request in requests {
            match request {
                BatchRequest::Get { key } => responses.push(BatchResponse::Get(read_state(
                    &self.state,
                    key,
                    &mut right_link_corrections,
                )?)),
                BatchRequest::Query {
                    pk,
                    exclusive_after_sk,
                    limit,
                } => responses.push(BatchResponse::Query(query_state(
                    &self.state,
                    pk,
                    exclusive_after_sk.as_ref(),
                    *limit,
                    &mut right_link_corrections,
                )?)),
                BatchRequest::Scan {
                    exclusive_after_key,
                    limit,
                } => responses.push(BatchResponse::Scan(scan_state(
                    &self.state,
                    exclusive_after_key.as_ref(),
                    *limit,
                    &mut right_link_corrections,
                )?)),
                BatchRequest::Put { key, value } => {
                    let result = self.transact(TransactionRequest::new(
                        Vec::new(),
                        vec![TransactionMutation::Put {
                            key: key.clone(),
                            value: value.clone(),
                        }],
                    ))?;
                    responses.push(BatchResponse::Put(Revision::from(result.commit_lsn)));
                }
                BatchRequest::Delete { key } => {
                    let result = self.transact(TransactionRequest::new(
                        Vec::new(),
                        vec![TransactionMutation::Delete { key: key.clone() }],
                    ))?;
                    responses.push(BatchResponse::Delete(Revision::from(result.commit_lsn)));
                }
            }
        }
        self.split_metrics.right_link_corrections = self
            .split_metrics
            .right_link_corrections
            .saturating_add(right_link_corrections);
        self.storage_metrics.btree_preparation_nanos = self
            .storage_metrics
            .btree_preparation_nanos
            .saturating_add(elapsed_nanos(started));
        Ok(responses)
    }

    pub fn transact(&mut self, request: TransactionRequest) -> Result<TransactionResult> {
        self.apply_transaction_group(std::slice::from_ref(&request))?
            .into_iter()
            .next()
            .ok_or_else(|| Error::invariant("Blink transaction group returned no result"))?
    }

    /// Serializes the logical requests in order. Every accepted request gets
    /// its own commit LSN and commit marker, while the physical WAL sync is
    /// shared by the group.
    pub fn apply_transaction_group(
        &mut self,
        requests: &[TransactionRequest],
    ) -> Result<Vec<Result<TransactionResult>>> {
        if self.planned_execution {
            self.apply_planned_transaction_group(requests)
        } else {
            self.apply_serial_transaction_group(requests)
        }
    }

    fn apply_serial_transaction_group(
        &mut self,
        requests: &[TransactionRequest],
    ) -> Result<Vec<Result<TransactionResult>>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        if self.broken.is_some() {
            return Err(Error::durability(
                "experimental storage shard is degraded after an uncertain write",
            ));
        }
        let started = Instant::now();
        let mut working = self.state.clone();
        self.batch_metrics.full_state_clones =
            self.batch_metrics.full_state_clones.saturating_add(1);
        // A free page is reusable only when no reader can still reference the
        // generation that contains its previous contents. Otherwise the
        // allocator conservatively grows the file and leaves the ID retired.
        working.allow_page_reuse = self.publisher.can_reuse_pages();
        let mut working_sb = self.current_superblock.clone();
        let mut working_slot = self.active_slot;
        let mut next_lsn = self.wal.as_ref().map_or(self.next_lsn, WalLog::next_lsn);
        let mut next_batch = self
            .wal
            .as_ref()
            .map_or(self.next_batch_id, WalLog::next_batch_id);
        let mut next_revision = self.next_revision;
        let mut candidates = Vec::new();
        let mut results = Vec::with_capacity(requests.len());

        for request in requests {
            let validation_started = Instant::now();
            let candidate_state = working.clone();
            self.batch_metrics.full_state_clones =
                self.batch_metrics.full_state_clones.saturating_add(1);
            if let Err(error) = request.validate() {
                if matches!(error, Error::InvalidInput(_) | Error::InvalidRequest(_)) {
                    results.push(Err(error));
                    continue;
                }
                return Err(error);
            }
            if let Err(error) = validate_request_values(request, &self.config.limits) {
                results.push(Err(error));
                continue;
            }
            if let Err(error) = validate_conditions(&candidate_state, &request.conditions) {
                results.push(Err(error));
                continue;
            }
            self.storage_metrics.validation_nanos = self
                .storage_metrics
                .validation_nanos
                .saturating_add(elapsed_nanos(validation_started));

            let provisional = next_revision;
            next_revision = Revision::new(
                next_revision
                    .get()
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("experimental revision exhausted"))?,
            );
            let mut dirty = BTreeSet::new();
            let mutated_keys = request
                .mutations
                .iter()
                .map(|mutation| mutation.key().encode())
                .collect::<BTreeSet<_>>();
            let mut candidate = candidate_state;
            for mutation in &request.mutations {
                apply_mutation(
                    &mut candidate,
                    &mut dirty,
                    &mut self.split_metrics,
                    mutation,
                    provisional,
                )?;
            }
            let commit_lsn = Lsn::new(
                next_lsn
                    .get()
                    .checked_add(
                        u64::try_from(dirty.len() + 1)
                            .map_err(|_| Error::invariant("Blink page count overflows LSN"))?,
                    )
                    .ok_or_else(|| Error::invariant("experimental LSN exhausted"))?,
            );
            for page_id in &dirty {
                Arc::make_mut(
                    candidate
                        .pages
                        .get_mut(page_id)
                        .ok_or_else(|| Error::invariant("dirty experimental page disappeared"))?,
                )
                .restamp(provisional, commit_lsn, &mutated_keys);
            }
            working_sb = BlinkSuperblock {
                generation: working_sb
                    .generation
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("experimental generation exhausted"))?,
                root_page_id: candidate.root_page_id,
                free_list_head: candidate.free_list_head,
                high_water_page_id: candidate.high_water_page_id,
                ..working_sb
            };
            working_slot = match working_slot {
                SuperblockSlot::A => SuperblockSlot::B,
                SuperblockSlot::B => SuperblockSlot::A,
            };
            let result = TransactionResult { commit_lsn };
            candidates.push(Candidate {
                state: {
                    self.batch_metrics.full_state_clones =
                        self.batch_metrics.full_state_clones.saturating_add(1);
                    candidate.clone()
                },
                dirty,
                result,
                superblock: working_sb.clone(),
                slot: working_slot,
                next_revision,
                next_lsn: Lsn::new(
                    commit_lsn
                        .get()
                        .checked_add(1)
                        .ok_or_else(|| Error::invariant("experimental LSN exhausted"))?,
                ),
                next_batch_id: next_batch
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("experimental batch id exhausted"))?,
            });
            results.push(Ok(result));
            working = candidate;
            next_lsn = Lsn::new(
                commit_lsn
                    .get()
                    .checked_add(1)
                    .ok_or_else(|| Error::invariant("experimental LSN exhausted"))?,
            );
            next_batch = next_batch
                .checked_add(1)
                .ok_or_else(|| Error::invariant("experimental batch id exhausted"))?;
        }
        if candidates.is_empty() {
            return Ok(results);
        }

        let final_candidate = candidates.last().unwrap();
        // Build the immutable catalog before WAL append. It is not visible
        // until the durable append/sync below succeeds.
        let all_dirty = candidates
            .iter()
            .flat_map(|candidate| candidate.dirty.iter().copied())
            .collect::<BTreeSet<_>>();
        let (published_generation, _) = self.publisher.prepare(
            &final_candidate.state,
            &final_candidate.superblock,
            &all_dirty,
        )?;

        let mut wal_commits = Vec::with_capacity(candidates.len());
        let mut final_images = BTreeMap::new();
        for candidate in &candidates {
            let mut images = Vec::with_capacity(candidate.dirty.len() + 1);
            for page_id in &candidate.dirty {
                let page = candidate
                    .state
                    .pages
                    .get(page_id)
                    .ok_or_else(|| Error::invariant("candidate page is missing"))?;
                let image = encode_blink_page(*page_id, page)?;
                images.push(WalPageImage {
                    page_id: *page_id,
                    image,
                });
            }
            let sb_image = encode_blink_superblock(&candidate.superblock)?;
            images.push(WalPageImage {
                page_id: match candidate.slot {
                    SuperblockSlot::A => PageId::ZERO,
                    SuperblockSlot::B => PageId::new(1),
                },
                image: sb_image,
            });
            if self.wal.is_some() {
                wal_commits.push(WalCommit {
                    batch_id: next_batch - (candidates.len() as u64) + wal_commits.len() as u64,
                    commit_lsn: candidate.result.commit_lsn,
                    pages: images.clone(),
                });
            }
            for image in images {
                final_images.insert(image.page_id, image.image);
            }
        }
        if let Some(wal) = self.wal.as_mut() {
            if let Err(error) = wal.append_group(&wal_commits, self.fault_injector.as_deref_mut()) {
                self.broken = Some(error.to_string());
                return Err(error);
            }
        } else {
            for (page_id, image) in &final_images {
                write_all_at(&mut self.file, page_id.get() * PAGE_SIZE as u64, image)?;
            }
            self.file.sync_data()?;
        }
        let final_sb = encode_blink_superblock(&final_candidate.superblock)?;
        self.state = final_candidate.state.clone();
        self.current_superblock = final_candidate.superblock.clone();
        self.active_slot = final_candidate.slot;
        self.next_revision = final_candidate.next_revision;
        self.next_lsn = final_candidate.next_lsn;
        self.next_batch_id = final_candidate.next_batch_id;
        self.publisher.publish(published_generation);
        self.dirty_pages.extend(
            final_images
                .iter()
                .filter(|(id, _)| **id != PageId::ZERO && **id != PageId::new(1))
                .map(|(id, image)| (*id, Arc::new(*image))),
        );
        self.dirty_superblock = Some(final_sb);
        self.split_metrics.pages_touched = self
            .split_metrics
            .pages_touched
            .saturating_add(final_images.len() as u64);
        self.split_metrics.page_images = self
            .split_metrics
            .page_images
            .saturating_add(final_images.len() as u64);
        self.storage_metrics.publication_nanos = self
            .storage_metrics
            .publication_nanos
            .saturating_add(elapsed_nanos(started));
        Ok(results)
    }

    fn apply_planned_transaction_group(
        &mut self,
        requests: &[TransactionRequest],
    ) -> Result<Vec<Result<TransactionResult>>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        if self.broken.is_some() {
            return Err(Error::durability(
                "experimental storage shard is degraded after an uncertain write",
            ));
        }
        self.group_scratch.reset();
        let _group_site = churn::enter(ChurnSite::OtherStorage);

        self.batch_metrics.logical_groups = self.batch_metrics.logical_groups.saturating_add(1);
        self.batch_metrics.logical_transactions = self
            .batch_metrics
            .logical_transactions
            .saturating_add(requests.len() as u64);
        let admission_started = Instant::now();
        let admission_site = churn::enter(ChurnSite::Admission);
        let committed_state = &self.state;
        let mut overlay = LogicalOverlay::new(committed_state);
        let provisional_start = self.next_revision.get();
        let mut admitted = Vec::new();
        let mut results = Vec::with_capacity(requests.len());
        let mut accepted_count = 0u64;
        for (fifo_position, request) in requests.iter().enumerate() {
            if let Err(error) = request.validate() {
                if matches!(error, Error::InvalidInput(_) | Error::InvalidRequest(_)) {
                    self.batch_metrics.rejected_transactions =
                        self.batch_metrics.rejected_transactions.saturating_add(1);
                    results.push(Err(error));
                    continue;
                }
                return Err(error);
            }
            let encoded_mutation_keys =
                match validate_and_encode_mutation_keys(request, &self.config.limits) {
                    Ok(encoded_mutation_keys) => encoded_mutation_keys,
                    Err(error) => {
                        self.batch_metrics.rejected_transactions =
                            self.batch_metrics.rejected_transactions.saturating_add(1);
                        results.push(Err(error));
                        continue;
                    }
                };
            if let Err(error) = overlay.validate_conditions(&request.conditions) {
                if matches!(error, Error::Conflict(_)) {
                    self.batch_metrics.conflicted_transactions =
                        self.batch_metrics.conflicted_transactions.saturating_add(1);
                    results.push(Err(error));
                    continue;
                }
                return Err(error);
            }
            let ordinal = provisional_start
                .checked_add(accepted_count)
                .ok_or_else(|| Error::invariant("experimental revision exhausted"))?;
            let provisional_revision = ProvisionalRevisionToken {
                transaction_position: fifo_position,
                ordinal,
            };
            overlay.accept_preencoded(request, &encoded_mutation_keys, provisional_revision)?;
            admitted.push(AdmittedTransaction {
                fifo_position,
                request,
                encoded_mutation_keys,
                provisional_revision,
            });
            accepted_count = accepted_count.saturating_add(1);
            results.push(Ok(TransactionResult {
                commit_lsn: Lsn::ZERO,
            }));
            self.batch_metrics.admitted_transactions =
                self.batch_metrics.admitted_transactions.saturating_add(1);
        }
        let admission_nanos = elapsed_nanos(admission_started);
        self.batch_metrics.logical_admission_nanos = self
            .batch_metrics
            .logical_admission_nanos
            .saturating_add(admission_nanos);
        self.storage_metrics.validation_nanos = self
            .storage_metrics
            .validation_nanos
            .saturating_add(admission_nanos);
        drop(overlay);
        drop(admission_site);
        if admitted.is_empty() {
            return Ok(results);
        }

        let planning_started = Instant::now();
        let planner_site = churn::enter(ChurnSite::Planner);
        let plan = Arc::new(plan_batch(
            &self.state,
            &admitted,
            &mut self.batch_metrics,
            &self.group_scratch,
        )?);
        drop(planner_site);
        self.batch_metrics.planning_nanos = self
            .batch_metrics
            .planning_nanos
            .saturating_add(elapsed_nanos(planning_started));

        let current_next_lsn = self.wal.as_ref().map_or(self.next_lsn, WalLog::next_lsn);
        let current_next_batch_id = self
            .wal
            .as_ref()
            .map_or(self.next_batch_id, WalLog::next_batch_id);
        let physical_started = Instant::now();
        let parallel_preparation = match self.parallel_worker_pool.as_ref() {
            Some(worker_pool) => prepare_leaf_parallel_execution(
                self.parallel_min_group_mutations,
                &self.state,
                &self.publisher.pin(),
                &plan,
                worker_pool,
                self.wal.as_ref(),
                &self.dirty_pages,
                &self.current_superblock,
                self.active_slot,
                current_next_lsn,
                current_next_batch_id,
                self.publisher.can_reuse_pages(),
                &mut self.batch_metrics,
                &mut self.fault_injector,
                #[cfg(test)]
                self.parallel_worker_fault,
            )?,
            None => None,
        };
        let serial_site = churn::enter(ChurnSite::PackedLeafMutationCow);
        let preparation = match parallel_preparation {
            Some(preparation) => preparation,
            None => prepare_planned_serial_execution(
                &self.state,
                &plan,
                self.publisher.can_reuse_pages(),
                &self.current_superblock,
                self.active_slot,
                current_next_lsn,
                current_next_batch_id,
                &mut self.split_metrics,
                &mut self.batch_metrics,
            )?,
        };
        drop(serial_site);
        let PlannedExecutionPreparation {
            working,
            mut executed,
            parallel_redo,
        } = preparation;
        self.batch_metrics.physical_execution_nanos = self
            .batch_metrics
            .physical_execution_nanos
            .saturating_add(elapsed_nanos(physical_started));

        let final_execution = executed
            .last()
            .ok_or_else(|| Error::invariant("planned execution produced no transaction"))?;
        let dirty_union_started = Instant::now();
        let all_dirty = executed
            .iter()
            .flat_map(|transaction| transaction.dirty.iter().copied())
            .collect::<BTreeSet<_>>();
        self.batch_metrics.dirty_union_nanos = self
            .batch_metrics
            .dirty_union_nanos
            .saturating_add(elapsed_nanos(dirty_union_started));
        let catalog_started = Instant::now();
        let catalog_site = churn::enter(ChurnSite::CatalogPublication);
        if !working
            .pages
            .keys()
            .all(|page_id| all_dirty.contains(page_id))
        {
            return Err(Error::invariant(
                "working overlay contains a non-dirty page",
            ));
        }
        let delta = working.into_delta();
        let (published_generation, prepare_timing) = self.publisher.prepare_shared_delta(
            &delta,
            &self.state,
            &final_execution.superblock,
            &all_dirty,
        )?;
        drop(catalog_site);
        self.batch_metrics.catalog_construction_nanos = self
            .batch_metrics
            .catalog_construction_nanos
            .saturating_add(elapsed_nanos(catalog_started));
        self.batch_metrics.catalog_map_clone_nanos = self
            .batch_metrics
            .catalog_map_clone_nanos
            .saturating_add(prepare_timing.catalog_map_clone_nanos);
        self.batch_metrics.catalog_directory_clone_nanos = self
            .batch_metrics
            .catalog_directory_clone_nanos
            .saturating_add(prepare_timing.catalog_directory_clone_nanos);
        self.batch_metrics.catalog_chunk_clone_nanos = self
            .batch_metrics
            .catalog_chunk_clone_nanos
            .saturating_add(prepare_timing.catalog_chunk_clone_nanos);
        self.batch_metrics.catalog_chunk_clones = self
            .batch_metrics
            .catalog_chunk_clones
            .saturating_add(prepare_timing.catalog_chunk_clones);
        self.batch_metrics.catalog_state_scan_nanos = self
            .batch_metrics
            .catalog_state_scan_nanos
            .saturating_add(prepare_timing.catalog_state_scan_nanos);

        let wal_assembly_started = Instant::now();
        let wal_assembly_site = churn::enter(ChurnSite::WalPreparation);
        let serial_redo_record_count = executed
            .iter()
            .map(|transaction| transaction.images.len() as u64)
            .sum::<u64>();
        let wal_commits = executed
            .iter_mut()
            .map(|transaction| WalCommit {
                batch_id: transaction.batch_id,
                commit_lsn: transaction.result.commit_lsn,
                pages: std::mem::take(&mut transaction.images),
            })
            .collect::<Vec<_>>();
        let final_execution = executed
            .last()
            .ok_or_else(|| Error::invariant("planned execution produced no transaction"))?;
        let mut final_images = BTreeMap::new();
        for commit in &wal_commits {
            for image in &commit.pages {
                final_images.insert(image.page_id, &image.image);
            }
        }
        let mut wal_bytes = 0u64;
        self.batch_metrics.wal_assembly_nanos = self
            .batch_metrics
            .wal_assembly_nanos
            .saturating_add(elapsed_nanos(wal_assembly_started));
        if let Some(parallel_redo) = &parallel_redo {
            let wal = self
                .wal
                .as_mut()
                .ok_or_else(|| Error::invariant("parallel Blink redo needs a WAL"))?;
            let prepared_assembly_started = Instant::now();
            let prepared_commits = executed
                .iter()
                .zip(&parallel_redo.transaction_records)
                .map(|(transaction, records)| PreparedWalCommit {
                    batch_id: transaction.batch_id,
                    commit_lsn: transaction.result.commit_lsn,
                    records: records
                        .iter()
                        .map(|(result_index, boundary_index)| {
                            let result = &parallel_redo.results[*result_index as usize];
                            let boundary = &result.boundaries[*boundary_index as usize];
                            PreparedWalRecord {
                                page_id: result.leaf_id,
                                page_lsn: boundary.commit_lsn,
                                image_crc: boundary.image_crc,
                                redo: match &boundary.redo {
                                    LeafChainRedo::Image(image) => PreparedWalRedo::Image(image),
                                    LeafChainRedo::Delta {
                                        payload,
                                        base_lsn,
                                        base_crc,
                                        spans,
                                        changed_bytes,
                                    } => PreparedWalRedo::Delta {
                                        payload,
                                        base_lsn: *base_lsn,
                                        base_crc: *base_crc,
                                        spans: *spans,
                                        changed_bytes: *changed_bytes,
                                    },
                                },
                            }
                        })
                        .collect(),
                })
                .collect::<Vec<_>>();
            self.batch_metrics.wal_assembly_nanos = self
                .batch_metrics
                .wal_assembly_nanos
                .saturating_add(elapsed_nanos(prepared_assembly_started));
            let _wal_append_site = churn::enter(ChurnSite::WalAppend);
            let reports = match wal
                .append_group_prepared(&prepared_commits, self.fault_injector.as_deref_mut())
            {
                Ok(reports) => reports,
                Err(error) => {
                    self.broken = Some(error.to_string());
                    return Err(error);
                }
            };
            wal_bytes = reports
                .iter()
                .map(|report| report.bytes_written as u64)
                .sum();
        } else if let Some(wal) = self.wal.as_mut() {
            let eligible_commits = executed
                .iter()
                .map(|transaction| !transaction.superblock_image_emitted)
                .collect::<Vec<_>>();
            let dirty_pages = &self.dirty_pages;
            let committed_pages = &self.state.pages;
            let mut base_source = |page_id: PageId| -> Option<Cow<'_, [u8; PAGE_SIZE]>> {
                if let Some(image) = dirty_pages.get(&page_id) {
                    return Some(Cow::Borrowed(&**image));
                }
                committed_pages
                    .get(&page_id)
                    .and_then(|page| encode_blink_page(page_id, page).ok())
                    .map(Cow::Owned)
            };
            let mut delta_request = WalDeltaRequest {
                eligible_commits: &eligible_commits,
                base_source: &mut base_source,
            };
            // Each WAL image above comes from this execution's successful
            // encode_blink_page or encode_blink_superblock call in this process.
            let _wal_append_site = churn::enter(ChurnSite::WalAppend);
            let reports = match wal.append_group_trusted_internal_with_page_deltas(
                &wal_commits,
                &mut delta_request,
                self.fault_injector.as_deref_mut(),
            ) {
                Ok(reports) => reports,
                Err(error) => {
                    self.broken = Some(error.to_string());
                    return Err(error);
                }
            };
            wal_bytes = reports
                .iter()
                .map(|report| report.bytes_written as u64)
                .sum();
        } else {
            for (page_id, image) in &final_images {
                write_all_at(&mut self.file, page_id.get() * PAGE_SIZE as u64, *image)?;
            }
            self.file.sync_data()?;
        }

        drop(wal_assembly_site);
        if let Some(injector) = self.fault_injector.as_deref_mut()
            && let Err(error) = injector.hit("before_generation_publication")
        {
            self.broken = Some(error.to_string());
            return Err(error);
        }
        for transaction in &executed {
            if transaction.superblock_image_emitted {
                self.batch_metrics.superblock_images_emitted = self
                    .batch_metrics
                    .superblock_images_emitted
                    .saturating_add(1);
            } else {
                self.batch_metrics.superblock_images_elided = self
                    .batch_metrics
                    .superblock_images_elided
                    .saturating_add(1);
            }
        }

        let state_install_started = Instant::now();
        let state_install_site = churn::enter(ChurnSite::StateInstall);
        for (page_id, page) in delta.pages {
            self.state.pages.insert(page_id, page);
        }
        self.state.root_page_id = delta.root_page_id;
        self.state.free_list_head = delta.free_list_head;
        self.state.high_water_page_id = delta.high_water_page_id;
        self.state.allow_page_reuse = delta.allow_page_reuse;
        self.current_superblock = final_execution.superblock.clone();
        self.active_slot = final_execution.slot;
        self.next_revision = final_execution.next_revision;
        self.next_lsn = final_execution.next_lsn;
        self.next_batch_id = final_execution.next_batch_id;
        drop(state_install_site);
        self.batch_metrics.state_install_nanos = self
            .batch_metrics
            .state_install_nanos
            .saturating_add(elapsed_nanos(state_install_started));
        if admitted.len() == executed.len() {
            for (admitted_transaction, transaction) in admitted.iter().zip(&executed) {
                let leaf_pages = transaction
                    .dirty
                    .iter()
                    .filter(|page_id| {
                        matches!(self.state.page_ref(**page_id), Some(BlinkPage::Leaf { .. }))
                    })
                    .count();
                record_histogram(
                    &mut self.batch_metrics.transaction_mutation_histogram,
                    admitted_transaction.request.mutations.len(),
                );
                record_histogram(
                    &mut self.batch_metrics.transaction_dirty_page_histogram,
                    transaction.dirty.len(),
                );
                record_histogram(
                    &mut self.batch_metrics.transaction_leaf_page_histogram,
                    leaf_pages,
                );
                if transaction.superblock_image_emitted {
                    self.batch_metrics.structural_transactions += 1;
                }
            }
        }
        let publication_started = Instant::now();
        let publication_site = churn::enter(ChurnSite::CatalogPublication);
        let publish_timing = self.publisher.publish(published_generation);
        drop(publication_site);
        self.batch_metrics.generation_publication_nanos = self
            .batch_metrics
            .generation_publication_nanos
            .saturating_add(elapsed_nanos(publication_started));
        self.batch_metrics.publication_swap_nanos = self
            .batch_metrics
            .publication_swap_nanos
            .saturating_add(publish_timing.swap_nanos);
        self.batch_metrics.retired_generation_drop_nanos = self
            .batch_metrics
            .retired_generation_drop_nanos
            .saturating_add(publish_timing.retired_generation_drop_nanos);
        let final_page_count = final_images.len() as u64
            + parallel_redo
                .as_ref()
                .map_or(0, |parallel_redo| parallel_redo.results.len() as u64);
        let redo_record_count = serial_redo_record_count
            + parallel_redo.as_ref().map_or(0, |parallel_redo| {
                parallel_redo
                    .transaction_records
                    .iter()
                    .map(|records| records.len() as u64)
                    .sum::<u64>()
            });
        let dirty_tracking_started = Instant::now();
        let dirty_tracking_site = churn::enter(ChurnSite::DirtyTracking);
        if let Some(parallel_redo) = parallel_redo {
            for result in parallel_redo.results {
                count_dirty_insert(&self.dirty_pages, result.leaf_id, false);
                self.dirty_pages.insert(result.leaf_id, result.final_image);
            }
        }
        for (page_id, image) in &final_images {
            if *page_id != PageId::ZERO && *page_id != PageId::new(1) {
                count_dirty_insert(&self.dirty_pages, *page_id, true);
                self.dirty_pages.insert(*page_id, Arc::new(**image));
            }
        }
        if let Some(image) = final_images.get(&match final_execution.slot {
            SuperblockSlot::A => PageId::ZERO,
            SuperblockSlot::B => PageId::new(1),
        }) {
            count_image_copy();
            self.dirty_superblock = Some(**image);
        }
        drop(dirty_tracking_site);
        self.batch_metrics.dirty_tracking_nanos = self
            .batch_metrics
            .dirty_tracking_nanos
            .saturating_add(elapsed_nanos(dirty_tracking_started));
        self.split_metrics.pages_touched = self
            .split_metrics
            .pages_touched
            .saturating_add(final_page_count);
        self.split_metrics.page_images = self
            .split_metrics
            .page_images
            .saturating_add(final_page_count);
        self.batch_metrics.page_images = self
            .batch_metrics
            .page_images
            .saturating_add(redo_record_count);
        self.batch_metrics.wal_bytes = self.batch_metrics.wal_bytes.saturating_add(wal_bytes);
        self.storage_metrics.btree_preparation_nanos = self
            .storage_metrics
            .btree_preparation_nanos
            .saturating_add(elapsed_nanos(admission_started));
        for (planned, transaction) in plan.transactions.iter().zip(&executed) {
            results[planned.fifo_position] = Ok(transaction.result);
        }
        self.storage_metrics.publication_nanos = self
            .storage_metrics
            .publication_nanos
            .saturating_add(elapsed_nanos(physical_started));
        Ok(results)
    }

    pub fn flush(&mut self) -> Result<()> {
        self.flush_dirty(false)
    }

    fn hit_checkpoint_fault(&mut self, checkpoint: bool, point: &str) -> Result<()> {
        if checkpoint && let Some(injector) = self.fault_injector.as_deref_mut() {
            injector.hit(point)?;
        }
        Ok(())
    }

    fn flush_dirty(&mut self, checkpoint: bool) -> Result<()> {
        if self.wal.is_none() {
            return self.file.sync_data();
        }
        self.hit_checkpoint_fault(checkpoint, "before_checkpoint_data_flush")?;
        let mut bytes = 0u64;
        for (page_id, image) in std::mem::take(&mut self.dirty_pages) {
            self.hit_checkpoint_fault(checkpoint, "during_checkpoint_page_write")?;
            write_all_at(&mut self.file, page_id.get() * PAGE_SIZE as u64, &image[..])?;
            bytes = bytes.saturating_add(PAGE_SIZE as u64);
        }
        if let Some(image) = self.dirty_superblock.take() {
            let slot = match self.active_slot {
                SuperblockSlot::A => 0,
                SuperblockSlot::B => PAGE_SIZE as u64,
            };
            write_all_at(&mut self.file, slot, &image)?;
            bytes = bytes.saturating_add(PAGE_SIZE as u64);
        }
        if bytes > 0 {
            self.hit_checkpoint_fault(checkpoint, "before_checkpoint_data_sync")?;
            self.file.sync_data()?;
            self.hit_checkpoint_fault(checkpoint, "after_checkpoint_data_sync")?;
        }
        Ok(())
    }

    pub fn checkpoint(&mut self) -> Result<BlinkCheckpointReport> {
        if let Some(message) = &self.broken {
            return Err(Error::checkpoint(format!(
                "experimental storage shard is degraded: {message}"
            )));
        }
        let result = self.checkpoint_inner();
        if let Err(error) = &result {
            self.broken = Some(error.to_string());
        }
        result
    }

    fn checkpoint_inner(&mut self) -> Result<BlinkCheckpointReport> {
        let started = Instant::now();
        let before = self.wal_metrics()?.map_or(0, |metrics| metrics.wal_bytes);
        let checkpoint_lsn = self
            .wal
            .as_ref()
            .and_then(WalLog::last_commit_lsn)
            .unwrap_or(self.current_superblock.checkpoint_lsn);
        self.flush_dirty(true)?;
        if checkpoint_lsn > self.current_superblock.checkpoint_lsn {
            let sb = BlinkSuperblock {
                generation: self.current_superblock.generation + 1,
                checkpoint_lsn,
                ..self.current_superblock.clone()
            };
            let slot = match self.active_slot {
                SuperblockSlot::A => SuperblockSlot::B,
                SuperblockSlot::B => SuperblockSlot::A,
            };
            let image = encode_blink_superblock(&sb)?;
            self.hit_checkpoint_fault(true, "before_checkpoint_superblock_write")?;
            write_all_at(
                &mut self.file,
                match slot {
                    SuperblockSlot::A => 0,
                    SuperblockSlot::B => PAGE_SIZE as u64,
                },
                &image,
            )?;
            self.hit_checkpoint_fault(true, "after_checkpoint_superblock_write")?;
            self.hit_checkpoint_fault(true, "before_checkpoint_metadata_sync")?;
            self.file.sync_data()?;
            self.hit_checkpoint_fault(true, "after_checkpoint_metadata_sync")?;
            self.current_superblock = sb;
            self.active_slot = slot;
            if let Some(wal) = self.wal.as_mut() {
                wal.reset(checkpoint_lsn, self.fault_injector.as_deref_mut())?;
            }
        }
        let after = self.wal_metrics()?.map_or(0, |metrics| metrics.wal_bytes);
        self.check_invariants()?;
        Ok(BlinkCheckpointReport {
            checkpoint_lsn,
            pages_flushed: self.state.pages.len(),
            bytes_written: 0,
            wal_bytes_reclaimed: before.saturating_sub(after),
            duration_nanos: elapsed_nanos(started),
        })
    }

    pub fn check_invariants(&self) -> Result<InvariantReport> {
        check_state(&self.state)
    }

    pub fn into_files(self) -> (F, Option<W>) {
        (self.file, self.wal.map(WalLog::into_file))
    }
}

fn validate_request_values(request: &TransactionRequest, limits: &StorageLimits) -> Result<()> {
    for mutation in &request.mutations {
        let key = mutation.key().encode();
        validate_encoded_key(&key)?;
        if let TransactionMutation::Put { value, .. } = mutation
            && value.len() > limits.max_value_size
        {
            return Err(Error::invalid_input("value exceeds Blink maximum"));
        }
    }
    Ok(())
}

fn validate_and_encode_mutation_keys(
    request: &TransactionRequest,
    limits: &StorageLimits,
) -> Result<Vec<Vec<u8>>> {
    let mut encoded_keys = Vec::with_capacity(request.mutations.len());
    for mutation in &request.mutations {
        let encoded_key = mutation.key().encode();
        validate_encoded_key(&encoded_key)?;
        if let TransactionMutation::Put { value, .. } = mutation
            && value.len() > limits.max_value_size
        {
            return Err(Error::invalid_input("value exceeds Blink maximum"));
        }
        encoded_keys.push(encoded_key);
    }
    Ok(encoded_keys)
}

fn validate_encoded_key(key: &[u8]) -> Result<()> {
    if key.len() > crate::MAX_ENCODED_KEY_SIZE {
        return Err(Error::invalid_input("encoded document key is too large"));
    }
    DocumentKey::validate_encoded(key)
        .map_err(|error| Error::invalid_input(format!("document key is not canonical: {error}")))
}

trait ReadPageSource {
    fn root_page_id(&self) -> PageId;
    fn page(&self, page_id: PageId) -> Result<Arc<BlinkPage>>;
}

impl ReadPageSource for BlinkState {
    fn root_page_id(&self) -> PageId {
        self.root_page_id
    }

    fn page(&self, page_id: PageId) -> Result<Arc<BlinkPage>> {
        self.pages
            .get(&page_id)
            .map(Arc::clone)
            .ok_or_else(|| Error::corruption("Blink page is missing"))
    }
}

impl ReadPageSource for GenerationPin {
    fn root_page_id(&self) -> PageId {
        self.generation.root_page_id
    }

    fn page(&self, page_id: PageId) -> Result<Arc<BlinkPage>> {
        if page_id > self.generation.high_water_page_id {
            return Err(Error::corruption(
                "published Blink page exceeds high-water mark",
            ));
        }
        self.generation
            .catalog
            .get(page_id)
            .ok_or_else(|| Error::corruption("published Blink page is missing"))?
            .page_at(self.generation.epoch)
    }
}

impl<'a> LogicalOverlay<'a> {
    fn new(committed: &'a BlinkState) -> Self {
        Self {
            committed,
            entries: BTreeMap::new(),
        }
    }

    fn observed_state(&self, key: &DocumentKey) -> Result<ObservedState> {
        let encoded = key.encode();
        validate_encoded_key(&encoded)?;
        if let Some(entry) = self.entries.get(&encoded) {
            let LogicalRevision::Provisional(token) = entry.revision;
            debug_assert_eq!(
                entry.originating_transaction_position,
                token.transaction_position
            );
            let revision = Revision::new(token.ordinal);
            return Ok(if entry.present {
                ObservedState::present(revision)
            } else {
                ObservedState::missing(revision)
            });
        }
        observed_state(self.committed, key)
    }

    fn validate_conditions(&self, conditions: &[TransactionCondition]) -> Result<()> {
        for condition in conditions {
            let actual = self.observed_state(condition.key())?;
            let matches = match condition {
                TransactionCondition::RevisionEquals {
                    expected_revision, ..
                } => actual.revision() == *expected_revision,
                TransactionCondition::Exists { .. } => !actual.is_missing(),
                TransactionCondition::NotExists { .. } => actual.is_missing(),
            };
            if !matches {
                return Err(Error::conflict(TransactionConflict {
                    key: condition.key().clone(),
                    expected: condition.expectation(),
                    actual,
                }));
            }
        }
        Ok(())
    }

    fn accept_preencoded(
        &mut self,
        request: &TransactionRequest,
        encoded_mutation_keys: &[Vec<u8>],
        token: ProvisionalRevisionToken,
    ) -> Result<()> {
        if request.mutations.len() != encoded_mutation_keys.len() {
            return Err(Error::invariant(
                "prepared mutation key count does not match request mutations",
            ));
        }
        for (mutation, encoded_key) in request.mutations.iter().zip(encoded_mutation_keys) {
            let present = matches!(mutation, TransactionMutation::Put { .. });
            self.entries.insert(
                encoded_key.clone(),
                LogicalEntry {
                    present,
                    revision: LogicalRevision::Provisional(token),
                    originating_transaction_position: token.transaction_position,
                },
            );
        }
        Ok(())
    }
}

fn validate_conditions<S: ReadPageSource>(
    state: &S,
    conditions: &[TransactionCondition],
) -> Result<()> {
    for condition in conditions {
        let actual = observed_state(state, condition.key())?;
        let matches = match condition {
            TransactionCondition::RevisionEquals {
                expected_revision, ..
            } => actual.revision() == *expected_revision,
            TransactionCondition::Exists { .. } => !actual.is_missing(),
            TransactionCondition::NotExists { .. } => actual.is_missing(),
        };
        if !matches {
            return Err(Error::conflict(TransactionConflict {
                key: condition.key().clone(),
                expected: condition.expectation(),
                actual,
            }));
        }
    }
    Ok(())
}

fn plan_batch(
    state: &BlinkState,
    admitted: &[AdmittedTransaction<'_>],
    metrics: &mut BlinkBatchMetrics,
    group_scratch: &Bump,
) -> Result<BatchPlan> {
    let mutation_count = admitted
        .iter()
        .map(|transaction| transaction.encoded_mutation_keys.len())
        .sum::<usize>();
    let fifo_limit = admitted
        .iter()
        .map(|transaction| transaction.fifo_position + 1)
        .max()
        .unwrap_or(0);
    let mut plan = BatchPlan {
        transactions: Vec::with_capacity(admitted.len()),
        leaf_groups: Vec::with_capacity(mutation_count),
        dependencies: Vec::new(),
    };
    let mut last_key_writer = HashMap::<&[u8], usize>::with_capacity(mutation_count);
    let mut last_leaf_writer = HashMap::<PageId, usize>::with_capacity(mutation_count);
    let mut leaf_group_indices = HashMap::<PageId, usize>::with_capacity(mutation_count);
    let mut transaction_groups = BumpVec::with_capacity_in(fifo_limit, group_scratch);
    for _ in 0..fifo_limit {
        transaction_groups.push(BumpVec::new_in(group_scratch));
    }

    for transaction in admitted {
        let mut dependency_metadata = DependencyMetadata::default();
        let mut mutations = Vec::with_capacity(transaction.request.mutations.len());
        for condition in &transaction.request.conditions {
            let encoded = condition.key().encode();
            validate_encoded_key(&encoded)?;
            if let Some(predecessor) = last_key_writer.get(encoded.as_slice()).copied() {
                dependency_metadata
                    .condition_key_predecessors
                    .push(predecessor);
                plan.dependencies.push(DependencyEdge {
                    predecessor,
                    successor: transaction.fifo_position,
                    kind: DependencyKind::ConditionKey,
                });
            }
        }
        if transaction.request.mutations.len() != transaction.encoded_mutation_keys.len() {
            return Err(Error::invariant(
                "prepared mutation key count does not match request mutations",
            ));
        }
        for (mutation_index, (mutation, encoded_key)) in transaction
            .request
            .mutations
            .iter()
            .zip(&transaction.encoded_mutation_keys)
            .enumerate()
        {
            let mut route_corrections = 0;
            let mut route_page_visits = 0;
            let route_started = Instant::now();
            let leaf_id = find_leaf_in_blink_state_borrowed(
                state,
                encoded_key,
                &mut route_corrections,
                &mut route_page_visits,
            )?;
            metrics.planner_route_nanos = metrics
                .planner_route_nanos
                .saturating_add(elapsed_nanos(route_started));
            metrics.planner_route_calls = metrics.planner_route_calls.saturating_add(1);
            metrics.planner_route_page_visits = metrics
                .planner_route_page_visits
                .saturating_add(route_page_visits);
            metrics.planner_route_right_link_hops = metrics
                .planner_route_right_link_hops
                .saturating_add(route_corrections);
            let route_hint = RouteHint { leaf_id };
            churn::add(ChurnCounter::PlannerKeyCopies, 1);
            mutations.push(PlannedMutation {
                write: match mutation {
                    TransactionMutation::Put { value, .. } => {
                        churn::add(ChurnCounter::PayloadArcsCreated, 1);
                        PlannedWrite::Put(Arc::from(value.as_slice()))
                    }
                    TransactionMutation::Delete { .. } => PlannedWrite::Delete,
                },
                encoded_key: encoded_key.clone(),
                route_hint,
            });
            metrics.routes_calculated = metrics.routes_calculated.saturating_add(1);
            if let Some(predecessor) = last_key_writer.get(encoded_key.as_slice()).copied() {
                dependency_metadata.same_key_predecessors.push(predecessor);
                plan.dependencies.push(DependencyEdge {
                    predecessor,
                    successor: transaction.fifo_position,
                    kind: DependencyKind::SameKey,
                });
            }
            if let Some(predecessor) = last_leaf_writer.get(&leaf_id).copied() {
                dependency_metadata
                    .same_target_page_predecessors
                    .push(predecessor);
                dependency_metadata
                    .structural_route_predecessors
                    .push(predecessor);
                plan.dependencies.push(DependencyEdge {
                    predecessor,
                    successor: transaction.fifo_position,
                    kind: DependencyKind::SameTargetPage,
                });
                plan.dependencies.push(DependencyEdge {
                    predecessor,
                    successor: transaction.fifo_position,
                    kind: DependencyKind::StructuralRoute,
                });
            }
            last_key_writer.insert(encoded_key.as_slice(), transaction.fifo_position);
            last_leaf_writer.insert(leaf_id, transaction.fifo_position);
            let leaf_group_index = *leaf_group_indices.entry(leaf_id).or_insert_with(|| {
                let index = plan.leaf_groups.len();
                plan.leaf_groups.push(LeafGroupPlan {
                    leaf_hint: leaf_id,
                    mutations: Vec::new(),
                });
                index
            });
            plan.leaf_groups[leaf_group_index]
                .mutations
                .push((transaction.fifo_position, mutation_index));
            let groups = &mut transaction_groups[transaction.fifo_position];
            if !groups.contains(&leaf_group_index) {
                groups.push(leaf_group_index);
            }
        }
        let same_transaction_positions = if mutations.len() > 1 {
            vec![transaction.fifo_position; mutations.len()]
        } else {
            Vec::new()
        };
        churn::add(ChurnCounter::PlannerKeyCopies, mutations.len() as u64);
        churn::add(ChurnCounter::PlannerMapInserts, mutations.len() as u64);
        let mutated_key_set = mutations
            .iter()
            .map(|mutation| mutation.encoded_key.clone())
            .collect::<BTreeSet<_>>();
        plan.transactions.push(PhysicalTransactionPlan {
            fifo_position: transaction.fifo_position,
            provisional_revision: transaction.provisional_revision,
            mutations,
            mutated_key_set,
            dependency_metadata: DependencyMetadata {
                same_transaction_positions,
                ..dependency_metadata
            },
        });
    }

    metrics.mutations_planned = metrics.mutations_planned.saturating_add(
        plan.transactions
            .iter()
            .map(|transaction| transaction.mutations.len() as u64)
            .sum(),
    );
    metrics.leaf_groups = metrics
        .leaf_groups
        .saturating_add(plan.leaf_groups.len() as u64);
    metrics.same_leaf_groups = metrics.same_leaf_groups.saturating_add(
        plan.leaf_groups
            .iter()
            .filter(|group| group.mutations.len() > 1)
            .count() as u64,
    );
    metrics.mutations_per_leaf_group = metrics.mutations_per_leaf_group.saturating_add(
        plan.leaf_groups
            .iter()
            .map(|group| group.mutations.len() as u64)
            .sum::<u64>(),
    );
    metrics.route_reuses = metrics.route_reuses.saturating_add(
        plan.leaf_groups
            .iter()
            .map(|group| group.mutations.len().saturating_sub(1) as u64)
            .sum::<u64>(),
    );
    metrics.dependency_edges = metrics
        .dependency_edges
        .saturating_add(plan.dependencies.len() as u64);
    churn::record_arena_alloc(plan.leaf_groups.len() * std::mem::size_of::<bool>());
    let mut dependent_group = BumpVec::with_capacity_in(plan.leaf_groups.len(), group_scratch);
    dependent_group.resize(plan.leaf_groups.len(), false);
    for groups in &transaction_groups {
        if groups.len() > 1 {
            for group in groups {
                dependent_group[*group] = true;
            }
        }
    }
    for dependency in &plan.dependencies {
        let Some(predecessor_groups) = transaction_groups.get(dependency.predecessor) else {
            continue;
        };
        let Some(successor_groups) = transaction_groups.get(dependency.successor) else {
            continue;
        };
        for predecessor_group in predecessor_groups {
            for successor_group in successor_groups {
                if predecessor_group != successor_group {
                    dependent_group[*predecessor_group] = true;
                    dependent_group[*successor_group] = true;
                }
            }
        }
    }
    let dependent_groups = dependent_group
        .iter()
        .filter(|dependent| **dependent)
        .count() as u64;
    metrics.independent_leaf_groups = metrics
        .independent_leaf_groups
        .saturating_add((plan.leaf_groups.len() as u64).saturating_sub(dependent_groups));
    Ok(plan)
}

fn record_parallel_fallback(metrics: &mut BlinkBatchMetrics, reason: ParallelFallbackReason) {
    metrics.parallel_fallback_groups = metrics.parallel_fallback_groups.saturating_add(1);
    match reason {
        ParallelFallbackReason::NoPageDeltaWal => {
            metrics.parallel_fallback_no_delta_wal =
                metrics.parallel_fallback_no_delta_wal.saturating_add(1);
        }
        ParallelFallbackReason::RouteMismatch => {
            metrics.parallel_fallback_route = metrics.parallel_fallback_route.saturating_add(1);
        }
        ParallelFallbackReason::OverflowOrAllocator => {
            metrics.parallel_fallback_overflow =
                metrics.parallel_fallback_overflow.saturating_add(1);
        }
        ParallelFallbackReason::Structural => {
            metrics.parallel_fallback_structural =
                metrics.parallel_fallback_structural.saturating_add(1);
        }
    }
}

/// Worker threads for leaf jobs. The coordinator is one more lane: after it
/// hands the queue to the threads it drains the same queue itself, so
/// `parallel_workers = n` runs n lanes on n - 1 threads plus the coordinator.
struct ParallelWorkerPool {
    workers: Vec<ParallelWorkerSlot>,
}

struct ParallelWorkerSlot {
    sender: Sender<ParallelWorkerCommand>,
    handle: Option<JoinHandle<()>>,
}

struct LeafChainQueue {
    jobs: Mutex<Vec<LeafChainJob>>,
    chunk: usize,
}

impl LeafChainQueue {
    fn take(&self) -> Vec<LeafChainJob> {
        let mut jobs = self
            .jobs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let keep = jobs.len().saturating_sub(self.chunk);
        jobs.split_off(keep)
    }
}

enum ParallelWorkerCommand {
    Execute {
        queue: Arc<LeafChainQueue>,
        results: Sender<ParallelWorkerResult>,
    },
    Shutdown,
}

struct ParallelWorkerResult {
    worker_index: usize,
    thread_id: ThreadId,
    lane: LaneOutput,
}

struct LaneOutput {
    outcomes: Vec<Result<LeafChainOutcome>>,
    panicked: bool,
    busy_nanos: u64,
}

struct ParallelWorkerRun {
    outcomes: Vec<LeafChainOutcome>,
    worker_nanos: u64,
    coordinator_lane_nanos: u64,
    join_nanos: u64,
    lanes: u64,
    worker_threads: Vec<(usize, ThreadId)>,
}

fn drain_leaf_chain_queue(queue: &LeafChainQueue) -> LaneOutput {
    let busy_started = Instant::now();
    let mut outcomes = Vec::new();
    let executed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        loop {
            let jobs = queue.take();
            if jobs.is_empty() {
                break;
            }
            outcomes.extend(jobs.into_iter().map(run_leaf_chain_job));
        }
    }));
    LaneOutput {
        outcomes,
        panicked: executed.is_err(),
        busy_nanos: elapsed_nanos(busy_started),
    }
}

impl ParallelWorkerPool {
    fn new(thread_count: usize) -> Result<Self> {
        let mut workers = Vec::with_capacity(thread_count);
        for worker_index in 0..thread_count {
            let (sender, receiver) = mpsc::channel();
            let handle = match thread::Builder::new()
                .name(format!("dodb-blink-leaf-{worker_index}"))
                .spawn(move || parallel_worker_loop(worker_index, receiver))
            {
                Ok(handle) => handle,
                Err(error) => {
                    shutdown_parallel_workers(&mut workers);
                    return Err(Error::Io(error));
                }
            };
            workers.push(ParallelWorkerSlot {
                sender,
                handle: Some(handle),
            });
        }
        Ok(Self { workers })
    }

    fn execute(&self, jobs: Vec<LeafChainJob>) -> Result<ParallelWorkerRun> {
        let lanes = self.workers.len() + 1;
        let chunk = (jobs.len() / (lanes * 8)).max(1);
        let threads_used = self.workers.len().min(jobs.len().saturating_sub(1));
        let queue = Arc::new(LeafChainQueue {
            jobs: Mutex::new(jobs),
            chunk,
        });
        let (result_sender, result_receiver) = mpsc::channel();
        let started = Instant::now();
        let mut dispatched = 0usize;
        let mut dispatch_error = false;
        for worker in &self.workers[..threads_used] {
            let command = ParallelWorkerCommand::Execute {
                queue: Arc::clone(&queue),
                results: result_sender.clone(),
            };
            if worker.sender.send(command).is_err() {
                dispatch_error = true;
                break;
            }
            dispatched += 1;
        }
        drop(result_sender);
        let coordinator_lane = drain_leaf_chain_queue(&queue);

        let mut outcomes = Vec::new();
        let mut worker_nanos = coordinator_lane.busy_nanos;
        let mut panicked = coordinator_lane.panicked;
        let mut worker_error = None;
        let mut worker_threads = Vec::with_capacity(dispatched);
        let mut received = 0usize;
        let mut lanes_output = vec![coordinator_lane];
        while received < dispatched {
            match result_receiver.recv() {
                Ok(response) => {
                    received += 1;
                    worker_threads.push((response.worker_index, response.thread_id));
                    lanes_output.push(response.lane);
                }
                Err(_) => break,
            }
        }
        let join_nanos = elapsed_nanos(started);
        let coordinator_lane_nanos = lanes_output[0].busy_nanos;
        for (lane_index, lane) in lanes_output.into_iter().enumerate() {
            if lane_index > 0 {
                worker_nanos = worker_nanos.saturating_add(lane.busy_nanos);
                panicked |= lane.panicked;
            }
            for outcome in lane.outcomes {
                match outcome {
                    Ok(outcome) => outcomes.push(outcome),
                    Err(error) => {
                        worker_error.get_or_insert(error);
                    }
                }
            }
        }
        if dispatch_error || received != dispatched {
            return Err(Error::invariant("parallel Blink worker dispatch failed"));
        }
        let distinct_worker_indices = worker_threads
            .iter()
            .map(|(worker_index, _)| *worker_index)
            .collect::<BTreeSet<_>>();
        let distinct_thread_ids = worker_threads
            .iter()
            .map(|(_, thread_id)| *thread_id)
            .collect::<HashSet<_>>();
        if distinct_worker_indices.len() != dispatched || distinct_thread_ids.len() != dispatched {
            return Err(Error::invariant(
                "parallel Blink pool returned duplicate worker identities",
            ));
        }
        if panicked {
            return Err(Error::invariant("parallel Blink leaf worker panicked"));
        }
        if let Some(error) = worker_error {
            return Err(error);
        }
        if !queue
            .jobs
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_empty()
        {
            return Err(Error::invariant(
                "parallel Blink leaf queue was not drained",
            ));
        }
        Ok(ParallelWorkerRun {
            outcomes,
            worker_nanos,
            coordinator_lane_nanos,
            join_nanos,
            lanes: dispatched as u64 + 1,
            worker_threads,
        })
    }
}

impl Drop for ParallelWorkerPool {
    fn drop(&mut self) {
        shutdown_parallel_workers(&mut self.workers);
    }
}

fn shutdown_parallel_workers(workers: &mut [ParallelWorkerSlot]) {
    for worker in workers.iter() {
        let _ = worker.sender.send(ParallelWorkerCommand::Shutdown);
    }
    for worker in workers.iter_mut() {
        if let Some(handle) = worker.handle.take() {
            let _ = handle.join();
        }
    }
}

fn parallel_worker_loop(worker_index: usize, receiver: Receiver<ParallelWorkerCommand>) {
    let _worker_site = churn::enter(ChurnSite::OtherStorage);
    let thread_id = thread::current().id();
    while let Ok(command) = receiver.recv() {
        match command {
            ParallelWorkerCommand::Shutdown => return,
            ParallelWorkerCommand::Execute { queue, results } => {
                let lane = drain_leaf_chain_queue(&queue);
                drop(queue);
                let _ = results.send(ParallelWorkerResult {
                    worker_index,
                    thread_id,
                    lane,
                });
            }
        }
    }
}

/// Leaf-partitioned physical execution of a whole WAL group. It runs only when
/// every transaction is an in-place change of leaves the planner already
/// routed, so each transaction writes exactly one redo record per touched leaf
/// and its commit LSN is known before any worker starts. Any other shape falls
/// back to the serial executor before anything reaches the WAL.
#[allow(clippy::too_many_arguments)]
fn prepare_leaf_parallel_execution<'a, W: DurableFile>(
    min_group_mutations: usize,
    state: &'a BlinkState,
    published: &GenerationPin,
    plan: &Arc<BatchPlan>,
    worker_pool: &ParallelWorkerPool,
    wal: Option<&WalLog<W>>,
    dirty_pages: &BTreeMap<PageId, Arc<[u8; PAGE_SIZE]>>,
    current_superblock: &BlinkSuperblock,
    active_slot: SuperblockSlot,
    starting_lsn: Lsn,
    starting_batch_id: u64,
    allow_page_reuse: bool,
    metrics: &mut BlinkBatchMetrics,
    fault_injector: &mut Option<Box<dyn FaultInjector + Send>>,
    #[cfg(test)] worker_fault: Option<ParallelWorkerFault>,
) -> Result<Option<PlannedExecutionPreparation<'a>>> {
    if plan.leaf_groups.len() <= 1 {
        metrics.parallel_skipped_single_leaf =
            metrics.parallel_skipped_single_leaf.saturating_add(1);
        return Ok(None);
    }
    if plan
        .leaf_groups
        .iter()
        .map(|leaf_group| leaf_group.mutations.len())
        .sum::<usize>()
        < min_group_mutations
    {
        metrics.parallel_skipped_small_group =
            metrics.parallel_skipped_small_group.saturating_add(1);
        return Ok(None);
    }
    let Some(wal) = wal.filter(|wal| wal.page_delta_enabled()) else {
        record_parallel_fallback(metrics, ParallelFallbackReason::NoPageDeltaWal);
        return Ok(None);
    };
    let dispatch_started = Instant::now();
    let dispatch_site = churn::enter(ChurnSite::LeafJobConstruction);
    let transaction_count = plan.transactions.len();
    let fifo_limit = plan
        .transactions
        .iter()
        .map(|transaction| transaction.fifo_position + 1)
        .max()
        .unwrap_or(0);
    let mut transaction_index_by_fifo = vec![u32::MAX; fifo_limit];
    for (transaction_index, transaction) in plan.transactions.iter().enumerate() {
        transaction_index_by_fifo[transaction.fifo_position] = u32::try_from(transaction_index)
            .map_err(|_| Error::invariant("parallel transaction index overflows u32"))?;
    }
    let mut record_counts = vec![0u64; transaction_count];
    let mut job_steps = Vec::with_capacity(plan.leaf_groups.len());
    for leaf_group in &plan.leaf_groups {
        let mut steps = Vec::with_capacity(leaf_group.mutations.len());
        let mut previous_transaction = None;
        for (fifo_position, mutation_index) in &leaf_group.mutations {
            let transaction_index = transaction_index_by_fifo
                .get(*fifo_position)
                .copied()
                .filter(|transaction_index| *transaction_index != u32::MAX)
                .ok_or_else(|| Error::invariant("parallel leaf references unknown transaction"))?;
            if previous_transaction != Some(transaction_index) {
                if previous_transaction.is_some_and(|previous| previous > transaction_index) {
                    return Err(Error::invariant(
                        "parallel leaf chain is not in transaction order",
                    ));
                }
                record_counts[transaction_index as usize] += 1;
                previous_transaction = Some(transaction_index);
            }
            if let PlannedWrite::Put(value) = &plan.transactions[transaction_index as usize]
                .mutations
                .get(*mutation_index)
                .ok_or_else(|| Error::invariant("parallel leaf references unknown mutation"))?
                .write
                && value.len() > INLINE_VALUE_LIMIT
            {
                record_parallel_fallback(metrics, ParallelFallbackReason::OverflowOrAllocator);
                return Ok(None);
            }
            let mutation_index = u32::try_from(*mutation_index)
                .map_err(|_| Error::invariant("parallel mutation index overflows u32"))?;
            steps.push((transaction_index, mutation_index));
        }
        job_steps.push(steps);
    }
    let mut commit_lsns = Vec::with_capacity(transaction_count);
    let mut next_lsn = starting_lsn;
    for record_count in &record_counts {
        if *record_count == 0 {
            return Err(Error::invariant("planned transaction has no target leaf"));
        }
        let commit_lsn = Lsn::new(
            next_lsn
                .get()
                .checked_add(*record_count)
                .ok_or_else(|| Error::invariant("experimental LSN exhausted"))?,
        );
        commit_lsns.push(commit_lsn);
        next_lsn = Lsn::new(
            commit_lsn
                .get()
                .checked_add(1)
                .ok_or_else(|| Error::invariant("experimental LSN exhausted"))?,
        );
    }
    let commit_lsns: Arc<[Lsn]> = commit_lsns.into();

    let mut jobs = Vec::with_capacity(plan.leaf_groups.len());
    for (leaf_group, steps) in plan.leaf_groups.iter().zip(job_steps) {
        let leaf_id = leaf_group.leaf_hint;
        let initial_page = published.page(leaf_id)?;
        if !matches!(*initial_page, BlinkPage::Leaf { .. }) {
            record_parallel_fallback(metrics, ParallelFallbackReason::RouteMismatch);
            return Ok(None);
        }
        #[cfg(debug_assertions)]
        if state.page_ref(leaf_id) != Some(&*initial_page) {
            return Err(Error::invariant(
                "published Blink leaf differs from the committed working state",
            ));
        }
        let chain_entry = wal.page_chain_entry(leaf_id);
        let base_image = if chain_entry.is_some() {
            dirty_pages.get(&leaf_id).map(Arc::clone)
        } else {
            None
        };
        churn::add(ChurnCounter::JobsBuilt, 1);
        jobs.push(LeafChainJob {
            leaf_id,
            initial_page,
            base_image,
            chain_entry,
            steps,
            plan: Arc::clone(plan),
            commit_lsns: Arc::clone(&commit_lsns),
            #[cfg(test)]
            fault: None,
        });
    }
    #[cfg(test)]
    if let Some(
        fault @ (ParallelWorkerFault::Error { leaf_group_index }
        | ParallelWorkerFault::Panic { leaf_group_index }),
    ) = worker_fault
        && let Some(job) = jobs.get_mut(leaf_group_index)
    {
        job.fault = Some(fault);
    }
    metrics.parallel_dispatch_nanos = metrics
        .parallel_dispatch_nanos
        .saturating_add(elapsed_nanos(dispatch_started));
    if let Some(injector) = fault_injector.as_deref_mut() {
        injector.hit("before_parallel_leaf_dispatch")?;
    }

    let worker_run = worker_pool.execute(jobs)?;
    drop(dispatch_site);
    let _collect_site = churn::enter(ChurnSite::JobResultCollection);
    if worker_run.worker_threads.len() as u64 + 1 != worker_run.lanes {
        return Err(Error::invariant(
            "parallel Blink worker dispatch result count is inconsistent",
        ));
    }
    metrics.parallel_join_nanos = metrics
        .parallel_join_nanos
        .saturating_add(worker_run.join_nanos);
    metrics.parallel_worker_slot_nanos = metrics
        .parallel_worker_slot_nanos
        .saturating_add(worker_run.join_nanos.saturating_mul(worker_run.lanes));
    metrics.parallel_worker_dispatches = metrics
        .parallel_worker_dispatches
        .saturating_add(worker_run.lanes);
    metrics.parallel_worker_nanos = metrics
        .parallel_worker_nanos
        .saturating_add(worker_run.worker_nanos);
    metrics.parallel_coordinator_lane_nanos = metrics
        .parallel_coordinator_lane_nanos
        .saturating_add(worker_run.coordinator_lane_nanos);
    if let Some(injector) = fault_injector.as_deref_mut() {
        injector.hit("after_parallel_leaf_join")?;
    }

    let collect_started = Instant::now();
    let mut results = Vec::with_capacity(worker_run.outcomes.len());
    let mut fallback = None;
    for outcome in worker_run.outcomes {
        match outcome {
            LeafChainOutcome::Prepared(result) => results.push(result),
            LeafChainOutcome::Fallback { reason } => {
                fallback.get_or_insert(reason);
            }
        }
    }
    let mut timing = LeafChainTiming::default();
    for result in &results {
        timing.base_nanos = timing.base_nanos.saturating_add(result.timing.base_nanos);
        timing.mutation_nanos = timing
            .mutation_nanos
            .saturating_add(result.timing.mutation_nanos);
        timing.encode_nanos = timing
            .encode_nanos
            .saturating_add(result.timing.encode_nanos);
        timing.delta_nanos = timing.delta_nanos.saturating_add(result.timing.delta_nanos);
    }
    metrics.parallel_worker_base_nanos = metrics
        .parallel_worker_base_nanos
        .saturating_add(timing.base_nanos);
    metrics.parallel_worker_mutation_nanos = metrics
        .parallel_worker_mutation_nanos
        .saturating_add(timing.mutation_nanos);
    metrics.parallel_worker_encode_nanos = metrics
        .parallel_worker_encode_nanos
        .saturating_add(timing.encode_nanos);
    metrics.parallel_worker_delta_nanos = metrics
        .parallel_worker_delta_nanos
        .saturating_add(timing.delta_nanos);
    if let Some(reason) = fallback {
        record_parallel_fallback(metrics, reason);
        metrics.parallel_fallback_after_dispatch =
            metrics.parallel_fallback_after_dispatch.saturating_add(1);
        return Ok(None);
    }
    if results.len() != plan.leaf_groups.len() {
        return Err(Error::invariant(
            "parallel Blink workers returned the wrong number of leaf chains",
        ));
    }

    let mut transaction_records = record_counts
        .iter()
        .map(|record_count| Vec::with_capacity(*record_count as usize))
        .collect::<Vec<Vec<(u32, u32)>>>();
    let mut boundary_count = 0u64;
    for (result_index, result) in results.iter().enumerate() {
        for (boundary_index, boundary) in result.boundaries.iter().enumerate() {
            if commit_lsns.get(boundary.transaction_index) != Some(&boundary.commit_lsn) {
                return Err(Error::invariant(
                    "parallel leaf boundary has the wrong commit LSN",
                ));
            }
            transaction_records[boundary.transaction_index]
                .push((result_index as u32, boundary_index as u32));
            boundary_count += 1;
        }
    }
    for (records, record_count) in transaction_records.iter_mut().zip(&record_counts) {
        records.sort_unstable_by_key(|(result_index, _)| results[*result_index as usize].leaf_id);
        if records.len() as u64 != *record_count
            || records.windows(2).any(|pair| {
                results[pair[0].0 as usize].leaf_id == results[pair[1].0 as usize].leaf_id
            })
        {
            return Err(Error::invariant(
                "parallel transaction does not have one redo record per leaf",
            ));
        }
    }

    let mut working = WorkingBlinkState::new(state, allow_page_reuse);
    for result in &mut results {
        let final_page = std::mem::replace(
            &mut result.final_page,
            BlinkPage::Free {
                lsn: Lsn::ZERO,
                next: None,
            },
        );
        working.insert_page(result.leaf_id, final_page);
    }
    let mut executed = Vec::with_capacity(transaction_count);
    let mut superblock = current_superblock.clone();
    let mut next_batch_id = starting_batch_id;
    for (transaction_index, records) in transaction_records.iter().enumerate() {
        let commit_lsn = commit_lsns[transaction_index];
        superblock = BlinkSuperblock {
            generation: superblock
                .generation
                .checked_add(1)
                .ok_or_else(|| Error::invariant("experimental generation exhausted"))?,
            ..superblock
        };
        let dirty = records
            .iter()
            .map(|(result_index, _)| results[*result_index as usize].leaf_id)
            .collect::<BTreeSet<_>>();
        let next_lsn_after = Lsn::new(
            commit_lsn
                .get()
                .checked_add(1)
                .ok_or_else(|| Error::invariant("experimental LSN exhausted"))?,
        );
        let following_batch_id = next_batch_id
            .checked_add(1)
            .ok_or_else(|| Error::invariant("experimental batch id exhausted"))?;
        executed.push(ExecutedPlanTransaction {
            batch_id: next_batch_id,
            result: TransactionResult { commit_lsn },
            dirty,
            images: Vec::new(),
            superblock: superblock.clone(),
            slot: active_slot,
            superblock_image_emitted: false,
            next_revision: Revision::from(next_lsn_after),
            next_lsn: next_lsn_after,
            next_batch_id: following_batch_id,
        });
        next_batch_id = following_batch_id;
    }
    metrics.parallel_collect_nanos = metrics
        .parallel_collect_nanos
        .saturating_add(elapsed_nanos(collect_started));
    metrics.parallel_groups = metrics.parallel_groups.saturating_add(1);
    metrics.parallel_leaf_jobs = metrics
        .parallel_leaf_jobs
        .saturating_add(results.len() as u64);
    metrics.parallel_job_operations = metrics
        .parallel_job_operations
        .saturating_add(boundary_count);
    metrics.parallel_transactions = metrics
        .parallel_transactions
        .saturating_add(transaction_count as u64);
    metrics.parallel_mutations = metrics.parallel_mutations.saturating_add(
        plan.transactions
            .iter()
            .map(|transaction| transaction.mutations.len() as u64)
            .sum::<u64>(),
    );
    metrics.leaf_encodes = metrics.leaf_encodes.saturating_add(boundary_count);
    Ok(Some(PlannedExecutionPreparation {
        working,
        executed,
        parallel_redo: Some(LeafParallelRedo {
            results,
            transaction_records,
        }),
    }))
}

fn run_leaf_chain_job(job: LeafChainJob) -> Result<LeafChainOutcome> {
    let _lane_site = churn::enter(ChurnSite::LeafLaneExecution);
    #[cfg(test)]
    match job.fault {
        Some(ParallelWorkerFault::Error { .. }) => {
            return Err(Error::invariant("injected parallel leaf worker failure"));
        }
        Some(ParallelWorkerFault::Panic { .. }) => {
            panic!("injected parallel leaf worker panic");
        }
        None => {}
    }
    let mut timing = LeafChainTiming::default();
    let leaf_id = job.leaf_id;
    let base_started = Instant::now();
    if churn::ENABLED
        && let BlinkPage::Leaf { entries, .. } = &*job.initial_page
    {
        churn::record_leaf_sample(
            entries.len(),
            entries.iter().map(|entry| entry.key.len()).sum(),
            entries
                .iter()
                .map(|entry| match &entry.value {
                    Some(BlinkValueRef::Inline(value)) => value.len(),
                    _ => 0,
                })
                .sum(),
        );
    }
    let mut page = BlinkPage::clone(&job.initial_page);
    drop(job.initial_page);
    let mut base = match job.chain_entry {
        None => None,
        Some((chain_lsn, chain_crc)) => {
            let image = match job.base_image {
                Some(image) => image,
                None => {
                    churn::add(ChurnCounter::PageEncodes, 1);
                    churn::add(ChurnCounter::PageImageBuffers, 1);
                    encode_blink_page_arc(leaf_id, &page)?
                }
            };
            let image_crc = crc32c::crc32c(&image[..]);
            if blink_image_lsn(&image) != chain_lsn || image_crc != chain_crc {
                return Err(Error::invariant(format!(
                    "page {leaf_id} parallel delta base does not match the WAL page chain"
                )));
            }
            Some((image, chain_lsn, image_crc))
        }
    };
    timing.base_nanos = elapsed_nanos(base_started);
    let plan = &*job.plan;
    let mut boundaries = Vec::with_capacity(job.steps.len());
    let mut step_position = 0usize;
    while step_position < job.steps.len() {
        let transaction_index = job.steps[step_position].0 as usize;
        let commit_lsn = *job
            .commit_lsns
            .get(transaction_index)
            .ok_or_else(|| Error::invariant("parallel transaction LSN is missing"))?;
        let transaction = plan
            .transactions
            .get(transaction_index)
            .ok_or_else(|| Error::invariant("parallel leaf references unknown transaction"))?;
        let mutation_started = Instant::now();
        while step_position < job.steps.len()
            && job.steps[step_position].0 as usize == transaction_index
        {
            let planned_mutation = transaction
                .mutations
                .get(job.steps[step_position].1 as usize)
                .ok_or_else(|| Error::invariant("parallel leaf references unknown mutation"))?;
            if let Some(reason) =
                apply_leaf_chain_mutation(&mut page, planned_mutation, commit_lsn)?
            {
                return Ok(LeafChainOutcome::Fallback { reason });
            }
            step_position += 1;
        }
        let BlinkPage::Leaf { lsn, .. } = &mut page else {
            return Err(Error::corruption("parallel Blink candidate is not a leaf"));
        };
        *lsn = commit_lsn;
        timing.mutation_nanos = timing
            .mutation_nanos
            .saturating_add(elapsed_nanos(mutation_started));

        let encode_started = Instant::now();
        churn::add(ChurnCounter::PageEncodes, 1);
        churn::add(ChurnCounter::PageImageBuffers, 1);
        let image = encode_blink_page_arc(leaf_id, &page)?;
        timing.encode_nanos = timing
            .encode_nanos
            .saturating_add(elapsed_nanos(encode_started));

        let delta_started = Instant::now();
        let image_crc = crc32c::crc32c(&image[..]);
        let redo = match &base {
            None => LeafChainRedo::Image(Arc::clone(&image)),
            Some((base_image, base_lsn, base_crc)) => {
                let payload = encode_page_delta(leaf_id, base_image, &image)?;
                if payload.len() >= PAGE_IMAGE_PAYLOAD_SIZE {
                    LeafChainRedo::Image(Arc::clone(&image))
                } else {
                    let view = decode_page_delta(&payload)?;
                    if !page_delta_rebuilds(base_image, &view, &image)? {
                        return Err(Error::invariant(format!(
                            "page {leaf_id} parallel delta does not rebuild its after-image"
                        )));
                    }
                    let spans = view.spans.len() as u64;
                    let changed_bytes = view.changed_bytes() as u64;
                    drop(view);
                    LeafChainRedo::Delta {
                        payload,
                        base_lsn: *base_lsn,
                        base_crc: *base_crc,
                        spans,
                        changed_bytes,
                    }
                }
            }
        };
        base = Some((image, commit_lsn, image_crc));
        timing.delta_nanos = timing
            .delta_nanos
            .saturating_add(elapsed_nanos(delta_started));
        boundaries.push(LeafChainBoundary {
            transaction_index,
            commit_lsn,
            image_crc,
            redo,
        });
    }
    let (final_image, _, _) =
        base.ok_or_else(|| Error::invariant("parallel leaf chain produced no image"))?;
    Ok(LeafChainOutcome::Prepared(LeafChainResult {
        leaf_id,
        boundaries,
        final_page: page,
        final_image,
        timing,
    }))
}

fn count_image_copy() {
    if churn::ENABLED {
        churn::add(ChurnCounter::PageImageCopies, 1);
        churn::add(ChurnCounter::PageImageBytesCopied, PAGE_SIZE as u64);
    }
}

fn count_dirty_insert<V>(dirty_pages: &BTreeMap<PageId, V>, page_id: PageId, copied: bool) {
    if churn::ENABLED {
        if dirty_pages.contains_key(&page_id) {
            churn::add(ChurnCounter::DirtyPageReplaces, 1);
        } else {
            churn::add(ChurnCounter::DirtyPageInserts, 1);
        }
        if copied {
            churn::add(ChurnCounter::DirtyPageBytesCopied, PAGE_SIZE as u64);
            count_image_copy();
        }
    }
}

fn blink_image_lsn(image: &[u8; PAGE_SIZE]) -> Lsn {
    let mut lsn_bytes = [0u8; 8];
    lsn_bytes.copy_from_slice(&image[16..24]);
    Lsn::new(u64::from_le_bytes(lsn_bytes))
}

/// Applies one planned mutation in place, with the same entry layout the
/// serial executor produces after its restamp. It never changes anything
/// outside this leaf; anything that would (a split, an overflow value, or a
/// key that does not belong to this leaf) is reported as a fallback reason.
fn apply_leaf_chain_mutation(
    page: &mut BlinkPage,
    planned_mutation: &PlannedMutation,
    commit_lsn: Lsn,
) -> Result<Option<ParallelFallbackReason>> {
    let BlinkPage::Leaf {
        high_key,
        right_sibling,
        entries,
        ..
    } = page
    else {
        return Err(Error::corruption("parallel Blink candidate is not a leaf"));
    };
    let encoded_key = planned_mutation.encoded_key.as_slice();
    if high_key
        .as_deref()
        .is_some_and(|high_key| encoded_key >= high_key)
    {
        return Ok(Some(ParallelFallbackReason::RouteMismatch));
    }
    let value = match &planned_mutation.write {
        PlannedWrite::Put(value) if value.len() <= INLINE_VALUE_LIMIT => {
            Some(BlinkValueRef::Inline(value))
        }
        PlannedWrite::Put(_) => {
            return Ok(Some(ParallelFallbackReason::OverflowOrAllocator));
        }
        PlannedWrite::Delete => None,
    };
    let revision = Revision::from(commit_lsn);
    match entries.search(encoded_key) {
        Ok(entry_index) => {
            if matches!(
                entries.get(entry_index).value,
                Some(BlinkValueRef::Overflow { .. })
            ) {
                return Ok(Some(ParallelFallbackReason::OverflowOrAllocator));
            }
            entries.replace(entry_index, encoded_key, revision, value);
        }
        Err(entry_index) => entries.insert(
            entry_index,
            LeafEntryRef {
                key: encoded_key,
                revision,
                value,
            },
        ),
    }
    if !leaf_fits(entries.all(), high_key.as_deref(), *right_sibling) {
        return Ok(Some(ParallelFallbackReason::Structural));
    }
    Ok(None)
}

fn prepare_planned_serial_execution<'a>(
    state: &'a BlinkState,
    plan: &BatchPlan,
    allow_page_reuse: bool,
    current_superblock: &BlinkSuperblock,
    active_slot: SuperblockSlot,
    starting_lsn: Lsn,
    starting_batch_id: u64,
    split_metrics: &mut BlinkSplitMetrics,
    batch_metrics: &mut BlinkBatchMetrics,
) -> Result<PlannedExecutionPreparation<'a>> {
    let mut working = WorkingBlinkState::new(state, allow_page_reuse);
    let mut working_superblock = current_superblock.clone();
    let mut working_slot = active_slot;
    let mut next_lsn = starting_lsn;
    let mut next_batch_id = starting_batch_id;
    let mut execution_state = PhysicalExecutionState { cached_leaf: None };
    let mut executed = Vec::with_capacity(plan.transactions.len());
    for transaction_plan in &plan.transactions {
        let mut dirty = BTreeSet::new();
        let mutation_started = Instant::now();
        for planned_mutation in &transaction_plan.mutations {
            apply_planned_mutation(
                &mut working,
                &mut dirty,
                split_metrics,
                batch_metrics,
                &mut execution_state,
                planned_mutation,
                Revision::new(transaction_plan.provisional_revision.ordinal),
            )?;
        }
        batch_metrics.physical_mutation_nanos = batch_metrics
            .physical_mutation_nanos
            .saturating_add(elapsed_nanos(mutation_started));
        let previous_root_page_id = working_superblock.root_page_id;
        let previous_free_list_head = working_superblock.free_list_head;
        let previous_high_water_page_id = working_superblock.high_water_page_id;
        working_superblock = BlinkSuperblock {
            generation: working_superblock
                .generation
                .checked_add(1)
                .ok_or_else(|| Error::invariant("experimental generation exhausted"))?,
            root_page_id: working.root_page_id(),
            free_list_head: working.free_list_head(),
            high_water_page_id: working.high_water_page_id(),
            ..working_superblock
        };
        let metadata_changed = working_superblock.root_page_id != previous_root_page_id
            || working_superblock.free_list_head != previous_free_list_head
            || working_superblock.high_water_page_id != previous_high_water_page_id;
        if metadata_changed {
            working_slot = match working_slot {
                SuperblockSlot::A => SuperblockSlot::B,
                SuperblockSlot::B => SuperblockSlot::A,
            };
        }
        let commit_image_count = dirty.len() + usize::from(metadata_changed);
        let commit_lsn = Lsn::new(
            next_lsn
                .get()
                .checked_add(
                    u64::try_from(commit_image_count)
                        .map_err(|_| Error::invariant("Blink page count overflows LSN"))?,
                )
                .ok_or_else(|| Error::invariant("experimental LSN exhausted"))?,
        );
        let restamp_started = Instant::now();
        for page_id in &dirty {
            working
                .overlay_page_mut(*page_id)
                .ok_or_else(|| Error::invariant("dirty experimental page disappeared"))?
                .restamp(
                    Revision::new(transaction_plan.provisional_revision.ordinal),
                    commit_lsn,
                    &transaction_plan.mutated_key_set,
                );
        }
        batch_metrics.physical_restamp_nanos = batch_metrics
            .physical_restamp_nanos
            .saturating_add(elapsed_nanos(restamp_started));
        let mut images = Vec::with_capacity(commit_image_count);
        let page_encode_started = Instant::now();
        for page_id in &dirty {
            let page = working
                .page(*page_id)
                .ok_or_else(|| Error::invariant("planned page is missing"))?;
            if matches!(page, BlinkPage::Leaf { .. }) {
                batch_metrics.leaf_encodes = batch_metrics.leaf_encodes.saturating_add(1);
            }
            churn::add(ChurnCounter::PageEncodes, 1);
            images.push(WalPageImage {
                page_id: *page_id,
                image: encode_blink_page(*page_id, page)?,
            });
        }
        batch_metrics.physical_page_encode_nanos = batch_metrics
            .physical_page_encode_nanos
            .saturating_add(elapsed_nanos(page_encode_started));
        if metadata_changed {
            let superblock_encode_started = Instant::now();
            let superblock_image = encode_blink_superblock(&working_superblock)?;
            batch_metrics.physical_superblock_encode_nanos = batch_metrics
                .physical_superblock_encode_nanos
                .saturating_add(elapsed_nanos(superblock_encode_started));
            images.push(WalPageImage {
                page_id: match working_slot {
                    SuperblockSlot::A => PageId::ZERO,
                    SuperblockSlot::B => PageId::new(1),
                },
                image: superblock_image,
            });
        }
        let next_lsn_after = Lsn::new(
            commit_lsn
                .get()
                .checked_add(1)
                .ok_or_else(|| Error::invariant("experimental LSN exhausted"))?,
        );
        executed.push(ExecutedPlanTransaction {
            batch_id: next_batch_id,
            result: TransactionResult { commit_lsn },
            dirty,
            images,
            superblock: working_superblock.clone(),
            slot: working_slot,
            superblock_image_emitted: metadata_changed,
            next_revision: Revision::from(next_lsn_after),
            next_lsn: next_lsn_after,
            next_batch_id: next_batch_id
                .checked_add(1)
                .ok_or_else(|| Error::invariant("experimental batch id exhausted"))?,
        });
        next_lsn = next_lsn_after;
        next_batch_id = next_batch_id
            .checked_add(1)
            .ok_or_else(|| Error::invariant("experimental batch id exhausted"))?;
    }
    Ok(PlannedExecutionPreparation {
        working,
        executed,
        parallel_redo: None,
    })
}

fn apply_planned_mutation(
    state: &mut WorkingBlinkState<'_>,
    dirty: &mut BTreeSet<PageId>,
    split_metrics: &mut BlinkSplitMetrics,
    batch_metrics: &mut BlinkBatchMetrics,
    execution_state: &mut PhysicalExecutionState,
    planned_mutation: &PlannedMutation,
    revision: Revision,
) -> Result<()> {
    let cached_matches = execution_state
        .cached_leaf
        .as_ref()
        .is_some_and(|cached| cached_leaf_contains(state, cached, &planned_mutation.encoded_key));
    if cached_matches {
        batch_metrics.coalesced_mutations = batch_metrics.coalesced_mutations.saturating_add(1);
        let mut cached = execution_state.cached_leaf.take().unwrap();
        let split = apply_cached_leaf_mutation(
            state,
            dirty,
            split_metrics,
            batch_metrics,
            &mut cached,
            planned_mutation,
            revision,
        )?;
        if split {
            batch_metrics.route_invalidations = batch_metrics.route_invalidations.saturating_add(1);
            batch_metrics.split_triggered_reroutes =
                batch_metrics.split_triggered_reroutes.saturating_add(1);
        } else {
            execution_state.cached_leaf = Some(cached);
        }
        return Ok(());
    }

    let mut route_corrections = 0;
    let leaf_id = find_leaf_from_hint(
        state,
        &planned_mutation.encoded_key,
        planned_mutation.route_hint.leaf_id,
        &mut route_corrections,
    )?;
    if leaf_id != planned_mutation.route_hint.leaf_id {
        batch_metrics.reroutes = batch_metrics.reroutes.saturating_add(1);
        batch_metrics.split_triggered_reroutes =
            batch_metrics.split_triggered_reroutes.saturating_add(1);
    }
    let leaf_load_started = Instant::now();
    let cloned = state.ensure_overlay_page(leaf_id)?;
    if cloned {
        batch_metrics.leaf_load_clones = batch_metrics.leaf_load_clones.saturating_add(1);
        batch_metrics.leaf_load_clone_nanos = batch_metrics
            .leaf_load_clone_nanos
            .saturating_add(elapsed_nanos(leaf_load_started));
    }
    let page = state
        .overlay_page_mut(leaf_id)
        .ok_or_else(|| Error::invariant("planned Blink overlay page disappeared"))?;
    if !matches!(page, BlinkPage::Leaf { .. }) {
        return Err(Error::corruption("planned Blink route ended at non-leaf"));
    }
    batch_metrics.leaf_loads = batch_metrics.leaf_loads.saturating_add(1);
    let mut cached = CachedLeaf { leaf_id };
    let split = apply_cached_leaf_mutation(
        state,
        dirty,
        split_metrics,
        batch_metrics,
        &mut cached,
        planned_mutation,
        revision,
    )?;
    if split {
        batch_metrics.route_invalidations = batch_metrics.route_invalidations.saturating_add(1);
        batch_metrics.split_triggered_reroutes =
            batch_metrics.split_triggered_reroutes.saturating_add(1);
    } else {
        execution_state.cached_leaf = Some(cached);
    }
    Ok(())
}

fn cached_leaf_contains(
    state: &WorkingBlinkState<'_>,
    cached: &CachedLeaf,
    encoded_key: &[u8],
) -> bool {
    let Some(page) = state.page(cached.leaf_id) else {
        return false;
    };
    let BlinkPage::Leaf {
        high_key, entries, ..
    } = page
    else {
        return false;
    };
    if high_key
        .as_ref()
        .is_some_and(|high_key| encoded_key >= high_key.as_slice())
    {
        return false;
    }
    entries.first().is_none_or(|entry| encoded_key >= entry.key)
}

fn apply_cached_leaf_mutation(
    state: &mut WorkingBlinkState<'_>,
    dirty: &mut BTreeSet<PageId>,
    split_metrics: &mut BlinkSplitMetrics,
    batch_metrics: &mut BlinkBatchMetrics,
    cached: &mut CachedLeaf,
    planned_mutation: &PlannedMutation,
    revision: Revision,
) -> Result<bool> {
    let encoded = &planned_mutation.encoded_key;
    let value_ref = match &planned_mutation.write {
        PlannedWrite::Put(value) => Some(allocate_value(state, dirty, value)?),
        PlannedWrite::Delete => None,
    };
    let mut old_value = None;
    let mut split_required = false;
    {
        let page = state
            .overlay_page_mut(cached.leaf_id)
            .ok_or_else(|| Error::invariant("cached planned leaf is not overlay-owned"))?;
        let BlinkPage::Leaf {
            lsn,
            high_key,
            right_sibling,
            entries,
        } = page
        else {
            return Err(Error::corruption("cached Blink page is not a leaf"));
        };
        match entries.search(encoded) {
            Ok(entry_index) => {
                old_value = Some(entries.replace(entry_index, encoded, revision, value_ref));
                if !leaf_fits(entries.all(), high_key.as_deref(), *right_sibling) {
                    return Err(Error::invalid_input(
                        "document key and value cannot fit in a Blink leaf",
                    ));
                }
                *lsn = Lsn::new(revision.get());
            }
            Err(entry_index) => {
                entries.insert(
                    entry_index,
                    LeafEntryRef {
                        key: encoded,
                        revision,
                        value: value_ref,
                    },
                );
                split_required = !leaf_fits(entries.all(), high_key.as_deref(), *right_sibling);
                if !split_required {
                    *lsn = Lsn::new(revision.get());
                }
            }
        }
    }
    if let Some(old_value) = old_value {
        free_value(state, dirty, old_value)?;
    }
    dirty.insert(cached.leaf_id);
    if split_required {
        let mut path = Vec::new();
        let routed_leaf = find_leaf_with_path(state, encoded, &mut path, split_metrics)?;
        if routed_leaf != cached.leaf_id {
            batch_metrics.reroutes = batch_metrics.reroutes.saturating_add(1);
            batch_metrics.split_triggered_reroutes =
                batch_metrics.split_triggered_reroutes.saturating_add(1);
            return Err(Error::invariant(
                "planned leaf cache became stale before split",
            ));
        }
        batch_metrics.structural_fallbacks = batch_metrics.structural_fallbacks.saturating_add(1);
        batch_metrics.coalescing_interruptions =
            batch_metrics.coalescing_interruptions.saturating_add(1);
        let (high_key, right_sibling, entries) = {
            let page = state
                .overlay_page_mut(cached.leaf_id)
                .ok_or_else(|| Error::invariant("planned split leaf disappeared"))?;
            let BlinkPage::Leaf {
                high_key,
                right_sibling,
                entries,
                ..
            } = page
            else {
                return Err(Error::corruption("planned split page is not a leaf"));
            };
            (high_key.take(), *right_sibling, std::mem::take(entries))
        };
        split_leaf(
            state,
            dirty,
            split_metrics,
            cached.leaf_id,
            path,
            high_key,
            right_sibling,
            entries,
            revision,
        )?;
        return Ok(true);
    }
    Ok(false)
}

fn find_leaf_from_hint<S: BlinkMutationState>(
    state: &S,
    key: &[u8],
    hint: PageId,
    right_link_corrections: &mut u64,
) -> Result<PageId> {
    let Some(BlinkPage::Leaf { .. }) = state.page(hint) else {
        return find_mutation_leaf_with_metrics(state, key, right_link_corrections);
    };
    let mut page_id = hint;
    let mut visited = HashSet::new();
    loop {
        if !visited.insert(page_id) {
            return Err(Error::corruption("planned Blink route contains a cycle"));
        }
        let page = state
            .page(page_id)
            .ok_or_else(|| Error::corruption("planned Blink route page is missing"))?;
        let BlinkPage::Leaf {
            high_key,
            right_sibling,
            entries,
            ..
        } = page
        else {
            return find_mutation_leaf_with_metrics(state, key, right_link_corrections);
        };
        if entries.first().is_some_and(|entry| key < entry.key) {
            return find_mutation_leaf_with_metrics(state, key, right_link_corrections);
        }
        if high_key
            .as_ref()
            .is_some_and(|high_key| key >= high_key.as_slice())
        {
            let Some(next) = *right_sibling else {
                return find_mutation_leaf_with_metrics(state, key, right_link_corrections);
            };
            *right_link_corrections = right_link_corrections.saturating_add(1);
            page_id = next;
            continue;
        }
        return Ok(page_id);
    }
}

fn find_mutation_leaf_with_metrics<S: BlinkMutationState>(
    state: &S,
    key: &[u8],
    right_link_corrections: &mut u64,
) -> Result<PageId> {
    let mut page_id = state.root_page_id();
    let mut visited = HashSet::new();
    loop {
        if !visited.insert(page_id) {
            return Err(Error::corruption("Blink tree route contains a cycle"));
        }
        let page = state
            .page(page_id)
            .ok_or_else(|| Error::corruption("Blink page is missing"))?;
        let (high_key, right_sibling) = match page {
            BlinkPage::Leaf {
                high_key,
                right_sibling,
                ..
            }
            | BlinkPage::Internal {
                high_key,
                right_sibling,
                ..
            } => (high_key, right_sibling),
            _ => return Err(Error::corruption("Blink route reached non-tree page")),
        };
        if high_key.as_ref().is_some_and(|high| key >= high.as_slice()) {
            let Some(next) = *right_sibling else {
                return Err(Error::corruption("Blink finite fence has no right sibling"));
            };
            *right_link_corrections = right_link_corrections.saturating_add(1);
            page_id = next;
            continue;
        }
        match page {
            BlinkPage::Leaf { .. } => return Ok(page_id),
            BlinkPage::Internal {
                leftmost_child,
                entries,
                ..
            } => {
                let index = entries.partition_point(|entry| key >= entry.key.as_ref());
                page_id = if index == 0 {
                    *leftmost_child
                } else {
                    entries[index - 1].right_child
                };
            }
            _ => unreachable!(),
        }
    }
}

fn observed_state<S: ReadPageSource>(state: &S, key: &DocumentKey) -> Result<ObservedState> {
    let mut corrections = 0;
    let (page, index) = find_entry_with_metrics(state, &key.encode(), &mut corrections)?;
    let BlinkPage::Leaf { entries, .. } = page.as_ref() else {
        return Err(Error::corruption("Blink route ended at non-leaf"));
    };
    match index.map(|index| entries.get(index)) {
        Some(entry) if entry.value.is_some() => Ok(ObservedState::present(entry.revision)),
        Some(entry) => Ok(ObservedState::missing(entry.revision)),
        None => Ok(ObservedState::missing(Revision::ZERO)),
    }
}

fn read_state<S: ReadPageSource>(
    state: &S,
    key: &DocumentKey,
    right_link_corrections: &mut u64,
) -> Result<RevisionState> {
    let encoded = key.encode();
    validate_encoded_key(&encoded)?;
    let (page, index) = find_entry_with_metrics(state, &encoded, right_link_corrections)?;
    let BlinkPage::Leaf { entries, .. } = page.as_ref() else {
        return Err(Error::corruption("Blink route ended at non-leaf"));
    };
    let Some(entry) = index.map(|index| entries.get(index)) else {
        return Ok(RevisionState::missing(Revision::ZERO));
    };
    match entry.value {
        Some(value) => Ok(RevisionState::present(
            materialize_value(state, value)?,
            entry.revision,
        )),
        None => Ok(RevisionState::missing(entry.revision)),
    }
}

fn query_state<S: ReadPageSource>(
    state: &S,
    pk: &PrimaryKey,
    exclusive_after_sk: Option<&SortKey>,
    limit: usize,
    right_link_corrections: &mut u64,
) -> Result<Vec<Document>> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let start = DocumentKey::new(pk.as_bytes().to_vec(), Vec::new());
    let cursor = exclusive_after_sk
        .map(|sk| DocumentKey::new(pk.as_bytes().to_vec(), sk.as_bytes().to_vec()));
    let mut leaf_id = find_leaf_with_metrics(state, &start.encode(), right_link_corrections, None)?;
    let mut first = true;
    let mut visited = HashSet::new();
    let mut output = Vec::new();
    while output.len() < limit {
        if !visited.insert(leaf_id) {
            return Err(Error::corruption("Blink leaf chain cycle during query"));
        }
        let page = state.page(leaf_id)?;
        let BlinkPage::Leaf {
            entries,
            right_sibling,
            ..
        } = page.as_ref()
        else {
            return Err(Error::corruption("Blink query reached non-leaf page"));
        };
        for entry in entries.iter() {
            let key = DocumentKey::decode(entry.key)
                .map_err(|error| Error::corruption(format!("Blink key decode failed: {error}")))?;
            if first && cursor.as_ref().is_some_and(|cursor| key <= cursor.clone()) {
                continue;
            }
            first = false;
            if key.pk != *pk {
                if key.pk > *pk {
                    return Ok(output);
                }
                continue;
            }
            if cursor.as_ref().is_some_and(|cursor| key <= cursor.clone()) {
                continue;
            }
            if let Some(value) = entry.value {
                output.push(Document {
                    key,
                    value: materialize_value(state, value)?,
                    revision: entry.revision,
                });
                if output.len() == limit {
                    return Ok(output);
                }
            }
        }
        let Some(next) = *right_sibling else { break };
        leaf_id = next;
        first = false;
    }
    Ok(output)
}

fn scan_state<S: ReadPageSource>(
    state: &S,
    cursor: Option<&DocumentKey>,
    limit: usize,
    right_link_corrections: &mut u64,
) -> Result<Vec<Document>> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let mut leaf_id = match cursor {
        Some(key) => find_leaf_with_metrics(state, &key.encode(), right_link_corrections, None)?,
        None => leftmost_leaf(state)?,
    };
    let cursor = cursor.map(DocumentKey::encode);
    let mut visited = HashSet::new();
    let mut output = Vec::new();
    loop {
        if !visited.insert(leaf_id) {
            return Err(Error::corruption("Blink leaf chain cycle during scan"));
        }
        let page = state.page(leaf_id)?;
        let BlinkPage::Leaf {
            entries,
            right_sibling,
            ..
        } = page.as_ref()
        else {
            return Err(Error::corruption("Blink scan reached non-leaf page"));
        };
        for entry in entries.iter() {
            if cursor
                .as_ref()
                .is_some_and(|cursor| entry.key <= cursor.as_slice())
            {
                continue;
            }
            let key = DocumentKey::decode(entry.key)
                .map_err(|error| Error::corruption(format!("Blink key decode failed: {error}")))?;
            if let Some(value) = entry.value {
                output.push(Document {
                    key,
                    value: materialize_value(state, value)?,
                    revision: entry.revision,
                });
                if output.len() == limit {
                    return Ok(output);
                }
            }
        }
        let Some(next) = *right_sibling else { break };
        leaf_id = next;
    }
    Ok(output)
}

fn find_entry_with_metrics<S: ReadPageSource>(
    state: &S,
    key: &[u8],
    right_link_corrections: &mut u64,
) -> Result<(Arc<BlinkPage>, Option<usize>)> {
    let leaf_id = find_leaf_with_metrics(state, key, right_link_corrections, None)?;
    let page = state.page(leaf_id)?;
    let BlinkPage::Leaf { entries, .. } = page.as_ref() else {
        return Err(Error::corruption("Blink route ended at non-leaf"));
    };
    let index = entries.iter().position(|entry| entry.key == key);
    Ok((page, index))
}

fn find_leaf_in_blink_state_borrowed(
    state: &BlinkState,
    key: &[u8],
    right_link_corrections: &mut u64,
    page_visits: &mut u64,
) -> Result<PageId> {
    let mut page_id = state.root_page_id;
    let mut guard = HashSet::new();
    loop {
        if !guard.insert(page_id) {
            return Err(Error::corruption("Blink tree route contains a cycle"));
        }
        let page = state
            .page_ref(page_id)
            .ok_or_else(|| Error::corruption("Blink page is missing"))?;
        *page_visits = page_visits.saturating_add(1);
        let (high_key, right_sibling) = match page {
            BlinkPage::Leaf {
                high_key,
                right_sibling,
                ..
            }
            | BlinkPage::Internal {
                high_key,
                right_sibling,
                ..
            } => (high_key, right_sibling),
            _ => return Err(Error::corruption("Blink route reached non-tree page")),
        };
        if high_key.as_ref().is_some_and(|high| key >= high.as_slice()) {
            let Some(next) = *right_sibling else {
                return Err(Error::corruption("Blink finite fence has no right sibling"));
            };
            *right_link_corrections = right_link_corrections.saturating_add(1);
            page_id = next;
            continue;
        }
        match page {
            BlinkPage::Leaf { .. } => return Ok(page_id),
            BlinkPage::Internal {
                leftmost_child,
                entries,
                ..
            } => {
                let index = entries.partition_point(|entry| key >= entry.key.as_ref());
                page_id = if index == 0 {
                    *leftmost_child
                } else {
                    entries[index - 1].right_child
                };
            }
            _ => unreachable!(),
        }
    }
}

fn find_leaf_with_metrics<S: ReadPageSource>(
    state: &S,
    key: &[u8],
    right_link_corrections: &mut u64,
    mut page_visits: Option<&mut u64>,
) -> Result<PageId> {
    let mut page_id = state.root_page_id();
    let mut guard = HashSet::new();
    loop {
        if !guard.insert(page_id) {
            return Err(Error::corruption("Blink tree route contains a cycle"));
        }
        let page = state.page(page_id)?;
        if let Some(page_visits) = page_visits.as_deref_mut() {
            *page_visits = page_visits.saturating_add(1);
        }
        let (high_key, right_sibling) = match page.as_ref() {
            BlinkPage::Leaf {
                high_key,
                right_sibling,
                ..
            }
            | BlinkPage::Internal {
                high_key,
                right_sibling,
                ..
            } => (high_key, right_sibling),
            _ => return Err(Error::corruption("Blink route reached non-tree page")),
        };
        if high_key.as_ref().is_some_and(|high| key >= high.as_slice()) {
            let Some(next) = *right_sibling else {
                return Err(Error::corruption("Blink finite fence has no right sibling"));
            };
            *right_link_corrections = right_link_corrections.saturating_add(1);
            page_id = next;
            continue;
        }
        match page.as_ref() {
            BlinkPage::Leaf { .. } => return Ok(page_id),
            BlinkPage::Internal {
                leftmost_child,
                entries,
                ..
            } => {
                let index = entries.partition_point(|entry| key >= entry.key.as_ref());
                page_id = if index == 0 {
                    *leftmost_child
                } else {
                    entries[index - 1].right_child
                };
            }
            _ => unreachable!(),
        }
    }
}

fn leftmost_leaf<S: ReadPageSource>(state: &S) -> Result<PageId> {
    let mut page_id = state.root_page_id();
    loop {
        match state.page(page_id)?.as_ref() {
            BlinkPage::Leaf { .. } => return Ok(page_id),
            BlinkPage::Internal { leftmost_child, .. } => page_id = *leftmost_child,
            _ => return Err(Error::corruption("Blink leftmost path is invalid")),
        }
    }
}

fn materialize_value<S: ReadPageSource>(state: &S, value: BlinkValueRef<'_>) -> Result<Vec<u8>> {
    match value {
        BlinkValueRef::Inline(value) => Ok(value.to_vec()),
        BlinkValueRef::Overflow { head, length } => {
            let mut output = Vec::with_capacity(length as usize);
            let mut page_id = Some(head);
            let mut visited = HashSet::new();
            while let Some(id) = page_id {
                if !visited.insert(id) {
                    return Err(Error::corruption("Blink overflow cycle"));
                }
                let page = state.page(id)?;
                let BlinkPage::Overflow { next, chunk, .. } = page.as_ref() else {
                    return Err(Error::corruption(
                        "Blink overflow reference has wrong page type",
                    ));
                };
                output.extend_from_slice(chunk);
                page_id = *next;
            }
            output.truncate(length as usize);
            if output.len() != length as usize {
                return Err(Error::corruption("Blink overflow length mismatch"));
            }
            Ok(output)
        }
    }
}

fn apply_mutation<S: BlinkMutationState>(
    state: &mut S,
    dirty: &mut BTreeSet<PageId>,
    metrics: &mut BlinkSplitMetrics,
    mutation: &TransactionMutation,
    revision: Revision,
) -> Result<()> {
    let (key, value) = match mutation {
        TransactionMutation::Put { key, value } => (key, Some(value.as_slice())),
        TransactionMutation::Delete { key } => (key, None),
    };
    let encoded = key.encode();
    validate_encoded_key(&encoded)?;
    let mut path = Vec::new();
    let leaf_id = find_leaf_with_path(state, &encoded, &mut path, metrics)?;
    let BlinkPage::Leaf {
        high_key,
        right_sibling,
        mut entries,
        ..
    } = state
        .page(leaf_id)
        .cloned()
        .ok_or_else(|| Error::corruption("Blink mutation leaf is missing"))?
    else {
        return Err(Error::corruption("Blink mutation route ended at non-leaf"));
    };
    let existing = entries.search(&encoded);
    let value_ref = match value {
        Some(bytes) => Some(allocate_value(state, dirty, bytes)?),
        None => None,
    };
    match existing {
        Ok(index) => {
            let old = entries.replace(index, &encoded, revision, value_ref);
            if !leaf_fits(entries.all(), high_key.as_deref(), right_sibling) {
                return Err(Error::invalid_input(
                    "document key and value cannot fit in a Blink leaf",
                ));
            }
            free_value(state, dirty, old)?;
            state.insert_page(
                leaf_id,
                BlinkPage::Leaf {
                    lsn: Lsn::new(revision.get()),
                    high_key,
                    right_sibling,
                    entries,
                },
            );
            dirty.insert(leaf_id);
        }
        Err(index) => {
            entries.insert(
                index,
                LeafEntryRef {
                    key: &encoded,
                    revision,
                    value: value_ref,
                },
            );
            if leaf_fits(entries.all(), high_key.as_deref(), right_sibling) {
                state.insert_page(
                    leaf_id,
                    BlinkPage::Leaf {
                        lsn: Lsn::new(revision.get()),
                        high_key,
                        right_sibling,
                        entries,
                    },
                );
                dirty.insert(leaf_id);
            } else {
                split_leaf(
                    state,
                    dirty,
                    metrics,
                    leaf_id,
                    path,
                    high_key,
                    right_sibling,
                    entries,
                    revision,
                )?;
            }
        }
    }
    Ok(())
}

fn find_leaf_with_path<S: BlinkMutationState>(
    state: &S,
    key: &[u8],
    path: &mut Vec<PageId>,
    metrics: &mut BlinkSplitMetrics,
) -> Result<PageId> {
    let mut page_id = state.root_page_id();
    let mut guard = HashSet::new();
    loop {
        if !guard.insert(page_id) {
            return Err(Error::corruption("Blink route contains a cycle"));
        }
        let page = state
            .page(page_id)
            .ok_or_else(|| Error::corruption("Blink route page is missing"))?;
        let (high_key, right_sibling) = match page {
            BlinkPage::Leaf {
                high_key,
                right_sibling,
                ..
            }
            | BlinkPage::Internal {
                high_key,
                right_sibling,
                ..
            } => (high_key, right_sibling),
            _ => return Err(Error::corruption("Blink route reached non-tree page")),
        };
        if high_key.as_ref().is_some_and(|high| key >= high.as_slice()) {
            let Some(next) = *right_sibling else {
                return Err(Error::corruption("Blink finite fence has no right sibling"));
            };
            metrics.right_link_corrections = metrics.right_link_corrections.saturating_add(1);
            page_id = next;
            continue;
        }
        match page {
            BlinkPage::Leaf { .. } => return Ok(page_id),
            BlinkPage::Internal {
                leftmost_child,
                entries,
                ..
            } => {
                path.push(page_id);
                let index = entries.partition_point(|entry| key >= entry.key.as_slice());
                page_id = if index == 0 {
                    *leftmost_child
                } else {
                    entries[index - 1].right_child
                };
            }
            _ => unreachable!(),
        }
    }
}

fn allocate_value<'value, S: BlinkMutationState>(
    state: &mut S,
    dirty: &mut BTreeSet<PageId>,
    bytes: &'value [u8],
) -> Result<BlinkValueRef<'value>> {
    if bytes.len() <= INLINE_VALUE_LIMIT {
        return Ok(BlinkValueRef::Inline(bytes));
    }
    if bytes.len() > MAX_VALUE_SIZE {
        return Err(Error::invalid_input("value exceeds Blink maximum"));
    }
    let chunk_size = BODY_SIZE - OVERFLOW_HEADER_SIZE;
    let mut chunks = Vec::new();
    for chunk in bytes.chunks(chunk_size) {
        let id = allocate_page(state, dirty);
        chunks.push((id, chunk.to_vec()));
    }
    for (index, (id, chunk)) in chunks.iter().enumerate() {
        let next = chunks.get(index + 1).map(|(id, _)| *id);
        state.insert_page(
            *id,
            BlinkPage::Overflow {
                lsn: Lsn::ZERO,
                next,
                total_length: bytes.len() as u64,
                chunk: chunk.clone(),
            },
        );
        dirty.insert(*id);
    }
    Ok(BlinkValueRef::Overflow {
        head: chunks
            .first()
            .map(|(id, _)| *id)
            .ok_or_else(|| Error::invariant("overflow allocation produced no pages"))?,
        length: bytes.len() as u64,
    })
}

fn free_value<S: BlinkMutationState>(
    state: &mut S,
    dirty: &mut BTreeSet<PageId>,
    value: Option<StoredValue>,
) -> Result<()> {
    let Some(StoredValue::Overflow { head, .. }) = value else {
        return Ok(());
    };
    let mut page_id = Some(head);
    let mut visited = HashSet::new();
    while let Some(id) = page_id {
        if !visited.insert(id) {
            return Err(Error::corruption("Blink overflow cycle while freeing"));
        }
        let next = match state.page(id) {
            Some(BlinkPage::Overflow { next, .. }) => *next,
            Some(_) => return Err(Error::corruption("Blink overflow free has wrong page type")),
            None => return Err(Error::corruption("Blink overflow free page is missing")),
        };
        state.insert_page(
            id,
            BlinkPage::Free {
                lsn: Lsn::ZERO,
                next: state.free_list_head(),
            },
        );
        state.set_free_list_head(Some(id));
        dirty.insert(id);
        page_id = next;
    }
    Ok(())
}

fn allocate_page<S: BlinkMutationState>(state: &mut S, dirty: &mut BTreeSet<PageId>) -> PageId {
    if state.allow_page_reuse()
        && let Some(id) = state.free_list_head()
    {
        let next = match state.page(id) {
            Some(BlinkPage::Free { next, .. }) => *next,
            _ => None,
        };
        state.set_free_list_head(next);
        dirty.insert(id);
        return id;
    }
    let id = PageId::new(state.high_water_page_id().get() + 1);
    state.set_high_water_page_id(id);
    dirty.insert(id);
    id
}

fn split_leaf<S: BlinkMutationState>(
    state: &mut S,
    dirty: &mut BTreeSet<PageId>,
    metrics: &mut BlinkSplitMetrics,
    leaf_id: PageId,
    path: Vec<PageId>,
    old_high: Option<Vec<u8>>,
    old_right: Option<PageId>,
    entries: LeafEntries,
    revision: Revision,
) -> Result<()> {
    let split = choose_leaf_split(&entries, old_high.as_deref(), old_right);
    let right_id = allocate_page(state, dirty);
    let separator = entries.key(split).to_vec();
    let mut left_entries = entries;
    let right_entries = left_entries.split_off(split);
    state.insert_page(
        leaf_id,
        BlinkPage::Leaf {
            lsn: Lsn::new(revision.get()),
            high_key: Some(separator.clone()),
            right_sibling: Some(right_id),
            entries: left_entries,
        },
    );
    state.insert_page(
        right_id,
        BlinkPage::Leaf {
            lsn: Lsn::new(revision.get()),
            high_key: old_high,
            right_sibling: old_right,
            entries: right_entries,
        },
    );
    dirty.insert(leaf_id);
    dirty.insert(right_id);
    metrics.leaf_splits = metrics.leaf_splits.saturating_add(1);
    install_separator(
        state, dirty, metrics, path, leaf_id, separator, right_id, revision,
    )
}

fn install_separator<S: BlinkMutationState>(
    state: &mut S,
    dirty: &mut BTreeSet<PageId>,
    metrics: &mut BlinkSplitMetrics,
    path: Vec<PageId>,
    left_child: PageId,
    separator: Vec<u8>,
    right_child: PageId,
    revision: Revision,
) -> Result<()> {
    let Some(parent_id) = path.last().copied() else {
        let old_root = state.root_page_id();
        let level = page_level(state, old_root)? + 1;
        let root = allocate_page(state, dirty);
        state.insert_page(
            root,
            BlinkPage::Internal {
                lsn: Lsn::new(revision.get()),
                level,
                high_key: None,
                right_sibling: None,
                leftmost_child: left_child,
                entries: vec![InternalEntry {
                    key: separator,
                    right_child,
                }],
            },
        );
        state.set_root_page_id(root);
        dirty.insert(root);
        metrics.root_splits = metrics.root_splits.saturating_add(1);
        return Ok(());
    };
    let BlinkPage::Internal {
        lsn: _,
        level,
        high_key,
        right_sibling,
        leftmost_child,
        mut entries,
    } = state
        .page(parent_id)
        .cloned()
        .ok_or_else(|| Error::corruption("Blink parent is missing"))?
    else {
        return Err(Error::corruption("Blink separator parent is not internal"));
    };
    let index = if leftmost_child == left_child {
        0
    } else {
        entries
            .iter()
            .position(|entry| entry.right_child == left_child)
            .map(|index| index + 1)
            .ok_or_else(|| Error::corruption("Blink parent does not contain split child"))?
    };
    entries.insert(
        index,
        InternalEntry {
            key: separator,
            right_child,
        },
    );
    if internal_fits(
        leftmost_child,
        &entries,
        high_key.as_deref(),
        right_sibling,
        level,
    ) {
        state.insert_page(
            parent_id,
            BlinkPage::Internal {
                lsn: Lsn::new(revision.get()),
                level,
                high_key,
                right_sibling,
                leftmost_child,
                entries,
            },
        );
        dirty.insert(parent_id);
        return Ok(());
    }
    let (promote, new_right) = split_internal(
        state,
        dirty,
        metrics,
        parent_id,
        level,
        high_key,
        right_sibling,
        leftmost_child,
        entries,
        revision,
    )?;
    let mut ancestor_path = path;
    ancestor_path.pop();
    install_separator(
        state,
        dirty,
        metrics,
        ancestor_path,
        parent_id,
        promote,
        new_right,
        revision,
    )
}

fn split_internal<S: BlinkMutationState>(
    state: &mut S,
    dirty: &mut BTreeSet<PageId>,
    metrics: &mut BlinkSplitMetrics,
    page_id: PageId,
    level: u16,
    old_high: Option<Vec<u8>>,
    old_right: Option<PageId>,
    leftmost_child: PageId,
    entries: Vec<InternalEntry>,
    revision: Revision,
) -> Result<(Vec<u8>, PageId)> {
    if entries.len() < 2 {
        return Err(Error::invalid_input("Blink internal entry cannot fit"));
    }
    let middle = entries.len() / 2;
    let promote = entries[middle].key.clone();
    let new_right = allocate_page(state, dirty);
    let right_leftmost = entries[middle].right_child;
    let left_entries = entries[..middle].to_vec();
    let right_entries = entries[middle + 1..].to_vec();
    if !internal_fits(
        leftmost_child,
        &left_entries,
        Some(&promote),
        Some(new_right),
        level,
    ) || !internal_fits(
        right_leftmost,
        &right_entries,
        old_high.as_deref(),
        old_right,
        level,
    ) {
        return Err(Error::invalid_input(
            "Blink internal split halves do not fit",
        ));
    }
    state.insert_page(
        page_id,
        BlinkPage::Internal {
            lsn: Lsn::new(revision.get()),
            level,
            high_key: Some(promote.clone()),
            right_sibling: Some(new_right),
            leftmost_child,
            entries: left_entries,
        },
    );
    state.insert_page(
        new_right,
        BlinkPage::Internal {
            lsn: Lsn::new(revision.get()),
            level,
            high_key: old_high,
            right_sibling: old_right,
            leftmost_child: right_leftmost,
            entries: right_entries,
        },
    );
    dirty.insert(page_id);
    dirty.insert(new_right);
    metrics.internal_splits = metrics.internal_splits.saturating_add(1);
    Ok((promote, new_right))
}

fn encode_blink_page(page_id: PageId, page: &BlinkPage) -> Result<[u8; PAGE_SIZE]> {
    let mut encoded = [0u8; PAGE_SIZE];
    encode_blink_page_into(page_id, page, &mut encoded)?;
    Ok(encoded)
}

fn encode_blink_page_arc(page_id: PageId, page: &BlinkPage) -> Result<Arc<[u8; PAGE_SIZE]>> {
    let mut encoded = Arc::new([0u8; PAGE_SIZE]);
    encode_blink_page_into(
        page_id,
        page,
        Arc::get_mut(&mut encoded)
            .ok_or_else(|| Error::invariant("fresh Blink page buffer is shared"))?,
    )?;
    Ok(encoded)
}

fn encode_blink_page_into(
    page_id: PageId,
    page: &BlinkPage,
    encoded: &mut [u8; PAGE_SIZE],
) -> Result<()> {
    {
        let body = &mut encoded[PAGE_HEADER_SIZE..];
        match page {
            BlinkPage::Leaf {
                high_key,
                right_sibling,
                entries,
                ..
            } => encode_leaf_body_into(body, high_key.as_deref(), *right_sibling, entries.all())?,
            BlinkPage::Internal {
                level,
                high_key,
                right_sibling,
                leftmost_child,
                entries,
                ..
            } => encode_internal_body_into(
                body,
                *level,
                high_key.as_deref(),
                *right_sibling,
                *leftmost_child,
                entries,
            )?,
            BlinkPage::Overflow {
                next,
                total_length,
                chunk,
                ..
            } => encode_overflow_body_into(body, *next, *total_length, chunk)?,
            BlinkPage::Free { next, .. } => encode_free_body_into(body, *next)?,
        }
    }
    finalize_encoded_page(
        PageHeader::new(page.page_type(), page_id, page.lsn()),
        encoded,
    )
}

#[cfg(test)]
fn encode_blink_page_reference(page_id: PageId, page: &BlinkPage) -> Result<[u8; PAGE_SIZE]> {
    let body = match page {
        BlinkPage::Leaf {
            high_key,
            right_sibling,
            entries,
            ..
        } => encode_leaf_body(
            high_key.as_deref(),
            *right_sibling,
            &entries.iter().collect::<Vec<_>>(),
        )?,
        BlinkPage::Internal {
            level,
            high_key,
            right_sibling,
            leftmost_child,
            entries,
            ..
        } => encode_internal_body(
            *level,
            high_key.as_deref(),
            *right_sibling,
            *leftmost_child,
            entries,
        )?,
        BlinkPage::Overflow {
            next,
            total_length,
            chunk,
            ..
        } => encode_overflow_body(*next, *total_length, chunk)?,
        BlinkPage::Free { next, .. } => encode_free_body(*next)?,
    };
    encode_page(
        PageHeader::new(page.page_type(), page_id, page.lsn()),
        &body,
    )
}

fn decode_blink_page(bytes: &[u8], expected_page_id: PageId) -> Result<BlinkPage> {
    let decoded = decode_page_at(bytes, Some(expected_page_id))?;
    if decoded.header.flags != 0 {
        return Err(Error::corruption("Blink page flags are non-zero"));
    }
    match decoded.header.page_type {
        PageType::Leaf => decode_leaf_body(decoded.header.page_lsn, &decoded.body),
        PageType::Internal => decode_internal_body(decoded.header.page_lsn, &decoded.body),
        PageType::Overflow => decode_overflow_body(decoded.header.page_lsn, &decoded.body),
        PageType::Free => decode_free_body(decoded.header.page_lsn, &decoded.body),
        PageType::Test => Err(Error::corruption("Blink test page is not a database page")),
    }
}

pub(crate) fn validate_blink_page_image(bytes: &[u8], expected_page_id: PageId) -> Result<()> {
    decode_blink_page(bytes, expected_page_id).map(|_| ())
}

fn encode_leaf_body_into(
    body: &mut [u8],
    high_key: Option<&[u8]>,
    right_sibling: Option<PageId>,
    entries: LeafRange<'_>,
) -> Result<()> {
    debug_assert_eq!(body.len(), BODY_SIZE);
    let layout = leaf_body_layout(entries, high_key)?;
    body[0..4].copy_from_slice(&LEAF_MAGIC);
    body[4..6].copy_from_slice(&BODY_VERSION.to_le_bytes());
    body[8..10].copy_from_slice(&(entries.len() as u16).to_le_bytes());
    body[10..12].copy_from_slice(&(layout.slot_end as u16).to_le_bytes());
    body[12..14].copy_from_slice(&(layout.records_start as u16).to_le_bytes());
    let high_offset = if high_key.is_some() {
        layout.slot_end
    } else {
        0
    };
    body[14..16].copy_from_slice(&(high_offset as u16).to_le_bytes());
    body[16..18].copy_from_slice(&(high_key.map_or(0, <[u8]>::len) as u16).to_le_bytes());
    body[20..28].copy_from_slice(&encode_page_id(right_sibling).to_le_bytes());
    if let Some(high_key) = high_key {
        body[layout.slot_end..layout.records_start].copy_from_slice(high_key);
    }

    let mut record_offset = BODY_SIZE;
    for entry_index in (0..entries.len()).rev() {
        let entry = entries.get(entry_index);
        let record_length = leaf_record_encoded_len(entry)?;
        record_offset = record_offset
            .checked_sub(record_length)
            .ok_or_else(|| Error::invalid_input("Blink leaf records exceed page"))?;
        if record_offset < layout.records_start {
            return Err(Error::invalid_input("Blink leaf records exceed page"));
        }
        encode_leaf_record_into_validated(
            &mut body[record_offset..record_offset + record_length],
            entry,
        )?;
        let slot = LEAF_HEADER_SIZE + entry_index * SLOT_SIZE;
        body[slot..slot + 2].copy_from_slice(&(record_offset as u16).to_le_bytes());
        body[slot + 2..slot + 4].copy_from_slice(&(record_length as u16).to_le_bytes());
        body[slot + 4..slot + 6].copy_from_slice(&(entry.key.len() as u16).to_le_bytes());
    }
    body[18..20].copy_from_slice(&(layout.records_end as u16).to_le_bytes());
    Ok(())
}

fn encode_leaf_record_into_validated(target: &mut [u8], entry: LeafEntryRef<'_>) -> Result<()> {
    let (flags, value_length, aux, inline) = match entry.value {
        None => (0u8, 0u64, NULL_PAGE_ID, &[][..]),
        Some(BlinkValueRef::Inline(value)) => (1u8, value.len() as u64, 0, value),
        Some(BlinkValueRef::Overflow { head, length }) => (2u8, length, head.get(), &[][..]),
    };
    let record_length = leaf_record_encoded_len(entry)?;
    if target.len() != record_length {
        return Err(Error::invalid_input(
            "Blink leaf record target has invalid size",
        ));
    }
    target[0..8].copy_from_slice(&entry.revision.get().to_le_bytes());
    target[8..16].copy_from_slice(&value_length.to_le_bytes());
    target[16..24].copy_from_slice(&aux.to_le_bytes());
    target[24] = flags;
    target[26..28].copy_from_slice(&(entry.key.len() as u16).to_le_bytes());
    target[LEAF_RECORD_HEADER_SIZE..LEAF_RECORD_HEADER_SIZE + entry.key.len()]
        .copy_from_slice(entry.key);
    target[LEAF_RECORD_HEADER_SIZE + entry.key.len()..].copy_from_slice(inline);
    Ok(())
}

fn encode_internal_body_into(
    body: &mut [u8],
    level: u16,
    high_key: Option<&[u8]>,
    right_sibling: Option<PageId>,
    leftmost_child: PageId,
    entries: &[InternalEntry],
) -> Result<()> {
    debug_assert_eq!(body.len(), BODY_SIZE);
    let layout = internal_body_layout(leftmost_child, entries, high_key)?;
    body[0..4].copy_from_slice(&INTERNAL_MAGIC);
    body[4..6].copy_from_slice(&BODY_VERSION.to_le_bytes());
    body[8..10].copy_from_slice(&level.to_le_bytes());
    body[10..12].copy_from_slice(&(entries.len() as u16).to_le_bytes());
    body[12..14].copy_from_slice(&(layout.slot_end as u16).to_le_bytes());
    body[14..16].copy_from_slice(&(layout.records_start as u16).to_le_bytes());
    let high_offset = if high_key.is_some() {
        layout.slot_end
    } else {
        0
    };
    body[16..18].copy_from_slice(&(high_offset as u16).to_le_bytes());
    body[18..20].copy_from_slice(&(high_key.map_or(0, <[u8]>::len) as u16).to_le_bytes());
    body[20..28].copy_from_slice(&encode_page_id(right_sibling).to_le_bytes());
    body[28..36].copy_from_slice(&leftmost_child.get().to_le_bytes());
    if let Some(high_key) = high_key {
        body[layout.slot_end..layout.records_start].copy_from_slice(high_key);
    }

    let mut record_offset = BODY_SIZE;
    for entry_index in (0..entries.len()).rev() {
        let entry = &entries[entry_index];
        if entry.key.len() > u16::MAX as usize {
            return Err(Error::invalid_input("Blink separator is too large"));
        }
        let record_length = INTERNAL_RECORD_HEADER_SIZE
            .checked_add(entry.key.len())
            .ok_or_else(|| Error::invalid_input("Blink internal record size overflow"))?;
        record_offset = record_offset
            .checked_sub(record_length)
            .ok_or_else(|| Error::invalid_input("Blink internal records exceed page"))?;
        if record_offset < layout.records_start {
            return Err(Error::invalid_input("Blink internal records exceed page"));
        }
        body[record_offset..record_offset + 8]
            .copy_from_slice(&entry.right_child.get().to_le_bytes());
        body[record_offset + 12..record_offset + 14]
            .copy_from_slice(&(entry.key.len() as u16).to_le_bytes());
        body[record_offset + INTERNAL_RECORD_HEADER_SIZE..record_offset + record_length]
            .copy_from_slice(&entry.key);
        let slot = INTERNAL_HEADER_SIZE + entry_index * SLOT_SIZE;
        body[slot..slot + 2].copy_from_slice(&(record_offset as u16).to_le_bytes());
        body[slot + 2..slot + 4].copy_from_slice(&(record_length as u16).to_le_bytes());
        body[slot + 4..slot + 6].copy_from_slice(&(entry.key.len() as u16).to_le_bytes());
    }
    body[36..38].copy_from_slice(&(layout.records_end as u16).to_le_bytes());
    Ok(())
}

fn encode_overflow_body_into(
    body: &mut [u8],
    next: Option<PageId>,
    total_length: u64,
    chunk: &[u8],
) -> Result<()> {
    debug_assert_eq!(body.len(), BODY_SIZE);
    if chunk.len() > BODY_SIZE - OVERFLOW_HEADER_SIZE {
        return Err(Error::invalid_input("Blink overflow chunk is too large"));
    }
    body[0..4].copy_from_slice(&OVERFLOW_MAGIC);
    body[4..6].copy_from_slice(&BODY_VERSION.to_le_bytes());
    body[8..16].copy_from_slice(&encode_page_id(next).to_le_bytes());
    body[16..24].copy_from_slice(&total_length.to_le_bytes());
    body[24..28].copy_from_slice(&(chunk.len() as u32).to_le_bytes());
    body[OVERFLOW_HEADER_SIZE..OVERFLOW_HEADER_SIZE + chunk.len()].copy_from_slice(chunk);
    Ok(())
}

fn encode_free_body_into(body: &mut [u8], next: Option<PageId>) -> Result<()> {
    debug_assert_eq!(body.len(), BODY_SIZE);
    body[0..4].copy_from_slice(&FREE_MAGIC);
    body[4..6].copy_from_slice(&BODY_VERSION.to_le_bytes());
    body[8..16].copy_from_slice(&encode_page_id(next).to_le_bytes());
    Ok(())
}

#[cfg(test)]
fn encode_leaf_body(
    high_key: Option<&[u8]>,
    right_sibling: Option<PageId>,
    entries: &[LeafEntryRef<'_>],
) -> Result<Vec<u8>> {
    for pair in entries.windows(2) {
        if pair[0].key >= pair[1].key {
            return Err(Error::corruption(
                "Blink leaf entries are not strictly ordered",
            ));
        }
    }
    for entry in entries {
        validate_encoded_key(entry.key)?;
    }
    let slot_end = LEAF_HEADER_SIZE
        .checked_add(
            entries
                .len()
                .checked_mul(SLOT_SIZE)
                .ok_or_else(|| Error::invalid_input("Blink leaf slot overflow"))?,
        )
        .ok_or_else(|| Error::invalid_input("Blink leaf slot overflow"))?;
    let high_len = high_key.map_or(0, <[u8]>::len);
    let records_start = slot_end
        .checked_add(high_len)
        .ok_or_else(|| Error::invalid_input("Blink leaf fence overflow"))?;
    if entries.len() > u16::MAX as usize || records_start > BODY_SIZE {
        return Err(Error::invalid_input("Blink leaf header exceeds page"));
    }
    let mut body = vec![0u8; BODY_SIZE];
    body[0..4].copy_from_slice(&LEAF_MAGIC);
    body[4..6].copy_from_slice(&BODY_VERSION.to_le_bytes());
    body[8..10].copy_from_slice(&(entries.len() as u16).to_le_bytes());
    body[10..12].copy_from_slice(&(slot_end as u16).to_le_bytes());
    body[12..14].copy_from_slice(&(records_start as u16).to_le_bytes());
    body[14..16]
        .copy_from_slice(&(if high_key.is_some() { slot_end } else { 0 } as u16).to_le_bytes());
    body[16..18].copy_from_slice(&(high_len as u16).to_le_bytes());
    body[20..28].copy_from_slice(&encode_page_id(right_sibling).to_le_bytes());
    if let Some(high_key) = high_key {
        validate_encoded_key(high_key)?;
        body[slot_end..records_start].copy_from_slice(high_key);
    }
    encode_leaf_records(&mut body, slot_end, records_start, entries)
}

#[cfg(test)]
fn encode_leaf_records(
    body: &mut [u8],
    slot_end: usize,
    records_start: usize,
    entries: &[LeafEntryRef<'_>],
) -> Result<Vec<u8>> {
    let mut upper = BODY_SIZE;
    let mut slots = Vec::with_capacity(entries.len());
    for entry in entries.iter().rev() {
        let record = encode_leaf_record(*entry)?;
        upper = upper
            .checked_sub(record.len())
            .ok_or_else(|| Error::invalid_input("Blink leaf records exceed page"))?;
        if upper < records_start {
            return Err(Error::invalid_input("Blink leaf records exceed page"));
        }
        body[upper..upper + record.len()].copy_from_slice(&record);
        slots.push((upper, record.len()));
    }
    slots.reverse();
    body[18..20].copy_from_slice(&(upper as u16).to_le_bytes());
    for (index, (offset, length)) in slots.into_iter().enumerate() {
        let slot = slot_end - entries.len() * SLOT_SIZE + index * SLOT_SIZE;
        body[slot..slot + 2].copy_from_slice(&(offset as u16).to_le_bytes());
        body[slot + 2..slot + 4].copy_from_slice(&(length as u16).to_le_bytes());
        body[slot + 4..slot + 6].copy_from_slice(&(entries[index].key.len() as u16).to_le_bytes());
    }
    Ok(body.to_vec())
}

#[cfg(test)]
fn encode_leaf_record(entry: LeafEntryRef<'_>) -> Result<Vec<u8>> {
    validate_encoded_key(entry.key)?;
    let (flags, value_length, aux, inline) = match entry.value {
        None => (0u8, 0u64, NULL_PAGE_ID, &[][..]),
        Some(BlinkValueRef::Inline(value)) => (1u8, value.len() as u64, 0, value),
        Some(BlinkValueRef::Overflow { head, length }) => (2u8, length, head.get(), &[][..]),
    };
    let length = LEAF_RECORD_HEADER_SIZE
        .checked_add(entry.key.len())
        .and_then(|length| length.checked_add(inline.len()))
        .ok_or_else(|| Error::invalid_input("Blink leaf record size overflow"))?;
    let mut record = vec![0u8; length];
    record[0..8].copy_from_slice(&entry.revision.get().to_le_bytes());
    record[8..16].copy_from_slice(&value_length.to_le_bytes());
    record[16..24].copy_from_slice(&aux.to_le_bytes());
    record[24] = flags;
    record[26..28].copy_from_slice(&(entry.key.len() as u16).to_le_bytes());
    record[LEAF_RECORD_HEADER_SIZE..LEAF_RECORD_HEADER_SIZE + entry.key.len()]
        .copy_from_slice(entry.key);
    record[LEAF_RECORD_HEADER_SIZE + entry.key.len()..].copy_from_slice(inline);
    Ok(record)
}

#[cfg(test)]
fn encode_internal_body(
    level: u16,
    high_key: Option<&[u8]>,
    right_sibling: Option<PageId>,
    leftmost_child: PageId,
    entries: &[InternalEntry],
) -> Result<Vec<u8>> {
    if leftmost_child.get() < FIRST_DATA_PAGE {
        return Err(Error::invalid_input(
            "Blink internal leftmost child is invalid",
        ));
    }
    ensure_sorted_internal(entries)?;
    let slot_end = INTERNAL_HEADER_SIZE
        .checked_add(
            entries
                .len()
                .checked_mul(SLOT_SIZE)
                .ok_or_else(|| Error::invalid_input("Blink internal slot overflow"))?,
        )
        .ok_or_else(|| Error::invalid_input("Blink internal slot overflow"))?;
    let high_len = high_key.map_or(0, <[u8]>::len);
    let records_start = slot_end
        .checked_add(high_len)
        .ok_or_else(|| Error::invalid_input("Blink internal fence overflow"))?;
    if entries.len() > u16::MAX as usize || records_start > BODY_SIZE {
        return Err(Error::invalid_input("Blink internal header exceeds page"));
    }
    let mut body = vec![0u8; BODY_SIZE];
    body[0..4].copy_from_slice(&INTERNAL_MAGIC);
    body[4..6].copy_from_slice(&BODY_VERSION.to_le_bytes());
    body[8..10].copy_from_slice(&level.to_le_bytes());
    body[10..12].copy_from_slice(&(entries.len() as u16).to_le_bytes());
    body[12..14].copy_from_slice(&(slot_end as u16).to_le_bytes());
    body[14..16].copy_from_slice(&(records_start as u16).to_le_bytes());
    body[16..18]
        .copy_from_slice(&(if high_key.is_some() { slot_end } else { 0 } as u16).to_le_bytes());
    body[18..20].copy_from_slice(&(high_len as u16).to_le_bytes());
    body[20..28].copy_from_slice(&encode_page_id(right_sibling).to_le_bytes());
    body[28..36].copy_from_slice(&leftmost_child.get().to_le_bytes());
    if let Some(high_key) = high_key {
        validate_encoded_key(high_key)?;
        body[slot_end..records_start].copy_from_slice(high_key);
    }
    let mut upper = BODY_SIZE;
    let mut slots = Vec::with_capacity(entries.len());
    for entry in entries.iter().rev() {
        if entry.key.len() > u16::MAX as usize {
            return Err(Error::invalid_input("Blink separator is too large"));
        }
        let length = INTERNAL_RECORD_HEADER_SIZE + entry.key.len();
        upper = upper
            .checked_sub(length)
            .ok_or_else(|| Error::invalid_input("Blink internal records exceed page"))?;
        if upper < records_start {
            return Err(Error::invalid_input("Blink internal records exceed page"));
        }
        body[upper..upper + 8].copy_from_slice(&entry.right_child.get().to_le_bytes());
        body[upper + 12..upper + 14].copy_from_slice(&(entry.key.len() as u16).to_le_bytes());
        body[upper + INTERNAL_RECORD_HEADER_SIZE..upper + length].copy_from_slice(&entry.key);
        slots.push((upper, length));
    }
    slots.reverse();
    body[36..38].copy_from_slice(&(upper as u16).to_le_bytes());
    for (index, (offset, length)) in slots.into_iter().enumerate() {
        let slot = slot_end - entries.len() * SLOT_SIZE + index * SLOT_SIZE;
        body[slot..slot + 2].copy_from_slice(&(offset as u16).to_le_bytes());
        body[slot + 2..slot + 4].copy_from_slice(&(length as u16).to_le_bytes());
        body[slot + 4..slot + 6].copy_from_slice(&(entries[index].key.len() as u16).to_le_bytes());
    }
    Ok(body)
}

#[cfg(test)]
fn encode_overflow_body(next: Option<PageId>, total_length: u64, chunk: &[u8]) -> Result<Vec<u8>> {
    if chunk.len() > BODY_SIZE - OVERFLOW_HEADER_SIZE {
        return Err(Error::invalid_input("Blink overflow chunk is too large"));
    }
    let mut body = vec![0u8; BODY_SIZE];
    body[0..4].copy_from_slice(&OVERFLOW_MAGIC);
    body[4..6].copy_from_slice(&BODY_VERSION.to_le_bytes());
    body[8..16].copy_from_slice(&encode_page_id(next).to_le_bytes());
    body[16..24].copy_from_slice(&total_length.to_le_bytes());
    body[24..28].copy_from_slice(&(chunk.len() as u32).to_le_bytes());
    body[OVERFLOW_HEADER_SIZE..OVERFLOW_HEADER_SIZE + chunk.len()].copy_from_slice(chunk);
    Ok(body)
}

#[cfg(test)]
fn encode_free_body(next: Option<PageId>) -> Result<Vec<u8>> {
    let mut body = vec![0u8; BODY_SIZE];
    body[0..4].copy_from_slice(&FREE_MAGIC);
    body[4..6].copy_from_slice(&BODY_VERSION.to_le_bytes());
    body[8..16].copy_from_slice(&encode_page_id(next).to_le_bytes());
    Ok(body)
}

fn decode_leaf_body(lsn: Lsn, body: &[u8]) -> Result<BlinkPage> {
    if body[0..4] != LEAF_MAGIC {
        return Err(Error::corruption("Blink leaf magic mismatch"));
    }
    check_body_version(body)?;
    if body[6..8].iter().any(|byte| *byte != 0) || body[20..48].iter().any(|byte| *byte != 0) {
        // bytes 20..28 contain the sibling, so only the rest of the header is reserved.
        if body[6..8].iter().any(|byte| *byte != 0) || body[28..48].iter().any(|byte| *byte != 0) {
            return Err(Error::corruption("Blink leaf reserved bytes are non-zero"));
        }
    }
    let count = u16::from_le_bytes(body[8..10].try_into().unwrap()) as usize;
    let slot_end = u16::from_le_bytes(body[10..12].try_into().unwrap()) as usize;
    let records_start = u16::from_le_bytes(body[12..14].try_into().unwrap()) as usize;
    let high_offset = u16::from_le_bytes(body[14..16].try_into().unwrap()) as usize;
    let high_len = u16::from_le_bytes(body[16..18].try_into().unwrap()) as usize;
    let records_end = u16::from_le_bytes(body[18..20].try_into().unwrap()) as usize;
    let right = decode_page_id(u64::from_le_bytes(body[20..28].try_into().unwrap()));
    validate_slot_header(
        count,
        slot_end,
        records_start,
        records_end,
        LEAF_HEADER_SIZE,
        body.len(),
    )?;
    let high_key = decode_high_key(body, high_offset, high_len, slot_end, records_start)?;
    let entries = decode_leaf_entries(body, count, slot_end, records_end)?;
    Ok(BlinkPage::Leaf {
        lsn,
        high_key,
        right_sibling: right,
        entries,
    })
}

fn decode_leaf_entries(
    body: &[u8],
    count: usize,
    slot_end: usize,
    records_end: usize,
) -> Result<LeafEntries> {
    let mut ranges = Vec::with_capacity(count);
    let mut entries = LeafEntries::with_capacity(count, body.len().saturating_sub(records_end));
    for index in 0..count {
        let slot = LEAF_HEADER_SIZE + index * SLOT_SIZE;
        let offset = u16::from_le_bytes(body[slot..slot + 2].try_into().unwrap()) as usize;
        let length = u16::from_le_bytes(body[slot + 2..slot + 4].try_into().unwrap()) as usize;
        let key_length = u16::from_le_bytes(body[slot + 4..slot + 6].try_into().unwrap()) as usize;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| Error::corruption("Blink leaf record overflow"))?;
        if offset < records_end || end > body.len() || length < LEAF_RECORD_HEADER_SIZE {
            return Err(Error::corruption("Blink leaf record range is invalid"));
        }
        if offset < slot_end || end > body.len() {
            return Err(Error::corruption("Blink leaf record overlaps header"));
        }
        ranges.push((offset, end));
        entries.push(decode_leaf_record(&body[offset..end], key_length)?);
    }
    ensure_non_overlapping(&mut ranges)?;
    ensure_sorted_leaf(entries.all())?;
    Ok(entries)
}

fn decode_leaf_record(bytes: &[u8], slot_key_length: usize) -> Result<LeafEntryRef<'_>> {
    let revision = Revision::new(u64::from_le_bytes(bytes[0..8].try_into().unwrap()));
    let value_length = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let aux = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    if bytes[25] != 0 || bytes[28..32].iter().any(|byte| *byte != 0) {
        return Err(Error::corruption(
            "Blink leaf record reserved bytes are non-zero",
        ));
    }
    let flags = bytes[24];
    let key_length = u16::from_le_bytes(bytes[26..28].try_into().unwrap()) as usize;
    if key_length != slot_key_length {
        return Err(Error::corruption("Blink leaf slot key length mismatch"));
    }
    let key_end = LEAF_RECORD_HEADER_SIZE
        .checked_add(key_length)
        .ok_or_else(|| Error::corruption("Blink leaf key length overflow"))?;
    if key_end > bytes.len() {
        return Err(Error::corruption("Blink leaf key exceeds record"));
    }
    let key = &bytes[LEAF_RECORD_HEADER_SIZE..key_end];
    validate_encoded_key(key).map_err(|_| Error::corruption("Blink leaf key is not canonical"))?;
    let value = match flags {
        0 if value_length == 0 && aux == NULL_PAGE_ID && bytes.len() == key_end => None,
        1 => {
            let end = key_end
                .checked_add(value_length as usize)
                .ok_or_else(|| Error::corruption("Blink inline value overflow"))?;
            if end != bytes.len() || aux != 0 {
                return Err(Error::corruption("Blink inline value record is invalid"));
            }
            Some(BlinkValueRef::Inline(&bytes[key_end..end]))
        }
        2 if bytes.len() == key_end && aux != NULL_PAGE_ID && value_length > 0 => {
            Some(BlinkValueRef::Overflow {
                head: PageId::new(aux),
                length: value_length,
            })
        }
        _ => return Err(Error::corruption("Blink leaf value record is invalid")),
    };
    Ok(LeafEntryRef {
        key,
        revision,
        value,
    })
}

fn decode_internal_body(lsn: Lsn, body: &[u8]) -> Result<BlinkPage> {
    if body[0..4] != INTERNAL_MAGIC {
        return Err(Error::corruption("Blink internal magic mismatch"));
    }
    check_body_version(body)?;
    if body[6..8].iter().any(|byte| *byte != 0) || body[40..56].iter().any(|byte| *byte != 0) {
        return Err(Error::corruption(
            "Blink internal reserved bytes are non-zero",
        ));
    }
    let level = u16::from_le_bytes(body[8..10].try_into().unwrap());
    if level == 0 {
        return Err(Error::corruption("Blink internal level is zero"));
    }
    let count = u16::from_le_bytes(body[10..12].try_into().unwrap()) as usize;
    let slot_end = u16::from_le_bytes(body[12..14].try_into().unwrap()) as usize;
    let records_start = u16::from_le_bytes(body[14..16].try_into().unwrap()) as usize;
    let high_offset = u16::from_le_bytes(body[16..18].try_into().unwrap()) as usize;
    let high_len = u16::from_le_bytes(body[18..20].try_into().unwrap()) as usize;
    let records_end = u16::from_le_bytes(body[36..38].try_into().unwrap()) as usize;
    let right = decode_page_id(u64::from_le_bytes(body[20..28].try_into().unwrap()));
    let leftmost = PageId::new(u64::from_le_bytes(body[28..36].try_into().unwrap()));
    validate_slot_header(
        count,
        slot_end,
        records_start,
        records_end,
        INTERNAL_HEADER_SIZE,
        body.len(),
    )?;
    let high_key = decode_high_key(body, high_offset, high_len, slot_end, records_start)?;
    let mut ranges = Vec::with_capacity(count);
    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        let slot = INTERNAL_HEADER_SIZE + index * SLOT_SIZE;
        let offset = u16::from_le_bytes(body[slot..slot + 2].try_into().unwrap()) as usize;
        let length = u16::from_le_bytes(body[slot + 2..slot + 4].try_into().unwrap()) as usize;
        let key_length = u16::from_le_bytes(body[slot + 4..slot + 6].try_into().unwrap()) as usize;
        let end = offset
            .checked_add(length)
            .ok_or_else(|| Error::corruption("Blink internal record overflow"))?;
        if offset < records_end
            || end > body.len()
            || length != INTERNAL_RECORD_HEADER_SIZE + key_length
        {
            return Err(Error::corruption("Blink internal record range is invalid"));
        }
        ranges.push((offset, end));
        let key_start = offset + INTERNAL_RECORD_HEADER_SIZE;
        let key = body[key_start..end].to_vec();
        validate_encoded_key(&key)
            .map_err(|_| Error::corruption("Blink separator is not canonical"))?;
        entries.push(InternalEntry {
            key,
            right_child: PageId::new(u64::from_le_bytes(
                body[offset..offset + 8].try_into().unwrap(),
            )),
        });
    }
    ensure_non_overlapping(&mut ranges)?;
    ensure_sorted_internal(&entries)?;
    Ok(BlinkPage::Internal {
        lsn,
        level,
        high_key,
        right_sibling: right,
        leftmost_child: leftmost,
        entries,
    })
}

fn decode_overflow_body(lsn: Lsn, body: &[u8]) -> Result<BlinkPage> {
    if body[0..4] != OVERFLOW_MAGIC {
        return Err(Error::corruption("Blink overflow magic mismatch"));
    }
    check_body_version(body)?;
    if body[6..8].iter().any(|byte| *byte != 0) || body[28..32].iter().any(|byte| *byte != 0) {
        return Err(Error::corruption(
            "Blink overflow reserved bytes are non-zero",
        ));
    }
    let next = decode_page_id(u64::from_le_bytes(body[8..16].try_into().unwrap()));
    let total_length = u64::from_le_bytes(body[16..24].try_into().unwrap());
    let chunk_length = u32::from_le_bytes(body[24..28].try_into().unwrap()) as usize;
    if chunk_length > BODY_SIZE - OVERFLOW_HEADER_SIZE {
        return Err(Error::corruption("Blink overflow chunk length is invalid"));
    }
    if body[OVERFLOW_HEADER_SIZE + chunk_length..]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err(Error::corruption(
            "Blink overflow trailing bytes are non-zero",
        ));
    }
    Ok(BlinkPage::Overflow {
        lsn,
        next,
        total_length,
        chunk: body[OVERFLOW_HEADER_SIZE..OVERFLOW_HEADER_SIZE + chunk_length].to_vec(),
    })
}

fn decode_free_body(lsn: Lsn, body: &[u8]) -> Result<BlinkPage> {
    if body[0..4] != FREE_MAGIC {
        return Err(Error::corruption("Blink free magic mismatch"));
    }
    check_body_version(body)?;
    if body[6..8].iter().any(|byte| *byte != 0) || body[16..].iter().any(|byte| *byte != 0) {
        return Err(Error::corruption("Blink free reserved bytes are non-zero"));
    }
    Ok(BlinkPage::Free {
        lsn,
        next: decode_page_id(u64::from_le_bytes(body[8..16].try_into().unwrap())),
    })
}

fn check_body_version(body: &[u8]) -> Result<()> {
    let version = u16::from_le_bytes(body[4..6].try_into().unwrap());
    if version != BODY_VERSION {
        return Err(Error::unsupported_format(format!(
            "Blink body version {version}, supported {BODY_VERSION}"
        )));
    }
    Ok(())
}

fn validate_slot_header(
    count: usize,
    slot_end: usize,
    records_start: usize,
    records_end: usize,
    header_size: usize,
    body_size: usize,
) -> Result<()> {
    if slot_end != header_size + count * SLOT_SIZE
        || slot_end > records_start
        || records_start > records_end
        || records_end > body_size
    {
        return Err(Error::corruption(
            "Blink slotted page boundaries are invalid",
        ));
    }
    Ok(())
}

fn decode_high_key(
    body: &[u8],
    offset: usize,
    length: usize,
    slot_end: usize,
    records_start: usize,
) -> Result<Option<Vec<u8>>> {
    if length == 0 {
        if offset != 0 {
            return Err(Error::corruption("Blink +infinity fence has an offset"));
        }
        return Ok(None);
    }
    if offset != slot_end || offset + length != records_start {
        return Err(Error::corruption("Blink high-key range is invalid"));
    }
    let key = body[offset..records_start].to_vec();
    validate_encoded_key(&key).map_err(|_| Error::corruption("Blink high key is not canonical"))?;
    Ok(Some(key))
}

fn ensure_sorted_leaf(entries: LeafRange<'_>) -> Result<()> {
    let mut previous: Option<&[u8]> = None;
    for entry in entries.iter() {
        if previous.is_some_and(|previous| previous >= entry.key) {
            return Err(Error::corruption(
                "Blink leaf entries are not strictly ordered",
            ));
        }
        previous = Some(entry.key);
    }
    for entry in entries.iter() {
        validate_encoded_key(entry.key)?;
    }
    Ok(())
}

fn ensure_sorted_internal(entries: &[InternalEntry]) -> Result<()> {
    for pair in entries.windows(2) {
        if pair[0].key >= pair[1].key {
            return Err(Error::corruption(
                "Blink separators are not strictly ordered",
            ));
        }
    }
    for entry in entries {
        validate_encoded_key(&entry.key)?;
        if entry.right_child.get() < FIRST_DATA_PAGE {
            return Err(Error::corruption(
                "Blink separator child page id is invalid",
            ));
        }
    }
    Ok(())
}

fn ensure_non_overlapping(ranges: &mut [(usize, usize)]) -> Result<()> {
    ranges.sort_unstable();
    for pair in ranges.windows(2) {
        if pair[0].1 > pair[1].0 {
            return Err(Error::corruption("Blink records overlap"));
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PageBodyLayout {
    slot_end: usize,
    records_start: usize,
    records_end: usize,
}

fn leaf_record_encoded_len(entry: LeafEntryRef<'_>) -> Result<usize> {
    let inline_value_len = match entry.value {
        None | Some(BlinkValueRef::Overflow { .. }) => 0,
        Some(BlinkValueRef::Inline(value)) => value.len(),
    };
    LEAF_RECORD_HEADER_SIZE
        .checked_add(entry.key.len())
        .and_then(|length| length.checked_add(inline_value_len))
        .ok_or_else(|| Error::invalid_input("Blink leaf record size overflow"))
}

fn leaf_body_layout(entries: LeafRange<'_>, high_key: Option<&[u8]>) -> Result<PageBodyLayout> {
    ensure_sorted_leaf(entries)?;
    let slot_end = LEAF_HEADER_SIZE
        .checked_add(
            entries
                .len()
                .checked_mul(SLOT_SIZE)
                .ok_or_else(|| Error::invalid_input("Blink leaf slot overflow"))?,
        )
        .ok_or_else(|| Error::invalid_input("Blink leaf slot overflow"))?;
    let records_start = slot_end
        .checked_add(high_key.map_or(0, <[u8]>::len))
        .ok_or_else(|| Error::invalid_input("Blink leaf fence overflow"))?;
    if entries.len() > u16::MAX as usize || records_start > BODY_SIZE {
        return Err(Error::invalid_input("Blink leaf header exceeds page"));
    }
    if let Some(high_key) = high_key {
        validate_encoded_key(high_key)?;
    }

    let mut records_end = BODY_SIZE;
    for entry in entries.iter().rev() {
        records_end = records_end
            .checked_sub(leaf_record_encoded_len(entry)?)
            .ok_or_else(|| Error::invalid_input("Blink leaf records exceed page"))?;
        if records_end < records_start {
            return Err(Error::invalid_input("Blink leaf records exceed page"));
        }
    }
    Ok(PageBodyLayout {
        slot_end,
        records_start,
        records_end,
    })
}

fn internal_body_layout(
    leftmost_child: PageId,
    entries: &[InternalEntry],
    high_key: Option<&[u8]>,
) -> Result<PageBodyLayout> {
    if leftmost_child.get() < FIRST_DATA_PAGE {
        return Err(Error::invalid_input(
            "Blink internal leftmost child is invalid",
        ));
    }
    ensure_sorted_internal(entries)?;
    let slot_end = INTERNAL_HEADER_SIZE
        .checked_add(
            entries
                .len()
                .checked_mul(SLOT_SIZE)
                .ok_or_else(|| Error::invalid_input("Blink internal slot overflow"))?,
        )
        .ok_or_else(|| Error::invalid_input("Blink internal slot overflow"))?;
    let records_start = slot_end
        .checked_add(high_key.map_or(0, <[u8]>::len))
        .ok_or_else(|| Error::invalid_input("Blink internal fence overflow"))?;
    if entries.len() > u16::MAX as usize || records_start > BODY_SIZE {
        return Err(Error::invalid_input("Blink internal header exceeds page"));
    }
    if let Some(high_key) = high_key {
        validate_encoded_key(high_key)?;
    }

    let mut records_end = BODY_SIZE;
    for entry in entries.iter().rev() {
        if entry.key.len() > u16::MAX as usize {
            return Err(Error::invalid_input("Blink separator is too large"));
        }
        let record_len = INTERNAL_RECORD_HEADER_SIZE
            .checked_add(entry.key.len())
            .ok_or_else(|| Error::invalid_input("Blink internal record size overflow"))?;
        records_end = records_end
            .checked_sub(record_len)
            .ok_or_else(|| Error::invalid_input("Blink internal records exceed page"))?;
        if records_end < records_start {
            return Err(Error::invalid_input("Blink internal records exceed page"));
        }
    }
    Ok(PageBodyLayout {
        slot_end,
        records_start,
        records_end,
    })
}

fn leaf_fits(entries: LeafRange<'_>, high_key: Option<&[u8]>, right: Option<PageId>) -> bool {
    let _ = right;
    leaf_body_layout(entries, high_key).is_ok()
}

fn internal_fits(
    leftmost: PageId,
    entries: &[InternalEntry],
    high_key: Option<&[u8]>,
    right: Option<PageId>,
    level: u16,
) -> bool {
    let _ = (right, level);
    internal_body_layout(leftmost, entries, high_key).is_ok()
}

fn choose_leaf_split(
    entries: &LeafEntries,
    high_key: Option<&[u8]>,
    right: Option<PageId>,
) -> usize {
    let middle = entries.len() / 2;
    (1..entries.len())
        .min_by_key(|index| {
            let left = leaf_fits(entries.range(0, *index), Some(entries.key(*index)), right);
            let right_fits = leaf_fits(entries.range(*index, entries.len()), high_key, right);
            if left && right_fits {
                (*index as isize - middle as isize).unsigned_abs()
            } else {
                usize::MAX
            }
        })
        .unwrap_or(middle)
}

fn page_level<S: BlinkMutationState>(state: &S, page_id: PageId) -> Result<u16> {
    match state.page(page_id) {
        Some(BlinkPage::Leaf { .. }) => Ok(0),
        Some(BlinkPage::Internal { level, .. }) => Ok(*level),
        _ => Err(Error::corruption("Blink page is not a tree page")),
    }
}

fn count_free_pages(state: &BlinkState) -> Result<u64> {
    let mut count = 0;
    let mut current = state.free_list_head;
    let mut visited = HashSet::new();
    while let Some(page_id) = current {
        if !visited.insert(page_id) {
            return Err(Error::corruption("Blink free list cycle"));
        }
        let Some(BlinkPage::Free { next, .. }) = state.page_ref(page_id) else {
            return Err(Error::corruption(
                "Blink free list points to a non-free page",
            ));
        };
        count += 1;
        current = *next;
    }
    Ok(count)
}

fn check_state(state: &BlinkState) -> Result<InvariantReport> {
    if state.root_page_id.get() < FIRST_DATA_PAGE
        || state.high_water_page_id.get() < state.root_page_id.get()
    {
        return Err(Error::corruption("Blink root/high-water invariant failed"));
    }
    let mut reachable = BTreeSet::new();
    let mut leaves = Vec::new();
    let mut max_revision = Revision::ZERO;
    walk_tree(
        state,
        state.root_page_id,
        None,
        None,
        &mut reachable,
        &mut leaves,
        &mut max_revision,
    )?;
    let mut overflow_owned = BTreeSet::new();
    for leaf_id in &leaves {
        let BlinkPage::Leaf { entries, .. } = state.page_ref(*leaf_id).unwrap() else {
            unreachable!()
        };
        for entry in entries.iter() {
            max_revision = max_revision.max(entry.revision);
            if let Some(BlinkValueRef::Overflow { head, length }) = entry.value {
                let mut current = Some(head);
                let mut total = 0u64;
                let mut local = HashSet::new();
                while let Some(id) = current {
                    if !local.insert(id) || !overflow_owned.insert(id) {
                        return Err(Error::corruption(
                            "Blink overflow chain is cyclic or multiply owned",
                        ));
                    }
                    let BlinkPage::Overflow {
                        next,
                        total_length,
                        chunk,
                        ..
                    } = state
                        .page_ref(id)
                        .ok_or_else(|| Error::corruption("Blink overflow page is missing"))?
                    else {
                        return Err(Error::corruption(
                            "Blink overflow owner points to wrong page",
                        ));
                    };
                    if *total_length != length {
                        return Err(Error::corruption("Blink overflow total length mismatch"));
                    }
                    total = total.saturating_add(chunk.len() as u64);
                    current = *next;
                }
                if total < length {
                    return Err(Error::corruption(
                        "Blink overflow chain is shorter than value",
                    ));
                }
            }
        }
    }
    reachable.extend(overflow_owned);
    let mut free = BTreeSet::new();
    let mut current = state.free_list_head;
    while let Some(id) = current {
        if !free.insert(id) || reachable.contains(&id) {
            return Err(Error::corruption(
                "Blink free list cycles or overlaps reachable pages",
            ));
        }
        let Some(BlinkPage::Free { next, .. }) = state.page_ref(id) else {
            return Err(Error::corruption(
                "Blink free list points to a non-free page",
            ));
        };
        current = *next;
    }
    let expected_ids = (FIRST_DATA_PAGE..=state.high_water_page_id.get())
        .map(PageId::new)
        .collect::<BTreeSet<_>>();
    let known = reachable.union(&free).copied().collect::<BTreeSet<_>>();
    let leaked = expected_ids.difference(&known).copied().collect::<Vec<_>>();
    if !leaked.is_empty() || state.pages.keys().any(|id| !expected_ids.contains(id)) {
        return Err(Error::corruption(format!(
            "Blink leaked or out-of-range pages: {leaked:?}"
        )));
    }
    check_sibling_links(state, &leaves)?;
    Ok(InvariantReport {
        reachable_pages: reachable.len(),
        free_pages: free.len(),
        leaked_pages: leaked,
        max_revision,
    })
}

fn walk_tree(
    state: &BlinkState,
    page_id: PageId,
    lower: Option<&[u8]>,
    upper: Option<&[u8]>,
    reachable: &mut BTreeSet<PageId>,
    leaves: &mut Vec<PageId>,
    max_revision: &mut Revision,
) -> Result<()> {
    if !reachable.insert(page_id) {
        return Err(Error::corruption(
            "Blink tree contains a multiply reachable page or cycle",
        ));
    }
    let page = state
        .page_ref(page_id)
        .ok_or_else(|| Error::corruption("Blink tree points outside the file"))?;
    match page {
        BlinkPage::Leaf {
            high_key, entries, ..
        } => {
            ensure_sorted_leaf(entries.all())?;
            if let Some(high) = high_key.as_deref()
                && upper.is_some_and(|upper| high > upper)
            {
                return Err(Error::corruption(
                    "Blink leaf fence exceeds parent boundary",
                ));
            }
            for entry in entries.iter() {
                if lower.is_some_and(|lower| entry.key < lower)
                    || upper.is_some_and(|upper| entry.key >= upper)
                    || high_key
                        .as_ref()
                        .is_some_and(|high| entry.key >= high.as_slice())
                {
                    return Err(Error::corruption(format!(
                        "Blink leaf key violates fence or parent range: key={:?} lower={:?} upper={:?} high={:?}",
                        entry.key, lower, upper, high_key
                    )));
                }
                *max_revision = (*max_revision).max(entry.revision);
            }
            leaves.push(page_id);
        }
        BlinkPage::Internal {
            level,
            high_key,
            leftmost_child,
            entries,
            ..
        } => {
            ensure_sorted_internal(entries)?;
            if *level == 0
                || high_key
                    .as_ref()
                    .is_some_and(|high| upper.is_some_and(|upper| high.as_slice() > upper))
            {
                return Err(Error::corruption(
                    "Blink internal fence or level is invalid",
                ));
            }
            let mut child_lower: Option<Vec<u8>> = lower.map(ToOwned::to_owned);
            walk_tree(
                state,
                *leftmost_child,
                child_lower.as_deref(),
                entries.first().map(|entry| entry.key.as_ref()).or(upper),
                reachable,
                leaves,
                max_revision,
            )?;
            for (index, entry) in entries.iter().enumerate() {
                child_lower = Some(entry.key.clone());
                let child_upper = entries
                    .get(index + 1)
                    .map(|next| next.key.as_slice())
                    .or(upper);
                walk_tree(
                    state,
                    entry.right_child,
                    child_lower.as_deref(),
                    child_upper,
                    reachable,
                    leaves,
                    max_revision,
                )?;
            }
        }
        BlinkPage::Overflow { .. } | BlinkPage::Free { .. } => {
            return Err(Error::corruption(
                "Blink tree route reaches data-management page",
            ));
        }
    }
    Ok(())
}

fn check_sibling_links(state: &BlinkState, leaves: &[PageId]) -> Result<()> {
    let leaf_set = leaves.iter().copied().collect::<BTreeSet<_>>();
    let first = *leaves
        .first()
        .ok_or_else(|| Error::corruption("Blink tree has no leaf"))?;
    let mut chain = Vec::new();
    let mut current = Some(first);
    let mut visited = HashSet::new();
    while let Some(id) = current {
        if !visited.insert(id) {
            return Err(Error::corruption("Blink leaf sibling chain cycles"));
        }
        if !leaf_set.contains(&id) {
            return Err(Error::corruption(
                "Blink leaf sibling points outside leaf set",
            ));
        }
        chain.push(id);
        current = match state.page_ref(id) {
            Some(BlinkPage::Leaf { right_sibling, .. }) => *right_sibling,
            _ => return Err(Error::corruption("Blink leaf chain points to non-leaf")),
        };
    }
    if chain != leaves {
        return Err(Error::corruption(
            "Blink leaf chain order differs from tree order",
        ));
    }
    for (index, id) in leaves.iter().enumerate() {
        let Some(BlinkPage::Leaf {
            right_sibling,
            high_key,
            entries,
            ..
        }) = state.page_ref(*id)
        else {
            unreachable!()
        };
        if let Some(next) = right_sibling {
            if *next == *id || page_level(state, *next)? != 0 {
                return Err(Error::corruption(
                    "Blink leaf right link has invalid target",
                ));
            }
            let Some(BlinkPage::Leaf {
                entries: next_entries,
                ..
            }) = state.page_ref(*next)
            else {
                unreachable!()
            };
            if next_entries
                .first()
                .is_some_and(|next_key| entries.last().is_some_and(|last| next_key.key <= last.key))
            {
                return Err(Error::corruption(
                    "Blink sibling key ranges are not increasing",
                ));
            }
            if index + 1 == leaves.len() {
                return Err(Error::corruption("Blink rightmost leaf has a sibling"));
            }
        } else if index + 1 != leaves.len() || high_key.is_some() {
            return Err(Error::corruption(
                "Blink finite/rightmost leaf fence is invalid",
            ));
        }
    }
    for (id, page) in &state.pages {
        let BlinkPage::Internal {
            level,
            right_sibling,
            high_key,
            ..
        } = &**page
        else {
            continue;
        };
        match right_sibling {
            Some(next) => {
                if *next == *id {
                    return Err(Error::corruption("Blink internal right link self-links"));
                }
                let Some(BlinkPage::Internal {
                    level: next_level, ..
                }) = state.page_ref(*next)
                else {
                    return Err(Error::corruption(
                        "Blink internal right link targets non-internal",
                    ));
                };
                if next_level != level {
                    return Err(Error::corruption(
                        "Blink internal siblings have different levels",
                    ));
                }
                if let Some(high) = high_key {
                    let next_min = subtree_min_key(state, *next)?;
                    if next_min.as_deref() != Some(high.as_slice()) {
                        return Err(Error::corruption(
                            "Blink internal sibling fence is not continuous",
                        ));
                    }
                } else {
                    return Err(Error::corruption(
                        "Blink internal right-linked page has an infinite fence",
                    ));
                }
            }
            None if high_key.is_some() => {
                return Err(Error::corruption(
                    "Blink finite internal fence has no right sibling",
                ));
            }
            None => {}
        }
    }
    Ok(())
}

fn subtree_min_key(state: &BlinkState, mut page_id: PageId) -> Result<Option<Vec<u8>>> {
    loop {
        match BlinkMutationState::page(state, page_id) {
            Some(BlinkPage::Leaf { entries, .. }) => {
                return Ok(entries.first().map(|entry| entry.key.as_ref().to_vec()));
            }
            Some(BlinkPage::Internal { leftmost_child, .. }) => page_id = *leftmost_child,
            _ => {
                return Err(Error::corruption(
                    "Blink subtree minimum reaches non-tree page",
                ));
            }
        }
    }
}

fn recover_data_file<F: DurableFile>(
    file: &mut F,
    pages: &BTreeMap<PageId, RecoveredWalPage>,
    checkpoint_hint: Lsn,
) -> Result<()> {
    let mut images = Vec::new();
    let mut high_water = FIRST_DATA_PAGE;
    for (page_id, page) in pages {
        if page.commit_lsn <= checkpoint_hint {
            continue;
        }
        high_water = high_water.max(page_id.get());
        images.push((*page_id, &page.image));
    }
    if images.is_empty() {
        return Ok(());
    }
    let length = high_water
        .checked_add(1)
        .and_then(|pages| pages.checked_mul(PAGE_SIZE as u64))
        .ok_or_else(|| Error::recovery("Blink recovery file length overflows"))?;
    if file.len()? < length {
        file.set_len(length)?;
    }
    for (page_id, image) in images {
        // The WAL validator already checked the format and checksum. Decode
        // again here so recovery never writes an image to the wrong physical
        // slot if the caller bypasses the normal open path in a test.
        if page_id.get() >= FIRST_DATA_PAGE {
            decode_blink_page(&image[..], page_id)?;
        } else {
            decode_blink_superblock_image(&image[..])?;
        }
        write_all_at(file, page_id.get() * PAGE_SIZE as u64, &image[..])?;
    }
    file.sync_data()
}

fn existing_checkpoint<F: DurableFile>(
    file: &mut F,
    identity: &WalIdentity,
    wal_length: u64,
) -> Result<Lsn> {
    if file.is_empty()? {
        return Ok(Lsn::ZERO);
    }
    if file.len()? < 2 * PAGE_SIZE as u64 {
        return Err(Error::corruption(
            "experimental database has no superblock pair",
        ));
    }
    let a = read_exact_at(file, 0, PAGE_SIZE)?;
    let b = read_exact_at(file, PAGE_SIZE as u64, PAGE_SIZE)?;
    let sb_a = decode_blink_superblock_image(&a);
    let sb_b = decode_blink_superblock_image(&b);
    let sb = match (sb_a, sb_b) {
        (Ok(a), Ok(b)) => {
            if a.generation >= b.generation {
                a
            } else {
                b
            }
        }
        (Ok(a), Err(_)) => a,
        (Err(_), Ok(b)) => b,
        (Err(a), Err(b)) => {
            return Err(Error::corruption(format!(
                "experimental superblock pair invalid: {a}; {b}"
            )));
        }
    };
    if sb.database_uuid != identity.database_uuid
        || sb.tenant_id != identity.tenant_id
        || sb.shard_id != identity.shard_id
        || sb.shard_epoch != identity.shard_epoch
    {
        return Err(Error::corruption("experimental database identity mismatch"));
    }
    // A non-empty data file with an empty WAL is valid only if it was already
    // checkpointed. The length is otherwise useful to keep this argument from
    // becoming an accidental unused assertion in format tests.
    if wal_length == 0 && sb.checkpoint_lsn > Lsn::ZERO {
        return Ok(sb.checkpoint_lsn);
    }
    Ok(sb.checkpoint_lsn)
}

fn encode_page_id(page_id: Option<PageId>) -> u64 {
    page_id.map_or(NULL_PAGE_ID, PageId::get)
}

fn decode_page_id(value: u64) -> Option<PageId> {
    (value != NULL_PAGE_ID).then(|| PageId::new(value))
}

fn elapsed_nanos(started: Instant) -> u64 {
    started.elapsed().as_nanos().try_into().unwrap_or(u64::MAX)
}

fn read_exact_at<F: DurableFile>(file: &mut F, offset: u64, length: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0u8; length];
    let mut position = 0usize;
    while position < length {
        let count = file.read_at(
            offset
                .checked_add(position as u64)
                .ok_or_else(|| Error::corruption("Blink read offset overflows"))?,
            &mut bytes[position..],
        )?;
        if count == 0 {
            return Err(Error::Io(std::io::Error::new(
                ErrorKind::UnexpectedEof,
                "unexpected end of Blink database",
            )));
        }
        position += count;
    }
    Ok(bytes)
}

fn write_all_at<F: DurableFile>(file: &mut F, offset: u64, bytes: &[u8]) -> Result<()> {
    let mut position = 0usize;
    while position < bytes.len() {
        let count = file.write_at(
            offset
                .checked_add(position as u64)
                .ok_or_else(|| Error::invalid_input("Blink write offset overflows"))?,
            &bytes[position..],
        )?;
        if count == 0 {
            return Err(Error::Io(std::io::Error::new(
                ErrorKind::WriteZero,
                "Blink file returned a zero-byte write",
            )));
        }
        position += count;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum OwnedValue {
        Inline(Vec<u8>),
        Overflow { head: PageId, length: u64 },
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct LeafEntry {
        key: Vec<u8>,
        revision: Revision,
        value: Option<OwnedValue>,
    }

    impl LeafEntry {
        fn as_entry_ref(&self) -> LeafEntryRef<'_> {
            LeafEntryRef {
                key: &self.key,
                revision: self.revision,
                value: self.value.as_ref().map(|value| match value {
                    OwnedValue::Inline(bytes) => BlinkValueRef::Inline(bytes),
                    OwnedValue::Overflow { head, length } => BlinkValueRef::Overflow {
                        head: *head,
                        length: *length,
                    },
                }),
            }
        }

        fn from_entry_ref(entry: LeafEntryRef<'_>) -> Self {
            Self {
                key: entry.key.to_vec(),
                revision: entry.revision,
                value: entry.value.map(|value| match value {
                    BlinkValueRef::Inline(bytes) => OwnedValue::Inline(bytes.to_vec()),
                    BlinkValueRef::Overflow { head, length } => {
                        OwnedValue::Overflow { head, length }
                    }
                }),
            }
        }
    }

    fn pack_leaf(entries: &[LeafEntry]) -> LeafEntries {
        entries.iter().map(LeafEntry::as_entry_ref).collect()
    }

    fn logical_entries(entries: &LeafEntries) -> Vec<LeafEntry> {
        entries.iter().map(LeafEntry::from_entry_ref).collect()
    }

    fn find_entry<S: ReadPageSource>(state: &S, key: &[u8]) -> Result<Option<LeafEntry>> {
        let mut corrections = 0;
        let (page, index) = find_entry_with_metrics(state, key, &mut corrections)?;
        let BlinkPage::Leaf { entries, .. } = page.as_ref() else {
            return Err(Error::corruption("Blink route ended at non-leaf"));
        };
        Ok(index.map(|index| LeafEntry::from_entry_ref(entries.get(index))))
    }

    fn shared_pages<const N: usize>(
        pages: [(PageId, BlinkPage); N],
    ) -> BTreeMap<PageId, Arc<BlinkPage>> {
        pages
            .into_iter()
            .map(|(page_id, page)| (page_id, Arc::new(page)))
            .collect()
    }

    fn payload_sharing_test_state() -> (BlinkState, PageId, Vec<u8>, Vec<u8>) {
        let page_id = PageId::new(FIRST_DATA_PAGE);
        let first_key = DocumentKey::new(b"shared".to_vec(), b"first".to_vec()).encode();
        let second_key = DocumentKey::new(b"shared".to_vec(), b"second".to_vec()).encode();
        let state = BlinkState {
            pages: shared_pages([(
                page_id,
                BlinkPage::Leaf {
                    lsn: Lsn::new(12),
                    high_key: None,
                    right_sibling: None,
                    entries: pack_leaf(&[
                        LeafEntry {
                            key: first_key.clone(),
                            revision: Revision::new(11),
                            value: Some(OwnedValue::Inline(b"first-value".to_vec())),
                        },
                        LeafEntry {
                            key: second_key.clone(),
                            revision: Revision::new(12),
                            value: Some(OwnedValue::Inline(b"second-value".to_vec())),
                        },
                    ]),
                },
            )]),
            root_page_id: page_id,
            free_list_head: None,
            high_water_page_id: page_id,
            allow_page_reuse: false,
        };
        (state, page_id, first_key, second_key)
    }

    fn leaf_payload_pointers(page: &BlinkPage) -> Vec<(*const u8, Option<*const u8>)> {
        let BlinkPage::Leaf { entries, .. } = page else {
            unreachable!();
        };
        entries
            .iter()
            .map(|entry| {
                (
                    entry.key.as_ptr(),
                    match entry.value {
                        Some(BlinkValueRef::Inline(bytes)) => Some(bytes.as_ptr()),
                        _ => None,
                    },
                )
            })
            .collect()
    }

    #[test]
    fn leaf_page_clone_copies_packed_entries_into_its_own_buffer() {
        let (state, page_id, _, _) = payload_sharing_test_state();
        let base_page = state.pages.get(&page_id).unwrap();
        let base_logical = BlinkPage::clone(base_page);
        let base_pointers = leaf_payload_pointers(base_page);
        let cloned_page = BlinkPage::clone(base_page);
        assert_eq!(cloned_page, **base_page);
        let cloned_pointers = leaf_payload_pointers(&cloned_page);
        assert_eq!(base_pointers.len(), 2);
        for ((base_key, base_value), (cloned_key, cloned_value)) in
            base_pointers.iter().zip(&cloned_pointers)
        {
            assert_ne!(base_key, cloned_key);
            assert_ne!(base_value, cloned_value);
        }
        drop(cloned_page);
        assert_eq!(**base_page, base_logical);
        assert_eq!(leaf_payload_pointers(base_page), base_pointers);
    }

    fn packed_test_key(state: &mut u64) -> Vec<u8> {
        *state = splitmix_for_test(*state);
        let secondary_length = (*state % 40) as usize;
        let mut secondary = random_test_bytes(state, secondary_length);
        secondary.insert(0, (*state >> 8) as u8);
        DocumentKey::new(b"packed".to_vec(), secondary).encode()
    }

    fn packed_test_value(state: &mut u64, nearly_full: bool) -> Option<OwnedValue> {
        *state = splitmix_for_test(*state);
        match *state % 8 {
            0 => None,
            1 => Some(OwnedValue::Overflow {
                head: PageId::new(FIRST_DATA_PAGE + *state % 500),
                length: 600 + (*state >> 16) % 5_000,
            }),
            _ => {
                let limit = if nearly_full { 400 } else { 96 };
                let length = ((*state >> 20) % limit) as usize;
                Some(OwnedValue::Inline(random_test_bytes(state, length)))
            }
        }
    }

    fn reference_fits(entries: &[LeafEntry], high_key: Option<&[u8]>) -> bool {
        let refs = entries
            .iter()
            .map(LeafEntry::as_entry_ref)
            .collect::<Vec<_>>();
        encode_leaf_body(high_key, None, &refs).is_ok()
    }

    fn reference_split(entries: &[LeafEntry], high_key: Option<&[u8]>) -> usize {
        let middle = entries.len() / 2;
        (1..entries.len())
            .min_by_key(|index| {
                if reference_fits(&entries[..*index], Some(&entries[*index].key))
                    && reference_fits(&entries[*index..], high_key)
                {
                    (*index as isize - middle as isize).unsigned_abs()
                } else {
                    usize::MAX
                }
            })
            .unwrap_or(middle)
    }

    fn assert_packed_matches_reference(
        packed: &LeafEntries,
        reference: &[LeafEntry],
        high_key: Option<&[u8]>,
        probes: &[Vec<u8>],
        label: &str,
    ) {
        assert_eq!(logical_entries(packed), reference, "{label}: entries");
        assert_eq!(packed.len(), reference.len(), "{label}: length");
        for probe in probes {
            assert_eq!(
                packed.search(probe),
                reference.binary_search_by(|entry| entry.key.as_slice().cmp(probe)),
                "{label}: search"
            );
        }
        let refs = reference
            .iter()
            .map(LeafEntry::as_entry_ref)
            .collect::<Vec<_>>();
        let reference_body = encode_leaf_body(high_key, None, &refs);
        let page_id = PageId::new(FIRST_DATA_PAGE + 3);
        let page = BlinkPage::Leaf {
            lsn: Lsn::new(77),
            high_key: high_key.map(<[u8]>::to_vec),
            right_sibling: None,
            entries: packed.clone(),
        };
        let direct = encode_blink_page(page_id, &page);
        assert_eq!(reference_body.is_ok(), direct.is_ok(), "{label}: fits");
        assert_eq!(
            leaf_fits(packed.all(), high_key, None),
            reference_body.is_ok(),
            "{label}: leaf_fits"
        );
        if let (Ok(reference_body), Ok(direct)) = (reference_body, direct) {
            let reference_image = encode_page(
                PageHeader::new(PageType::Leaf, page_id, Lsn::new(77)),
                &reference_body,
            )
            .unwrap();
            assert_eq!(direct, reference_image, "{label}: encoded page bytes");
            let decoded = decode_blink_page(&direct, page_id).unwrap();
            assert_eq!(decoded, page, "{label}: decode round trip");
            if reference.len() > 1 {
                let split = choose_leaf_split(packed, high_key, None);
                assert_eq!(
                    split,
                    reference_split(reference, high_key),
                    "{label}: split"
                );
                let mut left = packed.clone();
                let right = left.split_off(split);
                assert_eq!(logical_entries(&left), reference[..split], "{label}: left");
                assert_eq!(
                    logical_entries(&right),
                    reference[split..],
                    "{label}: right"
                );
            }
        }
    }

    #[test]
    fn parallel_min_group_mutations_sends_small_groups_to_the_serial_executor() {
        let (mut serial, _) = phase_d_store(0, 400);
        let (mut adaptive, _) = phase_d_store(2, 400);
        adaptive.set_parallel_min_group_mutations(8);
        let distinct = keys_on_distinct_leaves(&adaptive, 400, 12);
        let small = distinct[..3]
            .iter()
            .map(|(index, _)| put_request(&[(*index, b0_value(*index, 5))]))
            .collect::<Vec<_>>();
        let large = distinct
            .iter()
            .map(|(index, _)| put_request(&[(*index, b0_value(*index, 6))]))
            .collect::<Vec<_>>();
        for group in [&small, &large] {
            let expected = serial.apply_transaction_group(group).unwrap();
            let actual = adaptive.apply_transaction_group(group).unwrap();
            assert_eq!(
                successful_commit_lsns(&actual),
                successful_commit_lsns(&expected)
            );
        }
        let metrics = adaptive.batch_metrics();
        assert_eq!(metrics.parallel_skipped_small_group, 1);
        assert_eq!(metrics.parallel_groups, 1);
        assert_eq!(serial.dirty_pages, adaptive.dirty_pages);
        assert_eq!(
            serial.scan(None, usize::MAX).unwrap(),
            adaptive.scan(None, usize::MAX).unwrap()
        );
        let (_, serial_wal) = serial.into_files();
        let (_, adaptive_wal) = adaptive.into_files();
        assert_eq!(serial_wal.unwrap().0, adaptive_wal.unwrap().0);
    }

    #[test]
    fn packed_leaf_matches_reference_model_randomized() {
        let mut compactions_seen = false;
        for seed in 0..400u64 {
            let mut state = seed ^ 0x7ac4_ed00;
            state = splitmix_for_test(state);
            let nearly_full = state % 3 == 0;
            let initial = match state % 5 {
                0 => 0,
                1 => 1 + (state >> 8) % 4,
                _ => (state >> 8) % if nearly_full { 12 } else { 40 },
            };
            let mut reference = BTreeMap::<Vec<u8>, LeafEntry>::new();
            for _ in 0..initial {
                let key = packed_test_key(&mut state);
                let value = packed_test_value(&mut state, nearly_full);
                reference.insert(
                    key.clone(),
                    LeafEntry {
                        key,
                        revision: Revision::new(1 + state % 1_000),
                        value,
                    },
                );
            }
            let mut reference = reference.into_values().collect::<Vec<_>>();
            let mut packed = pack_leaf(&reference);
            let high_key = (seed % 4 == 0)
                .then(|| DocumentKey::new(b"packed".to_vec(), vec![0xff; 8]).encode());
            for operation in 0..120u64 {
                state = splitmix_for_test(state);
                let revision = Revision::new(2_000 + operation);
                let garbage_before = packed.garbage_bytes();
                match state % 5 {
                    0 | 1 => {
                        let key = packed_test_key(&mut state);
                        let value = packed_test_value(&mut state, nearly_full);
                        let entry = LeafEntry {
                            key: key.clone(),
                            revision,
                            value,
                        };
                        match reference.binary_search_by(|existing| existing.key.cmp(&key)) {
                            Ok(index) => {
                                let old = reference[index].value.clone();
                                let returned = packed.replace(
                                    index,
                                    &key,
                                    revision,
                                    entry.as_entry_ref().value,
                                );
                                assert_eq!(
                                    returned,
                                    old.map(|old| match old {
                                        OwnedValue::Inline(_) => StoredValue::Inline,
                                        OwnedValue::Overflow { head, length } => {
                                            StoredValue::Overflow { head, length }
                                        }
                                    })
                                );
                                reference[index] = entry;
                            }
                            Err(index) => {
                                packed.insert(index, entry.as_entry_ref());
                                reference.insert(index, entry);
                            }
                        }
                    }
                    2 if !reference.is_empty() => {
                        let index = (state >> 8) as usize % reference.len();
                        let key = reference[index].key.clone();
                        let length = match &reference[index].value {
                            Some(OwnedValue::Inline(bytes)) if state & 0x100 == 0 => bytes.len(),
                            _ => ((state >> 24) % 200) as usize,
                        };
                        let value = OwnedValue::Inline(random_test_bytes(&mut state, length));
                        packed.replace(
                            index,
                            &key,
                            revision,
                            Some(BlinkValueRef::Inline(match &value {
                                OwnedValue::Inline(bytes) => bytes,
                                _ => unreachable!(),
                            })),
                        );
                        reference[index].revision = revision;
                        reference[index].value = Some(value);
                    }
                    3 if !reference.is_empty() => {
                        let index = (state >> 8) as usize % reference.len();
                        packed.set_revision(index, revision);
                        reference[index].revision = revision;
                    }
                    4 if !reference.is_empty() => {
                        let index = (state >> 8) as usize % reference.len();
                        packed.remove(index);
                        reference.remove(index);
                    }
                    _ => {}
                }
                if packed.garbage_bytes() == 0 && garbage_before > 0 {
                    compactions_seen = true;
                }
                let clone = packed.clone();
                let probes = reference
                    .iter()
                    .map(|entry| entry.key.clone())
                    .chain([packed_test_key(&mut state), Vec::new(), vec![0xff; 3]])
                    .collect::<Vec<_>>();
                let label = format!("seed {seed} operation {operation}");
                assert_packed_matches_reference(
                    &packed,
                    &reference,
                    high_key.as_deref(),
                    &probes,
                    &label,
                );
                assert_eq!(clone, packed);
                if !reference.is_empty() {
                    assert_ne!(clone.key(0).as_ptr(), packed.key(0).as_ptr());
                }
                if packed.len() > 60 {
                    reference.truncate(30);
                    packed = pack_leaf(&reference);
                }
            }
        }
        assert!(compactions_seen);
    }

    #[test]
    fn working_overlay_isolates_packed_entries_and_restamps() {
        let (base, page_id, first_key, second_key) = payload_sharing_test_state();
        let base_page = Arc::clone(base.pages.get(&page_id).unwrap());
        let base_pointers = leaf_payload_pointers(&base_page);
        let base_logical = BlinkPage::clone(&base_page);
        let mut working = WorkingBlinkState::new(&base, false);
        assert!(working.ensure_overlay_page(page_id).unwrap());
        assert_eq!(working.pages.get(&page_id).unwrap(), &*base_page);

        let provisional = Revision::new(99);
        let committed = Lsn::new(123);
        let BlinkPage::Leaf { entries, .. } = working.pages.get_mut(&page_id).unwrap() else {
            unreachable!();
        };
        let old = entries.replace(
            1,
            &second_key,
            provisional,
            Some(BlinkValueRef::Inline(b"overlay-value")),
        );
        assert_eq!(old, Some(StoredValue::Inline));
        let mutated_keys = BTreeSet::from([second_key.clone()]);
        working
            .pages
            .get_mut(&page_id)
            .unwrap()
            .restamp(provisional, committed, &mutated_keys);

        assert!(Arc::ptr_eq(base.pages.get(&page_id).unwrap(), &base_page));
        assert_eq!(*base_page, base_logical);
        assert_eq!(leaf_payload_pointers(&base_page), base_pointers);
        let BlinkPage::Leaf {
            entries: base_entries,
            ..
        } = &*base_page
        else {
            unreachable!();
        };
        let base_entries = logical_entries(base_entries);
        assert_eq!(base_entries[0].key, first_key);
        assert_eq!(base_entries[0].revision, Revision::new(11));
        assert_eq!(base_entries[1].revision, Revision::new(12));
        assert_eq!(
            base_entries[1].value,
            Some(OwnedValue::Inline(b"second-value".to_vec()))
        );

        let BlinkPage::Leaf {
            entries: overlay_entries,
            ..
        } = working.pages.get(&page_id).unwrap()
        else {
            unreachable!();
        };
        let overlay_entries = logical_entries(overlay_entries);
        assert_eq!(overlay_entries[1].revision, Revision::from(committed));
        assert_eq!(overlay_entries[1].key, second_key);
        assert_eq!(
            overlay_entries[1].value,
            Some(OwnedValue::Inline(b"overlay-value".to_vec()))
        );
        assert_eq!(overlay_entries[0], base_entries[0]);
    }

    fn layout_test_leaf_entry(index: u64, inline_value_len: usize) -> LeafEntry {
        LeafEntry {
            key: DocumentKey::new(b"layout".to_vec(), index.to_be_bytes().to_vec()).encode(),
            revision: Revision::new(index + 1),
            value: Some(OwnedValue::Inline(vec![0x5a; inline_value_len])),
        }
    }

    fn layout_test_internal_entry(index: u64) -> InternalEntry {
        InternalEntry {
            key: DocumentKey::new(b"layout".to_vec(), index.to_be_bytes().to_vec()).encode(),
            right_child: PageId::new(FIRST_DATA_PAGE + index + 1),
        }
    }

    fn assert_leaf_layout_matches_encoder(entries: &[LeafEntry], high_key: Option<&[u8]>) {
        let packed = pack_leaf(entries);
        let layout = leaf_body_layout(packed.all(), high_key);
        let refs = entries
            .iter()
            .map(LeafEntry::as_entry_ref)
            .collect::<Vec<_>>();
        let encoded = encode_leaf_body(high_key, None, &refs);
        assert_eq!(layout.is_ok(), encoded.is_ok());
        if let (Ok(layout), Ok(encoded)) = (layout, encoded) {
            assert_eq!(
                layout.slot_end,
                u16::from_le_bytes(encoded[10..12].try_into().unwrap()) as usize
            );
            assert_eq!(
                layout.records_start,
                u16::from_le_bytes(encoded[12..14].try_into().unwrap()) as usize
            );
            assert_eq!(
                layout.records_end,
                u16::from_le_bytes(encoded[18..20].try_into().unwrap()) as usize
            );
        }
    }

    fn assert_internal_layout_matches_encoder(
        leftmost_child: PageId,
        entries: &[InternalEntry],
        high_key: Option<&[u8]>,
    ) {
        let layout = internal_body_layout(leftmost_child, entries, high_key);
        let encoded = encode_internal_body(1, high_key, None, leftmost_child, entries);
        assert_eq!(layout.is_ok(), encoded.is_ok());
        if let (Ok(layout), Ok(encoded)) = (layout, encoded) {
            assert_eq!(
                layout.slot_end,
                u16::from_le_bytes(encoded[12..14].try_into().unwrap()) as usize
            );
            assert_eq!(
                layout.records_start,
                u16::from_le_bytes(encoded[14..16].try_into().unwrap()) as usize
            );
            assert_eq!(
                layout.records_end,
                u16::from_le_bytes(encoded[36..38].try_into().unwrap()) as usize
            );
        }
    }

    fn next_layout_random(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *state
    }

    fn differential_key(index: u64) -> Vec<u8> {
        DocumentKey::new(b"direct-encoding".to_vec(), index.to_be_bytes().to_vec()).encode()
    }

    fn assert_direct_reference_round_trip(
        page_id: PageId,
        page: BlinkPage,
        seed: u64,
        iteration: usize,
    ) {
        let direct = encode_blink_page(page_id, &page).unwrap_or_else(|error| {
            panic!(
                "seed={seed:#x} iteration={iteration} page_type={:?} entry_count={} direct encode failed: {error}",
                page.page_type(),
                blink_page_entry_count(&page)
            )
        });
        let reference = encode_blink_page_reference(page_id, &page).unwrap_or_else(|error| {
            panic!(
                "seed={seed:#x} iteration={iteration} page_type={:?} entry_count={} reference encode failed: {error}",
                page.page_type(),
                blink_page_entry_count(&page)
            )
        });
        assert_eq!(
            direct,
            reference,
            "seed={seed:#x} iteration={iteration} page_type={:?} entry_count={}",
            page.page_type(),
            blink_page_entry_count(&page)
        );
        assert_eq!(
            decode_blink_page(&direct, page_id).unwrap(),
            page,
            "seed={seed:#x} iteration={iteration} page_type={:?} entry_count={} round trip",
            page.page_type(),
            blink_page_entry_count(&page)
        );
    }

    fn blink_page_entry_count(page: &BlinkPage) -> usize {
        match page {
            BlinkPage::Leaf { entries, .. } => entries.len(),
            BlinkPage::Internal { entries, .. } => entries.len(),
            BlinkPage::Overflow { .. } | BlinkPage::Free { .. } => 0,
        }
    }

    #[test]
    fn direct_blink_encoding_matches_reference_for_required_shapes() {
        let page_id = PageId::new(FIRST_DATA_PAGE);
        let lsn = Lsn::new(42);
        let sibling = Some(PageId::new(FIRST_DATA_PAGE + 1));
        let mut cases = vec![
            BlinkPage::Leaf {
                lsn,
                high_key: None,
                right_sibling: None,
                entries: LeafEntries::default(),
            },
            BlinkPage::Leaf {
                lsn,
                high_key: None,
                right_sibling: None,
                entries: pack_leaf(&[layout_test_leaf_entry(1, 32)]),
            },
            BlinkPage::Leaf {
                lsn,
                high_key: Some(differential_key(80)),
                right_sibling: sibling,
                entries: pack_leaf(
                    &(0..48)
                        .map(|entry_index| layout_test_leaf_entry(entry_index, 12))
                        .collect::<Vec<_>>(),
                ),
            },
            BlinkPage::Leaf {
                lsn,
                high_key: None,
                right_sibling: None,
                entries: pack_leaf(&[LeafEntry {
                    key: differential_key(1),
                    revision: Revision::new(2),
                    value: None,
                }]),
            },
            BlinkPage::Leaf {
                lsn,
                high_key: None,
                right_sibling: None,
                entries: pack_leaf(&[LeafEntry {
                    key: differential_key(1),
                    revision: Revision::new(3),
                    value: Some(OwnedValue::Overflow {
                        head: PageId::new(FIRST_DATA_PAGE + 9),
                        length: 4096,
                    }),
                }]),
            },
            BlinkPage::Leaf {
                lsn,
                high_key: Some(differential_key(32)),
                right_sibling: sibling,
                entries: pack_leaf(
                    &(0..9)
                        .map(|entry_index| layout_test_leaf_entry(entry_index, 330))
                        .collect::<Vec<_>>(),
                ),
            },
            BlinkPage::Internal {
                lsn,
                level: 1,
                high_key: None,
                right_sibling: None,
                leftmost_child: page_id,
                entries: vec![layout_test_internal_entry(1)],
            },
            BlinkPage::Internal {
                lsn,
                level: 7,
                high_key: Some(differential_key(80)),
                right_sibling: sibling,
                leftmost_child: page_id,
                entries: (0..60).map(layout_test_internal_entry).collect(),
            },
            BlinkPage::Internal {
                lsn,
                level: 3,
                high_key: Some(differential_key(20)),
                right_sibling: sibling,
                leftmost_child: page_id,
                entries: (0u64..42)
                    .map(|entry_index| InternalEntry {
                        key: DocumentKey::new(vec![b'k'; 48], entry_index.to_be_bytes().to_vec())
                            .encode(),
                        right_child: PageId::new(FIRST_DATA_PAGE + entry_index + 1),
                    })
                    .collect(),
            },
            BlinkPage::Overflow {
                lsn,
                next: sibling,
                total_length: (BODY_SIZE - OVERFLOW_HEADER_SIZE) as u64,
                chunk: vec![0xa7; BODY_SIZE - OVERFLOW_HEADER_SIZE],
            },
            BlinkPage::Free { lsn, next: sibling },
        ];
        for (iteration, page) in cases.drain(..).enumerate() {
            assert_direct_reference_round_trip(page_id, page, 0, iteration);
        }
    }

    #[test]
    fn randomized_direct_blink_encoding_matches_reference() {
        let seed = 0x91d4_2c73_5a06_b8ef;
        let mut random_state = seed;
        let page_id = PageId::new(FIRST_DATA_PAGE + 40);
        for iteration in 0..800 {
            let lsn = Lsn::new(next_layout_random(&mut random_state));
            let page = match next_layout_random(&mut random_state) % 4 {
                0 => {
                    let entry_count = (next_layout_random(&mut random_state) % 20) as usize;
                    let entries = (0..entry_count)
                        .map(|entry_index| {
                            let random_value = next_layout_random(&mut random_state);
                            LeafEntry {
                                key: differential_key(entry_index as u64),
                                revision: Revision::new(random_value.max(1)),
                                value: match random_value % 3 {
                                    0 => None,
                                    1 => Some(OwnedValue::Inline(vec![
                                        random_value as u8;
                                        random_value as usize % 48
                                    ])),
                                    _ => Some(OwnedValue::Overflow {
                                        head: PageId::new(FIRST_DATA_PAGE + random_value % 200),
                                        length: random_value.max(1),
                                    }),
                                },
                            }
                        })
                        .collect::<Vec<LeafEntry>>();
                    let high_key = (next_layout_random(&mut random_state) & 1 == 0)
                        .then(|| differential_key(entry_count as u64 + 10));
                    BlinkPage::Leaf {
                        lsn,
                        high_key,
                        right_sibling: (next_layout_random(&mut random_state) & 1 == 0)
                            .then_some(PageId::new(FIRST_DATA_PAGE + 41)),
                        entries: pack_leaf(&entries),
                    }
                }
                1 => {
                    let entry_count = (next_layout_random(&mut random_state) % 50) as usize;
                    let entries = (0..entry_count)
                        .map(|entry_index| InternalEntry {
                            key: differential_key(entry_index as u64),
                            right_child: PageId::new(FIRST_DATA_PAGE + entry_index as u64 + 1),
                        })
                        .collect();
                    let high_key = (next_layout_random(&mut random_state) & 1 == 0)
                        .then(|| differential_key(entry_count as u64 + 10));
                    BlinkPage::Internal {
                        lsn,
                        level: (next_layout_random(&mut random_state) % 20 + 1) as u16,
                        high_key,
                        right_sibling: (next_layout_random(&mut random_state) & 1 == 0)
                            .then_some(PageId::new(FIRST_DATA_PAGE + 41)),
                        leftmost_child: PageId::new(FIRST_DATA_PAGE),
                        entries,
                    }
                }
                2 => {
                    let chunk_length = (next_layout_random(&mut random_state)
                        % (BODY_SIZE - OVERFLOW_HEADER_SIZE + 1) as u64)
                        as usize;
                    BlinkPage::Overflow {
                        lsn,
                        next: Some(PageId::new(FIRST_DATA_PAGE + 41)),
                        total_length: chunk_length as u64,
                        chunk: vec![next_layout_random(&mut random_state) as u8; chunk_length],
                    }
                }
                _ => BlinkPage::Free {
                    lsn,
                    next: Some(PageId::new(FIRST_DATA_PAGE + 41)),
                },
            };
            assert_direct_reference_round_trip(page_id, page, seed, iteration);
        }
    }

    #[test]
    fn leaf_layout_matches_encoder_acceptance() {
        let empty = Vec::new();
        assert_leaf_layout_matches_encoder(&empty, None);

        let single = vec![layout_test_leaf_entry(0, 8)];
        assert_leaf_layout_matches_encoder(&single, None);

        let mut inline_entries = (0..40)
            .map(|index| layout_test_leaf_entry(index, index as usize % 97))
            .collect::<Vec<_>>();
        assert_leaf_layout_matches_encoder(&inline_entries, None);
        assert_leaf_layout_matches_encoder(
            &inline_entries,
            Some(&layout_test_leaf_entry(41, 0).key),
        );

        inline_entries[3].value = None;
        inline_entries[7].value = Some(OwnedValue::Overflow {
            head: PageId::new(FIRST_DATA_PAGE + 20),
            length: 10_000,
        });
        assert_leaf_layout_matches_encoder(&inline_entries, None);

        let mut invalid_order = inline_entries.clone();
        invalid_order.swap(1, 2);
        assert_leaf_layout_matches_encoder(&invalid_order, None);
        assert_leaf_layout_matches_encoder(&inline_entries, Some(&[0xff]));

        let mut boundary_entry = layout_test_leaf_entry(0, 0);
        let high_key = DocumentKey::new(b"layout".to_vec(), 1u64.to_be_bytes().to_vec()).encode();
        let fixed_size = LEAF_HEADER_SIZE
            + SLOT_SIZE
            + high_key.len()
            + LEAF_RECORD_HEADER_SIZE
            + boundary_entry.key.len();
        let exact_value_len = BODY_SIZE - fixed_size;
        boundary_entry.value = Some(OwnedValue::Inline(vec![0x33; exact_value_len]));
        assert_leaf_layout_matches_encoder(std::slice::from_ref(&boundary_entry), Some(&high_key));
        assert!(
            leaf_body_layout(
                pack_leaf(std::slice::from_ref(&boundary_entry)).all(),
                Some(&high_key)
            )
            .is_ok()
        );
        boundary_entry.value = Some(OwnedValue::Inline(vec![0x33; exact_value_len + 1]));
        assert_leaf_layout_matches_encoder(std::slice::from_ref(&boundary_entry), Some(&high_key));
        assert!(
            leaf_body_layout(
                pack_leaf(std::slice::from_ref(&boundary_entry)).all(),
                Some(&high_key)
            )
            .is_err()
        );
    }

    #[test]
    fn internal_layout_matches_encoder_acceptance() {
        let leftmost_child = PageId::new(FIRST_DATA_PAGE);
        let single = vec![layout_test_internal_entry(0)];
        assert_internal_layout_matches_encoder(leftmost_child, &single, None);

        let many = (0..40).map(layout_test_internal_entry).collect::<Vec<_>>();
        let high_key = layout_test_internal_entry(41).key;
        assert_internal_layout_matches_encoder(leftmost_child, &many, Some(&high_key));

        let mut invalid_order = many.clone();
        invalid_order.swap(2, 3);
        assert_internal_layout_matches_encoder(leftmost_child, &invalid_order, None);
        assert_internal_layout_matches_encoder(PageId::new(1), &single, None);
        assert_internal_layout_matches_encoder(leftmost_child, &many, Some(&[0xff]));

        let mut boundary_entries = Vec::new();
        for index in 0..u64::MAX {
            let candidate = layout_test_internal_entry(index);
            boundary_entries.push(candidate);
            if internal_body_layout(leftmost_child, &boundary_entries, None).is_err() {
                boundary_entries.pop();
                break;
            }
        }
        assert!(!boundary_entries.is_empty());
        assert_internal_layout_matches_encoder(leftmost_child, &boundary_entries, None);
        assert!(internal_body_layout(leftmost_child, &boundary_entries, None).is_ok());
        boundary_entries.push(layout_test_internal_entry(boundary_entries.len() as u64));
        assert_internal_layout_matches_encoder(leftmost_child, &boundary_entries, None);
        assert!(internal_body_layout(leftmost_child, &boundary_entries, None).is_err());
    }

    #[test]
    fn randomized_page_layouts_match_encoder_acceptance() {
        let seed = 0x6f31_9a42_7c05_d8e1;
        let mut random_state = seed;
        for iteration in 0..500 {
            let entry_count = (next_layout_random(&mut random_state) % 96) as usize;
            let mut leaf_entries = (0..entry_count)
                .map(|index| {
                    let key_length = (next_layout_random(&mut random_state) % 96 + 1) as usize;
                    let partition = vec![b'p'; key_length];
                    let sort_key = ((index as u64) << 32
                        | next_layout_random(&mut random_state) as u32 as u64)
                        .to_be_bytes()
                        .to_vec();
                    let value_length = (next_layout_random(&mut random_state) % 5_500) as usize;
                    LeafEntry {
                        key: DocumentKey::new(partition, sort_key).encode(),
                        revision: Revision::new(index as u64 + 1),
                        value: match next_layout_random(&mut random_state) % 3 {
                            0 => None,
                            1 => Some(OwnedValue::Inline(vec![0x61; value_length])),
                            _ => Some(OwnedValue::Overflow {
                                head: PageId::new(FIRST_DATA_PAGE + index as u64),
                                length: value_length as u64 + 1,
                            }),
                        },
                    }
                })
                .collect::<Vec<_>>();
            leaf_entries.sort_by(|left, right| left.key.cmp(&right.key));
            if iteration % 7 == 0 && leaf_entries.len() > 1 {
                leaf_entries.swap(0, 1);
            }
            let high_key = (iteration % 2 == 0).then(|| {
                DocumentKey::new(
                    b"layout-high".to_vec(),
                    (iteration as u64).to_be_bytes().to_vec(),
                )
                .encode()
            });
            let packed = pack_leaf(&leaf_entries);
            let leaf_layout = leaf_body_layout(packed.all(), high_key.as_deref());
            let leaf_refs = leaf_entries
                .iter()
                .map(LeafEntry::as_entry_ref)
                .collect::<Vec<_>>();
            let leaf_encoded = encode_leaf_body(high_key.as_deref(), None, &leaf_refs);
            assert_eq!(
                leaf_layout.is_ok(),
                leaf_encoded.is_ok(),
                "leaf seed={seed:#x} iteration={iteration} entry_count={}",
                leaf_entries.len()
            );

            let mut internal_entries = (0..entry_count)
                .map(|index| {
                    let key_length = (next_layout_random(&mut random_state) % 96 + 1) as usize;
                    let partition = vec![b'q'; key_length];
                    let sort_key = ((index as u64) << 32
                        | next_layout_random(&mut random_state) as u32 as u64)
                        .to_be_bytes()
                        .to_vec();
                    InternalEntry {
                        key: DocumentKey::new(partition, sort_key).encode(),
                        right_child: PageId::new(FIRST_DATA_PAGE + index as u64 + 1),
                    }
                })
                .collect::<Vec<_>>();
            internal_entries.sort_by(|left, right| left.key.cmp(&right.key));
            if iteration % 9 == 0 && internal_entries.len() > 1 {
                internal_entries.swap(0, 1);
            }
            let internal_high_key = (iteration % 2 == 1).then(|| {
                DocumentKey::new(
                    b"internal-high".to_vec(),
                    (iteration as u64).to_be_bytes().to_vec(),
                )
                .encode()
            });
            let leftmost_child = if iteration % 11 == 0 {
                PageId::new(1)
            } else {
                PageId::new(FIRST_DATA_PAGE)
            };
            let internal_layout = internal_body_layout(
                leftmost_child,
                &internal_entries,
                internal_high_key.as_deref(),
            );
            let internal_encoded = encode_internal_body(
                1,
                internal_high_key.as_deref(),
                None,
                leftmost_child,
                &internal_entries,
            );
            assert_eq!(
                internal_layout.is_ok(),
                internal_encoded.is_ok(),
                "internal seed={seed:#x} iteration={iteration} entry_count={}",
                internal_entries.len()
            );
        }
    }

    #[derive(Default)]
    struct MemoryFile(Vec<u8>);

    impl DurableFile for MemoryFile {
        fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
            let offset = offset as usize;
            if offset >= self.0.len() {
                return Ok(0);
            }
            let count = buffer.len().min(self.0.len() - offset);
            buffer[..count].copy_from_slice(&self.0[offset..offset + count]);
            Ok(count)
        }
        fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<usize> {
            let offset = offset as usize;
            let end = offset + bytes.len();
            if self.0.len() < end {
                self.0.resize(end, 0);
            }
            self.0[offset..end].copy_from_slice(bytes);
            Ok(bytes.len())
        }
        fn len(&self) -> Result<u64> {
            Ok(self.0.len() as u64)
        }
        fn set_len(&mut self, length: u64) -> Result<()> {
            self.0.resize(length as usize, 0);
            Ok(())
        }
        fn sync_data(&mut self) -> Result<()> {
            Ok(())
        }
        fn sync_all(&mut self) -> Result<()> {
            Ok(())
        }
    }

    fn planned_store() -> BlinkStore<MemoryFile, MemoryFile> {
        let mut store = BlinkStore::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            DatabaseConfig::default(),
        )
        .unwrap();
        store.enable_planned_execution();
        store
    }

    fn parallel_store() -> BlinkStore<MemoryFile, MemoryFile> {
        let mut store = BlinkStore::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            DatabaseConfig::default(),
        )
        .unwrap();
        store.enable_parallel_execution(2).unwrap();
        store
    }

    fn catalog_test_state(high_water_page_id: u64) -> BlinkState {
        BlinkState {
            pages: (FIRST_DATA_PAGE..=high_water_page_id)
                .map(|raw_page_id| {
                    (
                        PageId::new(raw_page_id),
                        Arc::new(BlinkPage::Free {
                            lsn: Lsn::ZERO,
                            next: None,
                        }),
                    )
                })
                .collect(),
            root_page_id: PageId::new(FIRST_DATA_PAGE),
            free_list_head: None,
            high_water_page_id: PageId::new(high_water_page_id),
            allow_page_reuse: true,
        }
    }

    fn catalog_test_superblock(generation: u64, high_water_page_id: u64) -> BlinkSuperblock {
        let mut superblock =
            BlinkSuperblock::new(&DatabaseConfig::default(), PageId::new(FIRST_DATA_PAGE));
        superblock.generation = generation;
        superblock.high_water_page_id = PageId::new(high_water_page_id);
        superblock
    }

    #[test]
    fn page_catalog_lookup_crosses_chunk_boundaries() {
        let state = catalog_test_state(128);
        let publisher = GenerationPublisher::new(&state, 1).unwrap();
        let pin = publisher.pin();
        for raw_page_id in [63, 64, 65, 127, 128] {
            assert!(
                pin.generation
                    .catalog
                    .get(PageId::new(raw_page_id))
                    .is_some()
            );
        }
        assert!(pin.generation.catalog.get(PageId::new(1)).is_none());
    }

    #[test]
    fn prepare_delta_reuses_untouched_catalog_chunks() {
        let state = catalog_test_state(191);
        let publisher = GenerationPublisher::new(&state, 1).unwrap();
        let old_pin = publisher.pin();
        let mut updated = state.clone();
        let changed_page_id = PageId::new(65);
        updated.pages.insert(
            changed_page_id,
            Arc::new(BlinkPage::Free {
                lsn: Lsn::new(2),
                next: None,
            }),
        );
        let dirty = BTreeSet::from([changed_page_id]);
        let (generation, timing) = publisher
            .prepare_delta(&updated, &catalog_test_superblock(2, 191), &dirty)
            .unwrap();
        assert_eq!(timing.catalog_chunk_clones, 1);
        assert!(!Arc::ptr_eq(
            &old_pin.generation.catalog.chunks[1],
            &generation.catalog.chunks[1]
        ));
        assert!(Arc::ptr_eq(
            &old_pin.generation.catalog.chunks[0],
            &generation.catalog.chunks[0]
        ));
        assert!(Arc::ptr_eq(
            &old_pin.generation.catalog.chunks[2],
            &generation.catalog.chunks[2]
        ));
    }

    #[test]
    fn prepare_delta_clones_each_dirty_chunk_once() {
        let state = catalog_test_state(191);
        let publisher = GenerationPublisher::new(&state, 1).unwrap();
        let mut updated = state.clone();
        let same_chunk_ids = [PageId::new(65), PageId::new(66), PageId::new(67)];
        for page_id in same_chunk_ids {
            updated.pages.insert(
                page_id,
                Arc::new(BlinkPage::Free {
                    lsn: Lsn::new(2),
                    next: None,
                }),
            );
        }
        let dirty = same_chunk_ids.into_iter().collect::<BTreeSet<_>>();
        let (_, timing) = publisher
            .prepare_delta(&updated, &catalog_test_superblock(2, 191), &dirty)
            .unwrap();
        assert_eq!(timing.catalog_chunk_clones, 1);

        let different_chunks = [PageId::new(3), PageId::new(65), PageId::new(129)];
        let dirty = different_chunks.into_iter().collect::<BTreeSet<_>>();
        let (_, timing) = publisher
            .prepare_delta(&updated, &catalog_test_superblock(2, 191), &dirty)
            .unwrap();
        assert_eq!(timing.catalog_chunk_clones, 3);
    }

    #[test]
    fn prepare_delta_extends_catalog_directory_and_rejects_missing_pages() {
        let state = catalog_test_state(63);
        let publisher = GenerationPublisher::new(&state, 1).unwrap();
        let mut extended = state.clone();
        for raw_page_id in 64..=65 {
            extended.pages.insert(
                PageId::new(raw_page_id),
                Arc::new(BlinkPage::Free {
                    lsn: Lsn::ZERO,
                    next: None,
                }),
            );
        }
        extended.high_water_page_id = PageId::new(65);
        let dirty = BTreeSet::from([PageId::new(64), PageId::new(65)]);
        let (generation, _) = publisher
            .prepare_delta(&extended, &catalog_test_superblock(2, 65), &dirty)
            .unwrap();
        assert_eq!(generation.catalog.chunks.len(), 2);
        assert!(generation.catalog.get(PageId::new(64)).is_some());
        assert!(generation.catalog.get(PageId::new(65)).is_some());
        assert!(generation.catalog.get(PageId::new(63)).is_some());

        let missing_dirty = BTreeSet::from([PageId::new(64)]);
        assert!(
            publisher
                .prepare_delta(&extended, &catalog_test_superblock(2, 65), &missing_dirty,)
                .is_err()
        );
    }

    #[test]
    fn persistent_parallel_pool_reuses_worker_threads_across_execute_cycles() {
        let pool = ParallelWorkerPool::new(2).unwrap();
        let key = DocumentKey::new(b"pool".to_vec(), b"key".to_vec());
        let plan = Arc::new(BatchPlan {
            transactions: vec![PhysicalTransactionPlan {
                fifo_position: 0,
                provisional_revision: ProvisionalRevisionToken {
                    transaction_position: 0,
                    ordinal: 1,
                },
                mutations: vec![PlannedMutation {
                    write: PlannedWrite::Put(Arc::from(&b"value"[..])),
                    encoded_key: key.encode(),
                    route_hint: RouteHint {
                        leaf_id: PageId::new(2),
                    },
                }],
                mutated_key_set: BTreeSet::new(),
                dependency_metadata: DependencyMetadata::default(),
            }],
            ..BatchPlan::default()
        });
        let commit_lsns: Arc<[Lsn]> = vec![Lsn::new(1)].into();
        let make_jobs = || {
            (0..64u64)
                .map(|leaf_index| LeafChainJob {
                    leaf_id: PageId::new(2 + leaf_index),
                    initial_page: Arc::new(BlinkPage::Leaf {
                        lsn: Lsn::ZERO,
                        high_key: None,
                        right_sibling: None,
                        entries: LeafEntries::default(),
                    }),
                    base_image: None,
                    chain_entry: None,
                    steps: vec![(0, 0)],
                    plan: Arc::clone(&plan),
                    commit_lsns: Arc::clone(&commit_lsns),
                    fault: None,
                })
                .collect::<Vec<_>>()
        };
        let first_run = pool.execute(make_jobs()).unwrap();
        let second_run = pool.execute(make_jobs()).unwrap();
        let mut first_workers = first_run.worker_threads;
        let mut second_workers = second_run.worker_threads;
        first_workers.sort_by_key(|(worker_index, _)| *worker_index);
        second_workers.sort_by_key(|(worker_index, _)| *worker_index);
        assert_eq!(first_workers, second_workers);
        assert_eq!(first_run.lanes, 3);
        assert_eq!(second_run.lanes, 3);
        assert_eq!(first_run.outcomes.len(), 64);
        assert_eq!(second_run.outcomes.len(), 64);
        let mut leaves = first_run
            .outcomes
            .iter()
            .map(|outcome| match outcome {
                LeafChainOutcome::Prepared(result) => result.leaf_id,
                LeafChainOutcome::Fallback { .. } => panic!("unexpected fallback"),
            })
            .collect::<Vec<_>>();
        leaves.sort();
        leaves.dedup();
        assert_eq!(leaves.len(), 64);
    }

    fn wide_key(index: u64) -> DocumentKey {
        let mut primary = vec![0x51; 280];
        primary.extend_from_slice(&index.to_be_bytes());
        DocumentKey::new(primary, vec![0x61; 280])
    }

    struct FailOnOccurrence {
        point: &'static str,
        remaining: usize,
    }

    impl FaultInjector for FailOnOccurrence {
        fn hit(&mut self, point: &str) -> Result<()> {
            if point == self.point {
                self.remaining = self.remaining.saturating_sub(1);
                if self.remaining == 0 {
                    return Err(Error::recovery(format!("injected failure at {point}")));
                }
            }
            Ok(())
        }
    }

    fn reference_apply(
        states: &mut BTreeMap<DocumentKey, RevisionState>,
        request: &TransactionRequest,
        commit_lsn: Lsn,
    ) -> bool {
        if request.validate().is_err() {
            return false;
        }
        for condition in &request.conditions {
            let observed = states
                .get(condition.key())
                .cloned()
                .unwrap_or_else(|| RevisionState::missing(Revision::ZERO));
            let matches = match condition {
                TransactionCondition::RevisionEquals {
                    expected_revision, ..
                } => observed.revision() == *expected_revision,
                TransactionCondition::Exists { .. } => !observed.is_missing(),
                TransactionCondition::NotExists { .. } => observed.is_missing(),
            };
            if !matches {
                return false;
            }
        }
        for mutation in &request.mutations {
            let state = match mutation {
                TransactionMutation::Put { value, .. } => {
                    RevisionState::present(value.clone(), Revision::from(commit_lsn))
                }
                TransactionMutation::Delete { .. } => {
                    RevisionState::missing(Revision::from(commit_lsn))
                }
            };
            states.insert(mutation.key().clone(), state);
        }
        true
    }

    fn successful_commit_lsns(results: &[Result<TransactionResult>]) -> Vec<Lsn> {
        results
            .iter()
            .map(|result| {
                result
                    .as_ref()
                    .expect("expected accepted transaction")
                    .commit_lsn
            })
            .collect()
    }

    #[test]
    fn experimental_superblock_is_not_baseline_format() {
        let config = DatabaseConfig::default();
        let sb = BlinkSuperblock::new(&config, PageId::new(FIRST_DATA_PAGE));
        let image = encode_blink_superblock(&sb).unwrap();
        assert!(crate::decode_superblock(&image).is_err());
        assert!(decode_blink_superblock_image(&image).is_ok());
    }

    #[test]
    fn high_key_right_link_correction_finds_stale_route() {
        let left = PageId::new(2);
        let right = PageId::new(3);
        let key = DocumentKey::new(b"p".to_vec(), b"z".to_vec());
        let left_key = DocumentKey::new(b"p".to_vec(), b"a".to_vec()).encode();
        let right_key = key.encode();
        let state = BlinkState {
            pages: shared_pages([
                (
                    left,
                    BlinkPage::Leaf {
                        lsn: Lsn::ZERO,
                        high_key: Some(right_key.clone()),
                        right_sibling: Some(right),
                        entries: pack_leaf(&[LeafEntry {
                            key: left_key,
                            revision: Revision::new(1),
                            value: Some(OwnedValue::Inline(vec![1])),
                        }]),
                    },
                ),
                (
                    right,
                    BlinkPage::Leaf {
                        lsn: Lsn::ZERO,
                        high_key: None,
                        right_sibling: None,
                        entries: pack_leaf(&[LeafEntry {
                            key: right_key,
                            revision: Revision::new(2),
                            value: Some(OwnedValue::Inline(vec![2])),
                        }]),
                    },
                ),
            ]),
            root_page_id: left,
            free_list_head: None,
            high_water_page_id: right,
            allow_page_reuse: true,
        };
        let mut generic_corrections = 0;
        let mut generic_visits = 0;
        let generic_leaf = find_leaf_with_metrics(
            &state,
            &key.encode(),
            &mut generic_corrections,
            Some(&mut generic_visits),
        )
        .unwrap();
        let mut borrowed_corrections = 0;
        let mut borrowed_visits = 0;
        let borrowed_leaf = find_leaf_in_blink_state_borrowed(
            &state,
            &key.encode(),
            &mut borrowed_corrections,
            &mut borrowed_visits,
        )
        .unwrap();
        assert_eq!(borrowed_leaf, generic_leaf);
        assert_eq!(borrowed_corrections, generic_corrections);
        assert_eq!(borrowed_visits, generic_visits);
        assert_eq!(borrowed_corrections, 1);
        assert_eq!(borrowed_visits, 2);
        let mut corrections = 0;
        let value = read_state(&state, &key, &mut corrections).unwrap();
        assert_eq!(value.value(), Some(&[2][..]));
        assert_eq!(corrections, 1);
    }

    #[test]
    fn borrowed_planner_routing_matches_generic_routing() {
        let page_id = |value| PageId::new(value);
        let single_leaf_id = page_id(2);
        let single_leaf_state = BlinkState {
            pages: shared_pages([(
                single_leaf_id,
                BlinkPage::Leaf {
                    lsn: Lsn::ZERO,
                    high_key: None,
                    right_sibling: None,
                    entries: LeafEntries::default(),
                },
            )]),
            root_page_id: single_leaf_id,
            free_list_head: None,
            high_water_page_id: single_leaf_id,
            allow_page_reuse: true,
        };
        let single_leaf_key = DocumentKey::new(b"single".to_vec(), b"leaf".to_vec()).encode();
        let mut generic_corrections = 0;
        let mut generic_visits = 0;
        let generic_leaf = find_leaf_with_metrics(
            &single_leaf_state,
            &single_leaf_key,
            &mut generic_corrections,
            Some(&mut generic_visits),
        )
        .unwrap();
        let mut borrowed_corrections = 0;
        let mut borrowed_visits = 0;
        let borrowed_leaf = find_leaf_in_blink_state_borrowed(
            &single_leaf_state,
            &single_leaf_key,
            &mut borrowed_corrections,
            &mut borrowed_visits,
        )
        .unwrap();
        assert_eq!(borrowed_leaf, generic_leaf);
        assert_eq!(borrowed_leaf, single_leaf_id);
        assert_eq!(borrowed_corrections, generic_corrections);
        assert_eq!(borrowed_visits, generic_visits);

        let leaf_ids = [page_id(2), page_id(3), page_id(4), page_id(5)];
        let left_internal_id = page_id(6);
        let right_internal_id = page_id(7);
        let root_id = page_id(8);
        let keys = [b"a".as_slice(), b"b", b"c", b"d"]
            .into_iter()
            .map(|sk| DocumentKey::new(b"p".to_vec(), sk.to_vec()).encode())
            .collect::<Vec<_>>();
        let make_leaf =
            |high_key: Option<Vec<u8>>, right_sibling: Option<PageId>| BlinkPage::Leaf {
                lsn: Lsn::ZERO,
                high_key,
                right_sibling,
                entries: LeafEntries::default(),
            };
        let state = BlinkState {
            pages: shared_pages([
                (
                    leaf_ids[0],
                    make_leaf(Some(keys[1].clone()), Some(leaf_ids[1])),
                ),
                (
                    leaf_ids[1],
                    make_leaf(Some(keys[2].clone()), Some(leaf_ids[2])),
                ),
                (
                    leaf_ids[2],
                    make_leaf(Some(keys[3].clone()), Some(leaf_ids[3])),
                ),
                (leaf_ids[3], make_leaf(None, None)),
                (
                    left_internal_id,
                    BlinkPage::Internal {
                        lsn: Lsn::ZERO,
                        level: 1,
                        high_key: Some(keys[2].clone()),
                        right_sibling: Some(right_internal_id),
                        leftmost_child: leaf_ids[0],
                        entries: vec![InternalEntry {
                            key: keys[1].clone(),
                            right_child: leaf_ids[1],
                        }],
                    },
                ),
                (
                    right_internal_id,
                    BlinkPage::Internal {
                        lsn: Lsn::ZERO,
                        level: 1,
                        high_key: None,
                        right_sibling: None,
                        leftmost_child: leaf_ids[2],
                        entries: vec![InternalEntry {
                            key: keys[3].clone(),
                            right_child: leaf_ids[3],
                        }],
                    },
                ),
                (
                    root_id,
                    BlinkPage::Internal {
                        lsn: Lsn::ZERO,
                        level: 2,
                        high_key: None,
                        right_sibling: None,
                        leftmost_child: left_internal_id,
                        entries: vec![InternalEntry {
                            key: keys[2].clone(),
                            right_child: right_internal_id,
                        }],
                    },
                ),
            ]),
            root_page_id: root_id,
            free_list_head: None,
            high_water_page_id: root_id,
            allow_page_reuse: true,
        };

        for (key, expected_leaf) in keys.iter().zip(leaf_ids) {
            let mut generic_corrections = 0;
            let mut generic_visits = 0;
            let generic_leaf = find_leaf_with_metrics(
                &state,
                key,
                &mut generic_corrections,
                Some(&mut generic_visits),
            )
            .unwrap();
            let mut borrowed_corrections = 0;
            let mut borrowed_visits = 0;
            let borrowed_leaf = find_leaf_in_blink_state_borrowed(
                &state,
                key,
                &mut borrowed_corrections,
                &mut borrowed_visits,
            )
            .unwrap();
            assert_eq!(generic_leaf, expected_leaf);
            assert_eq!(borrowed_leaf, generic_leaf);
            assert_eq!(borrowed_corrections, generic_corrections);
            assert_eq!(borrowed_visits, generic_visits);
        }

        let mut missing_state = state.clone();
        missing_state.pages.remove(&root_id);
        let mut generic_corrections = 0;
        let mut generic_visits = 0;
        let generic_error = find_leaf_with_metrics(
            &missing_state,
            &keys[0],
            &mut generic_corrections,
            Some(&mut generic_visits),
        )
        .unwrap_err();
        let mut borrowed_corrections = 0;
        let mut borrowed_visits = 0;
        let borrowed_error = find_leaf_in_blink_state_borrowed(
            &missing_state,
            &keys[0],
            &mut borrowed_corrections,
            &mut borrowed_visits,
        )
        .unwrap_err();
        assert_eq!(generic_error.to_string(), borrowed_error.to_string());

        let cyclic_page_id = page_id(9);
        let cyclic_state = BlinkState {
            pages: shared_pages([(
                cyclic_page_id,
                BlinkPage::Leaf {
                    lsn: Lsn::ZERO,
                    high_key: Some(keys[1].clone()),
                    right_sibling: Some(cyclic_page_id),
                    entries: LeafEntries::default(),
                },
            )]),
            root_page_id: cyclic_page_id,
            free_list_head: None,
            high_water_page_id: cyclic_page_id,
            allow_page_reuse: true,
        };
        let mut generic_corrections = 0;
        let mut generic_visits = 0;
        let generic_error = find_leaf_with_metrics(
            &cyclic_state,
            &keys[1],
            &mut generic_corrections,
            Some(&mut generic_visits),
        )
        .unwrap_err();
        let mut borrowed_corrections = 0;
        let mut borrowed_visits = 0;
        let borrowed_error = find_leaf_in_blink_state_borrowed(
            &cyclic_state,
            &keys[1],
            &mut borrowed_corrections,
            &mut borrowed_visits,
        )
        .unwrap_err();
        assert_eq!(generic_error.to_string(), borrowed_error.to_string());
        assert_eq!(borrowed_corrections, generic_corrections);
        assert_eq!(borrowed_visits, generic_visits);

        let mut corrupt_state = state;
        corrupt_state.pages.insert(
            root_id,
            Arc::new(BlinkPage::Overflow {
                lsn: Lsn::ZERO,
                next: None,
                total_length: 0,
                chunk: Vec::new(),
            }),
        );
        let mut generic_corrections = 0;
        let mut generic_visits = 0;
        let generic_error = find_leaf_with_metrics(
            &corrupt_state,
            &keys[0],
            &mut generic_corrections,
            Some(&mut generic_visits),
        )
        .unwrap_err();
        let mut borrowed_corrections = 0;
        let mut borrowed_visits = 0;
        let borrowed_error = find_leaf_in_blink_state_borrowed(
            &corrupt_state,
            &keys[0],
            &mut borrowed_corrections,
            &mut borrowed_visits,
        )
        .unwrap_err();
        assert_eq!(generic_error.to_string(), borrowed_error.to_string());
    }

    #[test]
    fn page_codec_rejects_version_checksum_and_reserved_bytes() {
        let page_id = PageId::new(2);
        let page = BlinkPage::Leaf {
            lsn: Lsn::ZERO,
            high_key: None,
            right_sibling: None,
            entries: LeafEntries::default(),
        };
        let image = encode_blink_page(page_id, &page).unwrap();
        assert!(decode_blink_page(&image, page_id).is_ok());
        let mut bad_version = image;
        bad_version[PAGE_HEADER_SIZE + 4..PAGE_HEADER_SIZE + 6]
            .copy_from_slice(&99u16.to_le_bytes());
        assert!(decode_blink_page(&bad_version, page_id).is_err());
        let mut bad_checksum = image;
        bad_checksum[100] ^= 1;
        assert!(decode_blink_page(&bad_checksum, page_id).is_err());
    }

    #[test]
    fn page_encoder_rejects_noncanonical_leaf_key() {
        let page_id = PageId::new(FIRST_DATA_PAGE);
        let page = BlinkPage::Leaf {
            lsn: Lsn::ZERO,
            high_key: None,
            right_sibling: None,
            entries: pack_leaf(&[LeafEntry {
                key: vec![0xff],
                revision: Revision::new(1),
                value: None,
            }]),
        };

        assert!(encode_blink_page(page_id, &page).is_err());
        assert!(encode_blink_page_reference(page_id, &page).is_err());
    }

    #[test]
    fn public_wal_append_rejects_malformed_blink_images() {
        let config = DatabaseConfig::default();
        let identity = WalIdentity::new(
            config.database_uuid,
            config.tenant_id,
            config.shard_id,
            config.shard_epoch,
        );
        let page_id = PageId::new(FIRST_DATA_PAGE);
        let page_lsn = Lsn::new(2);
        let page = BlinkPage::Leaf {
            lsn: page_lsn,
            high_key: None,
            right_sibling: None,
            entries: LeafEntries::default(),
        };
        let valid_image = encode_blink_page(page_id, &page).unwrap();

        let mut bad_checksum_image = valid_image;
        bad_checksum_image[100] ^= 1;
        let bad_checksum_commit = WalCommit {
            batch_id: 1,
            commit_lsn: page_lsn,
            pages: vec![WalPageImage {
                page_id,
                image: bad_checksum_image,
            }],
        };

        let wrong_page_id_commit = WalCommit {
            batch_id: 1,
            commit_lsn: page_lsn,
            pages: vec![WalPageImage {
                page_id: PageId::new(FIRST_DATA_PAGE + 1),
                image: valid_image,
            }],
        };

        let mut malformed_body_image = valid_image;
        malformed_body_image[PAGE_HEADER_SIZE..PAGE_HEADER_SIZE + 4].copy_from_slice(b"NOPE");
        let page_header = decode_page_at(&valid_image, Some(page_id)).unwrap().header;
        finalize_encoded_page(page_header, &mut malformed_body_image).unwrap();
        let malformed_body_commit = WalCommit {
            batch_id: 1,
            commit_lsn: page_lsn,
            pages: vec![WalPageImage {
                page_id,
                image: malformed_body_image,
            }],
        };

        for commit in [
            bad_checksum_commit,
            wrong_page_id_commit,
            malformed_body_commit,
        ] {
            let mut wal = WalLog::open_with_page_image_format(
                MemoryFile::default(),
                identity.clone(),
                WalPageImageFormat::ExperimentalBlink,
            )
            .unwrap();
            assert!(wal.append_group(&[commit], None).is_err());
            assert_eq!(wal.committed_batch_count(), 0);
            assert_eq!(wal.last_commit_lsn(), None);
        }
    }

    #[test]
    fn trusted_wal_images_match_strict_blink_encoding() {
        let config = DatabaseConfig::default();
        let identity = WalIdentity::new(
            config.database_uuid,
            config.tenant_id,
            config.shard_id,
            config.shard_epoch,
        );
        let commit_lsn = Lsn::new(3);
        let page_id = PageId::new(FIRST_DATA_PAGE);
        let page = BlinkPage::Leaf {
            lsn: commit_lsn,
            high_key: None,
            right_sibling: None,
            entries: LeafEntries::default(),
        };
        let superblock = BlinkSuperblock::new(&config, page_id);
        let commits = [WalCommit {
            batch_id: 1,
            commit_lsn,
            pages: vec![
                WalPageImage {
                    page_id,
                    image: encode_blink_page(page_id, &page).unwrap(),
                },
                WalPageImage {
                    page_id: PageId::new(1),
                    image: encode_blink_superblock(&superblock).unwrap(),
                },
            ],
        }];

        let mut strict_wal = WalLog::open_with_page_image_format(
            MemoryFile::default(),
            identity.clone(),
            WalPageImageFormat::ExperimentalBlink,
        )
        .unwrap();
        let strict_reports = strict_wal.append_group(&commits, None).unwrap();
        let strict_metrics = strict_wal.metrics().unwrap();
        let strict_next_lsn = strict_wal.next_lsn();
        let strict_next_batch_id = strict_wal.next_batch_id();
        let strict_bytes = strict_wal.into_file().0;

        let mut trusted_wal = WalLog::open_with_page_image_format(
            MemoryFile::default(),
            identity,
            WalPageImageFormat::ExperimentalBlink,
        )
        .unwrap();
        let trusted_reports = trusted_wal
            .append_group_trusted_internal(&commits, None)
            .unwrap();
        let trusted_metrics = trusted_wal.metrics().unwrap();

        assert_eq!(trusted_reports, strict_reports);
        assert_eq!(trusted_wal.next_lsn(), strict_next_lsn);
        assert_eq!(trusted_wal.next_batch_id(), strict_next_batch_id);
        assert_eq!(
            trusted_metrics.group_page_validations,
            if cfg!(debug_assertions) { 2 } else { 0 }
        );
        assert_eq!(strict_metrics.group_page_validations, 2);
        assert_eq!(trusted_wal.into_file().0, strict_bytes);
    }

    #[test]
    fn direct_wal_group_encoding_matches_reference_for_blink_page_shapes() {
        struct NoopWalInjector;

        impl FaultInjector for NoopWalInjector {
            fn hit(&mut self, _point: &str) -> Result<()> {
                Ok(())
            }
        }

        let config = DatabaseConfig::default();
        let identity = WalIdentity::new(
            config.database_uuid,
            config.tenant_id,
            config.shard_id,
            config.shard_epoch,
        );
        let commit_lsn = Lsn::new(5);
        let leaf_page_id = PageId::new(FIRST_DATA_PAGE + 20);
        let internal_page_id = PageId::new(FIRST_DATA_PAGE + 21);
        let overflow_page_id = PageId::new(FIRST_DATA_PAGE + 22);
        let superblock = BlinkSuperblock::new(&config, leaf_page_id);
        let commits = [WalCommit {
            batch_id: 1,
            commit_lsn,
            pages: vec![
                WalPageImage {
                    page_id: leaf_page_id,
                    image: encode_blink_page(
                        leaf_page_id,
                        &BlinkPage::Leaf {
                            lsn: commit_lsn,
                            high_key: None,
                            right_sibling: None,
                            entries: LeafEntries::default(),
                        },
                    )
                    .unwrap(),
                },
                WalPageImage {
                    page_id: internal_page_id,
                    image: encode_blink_page(
                        internal_page_id,
                        &BlinkPage::Internal {
                            lsn: commit_lsn,
                            level: 1,
                            high_key: None,
                            right_sibling: None,
                            leftmost_child: leaf_page_id,
                            entries: vec![layout_test_internal_entry(1)],
                        },
                    )
                    .unwrap(),
                },
                WalPageImage {
                    page_id: overflow_page_id,
                    image: encode_blink_page(
                        overflow_page_id,
                        &BlinkPage::Overflow {
                            lsn: commit_lsn,
                            next: None,
                            total_length: 16,
                            chunk: vec![0x61; 16],
                        },
                    )
                    .unwrap(),
                },
                WalPageImage {
                    page_id: PageId::new(1),
                    image: encode_blink_superblock(&superblock).unwrap(),
                },
            ],
        }];

        let mut direct_wal = WalLog::open_with_page_image_format(
            MemoryFile::default(),
            identity.clone(),
            WalPageImageFormat::ExperimentalBlink,
        )
        .unwrap();
        let direct_reports = direct_wal.append_group(&commits, None).unwrap();
        let direct_next_lsn = direct_wal.next_lsn();
        let direct_next_batch_id = direct_wal.next_batch_id();
        let direct_bytes = direct_wal.into_file().0;

        let mut reference_wal = WalLog::open_with_page_image_format(
            MemoryFile::default(),
            identity,
            WalPageImageFormat::ExperimentalBlink,
        )
        .unwrap();
        let mut injector = NoopWalInjector;
        let reference_reports = reference_wal
            .append_group(&commits, Some(&mut injector))
            .unwrap();
        assert_eq!(direct_reports, reference_reports);
        assert_eq!(direct_next_lsn, reference_wal.next_lsn());
        assert_eq!(direct_next_batch_id, reference_wal.next_batch_id());
        assert_eq!(direct_bytes, reference_wal.into_file().0);
    }

    #[test]
    fn serial_insert_reopen_and_checker() {
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config.clone(),
        )
        .unwrap();
        for index in 0usize..200 {
            store
                .put(
                    DocumentKey::new(b"p".to_vec(), index.to_be_bytes().to_vec()),
                    vec![index as u8; 32],
                )
                .unwrap();
        }
        assert!(store.split_metrics().leaf_splits > 0);
        store.check_invariants().unwrap();
        store.flush().unwrap();
        let (file, wal) = store.into_files();
        let mut reopened = BlinkStore::open_with_wal(file, wal.unwrap(), config).unwrap();
        assert_eq!(reopened.scan(None, 1_000).unwrap().len(), 200);
        reopened.check_invariants().unwrap();
    }

    #[test]
    fn transaction_group_preserves_logical_order_and_failed_atomicity() {
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        let a = DocumentKey::new(b"p".to_vec(), b"a".to_vec());
        let b = DocumentKey::new(b"p".to_vec(), b"b".to_vec());
        let c = DocumentKey::new(b"p".to_vec(), b"c".to_vec());
        let d = DocumentKey::new(b"p".to_vec(), b"d".to_vec());
        let results = store
            .apply_transaction_group(&[
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: a.clone(),
                        value: b"A".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    vec![TransactionCondition::Exists { key: a.clone() }],
                    vec![TransactionMutation::Put {
                        key: b.clone(),
                        value: b"B".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    vec![TransactionCondition::Exists { key: b.clone() }],
                    vec![TransactionMutation::Put {
                        key: c.clone(),
                        value: b"C".to_vec(),
                    }],
                ),
            ])
            .unwrap();
        assert!(results.iter().all(Result::is_ok));
        assert!(store.get(&a).unwrap().value().is_some());
        assert!(store.get(&b).unwrap().value().is_some());
        assert!(store.get(&c).unwrap().value().is_some());
        let failed = store
            .apply_transaction_group(&[
                TransactionRequest::new(
                    vec![TransactionCondition::NotExists { key: a.clone() }],
                    vec![TransactionMutation::Put {
                        key: d.clone(),
                        value: b"D".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    vec![TransactionCondition::Exists { key: c.clone() }],
                    vec![TransactionMutation::Put {
                        key: d.clone(),
                        value: b"D2".to_vec(),
                    }],
                ),
            ])
            .unwrap();
        assert!(failed[0].is_err());
        assert!(failed[1].is_ok());
        assert_eq!(store.get(&d).unwrap().value(), Some(&b"D2"[..]));
    }

    #[test]
    fn planned_admission_stages_dependencies_and_skips_failed_deltas() {
        let mut store = planned_store();
        let k1 = DocumentKey::new(b"planned".to_vec(), b"k1".to_vec());
        let k2 = DocumentKey::new(b"planned".to_vec(), b"k2".to_vec());
        let k3 = DocumentKey::new(b"planned".to_vec(), b"k3".to_vec());
        let results = store
            .apply_transaction_group(&[
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: k1.clone(),
                        value: b"one".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    vec![TransactionCondition::Exists { key: k1.clone() }],
                    vec![TransactionMutation::Put {
                        key: k2.clone(),
                        value: b"two".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    vec![TransactionCondition::Exists { key: k2.clone() }],
                    vec![TransactionMutation::Put {
                        key: k3.clone(),
                        value: b"three".to_vec(),
                    }],
                ),
            ])
            .unwrap();
        assert!(results.iter().all(Result::is_ok));
        assert_eq!(store.get(&k1).unwrap().value(), Some(&b"one"[..]));
        assert_eq!(store.get(&k2).unwrap().value(), Some(&b"two"[..]));
        assert_eq!(store.get(&k3).unwrap().value(), Some(&b"three"[..]));

        let failed_key = DocumentKey::new(b"planned".to_vec(), b"failed".to_vec());
        let after_failed_key = DocumentKey::new(b"planned".to_vec(), b"after".to_vec());
        let failed_results = store
            .apply_transaction_group(&[
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: failed_key.clone(),
                        value: b"first".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    vec![TransactionCondition::NotExists {
                        key: failed_key.clone(),
                    }],
                    vec![TransactionMutation::Put {
                        key: after_failed_key.clone(),
                        value: b"must-not-appear".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    vec![TransactionCondition::Exists {
                        key: failed_key.clone(),
                    }],
                    vec![TransactionMutation::Put {
                        key: after_failed_key.clone(),
                        value: b"after".to_vec(),
                    }],
                ),
            ])
            .unwrap();
        assert!(failed_results[0].is_ok());
        assert!(matches!(failed_results[1], Err(Error::Conflict(_))));
        assert!(failed_results[2].is_ok());
        assert_eq!(
            store.get(&after_failed_key).unwrap().value(),
            Some(&b"after"[..])
        );
        let metrics = store.batch_metrics();
        assert!(metrics.conflicted_transactions >= 1);
        assert!(metrics.dependency_edges >= 2);
        assert!(metrics.same_leaf_groups > 0);
        assert_eq!(metrics.full_state_clones, 0);
    }

    #[test]
    fn planned_group_uses_sparse_working_state() {
        let mut store = planned_store();
        for index in 0..512u64 {
            store
                .put(
                    DocumentKey::new(b"sparse".to_vec(), index.to_be_bytes().to_vec()),
                    vec![index as u8; 24],
                )
                .unwrap();
        }
        let base_page_count = store.state.pages.len();
        assert!(base_page_count > 20);
        let working = WorkingBlinkState::new(&store.state, false);
        assert!(working.pages.is_empty());
        assert_eq!(working.base.pages.len(), base_page_count);
        drop(working);

        let metrics_before = store.batch_metrics();
        store
            .apply_transaction_group(&[TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: DocumentKey::new(b"sparse".to_vec(), 64u64.to_be_bytes().to_vec()),
                    value: b"changed".to_vec(),
                }],
            )])
            .unwrap();
        let metrics_after = store.batch_metrics();
        assert_eq!(
            metrics_after.full_state_clones - metrics_before.full_state_clones,
            0
        );
        assert_eq!(
            metrics_after.state_clone_nanos - metrics_before.state_clone_nanos,
            0
        );

        let base = &store.state;
        let mut working = WorkingBlinkState::new(base, false);
        let mut dirty = BTreeSet::new();
        let mut split_metrics = BlinkSplitMetrics::default();
        apply_mutation(
            &mut working,
            &mut dirty,
            &mut split_metrics,
            &TransactionMutation::Put {
                key: DocumentKey::new(b"sparse".to_vec(), 64u64.to_be_bytes().to_vec()),
                value: b"overlay".to_vec(),
            },
            Revision::new(999),
        )
        .unwrap();
        assert!(working.pages.len() <= 3);
        assert!(working.pages.len() < base_page_count / 4);
        assert!(working.pages.keys().all(|page_id| dirty.contains(page_id)));
        assert_eq!(base.pages.len(), base_page_count);
    }

    #[test]
    fn planned_same_leaf_chain_preserves_wal_boundaries_and_revisions() {
        let mut store = planned_store();
        let key = DocumentKey::new(b"chain".to_vec(), b"key".to_vec());
        let other_key = DocumentKey::new(b"chain".to_vec(), b"other".to_vec());
        let before = store.batch_metrics();
        let results = store
            .apply_transaction_group(&[
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: key.clone(),
                        value: b"put".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Delete { key: key.clone() }],
                ),
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: key.clone(),
                        value: b"final".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: other_key.clone(),
                        value: b"other".to_vec(),
                    }],
                ),
            ])
            .unwrap();
        let first_lsn = results[0].as_ref().unwrap().commit_lsn;
        let second_lsn = results[1].as_ref().unwrap().commit_lsn;
        let third_lsn = results[2].as_ref().unwrap().commit_lsn;
        assert!(first_lsn < second_lsn && second_lsn < third_lsn);
        assert_eq!(
            store.get(&key).unwrap(),
            RevisionState::present(b"final", third_lsn.into())
        );
        assert_eq!(
            store.get(&other_key).unwrap().revision(),
            results[3].as_ref().unwrap().commit_lsn.into()
        );
        let metrics = store.batch_metrics();
        assert_eq!(metrics.full_state_clones - before.full_state_clones, 0);
        assert_eq!(metrics.leaf_load_clones - before.leaf_load_clones, 1);
        assert_eq!(metrics.leaf_entries_clones - before.leaf_entries_clones, 0);
        assert_eq!(
            metrics.leaf_entries_clone_nanos - before.leaf_entries_clone_nanos,
            0
        );
        assert_eq!(metrics.leaf_install_clones - before.leaf_install_clones, 0);
        assert_eq!(
            metrics.leaf_install_clone_nanos - before.leaf_install_clone_nanos,
            0
        );
        assert_eq!(
            metrics.cached_refresh_clones - before.cached_refresh_clones,
            0
        );
        assert_eq!(
            metrics.physical_cached_refresh_nanos - before.physical_cached_refresh_nanos,
            0
        );
        assert!(metrics.same_leaf_groups > before.same_leaf_groups);
        assert!(metrics.coalesced_mutations > before.coalesced_mutations);
        assert!(metrics.leaf_loads < metrics.mutations_planned);
        let committed = store.wal.as_mut().unwrap().committed_batches_on_disk();
        assert_eq!(committed.len(), 4);
        assert!(committed.iter().all(|batch| !batch.pages.is_empty()));
        let encoded_key = key.encode();
        for (transaction_index, expected_revision) in
            [first_lsn, second_lsn, third_lsn].into_iter().enumerate()
        {
            let page = committed[transaction_index]
                .pages
                .iter()
                .filter_map(|image| decode_blink_page(&image.image, image.page_id).ok())
                .find_map(|page| match page {
                    BlinkPage::Leaf { entries, .. }
                        if entries
                            .iter()
                            .any(|entry| entry.key.as_ref() == encoded_key.as_slice()) =>
                    {
                        Some(entries)
                    }
                    _ => None,
                })
                .unwrap();
            let entry = LeafEntry::from_entry_ref(
                page.iter()
                    .find(|entry| entry.key == encoded_key.as_slice())
                    .unwrap(),
            );
            assert_eq!(entry.revision, expected_revision.into());
            if transaction_index == 0 {
                assert_eq!(
                    entry.value,
                    Some(OwnedValue::Inline(Vec::from(&b"put"[..])))
                );
            } else if transaction_index == 1 {
                assert_eq!(entry.value, None);
            } else {
                assert_eq!(
                    entry.value,
                    Some(OwnedValue::Inline(Vec::from(&b"final"[..])))
                );
            }
        }
        store.check_invariants().unwrap();
    }

    #[test]
    fn planned_metadata_stable_transactions_elide_superblocks_and_recover() {
        let config = DatabaseConfig::default();
        let mut store = planned_store();
        let keys = (0u64..4)
            .map(|position| DocumentKey::new(b"stable".to_vec(), position.to_be_bytes().to_vec()))
            .collect::<Vec<_>>();
        for key in &keys {
            store.put(key.clone(), b"before".to_vec()).unwrap();
        }
        let root_page_id = store.current_superblock.root_page_id;
        let free_list_head = store.current_superblock.free_list_head;
        let high_water_page_id = store.current_superblock.high_water_page_id;
        let before_metrics = store.batch_metrics();
        let results = store
            .apply_transaction_group(
                &keys
                    .iter()
                    .enumerate()
                    .map(|(position, key)| {
                        TransactionRequest::new(
                            Vec::new(),
                            vec![TransactionMutation::Put {
                                key: key.clone(),
                                value: vec![position as u8 + 1],
                            }],
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        let batches = store.wal.as_mut().unwrap().committed_batches_on_disk();
        assert_eq!(batches.len(), 8);
        for batch in &batches[4..] {
            assert_eq!(batch.pages.len(), 1);
            assert!(
                batch
                    .pages
                    .iter()
                    .all(|image| image.page_id.get() >= FIRST_DATA_PAGE)
            );
        }
        assert_eq!(store.current_superblock.root_page_id, root_page_id);
        assert_eq!(store.current_superblock.free_list_head, free_list_head);
        assert_eq!(
            store.current_superblock.high_water_page_id,
            high_water_page_id
        );
        let metrics = store.batch_metrics();
        assert_eq!(
            metrics.superblock_images_emitted - before_metrics.superblock_images_emitted,
            0
        );
        assert_eq!(
            metrics.superblock_images_elided - before_metrics.superblock_images_elided,
            4
        );
        for (position, key) in keys.iter().enumerate() {
            let state = store.get(key).unwrap();
            assert_eq!(state.value(), Some(&[position as u8 + 1][..]));
            assert_eq!(
                state.revision(),
                results[position].as_ref().unwrap().commit_lsn.into()
            );
        }
        let (data, wal) = store.into_files();
        let mut reopened = BlinkStore::open_with_wal(data, wal.unwrap(), config).unwrap();
        for (position, key) in keys.iter().enumerate() {
            let state = reopened.get(key).unwrap();
            assert_eq!(state.value(), Some(&[position as u8 + 1][..]));
            assert_eq!(
                state.revision(),
                results[position].as_ref().unwrap().commit_lsn.into()
            );
        }
        reopened.check_invariants().unwrap();
    }

    #[test]
    fn planned_structural_transactions_emit_superblocks_and_later_updates_elide() {
        let config = DatabaseConfig::default();
        let mut store = planned_store();
        let replacement_key = DocumentKey::new(b"structure".to_vec(), b"replacement".to_vec());
        store.put(replacement_key.clone(), b"old".to_vec()).unwrap();
        let replacement_batch = store.wal.as_ref().unwrap().committed_batch_count();
        store.put(replacement_key.clone(), b"new".to_vec()).unwrap();
        let replacement =
            &store.wal.as_mut().unwrap().committed_batches_on_disk()[replacement_batch];
        assert_eq!(replacement.pages.len(), 1);

        let insert_key = DocumentKey::new(b"structure".to_vec(), b"insert".to_vec());
        let insert_batch = store.wal.as_ref().unwrap().committed_batch_count();
        store.put(insert_key.clone(), b"value".to_vec()).unwrap();
        let insert = &store.wal.as_mut().unwrap().committed_batches_on_disk()[insert_batch];
        assert_eq!(insert.pages.len(), 1);

        let overflow_key = DocumentKey::new(b"structure".to_vec(), b"overflow".to_vec());
        let overflow_batch = store.wal.as_ref().unwrap().committed_batch_count();
        store.put(overflow_key.clone(), vec![7; 2_000]).unwrap();
        let overflow = &store.wal.as_mut().unwrap().committed_batches_on_disk()[overflow_batch];
        assert!(
            overflow
                .pages
                .iter()
                .any(|image| image.page_id.get() < FIRST_DATA_PAGE)
        );

        let freed_overflow_batch = store.wal.as_ref().unwrap().committed_batch_count();
        store.delete(overflow_key.clone()).unwrap();
        let freed_overflow =
            &store.wal.as_mut().unwrap().committed_batches_on_disk()[freed_overflow_batch];
        assert!(
            freed_overflow
                .pages
                .iter()
                .any(|image| image.page_id.get() < FIRST_DATA_PAGE)
        );

        let reused_overflow_key =
            DocumentKey::new(b"structure".to_vec(), b"overflow-reused".to_vec());
        let reused_overflow_batch = store.wal.as_ref().unwrap().committed_batch_count();
        store
            .put(reused_overflow_key.clone(), vec![9; 2_000])
            .unwrap();
        let reused_overflow =
            &store.wal.as_mut().unwrap().committed_batches_on_disk()[reused_overflow_batch];
        assert!(
            reused_overflow
                .pages
                .iter()
                .any(|image| image.page_id.get() < FIRST_DATA_PAGE)
        );

        let mut observed_leaf_split = false;
        let mut observed_root_split = false;
        for position in 0..800u64 {
            let prior_metrics = store.split_metrics();
            let prior_batch_count = store.wal.as_ref().unwrap().committed_batch_count();
            store
                .put(
                    DocumentKey::new(b"split".to_vec(), position.to_be_bytes().to_vec()),
                    vec![position as u8; 32],
                )
                .unwrap();
            let next_metrics = store.split_metrics();
            if next_metrics.leaf_splits == prior_metrics.leaf_splits
                && next_metrics.root_splits == prior_metrics.root_splits
            {
                continue;
            }
            let batch = &store.wal.as_mut().unwrap().committed_batches_on_disk()[prior_batch_count];
            if next_metrics.leaf_splits > prior_metrics.leaf_splits {
                observed_leaf_split = true;
                assert!(
                    batch
                        .pages
                        .iter()
                        .any(|image| image.page_id.get() < FIRST_DATA_PAGE)
                );
            }
            if next_metrics.root_splits > prior_metrics.root_splits {
                observed_root_split = true;
                assert!(
                    batch
                        .pages
                        .iter()
                        .any(|image| image.page_id.get() < FIRST_DATA_PAGE)
                );
                break;
            }
        }
        assert!(observed_leaf_split);
        assert!(observed_root_split);

        let stable_batch = store.wal.as_ref().unwrap().committed_batch_count();
        let root_page_id = store.current_superblock.root_page_id;
        let high_water_page_id = store.current_superblock.high_water_page_id;
        store
            .put(replacement_key.clone(), b"after-split".to_vec())
            .unwrap();
        let stable = &store.wal.as_mut().unwrap().committed_batches_on_disk()[stable_batch];
        assert_eq!(stable.pages.len(), 1);
        let (data, wal) = store.into_files();
        let mut reopened = BlinkStore::open_with_wal(data, wal.unwrap(), config).unwrap();
        assert_eq!(reopened.current_superblock.root_page_id, root_page_id);
        assert_eq!(
            reopened.current_superblock.high_water_page_id,
            high_water_page_id
        );
        assert_eq!(
            reopened.get(&replacement_key).unwrap().value(),
            Some(&b"after-split"[..])
        );
        assert!(reopened.get(&insert_key).unwrap().value().is_some());
        assert!(reopened.get(&overflow_key).unwrap().is_missing());
        assert_eq!(
            reopened.get(&reused_overflow_key).unwrap().value(),
            Some(&vec![9; 2_000][..])
        );
        reopened.check_invariants().unwrap();
    }

    #[test]
    fn planned_checkpoint_after_superblock_elision_reopens() {
        let config = DatabaseConfig::default();
        let mut store = planned_store();
        let keys = (0u64..8)
            .map(|position| {
                DocumentKey::new(b"checkpoint".to_vec(), position.to_be_bytes().to_vec())
            })
            .collect::<Vec<_>>();
        for key in &keys {
            store.put(key.clone(), b"before".to_vec()).unwrap();
        }
        let metrics_before_updates = store.batch_metrics();
        for (position, key) in keys.iter().enumerate() {
            store.put(key.clone(), vec![position as u8]).unwrap();
        }
        let metrics_before_checkpoint = store.batch_metrics();
        assert_eq!(
            metrics_before_checkpoint.superblock_images_elided
                - metrics_before_updates.superblock_images_elided,
            8
        );
        store.checkpoint().unwrap();
        let (data, wal) = store.into_files();
        let mut reopened = BlinkStore::open_with_wal(data, wal.unwrap(), config).unwrap();
        for (position, key) in keys.iter().enumerate() {
            assert_eq!(
                reopened.get(key).unwrap().value(),
                Some(&[position as u8][..])
            );
        }
        reopened.check_invariants().unwrap();
    }

    #[test]
    fn planned_superblock_elision_wal_sync_failure_does_not_install_state() {
        let mut store = planned_store();
        let key = DocumentKey::new(b"elision-fault".to_vec(), b"key".to_vec());
        store.put(key.clone(), b"before".to_vec()).unwrap();
        let before_contents = store.scan(None, 100).unwrap();
        let before_lsn = store.next_lsn;
        let before_batches = store.wal.as_ref().unwrap().committed_batch_count();
        let before_publications = store.versioned_read_metrics().published_generations;
        store.set_fault_injector(FailOnce {
            point: "before_wal_sync",
            fired: false,
        });
        assert!(store.put(key.clone(), b"after".to_vec()).is_err());
        assert_eq!(store.scan(None, 100).unwrap(), before_contents);
        assert_eq!(store.next_lsn, before_lsn);
        assert_eq!(
            store.wal.as_ref().unwrap().committed_batch_count(),
            before_batches
        );
        assert_eq!(
            store.versioned_read_metrics().published_generations,
            before_publications
        );
        assert_eq!(store.get(&key).unwrap().value(), Some(&b"before"[..]));
    }

    #[test]
    fn planned_oversize_existing_key_update_does_not_publish_overlay() {
        let mut store = planned_store();
        let key = DocumentKey::new(vec![b'k'; 3_500], Vec::new());
        store.put(key.clone(), b"small".to_vec()).unwrap();
        let pages_before = store.state.pages.clone();
        let root_before = store.state.root_page_id;
        let free_list_before = store.state.free_list_head;
        let high_water_before = store.state.high_water_page_id;
        let superblock_before = store.current_superblock.clone();
        let slot_before = store.active_slot;
        let revision_before = store.next_revision;
        let lsn_before = store.next_lsn;
        let batch_before = store.next_batch_id;
        let generation_before = store.versioned_read_metrics().published_generations;
        let wal_before = store.wal.as_ref().unwrap().committed_batch_count();
        let error = store
            .apply_transaction_group(&[TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: key.clone(),
                    value: vec![b'x'; INLINE_VALUE_LIMIT],
                }],
            )])
            .unwrap_err();
        assert!(matches!(error, Error::InvalidInput(_)));
        assert_eq!(store.state.pages, pages_before);
        assert_eq!(store.state.root_page_id, root_before);
        assert_eq!(store.state.free_list_head, free_list_before);
        assert_eq!(store.state.high_water_page_id, high_water_before);
        assert_eq!(store.current_superblock, superblock_before);
        assert_eq!(store.active_slot, slot_before);
        assert_eq!(store.next_revision, revision_before);
        assert_eq!(store.next_lsn, lsn_before);
        assert_eq!(store.next_batch_id, batch_before);
        assert_eq!(
            store.versioned_read_metrics().published_generations,
            generation_before
        );
        assert_eq!(
            store.wal.as_ref().unwrap().committed_batch_count(),
            wal_before
        );
        assert_eq!(store.get(&key).unwrap().value(), Some(&b"small"[..]));
    }

    #[test]
    fn planned_independent_leaf_groups_are_exposed() {
        let mut store = BlinkStore::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            DatabaseConfig::default(),
        )
        .unwrap();
        for index in 0..120u64 {
            store.put(wide_key(index), vec![index as u8; 8]).unwrap();
        }
        store.enable_planned_execution();
        let first = wide_key(0);
        let last = wide_key(10_000);
        store
            .apply_transaction_group(&[
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: first.clone(),
                        value: b"first".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: last.clone(),
                        value: b"last".to_vec(),
                    }],
                ),
            ])
            .unwrap();
        let metrics = store.batch_metrics();
        assert!(metrics.independent_leaf_groups >= 2);
        assert!(metrics.dependency_edges < metrics.mutations_planned);
        assert_eq!(store.get(&first).unwrap().value(), Some(&b"first"[..]));
        assert_eq!(store.get(&last).unwrap().value(), Some(&b"last"[..]));
        store.check_invariants().unwrap();
    }

    #[test]
    fn planned_key_copy_cleanup_preserves_routes_dependencies_and_key_set() {
        let mut store = planned_store();
        for key_index in 0..120u64 {
            store
                .put(wide_key(key_index), vec![key_index as u8; 8])
                .unwrap();
        }
        let first_key = wide_key(0);
        let same_leaf_key = wide_key(1);
        let different_leaf_key = wide_key(10_000);
        let first_leaf =
            find_leaf_in_blink_state_borrowed(&store.state, &first_key.encode(), &mut 0, &mut 0)
                .unwrap();
        let same_leaf = find_leaf_in_blink_state_borrowed(
            &store.state,
            &same_leaf_key.encode(),
            &mut 0,
            &mut 0,
        )
        .unwrap();
        let different_leaf = find_leaf_in_blink_state_borrowed(
            &store.state,
            &different_leaf_key.encode(),
            &mut 0,
            &mut 0,
        )
        .unwrap();
        assert_eq!(first_leaf, same_leaf);
        assert_ne!(first_leaf, different_leaf);

        let requests = vec![
            TransactionRequest::new(
                Vec::new(),
                vec![
                    TransactionMutation::Put {
                        key: first_key.clone(),
                        value: b"first".to_vec(),
                    },
                    TransactionMutation::Put {
                        key: same_leaf_key.clone(),
                        value: b"same-leaf".to_vec(),
                    },
                    TransactionMutation::Put {
                        key: first_key.clone(),
                        value: b"duplicate".to_vec(),
                    },
                ],
            ),
            TransactionRequest::new(
                vec![TransactionCondition::Exists {
                    key: first_key.clone(),
                }],
                vec![TransactionMutation::Put {
                    key: first_key.clone(),
                    value: b"conditioned".to_vec(),
                }],
            ),
            TransactionRequest::new(
                vec![TransactionCondition::Exists {
                    key: first_key.clone(),
                }],
                vec![TransactionMutation::Put {
                    key: different_leaf_key.clone(),
                    value: b"different-leaf".to_vec(),
                }],
            ),
        ];
        let admitted = requests
            .iter()
            .enumerate()
            .map(|(fifo_position, request)| AdmittedTransaction {
                fifo_position,
                request,
                encoded_mutation_keys: request
                    .mutations
                    .iter()
                    .map(|mutation| mutation.key().encode())
                    .collect(),
                provisional_revision: ProvisionalRevisionToken {
                    transaction_position: fifo_position,
                    ordinal: fifo_position as u64 + 1,
                },
            })
            .collect::<Vec<_>>();
        let mut metrics = BlinkBatchMetrics::default();
        let group_scratch = Bump::new();
        let plan = plan_batch(&store.state, &admitted, &mut metrics, &group_scratch).unwrap();
        let first_encoded = first_key.encode();
        let same_leaf_encoded = same_leaf_key.encode();
        let different_leaf_encoded = different_leaf_key.encode();
        let first_transaction = &plan.transactions[0];

        assert_eq!(
            first_transaction
                .mutations
                .iter()
                .map(|mutation| mutation.route_hint.leaf_id)
                .collect::<Vec<_>>(),
            vec![first_leaf, same_leaf, first_leaf]
        );
        assert_eq!(
            first_transaction.mutated_key_set,
            BTreeSet::from([first_encoded.clone(), same_leaf_encoded])
        );
        assert_eq!(
            plan.transactions[1].mutated_key_set,
            BTreeSet::from([first_encoded.clone()])
        );
        assert_eq!(
            plan.transactions[2].mutated_key_set,
            BTreeSet::from([different_leaf_encoded])
        );
        assert_eq!(
            plan.transactions[1]
                .dependency_metadata
                .condition_key_predecessors,
            vec![0]
        );
        assert_eq!(
            plan.transactions[1]
                .dependency_metadata
                .same_key_predecessors,
            vec![0]
        );
        assert_eq!(
            plan.transactions[1]
                .dependency_metadata
                .same_target_page_predecessors,
            vec![0]
        );
        assert_eq!(
            plan.transactions[1]
                .dependency_metadata
                .structural_route_predecessors,
            vec![0]
        );
        assert!(plan.dependencies.iter().any(|edge| {
            edge.predecessor == 0
                && edge.successor == 1
                && edge.kind == DependencyKind::ConditionKey
        }));
        assert!(plan.dependencies.iter().any(|edge| {
            edge.predecessor == 0
                && edge.successor == 1
                && edge.kind == DependencyKind::SameTargetPage
        }));
        assert!(!plan.dependencies.iter().any(|edge| {
            edge.predecessor == 1
                && edge.successor == 2
                && edge.kind == DependencyKind::SameTargetPage
        }));
        assert_eq!(plan.transactions[1].provisional_revision.ordinal, 2);
    }

    #[test]
    fn prepared_mutation_keys_match_existing_encoding_and_preserve_limits() {
        let request = TransactionRequest::new(
            Vec::new(),
            vec![
                TransactionMutation::Put {
                    key: DocumentKey::new(vec![1, 0, 2], vec![3, 0]),
                    value: b"value".to_vec(),
                },
                TransactionMutation::Delete {
                    key: DocumentKey::new(Vec::new(), vec![4, 5]),
                },
            ],
        );
        let prepared =
            validate_and_encode_mutation_keys(&request, &StorageLimits::default()).unwrap();
        assert_eq!(prepared.len(), request.mutations.len());
        assert_eq!(
            prepared,
            request
                .mutations
                .iter()
                .map(|mutation| mutation.key().encode())
                .collect::<Vec<_>>()
        );

        let oversized_key_request = TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Delete {
                key: DocumentKey::new(vec![1; crate::MAX_ENCODED_KEY_SIZE], Vec::new()),
            }],
        );
        assert!(matches!(
            validate_and_encode_mutation_keys(&oversized_key_request, &StorageLimits::default()),
            Err(Error::InvalidInput(_))
        ));

        let oversized_value_request = TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Put {
                key: DocumentKey::new(b"key".to_vec(), Vec::new()),
                value: vec![1, 2, 3],
            }],
        );
        let limits = StorageLimits {
            max_value_size: 2,
            ..StorageLimits::default()
        };
        assert!(matches!(
            validate_and_encode_mutation_keys(&oversized_value_request, &limits),
            Err(Error::InvalidInput(_))
        ));
    }

    #[test]
    fn parallel_planned_executes_independent_leaf_jobs() {
        let mut serial = planned_store();
        let mut parallel = parallel_store();
        for index in 0..120u64 {
            let key = wide_key(index);
            let value = vec![index as u8; 8];
            serial.put(key.clone(), value.clone()).unwrap();
            parallel.put(key, value).unwrap();
        }
        serial.put(wide_key(10_000), b"seedlast".to_vec()).unwrap();
        parallel
            .put(wide_key(10_000), b"seedlast".to_vec())
            .unwrap();
        serial.enable_planned_execution();
        let first = wide_key(0);
        let last = wide_key(10_000);
        let requests = vec![
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: first.clone(),
                    value: b"first!!!".to_vec(),
                }],
            ),
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: last.clone(),
                    value: b"last!!!!".to_vec(),
                }],
            ),
        ];
        let serial_results = serial.apply_transaction_group(&requests).unwrap();
        let parallel_results = parallel.apply_transaction_group(&requests).unwrap();
        assert_eq!(
            successful_commit_lsns(&parallel_results),
            successful_commit_lsns(&serial_results)
        );
        for key in [&first, &last] {
            assert_eq!(parallel.get(key).unwrap(), serial.get(key).unwrap());
        }
        let metrics = parallel.batch_metrics();
        assert_eq!(metrics.parallel_groups, 1, "metrics={metrics:?}");
        assert!(metrics.parallel_leaf_jobs >= 2);
        assert!(metrics.parallel_worker_dispatches >= 2);
        assert_eq!(metrics.full_state_clones, 0);
        parallel.check_invariants().unwrap();
    }

    #[test]
    fn parallel_same_leaf_chain_preserves_order_and_wal_boundaries() {
        let mut serial = planned_store();
        let mut parallel = parallel_store();
        let key = DocumentKey::new(b"parallel-chain".to_vec(), b"key".to_vec());
        let requests = vec![
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: key.clone(),
                    value: b"A".to_vec(),
                }],
            ),
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: key.clone(),
                    value: b"B".to_vec(),
                }],
            ),
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: key.clone(),
                    value: b"C".to_vec(),
                }],
            ),
        ];
        let serial_results = serial.apply_transaction_group(&requests).unwrap();
        let parallel_results = parallel.apply_transaction_group(&requests).unwrap();
        assert_eq!(
            successful_commit_lsns(&parallel_results),
            successful_commit_lsns(&serial_results)
        );
        assert_eq!(parallel.get(&key).unwrap(), serial.get(&key).unwrap());
        assert_eq!(
            parallel.get(&key).unwrap().revision(),
            parallel_results[2].as_ref().unwrap().commit_lsn.into()
        );
        assert_eq!(parallel.batch_metrics().parallel_skipped_single_leaf, 1);
        let parallel_wal = parallel.wal.as_mut().unwrap().committed_batches_on_disk();
        let serial_wal = serial.wal.as_mut().unwrap().committed_batches_on_disk();
        assert_eq!(parallel_wal, serial_wal);
    }

    #[test]
    fn parallel_multi_leaf_transaction_runs_on_leaf_workers() {
        let mut serial = planned_store();
        let mut parallel = parallel_store();
        for index in 0..120u64 {
            let key = wide_key(index);
            serial.put(key.clone(), vec![index as u8; 8]).unwrap();
            parallel.put(key, vec![index as u8; 8]).unwrap();
        }
        serial.enable_planned_execution();
        let first = wide_key(0);
        let last = wide_key(119);
        let request = TransactionRequest::new(
            Vec::new(),
            vec![
                TransactionMutation::Put {
                    key: first.clone(),
                    value: b"multi-first".to_vec(),
                },
                TransactionMutation::Put {
                    key: last.clone(),
                    value: b"multi-last".to_vec(),
                },
            ],
        );
        let serial_result = serial.transact(request.clone()).unwrap();
        let parallel_result = parallel.transact(request).unwrap();
        assert_eq!(parallel_result, serial_result);
        assert_eq!(parallel.get(&first).unwrap(), serial.get(&first).unwrap());
        assert_eq!(parallel.get(&last).unwrap(), serial.get(&last).unwrap());
        assert_eq!(
            parallel.batch_metrics().parallel_groups,
            1,
            "{:?}",
            parallel.batch_metrics()
        );
        assert_eq!(parallel.batch_metrics().parallel_fallback_groups, 0);
        assert_eq!(
            parallel.wal.as_mut().unwrap().committed_batches_on_disk(),
            serial.wal.as_mut().unwrap().committed_batches_on_disk()
        );
        assert_eq!(
            parallel.wal.as_ref().unwrap().last_commit_lsn().unwrap(),
            parallel_result.commit_lsn
        );
        parallel.check_invariants().unwrap();
    }

    #[test]
    fn parallel_cross_leaf_condition_dependency_keeps_fifo_results() {
        let mut serial = planned_store();
        let mut parallel = parallel_store();
        for index in 0..120u64 {
            let key = wide_key(index);
            serial.put(key.clone(), vec![index as u8; 8]).unwrap();
            parallel.put(key, vec![index as u8; 8]).unwrap();
        }
        serial.enable_planned_execution();
        let first = wide_key(0);
        let last = wide_key(119);
        let requests = vec![
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: first.clone(),
                    value: b"dependency-source".to_vec(),
                }],
            ),
            TransactionRequest::new(
                vec![TransactionCondition::Exists { key: first.clone() }],
                vec![TransactionMutation::Put {
                    key: last.clone(),
                    value: b"dependency-target".to_vec(),
                }],
            ),
        ];
        let parallel_results = parallel.apply_transaction_group(&requests).unwrap();
        let serial_results = serial.apply_transaction_group(&requests).unwrap();
        assert_eq!(
            successful_commit_lsns(&parallel_results),
            successful_commit_lsns(&serial_results)
        );
        assert_eq!(
            parallel.batch_metrics().parallel_groups,
            1,
            "{:?}",
            parallel.batch_metrics()
        );
        assert_eq!(parallel.batch_metrics().parallel_fallback_groups, 0);
        assert_eq!(parallel.get(&first).unwrap(), serial.get(&first).unwrap());
        assert_eq!(parallel.get(&last).unwrap(), serial.get(&last).unwrap());
    }

    #[test]
    fn parallel_overflow_and_split_candidates_fall_back_as_whole_groups() {
        let mut serial = planned_store();
        let mut parallel = parallel_store();
        for index in 0..120u64 {
            let key = wide_key(index);
            serial.put(key.clone(), vec![index as u8; 8]).unwrap();
            parallel.put(key, vec![index as u8; 8]).unwrap();
        }
        let first = wide_key(0);
        let last = wide_key(10_000);
        serial.put(last.clone(), vec![0xa5; 2_000]).unwrap();
        parallel.put(last.clone(), vec![0xa5; 2_000]).unwrap();
        serial.enable_planned_execution();
        let overflow_requests = vec![
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: first.clone(),
                    value: b"inline-update".to_vec(),
                }],
            ),
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Delete { key: last.clone() }],
            ),
        ];
        let parallel_results = parallel
            .apply_transaction_group(&overflow_requests)
            .unwrap();
        let serial_results = serial.apply_transaction_group(&overflow_requests).unwrap();
        assert_eq!(
            successful_commit_lsns(&parallel_results),
            successful_commit_lsns(&serial_results)
        );
        assert_eq!(parallel.batch_metrics().parallel_fallback_overflow, 1);
        assert_eq!(parallel.get(&last).unwrap(), serial.get(&last).unwrap());

        let split_metrics_before = parallel.split_metrics();
        let split_requests = vec![
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: first.clone(),
                    value: b"still-inline".to_vec(),
                }],
            ),
            TransactionRequest::new(
                Vec::new(),
                (120..124u64)
                    .map(|index| TransactionMutation::Put {
                        key: wide_key(index),
                        value: vec![index as u8; 8],
                    })
                    .collect(),
            ),
        ];
        let parallel_results = parallel.apply_transaction_group(&split_requests).unwrap();
        let serial_results = serial.apply_transaction_group(&split_requests).unwrap();
        assert_eq!(
            successful_commit_lsns(&parallel_results),
            successful_commit_lsns(&serial_results)
        );
        assert_eq!(parallel.batch_metrics().parallel_fallback_structural, 1);
        assert!(parallel.split_metrics().leaf_splits > split_metrics_before.leaf_splits);
        parallel.check_invariants().unwrap();
        serial.check_invariants().unwrap();
    }

    #[test]
    fn planned_stale_route_reroutes_after_a_split() {
        let mut store = BlinkStore::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            DatabaseConfig::default(),
        )
        .unwrap();
        for index in 0..6u64 {
            store.put(wide_key(index), vec![index as u8; 8]).unwrap();
        }
        store.enable_planned_execution();
        let split_key = wide_key(6);
        let stale_route_key = wide_key(7);
        let results = store
            .apply_transaction_group(&[
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: split_key.clone(),
                        value: vec![6; 8],
                    }],
                ),
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: stale_route_key.clone(),
                        value: vec![7; 8],
                    }],
                ),
            ])
            .unwrap();
        assert!(results.iter().all(Result::is_ok));
        let metrics = store.batch_metrics();
        assert!(metrics.structural_fallbacks > 0);
        assert!(metrics.reroutes > 0 || metrics.split_triggered_reroutes > 0);
        assert_eq!(
            store.get(&stale_route_key).unwrap().value(),
            Some(&vec![7; 8][..])
        );
        store.check_invariants().unwrap();
    }

    #[test]
    fn planned_structural_fallback_handles_internal_and_root_splits() {
        let mut store = BlinkStore::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            DatabaseConfig::default(),
        )
        .unwrap();
        for index in 0..500u64 {
            store.put(wide_key(index), vec![index as u8; 8]).unwrap();
        }
        let before = store.split_metrics();
        store.enable_planned_execution();
        let requests = (500..5_000u64)
            .map(|index| {
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: wide_key(index),
                        value: vec![index as u8; 8],
                    }],
                )
            })
            .collect::<Vec<_>>();
        let results = store.apply_transaction_group(&requests).unwrap();
        assert!(results.iter().all(Result::is_ok));
        let after = store.split_metrics();
        assert!(after.leaf_splits > before.leaf_splits);
        assert!(after.internal_splits > before.internal_splits);
        assert!(
            after.root_splits > before.root_splits,
            "before={before:?} after={after:?}"
        );
        assert!(store.batch_metrics().structural_fallbacks > 0);
        store.check_invariants().unwrap();
    }

    #[test]
    fn planned_multi_leaf_transaction_has_one_commit_and_atomic_publication() {
        let mut store = planned_store();
        for index in 0..120u64 {
            store.put(wide_key(index), vec![index as u8; 8]).unwrap();
        }
        let first = wide_key(0);
        let last = wide_key(10_000);
        let old_pin = store.publisher.pin();
        let read_handle = store.versioned_read_handle();
        let before_batches = store.wal.as_ref().unwrap().committed_batch_count();
        let result = store
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![
                    TransactionMutation::Put {
                        key: first.clone(),
                        value: b"multi-first".to_vec(),
                    },
                    TransactionMutation::Put {
                        key: last.clone(),
                        value: b"multi-last".to_vec(),
                    },
                ],
            ))
            .unwrap();
        let mut corrections = 0;
        assert_eq!(
            read_state(&old_pin, &first, &mut corrections)
                .unwrap()
                .value(),
            Some(&vec![0; 8][..])
        );
        assert_eq!(
            read_state(&old_pin, &last, &mut corrections)
                .unwrap()
                .value(),
            None
        );
        assert_eq!(
            store.get(&first).unwrap(),
            RevisionState::present(b"multi-first", result.commit_lsn.into())
        );
        assert_eq!(
            store.get(&last).unwrap(),
            RevisionState::present(b"multi-last", result.commit_lsn.into())
        );
        assert_eq!(
            read_handle.get(&first).unwrap().value(),
            Some(&b"multi-first"[..])
        );
        assert_eq!(
            read_handle.get(&last).unwrap().value(),
            Some(&b"multi-last"[..])
        );
        assert_eq!(
            store.wal.as_ref().unwrap().committed_batch_count(),
            before_batches + 1
        );
        store.check_invariants().unwrap();
    }

    #[test]
    fn planned_wal_tail_keeps_the_first_logical_commit_boundary() {
        let mut store = planned_store();
        store.set_fault_injector(FailOnOccurrence {
            point: "before_commit_record",
            remaining: 2,
        });
        let first = DocumentKey::new(b"wal-boundary".to_vec(), b"first".to_vec());
        let second = DocumentKey::new(b"wal-boundary".to_vec(), b"second".to_vec());
        assert!(
            store
                .apply_transaction_group(&[
                    TransactionRequest::new(
                        Vec::new(),
                        vec![TransactionMutation::Put {
                            key: first.clone(),
                            value: b"first".to_vec(),
                        }],
                    ),
                    TransactionRequest::new(
                        Vec::new(),
                        vec![TransactionMutation::Put {
                            key: second.clone(),
                            value: b"second".to_vec(),
                        }],
                    ),
                ])
                .is_err()
        );
        let (data, wal) = store.into_files();
        let mut reopened =
            BlinkStore::open_with_wal(data, wal.unwrap(), DatabaseConfig::default()).unwrap();
        assert_eq!(reopened.get(&first).unwrap().value(), Some(&b"first"[..]));
        assert!(reopened.get(&second).unwrap().is_missing());
        reopened.check_invariants().unwrap();
    }

    #[test]
    fn query_scan_tombstone_overflow_checkpoint_and_reopen() {
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config.clone(),
        )
        .unwrap();
        let first = DocumentKey::new(b"p".to_vec(), b"a".to_vec());
        let second = DocumentKey::new(b"p".to_vec(), b"b".to_vec());
        store.put(first.clone(), vec![7; 2_000]).unwrap();
        store.put(second.clone(), b"small".to_vec()).unwrap();
        store.delete(first.clone()).unwrap();
        let query = store
            .query(&PrimaryKey::new(b"p".to_vec()), None, 10)
            .unwrap();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0].key, second);
        assert_eq!(store.scan(None, 10).unwrap(), query);
        let checkpoint_pin = store.publisher.pin();
        store.checkpoint().unwrap();
        let mut corrections = 0;
        assert_eq!(
            scan_state(&checkpoint_pin, None, 10, &mut corrections).unwrap(),
            query
        );
        let (file, wal) = store.into_files();
        let mut reopened = BlinkStore::open_with_wal(file, wal.unwrap(), config).unwrap();
        assert!(reopened.get(&first).unwrap().is_missing());
        assert_eq!(reopened.scan(None, 10).unwrap(), query);
        assert_eq!(
            reopened.versioned_read_handle().scan(None, 10).unwrap(),
            query
        );
        reopened.check_invariants().unwrap();
    }

    #[test]
    fn versioned_generation_is_atomic_for_multi_page_transaction() {
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        let a = DocumentKey::new(b"atomic".to_vec(), b"a".to_vec());
        let b = DocumentKey::new(b"atomic".to_vec(), b"b".to_vec());
        store.put(a.clone(), b"old-a".to_vec()).unwrap();
        store.put(b.clone(), b"old-b".to_vec()).unwrap();
        let old_pin = store.publisher.pin();
        let handle = store.versioned_read_handle();
        store
            .apply_transaction_group(&[
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: a.clone(),
                        value: b"new-a".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: b.clone(),
                        value: b"new-b".to_vec(),
                    }],
                ),
            ])
            .unwrap();

        let mut corrections = 0;
        assert_eq!(
            read_state(&old_pin, &a, &mut corrections).unwrap().value(),
            Some(&b"old-a"[..])
        );
        assert_eq!(
            read_state(&old_pin, &b, &mut corrections).unwrap().value(),
            Some(&b"old-b"[..])
        );
        assert_eq!(handle.get(&a).unwrap().value(), Some(&b"new-a"[..]));
        assert_eq!(handle.get(&b).unwrap().value(), Some(&b"new-b"[..]));
        assert_eq!(store.versioned_read_metrics().active_generation_pins, 1);
    }

    #[test]
    fn parallel_generation_publication_is_atomic_across_independent_leaves() {
        let mut store = parallel_store();
        for index in 0..120u64 {
            store.put(wide_key(index), vec![index as u8; 8]).unwrap();
        }
        let first = wide_key(0);
        let last = wide_key(10_000);
        store.put(last.clone(), b"seedlast".to_vec()).unwrap();
        store.put(last.clone(), b"old-last".to_vec()).unwrap();
        let old_pin = store.publisher.pin();
        let handle = store.versioned_read_handle();
        store
            .apply_transaction_group(&[
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: first.clone(),
                        value: b"newfirst".to_vec(),
                    }],
                ),
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: last.clone(),
                        value: b"newlast!".to_vec(),
                    }],
                ),
            ])
            .unwrap();
        let mut corrections = 0;
        assert_eq!(
            read_state(&old_pin, &first, &mut corrections)
                .unwrap()
                .value(),
            Some(&[0u8; 8][..])
        );
        assert_eq!(
            read_state(&old_pin, &last, &mut corrections)
                .unwrap()
                .value(),
            Some(&b"old-last"[..])
        );
        assert_eq!(handle.get(&first).unwrap().value(), Some(&b"newfirst"[..]));
        assert_eq!(handle.get(&last).unwrap().value(), Some(&b"newlast!"[..]));
        assert_eq!(store.batch_metrics().parallel_groups, 1);
    }

    #[test]
    fn reader_during_unpublished_install_sees_only_old_generation() {
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        let key = DocumentKey::new(b"install".to_vec(), b"key".to_vec());
        store.put(key.clone(), b"old".to_vec()).unwrap();
        let old_pin = store.publisher.pin();
        let handle = store.versioned_read_handle();

        let mut candidate = store.state.clone();
        candidate.allow_page_reuse = false;
        let mut dirty = BTreeSet::new();
        let mut split_metrics = BlinkSplitMetrics::default();
        apply_mutation(
            &mut candidate,
            &mut dirty,
            &mut split_metrics,
            &TransactionMutation::Put {
                key: key.clone(),
                value: b"prepared-but-not-published".to_vec(),
            },
            Revision::new(2),
        )
        .unwrap();
        let prepared_sb = BlinkSuperblock {
            generation: store.current_superblock.generation + 1,
            root_page_id: candidate.root_page_id,
            free_list_head: candidate.free_list_head,
            high_water_page_id: candidate.high_water_page_id,
            ..store.current_superblock.clone()
        };
        let (unpublished, _) = store
            .publisher
            .prepare(&candidate, &prepared_sb, &dirty)
            .unwrap();

        // The immutable page cells exist, but the generation pointer has not
        // moved. This is the install-before-publication window.
        assert_eq!(handle.get(&key).unwrap().value(), Some(&b"old"[..]));
        let mut corrections = 0;
        assert_eq!(
            read_state(&old_pin, &key, &mut corrections)
                .unwrap()
                .value(),
            Some(&b"old"[..])
        );
        store.publisher.publish(unpublished);
        assert_eq!(
            handle.get(&key).unwrap().value(),
            Some(&b"prepared-but-not-published"[..])
        );
    }

    #[test]
    fn root_split_and_leaf_links_are_safe_for_pinned_reader() {
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        let first = DocumentKey::new(b"root".to_vec(), 0u64.to_be_bytes().to_vec());
        store.put(first.clone(), b"before".to_vec()).unwrap();
        let old_pin = store.publisher.pin();
        let old_root = old_pin.generation.root_page_id;
        let handle = store.versioned_read_handle();
        for index in 1..600u64 {
            store
                .put(
                    DocumentKey::new(b"root".to_vec(), index.to_be_bytes().to_vec()),
                    vec![(index & 0xff) as u8; 32],
                )
                .unwrap();
        }
        assert_ne!(store.current_superblock.root_page_id, old_root);
        let mut corrections = 0;
        assert_eq!(
            read_state(&old_pin, &first, &mut corrections)
                .unwrap()
                .value(),
            Some(&b"before"[..])
        );
        assert_eq!(handle.scan(None, 1_000).unwrap().len(), 600);
        store.check_invariants().unwrap();
    }

    #[test]
    fn page_reuse_is_delayed_until_pinned_generation_releases() {
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        let old_key = DocumentKey::new(b"reuse".to_vec(), b"old".to_vec());
        let new_key = DocumentKey::new(b"reuse".to_vec(), b"new".to_vec());
        let value = vec![7; 2_000];
        store.put(old_key.clone(), value.clone()).unwrap();
        let old_head = match find_entry(&store.state, &old_key.encode())
            .unwrap()
            .unwrap()
            .value
        {
            Some(OwnedValue::Overflow { head, .. }) => head,
            _ => panic!("expected overflow value"),
        };
        let old_pin = store.publisher.pin();
        store.delete(old_key.clone()).unwrap();
        store.put(new_key.clone(), value.clone()).unwrap();
        let new_head = match find_entry(&store.state, &new_key.encode())
            .unwrap()
            .unwrap()
            .value
        {
            Some(OwnedValue::Overflow { head, .. }) => head,
            _ => panic!("expected overflow value"),
        };
        assert_ne!(old_head, new_head);
        assert_eq!(store.versioned_read_metrics().reusable_page_ids, 0);
        let mut corrections = 0;
        assert_eq!(
            read_state(&old_pin, &old_key, &mut corrections)
                .unwrap()
                .value(),
            Some(&value[..])
        );
        drop(old_pin);
        assert_eq!(store.versioned_read_metrics().active_generation_pins, 0);
        assert!(store.versioned_read_metrics().versions_reclaimed > 0);
        let after_key = DocumentKey::new(b"reuse".to_vec(), b"after".to_vec());
        store.put(after_key.clone(), value).unwrap();
        let after_head = match find_entry(&store.state, &after_key.encode())
            .unwrap()
            .unwrap()
            .value
        {
            Some(OwnedValue::Overflow { head, .. }) => head,
            _ => panic!("expected overflow value"),
        };
        assert_eq!(after_head, old_head);
    }

    #[test]
    fn query_and_scan_keep_one_generation_during_publication() {
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        for index in 0..40u64 {
            store
                .put(
                    DocumentKey::new(b"query".to_vec(), index.to_be_bytes().to_vec()),
                    vec![1],
                )
                .unwrap();
        }
        let old_pin = store.publisher.pin();
        let handle = store.versioned_read_handle();
        store
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: DocumentKey::new(b"query".to_vec(), 10u64.to_be_bytes().to_vec()),
                    value: vec![9],
                }],
            ))
            .unwrap();
        let mut corrections = 0;
        let old_query = query_state(
            &old_pin,
            &PrimaryKey::new(b"query".to_vec()),
            None,
            40,
            &mut corrections,
        )
        .unwrap();
        let old_scan = scan_state(&old_pin, None, 40, &mut corrections).unwrap();
        assert!(old_query.iter().all(|row| row.value == vec![1]));
        assert!(old_scan.iter().all(|row| row.value == vec![1]));
        assert_eq!(
            handle
                .query(&PrimaryKey::new(b"query".to_vec()), None, 40)
                .unwrap()
                .iter()
                .find(|row| row.key.sk == SortKey::new(10u64.to_be_bytes().to_vec()))
                .unwrap()
                .value,
            vec![9]
        );
    }

    struct FailOnce {
        point: &'static str,
        fired: bool,
    }

    impl FaultInjector for FailOnce {
        fn hit(&mut self, point: &str) -> Result<()> {
            if !self.fired && point == self.point {
                self.fired = true;
                return Err(Error::recovery(format!("injected failure at {point}")));
            }
            Ok(())
        }
    }

    #[test]
    fn wal_sync_failure_does_not_publish_a_generation() {
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        let handle = store.versioned_read_handle();
        store.set_fault_injector(FailOnce {
            point: "before_wal_sync",
            fired: false,
        });
        let key = DocumentKey::new(b"wal".to_vec(), b"failed".to_vec());
        assert!(store.put(key.clone(), b"not-visible".to_vec()).is_err());
        assert!(handle.get(&key).unwrap().is_missing());
        assert_eq!(store.versioned_read_metrics().published_generations, 0);
    }

    #[test]
    fn planned_wal_failure_does_not_install_working_delta() {
        let mut store = planned_store();
        let existing_key = DocumentKey::new(b"wal-atomic".to_vec(), b"existing".to_vec());
        store.put(existing_key.clone(), b"before".to_vec()).unwrap();
        let before_contents = store.scan(None, 100).unwrap();
        let before_metadata = (
            store.state.root_page_id,
            store.state.free_list_head,
            store.state.high_water_page_id,
            store.current_superblock.clone(),
            store.next_revision,
            store.next_lsn,
            store.next_batch_id,
            store.versioned_read_metrics().published_generations,
        );
        let before_pages = store
            .state
            .pages
            .iter()
            .map(|(page_id, page)| (*page_id, encode_blink_page(*page_id, page).unwrap()))
            .collect::<Vec<_>>();
        store.set_fault_injector(FailOnce {
            point: "before_wal_sync",
            fired: false,
        });
        let failed_key = DocumentKey::new(b"wal-atomic".to_vec(), b"failed".to_vec());
        assert!(
            store
                .apply_transaction_group(&[TransactionRequest::new(
                    Vec::new(),
                    vec![
                        TransactionMutation::Put {
                            key: existing_key,
                            value: b"after".to_vec(),
                        },
                        TransactionMutation::Put {
                            key: failed_key,
                            value: vec![8; 4_000],
                        },
                    ],
                )])
                .is_err()
        );
        assert_eq!(store.scan(None, 100).unwrap(), before_contents);
        assert_eq!(
            (
                store.state.root_page_id,
                store.state.free_list_head,
                store.state.high_water_page_id,
                store.current_superblock.clone(),
                store.next_revision,
                store.next_lsn,
                store.next_batch_id,
                store.versioned_read_metrics().published_generations,
            ),
            before_metadata
        );
        let after_pages = store
            .state
            .pages
            .iter()
            .map(|(page_id, page)| (*page_id, encode_blink_page(*page_id, page).unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(after_pages, before_pages);
    }

    #[test]
    fn parallel_wal_failure_does_not_install_working_delta() {
        let mut store = parallel_store();
        for index in 0..120u64 {
            store.put(wide_key(index), vec![index as u8; 8]).unwrap();
        }
        let first = wide_key(0);
        let last = wide_key(10_000);
        store.put(last.clone(), b"seedlast".to_vec()).unwrap();
        let before_contents = store.scan(None, 1_000).unwrap();
        let before_metadata = (
            store.state.root_page_id,
            store.state.free_list_head,
            store.state.high_water_page_id,
            store.current_superblock.clone(),
            store.next_revision,
            store.next_lsn,
            store.next_batch_id,
            store.versioned_read_metrics().published_generations,
        );
        let before_pages = store
            .state
            .pages
            .iter()
            .map(|(page_id, page)| (*page_id, encode_blink_page(*page_id, page).unwrap()))
            .collect::<Vec<_>>();
        store.set_fault_injector(FailOnce {
            point: "before_wal_sync",
            fired: false,
        });
        let failed = store.apply_transaction_group(&[
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: first,
                    value: b"fail-one".to_vec(),
                }],
            ),
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: last,
                    value: b"fail-two".to_vec(),
                }],
            ),
        ]);
        assert!(failed.is_err());
        assert_eq!(store.scan(None, 1_000).unwrap(), before_contents);
        assert_eq!(
            (
                store.state.root_page_id,
                store.state.free_list_head,
                store.state.high_water_page_id,
                store.current_superblock.clone(),
                store.next_revision,
                store.next_lsn,
                store.next_batch_id,
                store.versioned_read_metrics().published_generations,
            ),
            before_metadata
        );
        let after_pages = store
            .state
            .pages
            .iter()
            .map(|(page_id, page)| (*page_id, encode_blink_page(*page_id, page).unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(after_pages, before_pages);
        assert_eq!(store.batch_metrics().parallel_groups, 1);
        assert!(store.batch_metrics().parallel_leaf_jobs >= 2);
    }

    #[test]
    fn concurrent_versioned_readers_survive_serial_writer_publications() {
        use std::sync::Mutex as StdMutex;
        use std::thread;

        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        let key = DocumentKey::new(b"stress".to_vec(), b"key".to_vec());
        store.put(key.clone(), vec![0]).unwrap();
        let handle = store.versioned_read_handle();
        let writer = Arc::new(StdMutex::new(store));
        let mut readers = Vec::new();
        for _ in 0..4 {
            let handle = handle.clone();
            let key = key.clone();
            readers.push(thread::spawn(move || {
                for _ in 0..500 {
                    assert!(handle.get(&key).unwrap().value().is_some());
                    assert_eq!(handle.scan(None, 1).unwrap().len(), 1);
                }
            }));
        }
        for value in 1..100u8 {
            writer
                .lock()
                .unwrap()
                .put(key.clone(), vec![value])
                .unwrap();
        }
        for reader in readers {
            reader.join().unwrap();
        }
        let store = match Arc::try_unwrap(writer) {
            Ok(writer) => writer.into_inner().unwrap(),
            Err(_) => panic!("writer Arc should have no remaining references"),
        };
        assert_eq!(store.versioned_read_metrics().active_generation_pins, 0);
        store.check_invariants().unwrap();
    }

    #[test]
    fn randomized_concurrent_reads_and_serial_writes_preserve_ordering() {
        use std::sync::Mutex as StdMutex;
        use std::thread;

        let seed = 0x2a02_2026_u64;
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        for index in 0..128u64 {
            store
                .put(
                    DocumentKey::new(b"random".to_vec(), index.to_be_bytes().to_vec()),
                    vec![index as u8],
                )
                .unwrap();
        }
        let handle = store.versioned_read_handle();
        let writer = Arc::new(StdMutex::new(store));
        let mut readers = Vec::new();
        for reader_id in 0..6u64 {
            let handle = handle.clone();
            readers.push(thread::spawn(move || {
                let mut state = seed ^ reader_id;
                for _ in 0..1_000 {
                    state = splitmix_for_test(state);
                    let key =
                        DocumentKey::new(b"random".to_vec(), (state % 128).to_be_bytes().to_vec());
                    match state % 3 {
                        0 => {
                            let _ = handle.get(&key).unwrap();
                        }
                        1 => {
                            let rows = handle.scan(None, 16).unwrap();
                            assert!(rows.windows(2).all(|pair| pair[0].key < pair[1].key));
                        }
                        _ => {
                            let rows = handle
                                .query(&PrimaryKey::new(b"random".to_vec()), None, 16)
                                .unwrap();
                            assert!(rows.windows(2).all(|pair| pair[0].key < pair[1].key));
                        }
                    }
                }
            }));
        }
        let mut state = seed;
        for operation in 0..300u64 {
            state = splitmix_for_test(state);
            let key = DocumentKey::new(b"random".to_vec(), (state % 128).to_be_bytes().to_vec());
            let mut store = writer.lock().unwrap();
            if state & 1 == 0 {
                store.put(key, vec![(operation & 0xff) as u8]).unwrap();
            } else {
                store.delete(key).unwrap();
            }
        }
        for reader in readers {
            reader.join().unwrap();
        }
        let store = match Arc::try_unwrap(writer) {
            Ok(writer) => writer.into_inner().unwrap(),
            Err(_) => panic!("writer Arc should have no remaining references"),
        };
        assert_eq!(store.versioned_read_metrics().active_generation_pins, 0);
        store.check_invariants().unwrap();
    }

    #[test]
    fn baseline_and_experimental_formats_reject_each_other() {
        let config = DatabaseConfig::default();
        let mut blink = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config.clone(),
        )
        .unwrap();
        blink
            .put(
                DocumentKey::new(b"p".to_vec(), b"a".to_vec()),
                b"v".to_vec(),
            )
            .unwrap();
        blink.flush().unwrap();
        let (blink_file, blink_wal) = blink.into_files();
        assert!(
            crate::BTreeStore::open_with_wal(blink_file, blink_wal.unwrap(), config.clone())
                .is_err()
        );

        let mut baseline = crate::BTreeStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config.clone(),
        )
        .unwrap();
        baseline
            .put(
                DocumentKey::new(b"p".to_vec(), b"a".to_vec()),
                b"v".to_vec(),
            )
            .unwrap();
        baseline.flush().unwrap();
        let (baseline_file, baseline_wal) = baseline.into_files().unwrap();
        assert!(BlinkStore::open_with_wal(baseline_file, baseline_wal, config).is_err());
    }

    #[test]
    fn planned_randomized_transaction_differential_reports_seed_and_operation() {
        let seed = 0x3a03_2026_u64;
        let mut state = seed;
        let mut store = parallel_store();
        let mut reference = BTreeMap::<DocumentKey, RevisionState>::new();
        for operation_index in 0..300usize {
            state = splitmix_for_test(state);
            let mut requests = Vec::new();
            if operation_index % 7 == 0 {
                let first_key =
                    DocumentKey::new(b"staged".to_vec(), (state % 64).to_be_bytes().to_vec());
                let second_key = DocumentKey::new(
                    b"staged".to_vec(),
                    ((state + 1) % 64).to_be_bytes().to_vec(),
                );
                requests.push(TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: first_key.clone(),
                        value: vec![operation_index as u8],
                    }],
                ));
                requests.push(TransactionRequest::new(
                    vec![TransactionCondition::Exists { key: first_key }],
                    vec![TransactionMutation::Put {
                        key: second_key,
                        value: vec![(operation_index + 1) as u8],
                    }],
                ));
            } else {
                let key_index = state % 64;
                let key = if state & 8 == 0 {
                    wide_key(key_index)
                } else {
                    DocumentKey::new(b"random".to_vec(), key_index.to_be_bytes().to_vec())
                };
                state = splitmix_for_test(state);
                let condition = match state % 4 {
                    0 => None,
                    1 => Some(
                        if reference.get(&key).is_some_and(|value| !value.is_missing()) {
                            TransactionCondition::Exists { key: key.clone() }
                        } else {
                            TransactionCondition::NotExists { key: key.clone() }
                        },
                    ),
                    2 => Some(TransactionCondition::RevisionEquals {
                        key: key.clone(),
                        expected_revision: reference
                            .get(&key)
                            .map(RevisionState::revision)
                            .unwrap_or(Revision::ZERO),
                    }),
                    _ => Some(TransactionCondition::RevisionEquals {
                        key: key.clone(),
                        expected_revision: Revision::new(9_000_000 + operation_index as u64),
                    }),
                };
                state = splitmix_for_test(state);
                let mutation = if state & 1 == 0 {
                    TransactionMutation::Put {
                        key: key.clone(),
                        value: vec![(operation_index & 0xff) as u8; 8],
                    }
                } else {
                    TransactionMutation::Delete { key: key.clone() }
                };
                let mut mutations = vec![mutation];
                if state & 2 == 0 {
                    let second_key = DocumentKey::new(
                        b"random".to_vec(),
                        ((key_index + 1) % 64).to_be_bytes().to_vec(),
                    );
                    if second_key != key {
                        mutations.push(TransactionMutation::Put {
                            key: second_key,
                            value: vec![operation_index as u8; 4],
                        });
                    }
                }
                requests.push(TransactionRequest::new(
                    condition.into_iter().collect(),
                    mutations,
                ));
            }

            let actual_results = store
                .apply_transaction_group(&requests)
                .unwrap_or_else(|error| {
                    panic!("seed={seed:#x} operation={operation_index} error={error}")
                });
            assert_eq!(actual_results.len(), requests.len());
            for (request, actual_result) in requests.iter().zip(actual_results) {
                let expected_accept = {
                    let mut candidate = reference.clone();
                    reference_apply(&mut candidate, request, Lsn::new(1))
                };
                match (expected_accept, actual_result) {
                    (true, Ok(result)) => {
                        assert!(reference_apply(&mut reference, request, result.commit_lsn));
                    }
                    (false, Err(_)) => {}
                    (expected, actual) => panic!(
                        "seed={seed:#x} operation={operation_index} expected_accept={expected} actual={actual:?}"
                    ),
                }
            }
            if operation_index % 25 == 0 {
                for key_index in 0..64u64 {
                    let key =
                        DocumentKey::new(b"random".to_vec(), key_index.to_be_bytes().to_vec());
                    assert_eq!(
                        store.get(&key).unwrap(),
                        reference
                            .get(&key)
                            .cloned()
                            .unwrap_or_else(|| RevisionState::missing(Revision::ZERO)),
                        "seed={seed:#x} operation={operation_index} key={key:?}"
                    );
                }
            }
        }
        store.flush().unwrap();
        let (data, wal) = store.into_files();
        let mut reopened =
            BlinkStore::open_with_wal(data, wal.unwrap(), DatabaseConfig::default()).unwrap();
        for (key, expected) in &reference {
            assert_eq!(
                reopened.get(key).unwrap(),
                *expected,
                "seed={seed:#x} reopened key={key:?}"
            );
        }
        reopened.check_invariants().unwrap();
    }

    #[test]
    fn randomized_differential_seed_is_reproducible() {
        let seed = 0x51a1_2026_u64;
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        let mut reference = BTreeMap::<DocumentKey, (Vec<u8>, Revision)>::new();
        let mut state = seed;
        for operation in 0..500usize {
            state = splitmix_for_test(state);
            let index = (state as usize) % 100;
            let key = DocumentKey::new(b"p".to_vec(), index.to_be_bytes().to_vec());
            state = splitmix_for_test(state);
            if state & 1 == 0 {
                let value = vec![(operation & 0xff) as u8; 8 + operation % 16];
                let revision = store.put(key.clone(), value.clone()).unwrap();
                reference.insert(key, (value, revision));
            } else {
                store.delete(key.clone()).unwrap();
                reference.remove(&key);
            }
            state = splitmix_for_test(state);
            let get_key = DocumentKey::new(
                b"p".to_vec(),
                ((state as usize) % 100).to_be_bytes().to_vec(),
            );
            let actual = store.get(&get_key).unwrap();
            match reference.get(&get_key) {
                Some((value, revision)) => assert_eq!(
                    actual,
                    RevisionState::present(value.clone(), *revision),
                    "seed={seed:#x} operation={operation} key={get_key:?}"
                ),
                None => assert!(actual.is_missing(), "seed={seed:#x} operation={operation}"),
            }
        }
        store.check_invariants().unwrap();
    }

    #[test]
    fn structural_stress_reaches_internal_and_root_splits() {
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        for index in 0usize..1_000 {
            let mut pk = vec![0x51; 280];
            pk.extend_from_slice(&(index as u64).to_be_bytes());
            let key = DocumentKey::new(pk, vec![0x61; 280]);
            store.put(key, vec![index as u8; 8]).unwrap();
        }
        let metrics = store.split_metrics();
        assert!(metrics.leaf_splits > 0);
        assert!(metrics.internal_splits > 0);
        assert!(metrics.root_splits > 1);
        store.check_invariants().unwrap();
    }

    #[test]
    fn checker_rejects_sibling_cycle_wrong_level_and_unordered_separator() {
        let config = DatabaseConfig::default();
        let mut store = BlinkStore::<MemoryFile, MemoryFile>::open_with_wal(
            MemoryFile::default(),
            MemoryFile::default(),
            config,
        )
        .unwrap();
        for index in 0usize..300 {
            store
                .put(
                    DocumentKey::new(b"p".to_vec(), index.to_be_bytes().to_vec()),
                    vec![index as u8; 32],
                )
                .unwrap();
        }
        let leaf_id = store
            .state
            .pages
            .iter()
            .find_map(|(id, page)| {
                matches!(
                    &**page,
                    BlinkPage::Leaf {
                        right_sibling: Some(_),
                        ..
                    }
                )
                .then_some(*id)
            })
            .unwrap();
        let mut cycle = store.state.clone();
        if let Some(BlinkPage::Leaf { right_sibling, .. }) =
            cycle.pages.get_mut(&leaf_id).map(Arc::make_mut)
        {
            *right_sibling = Some(leaf_id);
        }
        assert!(check_state(&cycle).is_err());

        let mut wrong_level = store.state.clone();
        let root = wrong_level.root_page_id;
        if let Some(BlinkPage::Leaf { right_sibling, .. }) =
            wrong_level.pages.get_mut(&leaf_id).map(Arc::make_mut)
        {
            *right_sibling = Some(root);
        }
        assert!(check_state(&wrong_level).is_err());

        let internal_id = store
            .state
            .pages
            .iter()
            .find_map(|(id, page)| {
                matches!(&**page, BlinkPage::Internal { entries, .. } if entries.len() > 1)
                    .then_some(*id)
            })
            .unwrap();
        let mut unordered = store.state.clone();
        if let Some(BlinkPage::Internal { entries, .. }) =
            unordered.pages.get_mut(&internal_id).map(Arc::make_mut)
        {
            entries.swap(0, 1);
        }
        assert!(check_state(&unordered).is_err());
    }

    fn retention_key(index: u64) -> DocumentKey {
        DocumentKey::new(b"retention".to_vec(), (index % 257).to_be_bytes().to_vec())
    }

    fn assert_no_retained_wal_payload(store: &BlinkStore<MemoryFile, MemoryFile>) {
        let metrics = store.wal_metrics().unwrap().unwrap();
        assert_eq!(metrics.retained_recovery_batches, 0);
        assert_eq!(metrics.retained_recovery_page_images, 0);
        assert!(store.wal.as_ref().unwrap().recovery_batches().is_empty());
    }

    #[test]
    fn runtime_commits_do_not_retain_wal_page_images() {
        let mut store = planned_store();
        let before = store.wal_metrics().unwrap().unwrap();
        let mut last_revision = Revision::new(0);
        for index in 0..4_000u64 {
            last_revision = store
                .put(retention_key(index), index.to_be_bytes().to_vec())
                .unwrap();
            if index % 1_000 == 999 {
                assert_no_retained_wal_payload(&store);
            }
        }
        let after = store.wal_metrics().unwrap().unwrap();
        assert_eq!(after.committed_batches - before.committed_batches, 4_000);
        assert!(
            (after.page_images - before.page_images) as u64
                + (after.redo.page_delta_records - before.redo.page_delta_records)
                >= 4_000
        );
        assert_no_retained_wal_payload(&store);
        assert_eq!(
            store
                .wal
                .as_ref()
                .unwrap()
                .last_commit_lsn()
                .map(Revision::from),
            Some(last_revision)
        );
    }

    #[test]
    fn reopen_without_checkpoint_replays_every_commit_and_drops_payload() {
        let mut store = planned_store();
        let mut expected = BTreeMap::new();
        for index in 0..3_000u64 {
            let key = retention_key(index);
            let value = index.to_be_bytes().to_vec();
            store.put(key.clone(), value.clone()).unwrap();
            expected.insert(key, value);
        }
        let committed_before_close = store.wal_metrics().unwrap().unwrap().committed_batches;
        let last_commit_before_close = store.wal.as_ref().unwrap().last_commit_lsn();
        let (data, wal) = store.into_files();
        let config = DatabaseConfig::default();
        let reopened_wal = WalLog::open_with_page_image_format(
            wal.unwrap(),
            WalIdentity::new(
                config.database_uuid,
                config.tenant_id,
                config.shard_id,
                config.shard_epoch,
            ),
            WalPageImageFormat::ExperimentalBlink,
        )
        .unwrap();
        assert!(reopened_wal.recovery_batches().is_empty());
        assert!(!reopened_wal.recovery_pages().is_empty());
        assert_eq!(reopened_wal.committed_batch_count(), committed_before_close);
        assert_eq!(reopened_wal.last_commit_lsn(), last_commit_before_close);
        let wal = reopened_wal.into_file();
        let mut reopened = BlinkStore::open_with_wal(data, wal, DatabaseConfig::default()).unwrap();
        assert_no_retained_wal_payload(&reopened);
        assert_eq!(
            reopened.wal_metrics().unwrap().unwrap().committed_batches,
            committed_before_close
        );
        assert_eq!(
            reopened.wal.as_ref().unwrap().last_commit_lsn(),
            last_commit_before_close
        );
        for (key, value) in &expected {
            assert_eq!(reopened.get(key).unwrap().value(), Some(&value[..]));
        }
        reopened.check_invariants().unwrap();
        reopened
            .put(retention_key(0), b"after-reopen".to_vec())
            .unwrap();
        assert_no_retained_wal_payload(&reopened);
    }

    #[test]
    fn checkpoint_uses_last_commit_lsn_without_retained_payload() {
        let mut store = planned_store();
        let mut expected = BTreeMap::new();
        let mut last_revision = Revision::new(0);
        for index in 0..1_500u64 {
            let key = retention_key(index);
            let value = index.to_be_bytes().to_vec();
            last_revision = store.put(key.clone(), value.clone()).unwrap();
            expected.insert(key, value);
        }
        let report = store.checkpoint().unwrap();
        assert_eq!(Revision::from(report.checkpoint_lsn), last_revision);
        assert_eq!(
            store.current_superblock.checkpoint_lsn,
            report.checkpoint_lsn
        );
        assert_eq!(store.wal.as_ref().unwrap().last_commit_lsn(), None);
        assert_eq!(
            store.wal.as_ref().unwrap().history_start_lsn(),
            report.checkpoint_lsn
        );
        assert!(report.wal_bytes_reclaimed > 0);
        let idle = store.checkpoint().unwrap();
        assert_eq!(idle.checkpoint_lsn, report.checkpoint_lsn);
        assert_eq!(idle.wal_bytes_reclaimed, 0);

        for index in 1_500..2_500u64 {
            let key = retention_key(index);
            let value = index.to_be_bytes().to_vec();
            store.put(key.clone(), value.clone()).unwrap();
            expected.insert(key, value);
        }
        assert_no_retained_wal_payload(&store);
        let (data, wal) = store.into_files();
        let mut reopened =
            BlinkStore::open_with_wal(data, wal.unwrap(), DatabaseConfig::default()).unwrap();
        assert_no_retained_wal_payload(&reopened);
        assert_eq!(
            reopened.current_superblock.checkpoint_lsn,
            report.checkpoint_lsn
        );
        for (key, value) in &expected {
            assert_eq!(reopened.get(key).unwrap().value(), Some(&value[..]));
        }
        reopened.check_invariants().unwrap();
    }

    #[test]
    fn torn_final_commit_is_discarded_on_reopen_without_retained_payload() {
        let mut store = planned_store();
        let key = retention_key(7);
        store.put(key.clone(), b"durable".to_vec()).unwrap();
        for index in 0..500u64 {
            store
                .put(retention_key(index * 2 + 100), b"filler".to_vec())
                .unwrap();
        }
        store.put(key.clone(), b"durable-2".to_vec()).unwrap();
        let length_before_torn = store.wal_metrics().unwrap().unwrap().wal_bytes;
        let committed_before_torn = store.wal_metrics().unwrap().unwrap().committed_batches;
        store.put(key.clone(), b"torn".to_vec()).unwrap();
        let (data, wal) = store.into_files();
        let mut wal = wal.unwrap();
        let full_length = wal.len().unwrap();
        assert!(full_length > length_before_torn + 10);
        wal.set_len(full_length - 10).unwrap();
        let mut reopened = BlinkStore::open_with_wal(data, wal, DatabaseConfig::default()).unwrap();
        assert_no_retained_wal_payload(&reopened);
        let metrics = reopened.wal_metrics().unwrap().unwrap();
        assert!(metrics.wal_bytes >= length_before_torn);
        assert!(metrics.wal_bytes < full_length - 10);
        assert_eq!(metrics.committed_batches, committed_before_torn);
        assert_eq!(
            reopened.wal.as_ref().unwrap().scan_report().torn_tail_bytes as u64,
            full_length - 10 - metrics.wal_bytes
        );
        assert_eq!(reopened.get(&key).unwrap().value(), Some(&b"durable-2"[..]));
        reopened.check_invariants().unwrap();
        reopened.put(key.clone(), b"after-torn".to_vec()).unwrap();
        let (data, wal) = reopened.into_files();
        let mut again =
            BlinkStore::open_with_wal(data, wal.unwrap(), DatabaseConfig::default()).unwrap();
        assert_eq!(again.get(&key).unwrap().value(), Some(&b"after-torn"[..]));
        again.check_invariants().unwrap();
    }

    fn b0_key(index: u64) -> DocumentKey {
        let mut primary = vec![0x51u8; 8];
        primary[7] = (index % 128) as u8;
        let mut secondary = vec![0x61u8; 8];
        secondary.copy_from_slice(&index.to_be_bytes());
        DocumentKey::new(primary, secondary)
    }

    fn b0_value(index: u64, round: u64) -> Vec<u8> {
        let mut value = vec![0u8; 64];
        for (position, byte) in value.iter_mut().enumerate() {
            *byte = splitmix_for_test(index ^ (round << 32) ^ position as u64) as u8;
        }
        value
    }

    fn b0_percentile(values: &[u64], fraction: f64) -> u64 {
        if values.is_empty() {
            return 0;
        }
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        sorted[((sorted.len() - 1) as f64 * fraction).round() as usize]
    }

    fn b0_distribution(values: &[u64]) -> String {
        if values.is_empty() {
            return "{\"count\":0}".to_string();
        }
        let sum: u64 = values.iter().sum();
        format!(
            "{{\"count\":{},\"mean\":{:.1},\"p50\":{},\"p95\":{},\"max\":{},\"min\":{}}}",
            values.len(),
            sum as f64 / values.len() as f64,
            b0_percentile(values, 0.5),
            b0_percentile(values, 0.95),
            values.iter().max().unwrap(),
            values.iter().min().unwrap()
        )
    }

    fn b0_measure<Operation>(
        store: &mut BlinkStore<MemoryFile, MemoryFile>,
        scenario: &str,
        mut operation: Operation,
    ) where
        Operation: FnMut(&mut BlinkStore<MemoryFile, MemoryFile>),
    {
        let start_offset = store.wal_metrics().unwrap().unwrap().wal_bytes;
        let before = store.wal_metrics().unwrap().unwrap();
        let split_before = store.split_metrics();
        operation(store);
        let after = store.wal_metrics().unwrap().unwrap();
        let split_after = store.split_metrics();
        let frames = store
            .wal
            .as_mut()
            .unwrap()
            .frame_summaries_from(start_offset);
        let mut transaction_bytes: BTreeMap<u64, u64> = BTreeMap::new();
        let mut delta_frames = Vec::new();
        for (record_type, batch_id, frame_length) in &frames {
            *transaction_bytes.entry(*batch_id).or_default() += *frame_length as u64;
            if *record_type == crate::wal::WalRecordType::PageDelta {
                delta_frames.push(*frame_length as u64);
            }
        }
        let transaction_bytes = transaction_bytes.into_values().collect::<Vec<_>>();
        let redo = |metrics: &WalMetrics| metrics.redo;
        let (before_redo, after_redo) = (redo(&before), redo(&after));
        let delta_records = after_redo.page_delta_records - before_redo.page_delta_records;
        let image_records = after_redo.page_image_records - before_redo.page_image_records;
        let transactions = transaction_bytes.len() as u64;
        let data_records = delta_records + image_records
            - (after_redo.image_superblock - before_redo.image_superblock);
        println!(
            "B0 {{\"scenario\":\"{scenario}\",\"transactions\":{transactions},\"wal_bytes_per_tx\":{},\"page_delta_frame_bytes\":{},\"page_image_frame_bytes\":{},\"page_image_records\":{image_records},\"page_delta_records\":{delta_records},\"superblock_images\":{},\"delta_spans_mean\":{:.2},\"delta_changed_bytes_mean\":{:.1},\"delta_payload_bytes_mean\":{:.1},\"data_page_fallback_rate\":{:.4},\"fallback_ineligible_commit\":{},\"fallback_first_touch\":{},\"fallback_no_base\":{},\"fallback_not_smaller\":{},\"fallback_page_image_format\":{},\"leaf_splits\":{}}}",
            b0_distribution(&transaction_bytes),
            b0_distribution(&delta_frames),
            crate::wal::WAL_HEADER_SIZE + 8 + PAGE_SIZE + 4,
            after_redo.image_superblock - before_redo.image_superblock,
            (after_redo.page_delta_spans - before_redo.page_delta_spans) as f64
                / delta_records.max(1) as f64,
            (after_redo.page_delta_changed_bytes - before_redo.page_delta_changed_bytes) as f64
                / delta_records.max(1) as f64,
            (after_redo.page_delta_payload_bytes - before_redo.page_delta_payload_bytes) as f64
                / delta_records.max(1) as f64,
            (data_records - delta_records) as f64 / data_records.max(1) as f64,
            after_redo.image_ineligible_commit - before_redo.image_ineligible_commit,
            after_redo.image_first_touch - before_redo.image_first_touch,
            after_redo.image_no_base - before_redo.image_no_base,
            after_redo.image_not_smaller - before_redo.image_not_smaller,
            after_redo.image_page_image_format - before_redo.image_page_image_format,
            split_after.leaf_splits - split_before.leaf_splits,
        );
    }

    #[test]
    #[ignore = "B0 encoded-size probe; run explicitly with --ignored --nocapture"]
    fn phase_b0_encoded_size_probe() {
        const KEYS: u64 = 20_000;
        let mut store = planned_store();
        for chunk_start in (0..KEYS).step_by(25) {
            let requests = vec![TransactionRequest::new(
                Vec::new(),
                (chunk_start..(chunk_start + 25).min(KEYS))
                    .map(|index| TransactionMutation::Put {
                        key: b0_key(index),
                        value: b0_value(index, 0),
                    })
                    .collect(),
            )];
            for result in store.apply_transaction_group(&requests).unwrap() {
                result.unwrap();
            }
        }
        store.checkpoint().unwrap();
        let leaf_count = store
            .state
            .pages
            .values()
            .filter(|page| matches!(&***page, BlinkPage::Leaf { .. }))
            .count();
        println!("B0 {{\"setup_keys\":{KEYS},\"leaf_pages\":{leaf_count}}}");
        let mut state = 0xb0b0u64;
        let mut next_index = || {
            state = splitmix_for_test(state);
            state % KEYS
        };

        b0_measure(
            &mut store,
            "first_touch_after_checkpoint_width1_update",
            |store| {
                for index in (0..KEYS).step_by(40) {
                    store.put(b0_key(index), b0_value(index, 1)).unwrap();
                }
            },
        );
        for index in 0..KEYS {
            store.put(b0_key(index), b0_value(index, 2)).unwrap();
        }
        b0_measure(&mut store, "existing_key_width1_update_64b", |store| {
            for round in 0..4_000u64 {
                let index = next_index();
                store
                    .put(b0_key(index), b0_value(index, 10 + round))
                    .unwrap();
            }
        });
        b0_measure(&mut store, "delete_existing_key", |store| {
            for index in (1..KEYS).step_by(37).take(500) {
                store.delete(b0_key(index)).unwrap();
            }
        });
        b0_measure(&mut store, "insert_new_key", |store| {
            for index in (1..KEYS).step_by(37).take(500) {
                store.put(b0_key(index), b0_value(index, 7)).unwrap();
            }
        });
        b0_measure(&mut store, "existing_key_width16_update", |store| {
            for round in 0..500u64 {
                let mut indexes = BTreeSet::new();
                while indexes.len() < 16 {
                    indexes.insert(next_index());
                }
                let request = TransactionRequest::new(
                    Vec::new(),
                    indexes
                        .into_iter()
                        .map(|index| TransactionMutation::Put {
                            key: b0_key(index),
                            value: b0_value(index, 20_000 + round),
                        })
                        .collect(),
                );
                store.transact(request).unwrap();
            }
        });
        b0_measure(&mut store, "same_leaf_16_transactions_per_group", |store| {
            for round in 0..200u64 {
                let base = next_index() / 128 * 128;
                let requests = (0..16u64)
                    .map(|slot| {
                        let index = (base + slot * 128) % KEYS;
                        TransactionRequest::new(
                            Vec::new(),
                            vec![TransactionMutation::Put {
                                key: b0_key(index),
                                value: b0_value(index, 40_000 + round * 16 + slot),
                            }],
                        )
                    })
                    .collect::<Vec<_>>();
                for result in store.apply_transaction_group(&requests).unwrap() {
                    result.unwrap();
                }
            }
        });
        b0_measure(
            &mut store,
            "different_leaf_16_transactions_per_group",
            |store| {
                for round in 0..200u64 {
                    let requests = (0..16u64)
                        .map(|slot| {
                            let index = next_index();
                            TransactionRequest::new(
                                Vec::new(),
                                vec![TransactionMutation::Put {
                                    key: b0_key(index),
                                    value: b0_value(index, 80_000 + round * 16 + slot),
                                }],
                            )
                        })
                        .collect::<Vec<_>>();
                    for result in store.apply_transaction_group(&requests).unwrap() {
                        result.unwrap();
                    }
                }
            },
        );
        let (data, wal) = store.into_files();
        let reopened =
            BlinkStore::open_with_wal(data, wal.unwrap(), DatabaseConfig::default()).unwrap();
        reopened.check_invariants().unwrap();
    }

    fn blink_wal_identity() -> WalIdentity {
        let config = DatabaseConfig::default();
        WalIdentity::new(
            config.database_uuid,
            config.tenant_id,
            config.shard_id,
            config.shard_epoch,
        )
    }

    fn random_test_bytes(state: &mut u64, length: usize) -> Vec<u8> {
        (0..length)
            .map(|_| {
                *state = splitmix_for_test(*state);
                *state as u8
            })
            .collect()
    }

    fn random_leaf_entry(state: &mut u64, revision: u64) -> LeafEntry {
        *state = splitmix_for_test(*state);
        let key_length = 2 + (*state % 20) as usize;
        let primary = random_test_bytes(state, key_length);
        let secondary = random_test_bytes(state, key_length / 2 + 1);
        *state = splitmix_for_test(*state);
        let value = if (*state).is_multiple_of(6) {
            None
        } else {
            let value_length = (*state % 90) as usize;
            Some(OwnedValue::Inline(Vec::from(
                random_test_bytes(state, value_length).as_slice(),
            )))
        };
        LeafEntry {
            key: DocumentKey::new(primary, secondary).encode(),
            revision: Revision::new(revision),
            value,
        }
    }

    fn random_leaf_page(seed: u64, lsn: Lsn) -> BlinkPage {
        let mut state = seed;
        state = splitmix_for_test(state);
        let entry_count = 1 + (state % 30) as usize;
        let mut entries = BTreeMap::new();
        for _ in 0..entry_count {
            state = splitmix_for_test(state);
            let revision = 1 + state % lsn.get();
            let entry = random_leaf_entry(&mut state, revision);
            entries.insert(entry.key.to_vec(), entry);
        }
        BlinkPage::Leaf {
            lsn,
            high_key: None,
            right_sibling: None,
            entries: pack_leaf(&entries.into_values().collect::<Vec<_>>()),
        }
    }

    fn mutate_leaf_page(base: &BlinkPage, seed: u64) -> BlinkPage {
        let BlinkPage::Leaf {
            lsn,
            high_key,
            right_sibling,
            entries,
        } = base
        else {
            panic!("test page is a leaf");
        };
        let mut state = seed;
        state = splitmix_for_test(state);
        let target_lsn = Lsn::new(lsn.get() + 1 + state % 5);
        let revision = Revision::new(target_lsn.get());
        let mut entries = logical_entries(entries);
        state = splitmix_for_test(state);
        let position = (state as usize) % entries.len();
        state = splitmix_for_test(state);
        match state % 6 {
            0 => {
                let length = match &entries[position].value {
                    Some(OwnedValue::Inline(value)) => value.len(),
                    _ => 8,
                };
                entries[position].value = Some(OwnedValue::Inline(Vec::from(
                    random_test_bytes(&mut state, length).as_slice(),
                )));
                entries[position].revision = revision;
            }
            1 => {
                let length = (splitmix_for_test(state) % 90) as usize;
                entries[position].value = Some(OwnedValue::Inline(Vec::from(
                    random_test_bytes(&mut state, length).as_slice(),
                )));
                entries[position].revision = revision;
            }
            2 if entries.len() > 1 => {
                entries.remove(position);
            }
            3 => {
                let entry = random_leaf_entry(&mut state, target_lsn.get());
                if let Err(insert_at) =
                    entries.binary_search_by(|existing| existing.key.cmp(&entry.key))
                {
                    entries.insert(insert_at, entry);
                }
            }
            4 => {
                entries[position].value = None;
                entries[position].revision = revision;
            }
            _ => {
                entries[position].revision = revision;
            }
        }
        BlinkPage::Leaf {
            lsn: target_lsn,
            high_key: high_key.clone(),
            right_sibling: *right_sibling,
            entries: pack_leaf(&entries),
        }
    }

    #[test]
    fn page_delta_round_trips_randomized_blink_page_mutations() {
        use crate::wal::{apply_page_delta, decode_page_delta, encode_page_delta};
        let mut round_trips = 0;
        for seed in 0..3_000u64 {
            let page_id = PageId::new(FIRST_DATA_PAGE + seed % 50);
            let base_page = random_leaf_page(seed, Lsn::new(100 + seed));
            let Ok(base) = encode_blink_page(page_id, &base_page) else {
                continue;
            };
            let target_page = mutate_leaf_page(&base_page, seed ^ 0x5eed);
            let Ok(target) = encode_blink_page(page_id, &target_page) else {
                continue;
            };
            let payload = encode_page_delta(page_id, &base, &target).unwrap();
            let view = decode_page_delta(&payload).unwrap();
            assert_eq!(view.page_id, page_id);
            assert_eq!(view.base_page_lsn, base_page.lsn());
            let rebuilt = apply_page_delta(&base, &view).unwrap();
            assert_eq!(rebuilt, target, "seed {seed}");
            validate_blink_page_image(&rebuilt, page_id).unwrap();
            assert_eq!(
                encode_page_delta(page_id, &base, &rebuilt).unwrap(),
                payload
            );
            assert_eq!(encode_page_delta(page_id, &base, &target).unwrap(), payload);
            round_trips += 1;
        }
        assert!(round_trips > 2_500, "{round_trips}");
    }

    #[test]
    fn page_delta_round_trips_randomized_raw_byte_edits() {
        use crate::wal::{apply_page_delta, decode_page_delta, encode_page_delta};
        for seed in 0..2_000u64 {
            let mut state = seed ^ 0xdead_beef;
            let mut base = [0u8; PAGE_SIZE];
            base.copy_from_slice(&random_test_bytes(&mut state, PAGE_SIZE));
            let mut target = base;
            state = splitmix_for_test(state);
            let edit_count = 1 + state % 40;
            for _ in 0..edit_count {
                state = splitmix_for_test(state);
                let offset = match state % 10 {
                    0 => 0,
                    1 => PAGE_SIZE - 1,
                    _ => (state >> 8) as usize % PAGE_SIZE,
                };
                state = splitmix_for_test(state);
                let length = (1 + state % 12) as usize;
                for position in offset..(offset + length).min(PAGE_SIZE) {
                    target[position] = base[position].wrapping_add(1 + (state % 200) as u8);
                }
            }
            target[16..24].copy_from_slice(&(seed + 1).to_le_bytes());
            base[16..24].copy_from_slice(&seed.to_le_bytes());
            let payload = encode_page_delta(PageId::new(7), &base, &target).unwrap();
            let rebuilt = apply_page_delta(&base, &decode_page_delta(&payload).unwrap()).unwrap();
            assert_eq!(rebuilt, target, "seed {seed}");
            assert_eq!(
                encode_page_delta(PageId::new(7), &base, &rebuilt).unwrap(),
                payload
            );
        }
    }

    #[test]
    fn page_delta_rebuild_check_matches_apply_and_compare() {
        use crate::wal::{apply_page_delta, decode_page_delta, encode_page_delta};
        let mut checked_equal = 0u32;
        let mut checked_different = 0u32;
        let mut checked_errors = 0u32;
        for seed in 0..3_000u64 {
            let mut state = seed ^ 0x5eed_cafe;
            let mut base = [0u8; PAGE_SIZE];
            base.copy_from_slice(&random_test_bytes(&mut state, PAGE_SIZE));
            let mut target = base;
            state = splitmix_for_test(state);
            for _ in 0..1 + state % 30 {
                state = splitmix_for_test(state);
                let offset = (state >> 8) as usize % PAGE_SIZE;
                let length = (1 + state % 12) as usize;
                for position in offset..(offset + length).min(PAGE_SIZE) {
                    target[position] = base[position].wrapping_add(1 + (state % 200) as u8);
                }
            }
            base[16..24].copy_from_slice(&seed.to_le_bytes());
            target[16..24].copy_from_slice(&(seed + 1).to_le_bytes());
            let payload = encode_page_delta(PageId::new(9), &base, &target).unwrap();
            let view = decode_page_delta(&payload).unwrap();
            let mut candidates = vec![target];
            state = splitmix_for_test(state);
            let mut flipped = target;
            flipped[(state >> 8) as usize % PAGE_SIZE] ^= 1 << (state % 8);
            candidates.push(flipped);
            let mut other_base = base;
            other_base[(state >> 20) as usize % PAGE_SIZE] ^= 0x40;
            for (base_variant, candidate) in candidates
                .iter()
                .map(|candidate| (&base, candidate))
                .chain(std::iter::once((&other_base, &target)))
            {
                let expected =
                    apply_page_delta(base_variant, &view).map(|rebuilt| rebuilt == *candidate);
                let actual = page_delta_rebuilds(base_variant, &view, candidate);
                match (expected, actual) {
                    (Ok(expected), Ok(actual)) => {
                        assert_eq!(expected, actual, "seed {seed}");
                        if expected {
                            checked_equal += 1;
                        } else {
                            checked_different += 1;
                        }
                    }
                    (Err(expected), Err(actual)) => {
                        assert_eq!(expected.to_string(), actual.to_string(), "seed {seed}");
                        checked_errors += 1;
                    }
                    (expected, actual) => {
                        panic!("seed {seed}: apply {expected:?}, rebuild check {actual:?}")
                    }
                }
            }
        }
        assert!(checked_equal >= 3_000 && checked_different >= 2_000 && checked_errors > 0);
    }

    #[test]
    fn page_delta_spans_merge_only_small_unchanged_gaps() {
        use crate::wal::{PAGE_DELTA_SPAN_MERGE_GAP, decode_page_delta, encode_page_delta};
        let base = [0u8; PAGE_SIZE];
        let mut target = base;
        target[100] = 1;
        target[100 + PAGE_DELTA_SPAN_MERGE_GAP + 1] = 1;
        let merged = encode_page_delta(PageId::new(3), &base, &target).unwrap();
        let view = decode_page_delta(&merged).unwrap();
        assert_eq!(view.spans.len(), 1);
        assert_eq!(view.spans[0].0, 100);
        assert_eq!(view.spans[0].1.len(), PAGE_DELTA_SPAN_MERGE_GAP + 2);

        let mut target = base;
        target[100] = 1;
        target[100 + PAGE_DELTA_SPAN_MERGE_GAP + 2] = 1;
        let split = encode_page_delta(PageId::new(3), &base, &target).unwrap();
        let view = decode_page_delta(&split).unwrap();
        assert_eq!(view.spans.len(), 2);
        assert_eq!(view.spans[1].0, 100 + PAGE_DELTA_SPAN_MERGE_GAP + 2);
        assert!(encode_page_delta(PageId::new(3), &base, &base).is_err());
    }

    fn manual_delta_payload(
        page_id: u64,
        base_lsn: u64,
        span_count: u16,
        spans: &[(u16, u16, &[u8])],
    ) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&page_id.to_le_bytes());
        payload.extend_from_slice(&base_lsn.to_le_bytes());
        payload.extend_from_slice(&span_count.to_le_bytes());
        for (offset, length, bytes) in spans {
            payload.extend_from_slice(&offset.to_le_bytes());
            payload.extend_from_slice(&length.to_le_bytes());
            payload.extend_from_slice(bytes);
        }
        payload
    }

    fn delta_test_leaf(page_id: PageId, lsn: Lsn, value_byte: u8) -> [u8; PAGE_SIZE] {
        let entries = (0..8u8)
            .map(|index| LeafEntry {
                key: DocumentKey::new(b"delta".to_vec(), vec![index; 4]).encode(),
                revision: Revision::new(if index == 3 { lsn.get() } else { 1 }),
                value: Some(OwnedValue::Inline(Vec::from(
                    vec![if index == 3 { value_byte } else { index }; 64].as_slice(),
                ))),
            })
            .collect::<Vec<LeafEntry>>();
        encode_blink_page(
            page_id,
            &BlinkPage::Leaf {
                lsn,
                high_key: None,
                right_sibling: None,
                entries: pack_leaf(&entries),
            },
        )
        .unwrap()
    }

    #[test]
    fn malformed_page_deltas_are_rejected_by_the_codec() {
        use crate::wal::{
            PAGE_DELTA_MAX_SPANS, apply_page_delta, decode_page_delta, encode_page_delta,
        };
        let page_id = PageId::new(5);
        let base = delta_test_leaf(page_id, Lsn::new(10), 0xaa);
        let target = delta_test_leaf(page_id, Lsn::new(11), 0xbb);
        let valid = encode_page_delta(page_id, &base, &target).unwrap();
        assert_eq!(
            apply_page_delta(&base, &decode_page_delta(&valid).unwrap()).unwrap(),
            target
        );

        let mut other_lsn_base = base;
        other_lsn_base[16..24].copy_from_slice(&9u64.to_le_bytes());
        assert!(apply_page_delta(&other_lsn_base, &decode_page_delta(&valid).unwrap()).is_err());

        let mut bad_count = valid.clone();
        bad_count[16..18].copy_from_slice(&0u16.to_le_bytes());
        assert!(decode_page_delta(&bad_count).is_err());
        let mut bad_count = valid.clone();
        let count = u16::from_le_bytes(valid[16..18].try_into().unwrap());
        bad_count[16..18].copy_from_slice(&(count + 1).to_le_bytes());
        assert!(decode_page_delta(&bad_count).is_err());
        let mut bad_count = valid.clone();
        bad_count[16..18].copy_from_slice(&((PAGE_DELTA_MAX_SPANS + 1) as u16).to_le_bytes());
        assert!(decode_page_delta(&bad_count).is_err());

        assert!(decode_page_delta(&valid[..valid.len() - 1]).is_err());
        let mut trailing = valid.clone();
        trailing.push(0);
        assert!(decode_page_delta(&trailing).is_err());
        assert!(decode_page_delta(&valid[..10]).is_err());

        let zero_length = manual_delta_payload(5, 10, 1, &[(100, 0, &[])]);
        assert!(decode_page_delta(&zero_length).is_err());
        let past_end = manual_delta_payload(5, 10, 1, &[(4095, 2, &[1, 2])]);
        assert!(decode_page_delta(&past_end).is_err());
        let outside = manual_delta_payload(5, 10, 1, &[(4096, 1, &[1])]);
        assert!(decode_page_delta(&outside).is_err());
        let overlap = manual_delta_payload(5, 10, 2, &[(100, 10, &[1; 10]), (105, 5, &[1; 5])]);
        assert!(decode_page_delta(&overlap).is_err());
        let unsorted = manual_delta_payload(5, 10, 2, &[(200, 1, &[1]), (100, 1, &[1])]);
        assert!(decode_page_delta(&unsorted).is_err());
        let small_gap = manual_delta_payload(5, 10, 2, &[(100, 1, &[1]), (103, 1, &[1])]);
        assert!(decode_page_delta(&small_gap).is_err());
        let huge = manual_delta_payload(5, 10, 1, &[(0, 4096, &[1; 4096])]);
        assert!(decode_page_delta(&huge).is_err());

        let unchanged_start = manual_delta_payload(5, 10, 1, &[(40, 1, &[base[40]])]);
        assert!(apply_page_delta(&base, &decode_page_delta(&unchanged_start).unwrap()).is_err());
        let mut long_gap_bytes = base[40..50].to_vec();
        long_gap_bytes[0] ^= 1;
        long_gap_bytes[9] ^= 1;
        let long_gap = manual_delta_payload(5, 10, 1, &[(40, 10, &long_gap_bytes)]);
        assert!(apply_page_delta(&base, &decode_page_delta(&long_gap).unwrap()).is_err());

        let mut bad_checksum_target = target;
        bad_checksum_target[PAGE_SIZE - 3] ^= 0x40;
        let bad_checksum = encode_page_delta(page_id, &base, &bad_checksum_target).unwrap();
        let rebuilt = apply_page_delta(&base, &decode_page_delta(&bad_checksum).unwrap()).unwrap();
        assert!(validate_blink_page_image(&rebuilt, page_id).is_err());

        let mut wrong_page = valid.clone();
        wrong_page[0..8].copy_from_slice(&6u64.to_le_bytes());
        let view = decode_page_delta(&wrong_page).unwrap();
        let rebuilt = apply_page_delta(&base, &view).unwrap();
        assert!(validate_blink_page_image(&rebuilt, view.page_id).is_err());
    }

    fn blink_test_wal() -> WalLog<MemoryFile> {
        WalLog::open_with_page_image_format(
            MemoryFile::default(),
            blink_wal_identity(),
            WalPageImageFormat::ExperimentalBlink,
        )
        .unwrap()
    }

    fn delta_commits(
        wal: &WalLog<MemoryFile>,
        pages_per_commit: &[&[(u64, u8)]],
    ) -> Vec<WalCommit> {
        let mut next_lsn = wal.next_lsn().get();
        let mut batch_id = wal.next_batch_id();
        pages_per_commit
            .iter()
            .map(|pages| {
                let commit_lsn = Lsn::new(next_lsn + pages.len() as u64);
                let commit = WalCommit {
                    batch_id,
                    commit_lsn,
                    pages: pages
                        .iter()
                        .map(|(page_id, value_byte)| WalPageImage {
                            page_id: PageId::new(*page_id),
                            image: delta_test_leaf(PageId::new(*page_id), commit_lsn, *value_byte),
                        })
                        .collect(),
                };
                next_lsn = commit_lsn.get() + 1;
                batch_id += 1;
                commit
            })
            .collect()
    }

    fn append_with_bases(
        wal: &mut WalLog<MemoryFile>,
        commits: &[WalCommit],
        bases: &BTreeMap<PageId, [u8; PAGE_SIZE]>,
    ) -> Result<()> {
        let eligible = vec![true; commits.len()];
        let mut source = |page_id: PageId| bases.get(&page_id).map(Cow::Borrowed);
        let mut request = WalDeltaRequest {
            eligible_commits: &eligible,
            base_source: &mut source,
        };
        wal.append_group_with_page_deltas(commits, &mut request, None)
            .map(|_| ())
    }

    fn latest_images(commits: &[WalCommit], bases: &mut BTreeMap<PageId, [u8; PAGE_SIZE]>) {
        for commit in commits {
            for page in &commit.pages {
                bases.insert(page.page_id, page.image);
            }
        }
    }

    fn redo_kinds(wal: &mut WalLog<MemoryFile>) -> Vec<Vec<crate::wal::WalRedoKind>> {
        wal.committed_batches_on_disk()
            .into_iter()
            .map(|batch| batch.redo_kinds)
            .collect()
    }

    #[test]
    fn first_redo_after_wal_reset_is_a_full_image_then_deltas() {
        use crate::wal::WalRedoKind::{PageDelta, PageImage};
        let mut wal = blink_test_wal();
        let mut bases = BTreeMap::new();
        for value_byte in [1u8, 2, 3] {
            let commits = delta_commits(&wal, &[&[(5, value_byte)]]);
            append_with_bases(&mut wal, &commits, &bases).unwrap();
            latest_images(&commits, &mut bases);
        }
        assert_eq!(
            redo_kinds(&mut wal),
            vec![vec![PageImage], vec![PageDelta], vec![PageDelta]]
        );
        let metrics = wal.metrics().unwrap();
        assert_eq!(metrics.redo.image_first_touch, 1);
        assert_eq!(metrics.redo.page_delta_records, 2);
        let latest = bases[&PageId::new(5)];
        let last_commit = wal.last_commit_lsn().unwrap();

        let mut reopened = WalLog::open_with_page_image_format(
            wal.into_file(),
            blink_wal_identity(),
            WalPageImageFormat::ExperimentalBlink,
        )
        .unwrap();
        let recovered = reopened.take_recovery_pages();
        assert_eq!(*recovered[&PageId::new(5)].image, latest);
        assert_eq!(recovered[&PageId::new(5)].commit_lsn, last_commit);
        assert_eq!(reopened.metrics().unwrap().tracked_chain_pages, 1);

        let commits = delta_commits(&reopened, &[&[(5, 4)]]);
        append_with_bases(&mut reopened, &commits, &bases).unwrap();
        latest_images(&commits, &mut bases);
        reopened
            .reset(reopened.last_commit_lsn().unwrap(), None)
            .unwrap();
        assert_eq!(reopened.metrics().unwrap().tracked_chain_pages, 0);
        let commits = delta_commits(&reopened, &[&[(5, 5)]]);
        append_with_bases(&mut reopened, &commits, &bases).unwrap();
        assert_eq!(redo_kinds(&mut reopened), vec![vec![PageImage]]);
    }

    #[test]
    fn page_delta_base_mismatch_is_an_invariant_error() {
        let mut wal = blink_test_wal();
        let mut bases = BTreeMap::new();
        let commits = delta_commits(&wal, &[&[(5, 1)]]);
        append_with_bases(&mut wal, &commits, &bases).unwrap();
        latest_images(&commits, &mut bases);
        let length = wal.metrics().unwrap().wal_bytes;
        let committed_lsn = commits[0].commit_lsn;

        let mut wrong_bytes = bases.clone();
        wrong_bytes.insert(
            PageId::new(5),
            delta_test_leaf(PageId::new(5), committed_lsn, 9),
        );
        let next = delta_commits(&wal, &[&[(5, 2)]]);
        assert!(matches!(
            append_with_bases(&mut wal, &next, &wrong_bytes),
            Err(Error::InternalInvariantViolation(_))
        ));
        let mut stale_lsn = bases.clone();
        stale_lsn.insert(
            PageId::new(5),
            delta_test_leaf(PageId::new(5), Lsn::new(committed_lsn.get() - 1), 1),
        );
        assert!(matches!(
            append_with_bases(&mut wal, &next, &stale_lsn),
            Err(Error::InternalInvariantViolation(_))
        ));
        assert_eq!(wal.metrics().unwrap().wal_bytes, length);
        append_with_bases(&mut wal, &next, &bases).unwrap();
    }

    #[test]
    fn same_page_transactions_in_one_group_chain_in_fifo_order() {
        use crate::wal::WalRedoKind::{PageDelta, PageImage};
        let mut wal = blink_test_wal();
        let bases = BTreeMap::new();
        let group = delta_commits(&wal, &[&[(5, 1)], &[(5, 2)], &[(5, 3), (6, 3)], &[(6, 4)]]);
        append_with_bases(&mut wal, &group, &bases).unwrap();
        assert_eq!(
            redo_kinds(&mut wal),
            vec![
                vec![PageImage],
                vec![PageDelta],
                vec![PageDelta, PageImage],
                vec![PageDelta]
            ]
        );
        let mut file = wal.into_file();
        let frames = crate::wal::parse_wal_frames_for_test(&file.0);
        let delta_bases = frames
            .iter()
            .filter(|frame| frame.record_type == crate::wal::WalRecordType::PageDelta)
            .map(|frame| {
                (
                    u64::from_le_bytes(frame.payload[0..8].try_into().unwrap()),
                    u64::from_le_bytes(frame.payload[8..16].try_into().unwrap()),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            delta_bases,
            vec![
                (5, group[0].commit_lsn.get()),
                (5, group[1].commit_lsn.get()),
                (6, group[2].commit_lsn.get())
            ]
        );
        file.0.truncate(file.0.len());
        let mut reopened = WalLog::open_with_page_image_format(
            file,
            blink_wal_identity(),
            WalPageImageFormat::ExperimentalBlink,
        )
        .unwrap();
        let recovered = reopened.take_recovery_pages();
        assert_eq!(*recovered[&PageId::new(5)].image, group[2].pages[0].image);
        assert_eq!(*recovered[&PageId::new(6)].image, group[3].pages[0].image);
    }

    #[test]
    fn page_deltas_skip_superblocks_ineligible_commits_and_page_image_wals() {
        use crate::wal::WalRedoKind::PageImage;
        let store = planned_store();
        let superblock = encode_blink_superblock(&store.current_superblock).unwrap();
        let mut wal = blink_test_wal();
        let mut bases = BTreeMap::new();
        let commits = delta_commits(&wal, &[&[(5, 1)]]);
        append_with_bases(&mut wal, &commits, &bases).unwrap();
        latest_images(&commits, &mut bases);
        let commit_lsn = Lsn::new(wal.next_lsn().get() + 2);
        let commits = vec![WalCommit {
            batch_id: wal.next_batch_id(),
            commit_lsn,
            pages: vec![
                WalPageImage {
                    page_id: PageId::new(5),
                    image: delta_test_leaf(PageId::new(5), commit_lsn, 2),
                },
                WalPageImage {
                    page_id: PageId::ZERO,
                    image: superblock,
                },
            ],
        }];
        let eligible = vec![false];
        let mut source = |page_id: PageId| bases.get(&page_id).map(Cow::Borrowed);
        let mut request = WalDeltaRequest {
            eligible_commits: &eligible,
            base_source: &mut source,
        };
        wal.append_group_with_page_deltas(&commits, &mut request, None)
            .unwrap();
        assert_eq!(redo_kinds(&mut wal)[1], vec![PageImage, PageImage]);
        let redo = wal.metrics().unwrap().redo;
        assert_eq!(redo.image_superblock, 1);
        assert_eq!(redo.image_ineligible_commit, 1);

        let legacy = MemoryFile(crate::wal::init_frame_for_test(
            2,
            &blink_wal_identity(),
            Lsn::ZERO,
        ));
        let mut legacy_wal = WalLog::open_with_page_image_format(
            legacy,
            blink_wal_identity(),
            WalPageImageFormat::ExperimentalBlink,
        )
        .unwrap();
        assert!(!legacy_wal.page_delta_enabled());
        let mut legacy_bases = BTreeMap::new();
        for value_byte in [1u8, 2] {
            let commits = delta_commits(&legacy_wal, &[&[(5, value_byte)]]);
            append_with_bases(&mut legacy_wal, &commits, &legacy_bases).unwrap();
            latest_images(&commits, &mut legacy_bases);
        }
        assert_eq!(
            redo_kinds(&mut legacy_wal),
            vec![vec![PageImage], vec![PageImage]]
        );
        assert_eq!(
            legacy_wal.metrics().unwrap().redo.image_page_image_format,
            2
        );
        legacy_wal
            .reset(legacy_wal.last_commit_lsn().unwrap(), None)
            .unwrap();
        assert!(legacy_wal.page_delta_enabled());
    }

    fn two_commit_delta_wal() -> (Vec<crate::wal::TestWalFrame>, Vec<WalCommit>) {
        let mut wal = blink_test_wal();
        let mut bases = BTreeMap::new();
        let first = delta_commits(&wal, &[&[(5, 1), (6, 1)]]);
        append_with_bases(&mut wal, &first, &bases).unwrap();
        latest_images(&first, &mut bases);
        let second = delta_commits(&wal, &[&[(5, 2), (6, 2)]]);
        append_with_bases(&mut wal, &second, &bases).unwrap();
        let frames = crate::wal::parse_wal_frames_for_test(&wal.into_file().0);
        (frames, [first, second].concat())
    }

    fn open_test_frames(frames: &[crate::wal::TestWalFrame]) -> Result<WalLog<MemoryFile>> {
        WalLog::open_with_page_image_format(
            MemoryFile(crate::wal::serialize_wal_frames_for_test(frames)),
            blink_wal_identity(),
            WalPageImageFormat::ExperimentalBlink,
        )
    }

    #[test]
    fn malformed_page_delta_records_are_rejected_on_scan() {
        use crate::wal::{WalRecordType, encode_page_delta, recompute_commit_digests_for_test};
        let (frames, commits) = two_commit_delta_wal();
        let delta_frame = frames
            .iter()
            .position(|frame| frame.record_type == WalRecordType::PageDelta)
            .unwrap();
        let mut control = frames.clone();
        recompute_commit_digests_for_test(&mut control);
        let mut control_wal = open_test_frames(&control).unwrap();
        assert_eq!(
            *control_wal.take_recovery_pages()[&PageId::new(5)].image,
            commits[1].pages[0].image
        );

        let valid_payload = frames[delta_frame].payload.clone();
        let mut bad_checksum_target = commits[1].pages[0].image;
        bad_checksum_target[PAGE_SIZE - 3] ^= 0x40;
        let bad_checksum_payload = encode_page_delta(
            PageId::new(5),
            &commits[0].pages[0].image,
            &bad_checksum_target,
        )
        .unwrap();
        let mut variants: Vec<(&str, Vec<u8>)> = Vec::new();
        let mut payload = valid_payload.clone();
        payload[8..16].copy_from_slice(&(commits[0].commit_lsn.get() - 1).to_le_bytes());
        variants.push(("bad base LSN", payload));
        let mut payload = valid_payload.clone();
        payload[0..8].copy_from_slice(&6u64.to_le_bytes());
        variants.push(("wrong page ID with another base", payload));
        let mut payload = valid_payload.clone();
        payload[0..8].copy_from_slice(&77u64.to_le_bytes());
        variants.push(("page without base", payload));
        let mut payload = valid_payload.clone();
        payload[0..8].copy_from_slice(&0u64.to_le_bytes());
        variants.push(("superblock target", payload));
        let mut payload = valid_payload.clone();
        payload[20..22].copy_from_slice(&0u16.to_le_bytes());
        variants.push(("zero-length span", payload));
        let mut payload = valid_payload.clone();
        payload[18..20].copy_from_slice(&4095u16.to_le_bytes());
        variants.push(("out-of-range span", payload));
        let mut payload = valid_payload.clone();
        payload[16..18].copy_from_slice(&0u16.to_le_bytes());
        variants.push(("bad span count", payload));
        variants.push((
            "truncated payload",
            valid_payload[..valid_payload.len() - 1].to_vec(),
        ));
        variants.push(("invalid rebuilt checksum", bad_checksum_payload));
        variants.push((
            "overlapping spans",
            manual_delta_payload(
                5,
                commits[0].commit_lsn.get(),
                2,
                &[(100, 10, &[1; 10]), (105, 5, &[1; 5])],
            ),
        ));
        variants.push((
            "unsorted spans",
            manual_delta_payload(
                5,
                commits[0].commit_lsn.get(),
                2,
                &[(200, 1, &[1]), (100, 1, &[1])],
            ),
        ));
        for (name, payload) in variants {
            let mut malformed = frames.clone();
            malformed[delta_frame].payload = payload;
            recompute_commit_digests_for_test(&mut malformed);
            assert!(
                matches!(open_test_frames(&malformed), Err(Error::Corruption(_))),
                "{name}"
            );
        }

        let mut bad_digest = control.clone();
        let commit_frame = bad_digest
            .iter()
            .rposition(|frame| frame.record_type == WalRecordType::Commit)
            .unwrap();
        bad_digest[commit_frame].payload[12] ^= 1;
        assert!(matches!(
            open_test_frames(&bad_digest),
            Err(Error::Corruption(_))
        ));

        let mut in_page_image_wal = control.clone();
        for frame in &mut in_page_image_wal {
            frame.version = 2;
        }
        recompute_commit_digests_for_test(&mut in_page_image_wal);
        assert!(matches!(
            open_test_frames(&in_page_image_wal),
            Err(Error::Corruption(_))
        ));
    }

    #[test]
    fn commit_digest_binds_the_exact_redo_record_sequence() {
        use crate::wal::{WalRecordType, record_digest_for_test};
        let (frames, _) = two_commit_delta_wal();
        let first_delta = frames
            .iter()
            .position(|frame| frame.record_type == WalRecordType::PageDelta)
            .unwrap();
        let second_delta = first_delta + 1;
        assert_eq!(frames[second_delta].record_type, WalRecordType::PageDelta);
        let commit_frame = second_delta + 1;
        assert_eq!(frames[commit_frame].record_type, WalRecordType::Commit);
        assert!(open_test_frames(&frames).is_ok());

        let mut omission = frames.clone();
        omission.remove(second_delta);
        let commit = omission
            .iter_mut()
            .rfind(|frame| frame.record_type == WalRecordType::Commit)
            .unwrap();
        commit.record_index = 1;
        commit.payload[8..12].copy_from_slice(&1u32.to_le_bytes());
        assert!(matches!(
            open_test_frames(&omission),
            Err(Error::Corruption(_))
        ));

        let mut duplication = frames.clone();
        duplication[second_delta].payload = duplication[first_delta].payload.clone();
        assert!(matches!(
            open_test_frames(&duplication),
            Err(Error::Corruption(_))
        ));

        let mut reorder = frames.clone();
        let first_payload = reorder[first_delta].payload.clone();
        reorder[first_delta].payload = reorder[second_delta].payload.clone();
        reorder[second_delta].payload = first_payload;
        assert!(matches!(
            open_test_frames(&reorder),
            Err(Error::Corruption(_))
        ));

        let mut corrupted = frames.clone();
        let last = corrupted[first_delta].payload.len() - 1;
        corrupted[first_delta].payload[last] ^= 0x80;
        assert!(matches!(
            open_test_frames(&corrupted),
            Err(Error::Corruption(_))
        ));

        let mut substituted = frames.clone();
        substituted[first_delta].record_type = WalRecordType::PageImage;
        assert!(matches!(
            open_test_frames(&substituted),
            Err(Error::Corruption(_))
        ));
        let payload = frames[first_delta].payload.as_slice();
        assert_ne!(
            record_digest_for_test(3, &[(WalRecordType::PageDelta, 0, payload)]),
            record_digest_for_test(3, &[(WalRecordType::PageImage, 0, payload)])
        );
        assert_ne!(
            record_digest_for_test(3, &[(WalRecordType::PageDelta, 0, payload)]),
            record_digest_for_test(3, &[(WalRecordType::PageDelta, 1, payload)])
        );

        let mut other_wal = blink_test_wal();
        let mut other_bases = BTreeMap::new();
        let first = delta_commits(&other_wal, &[&[(5, 1), (6, 1)]]);
        append_with_bases(&mut other_wal, &first, &other_bases).unwrap();
        latest_images(&first, &mut other_bases);
        let second = delta_commits(&other_wal, &[&[(5, 9), (6, 9)]]);
        append_with_bases(&mut other_wal, &second, &other_bases).unwrap();
        let other_frames = crate::wal::parse_wal_frames_for_test(&other_wal.into_file().0);
        let mut spliced = frames.clone();
        spliced[commit_frame] = other_frames[commit_frame].clone();
        assert_ne!(spliced[commit_frame].payload, frames[commit_frame].payload);
        assert!(matches!(
            open_test_frames(&spliced),
            Err(Error::Corruption(_))
        ));
    }

    fn page_delta_store(keys: u64) -> (BlinkStore<MemoryFile, MemoryFile>, BTreeMap<u64, Vec<u8>>) {
        let mut store = planned_store();
        let mut expected = BTreeMap::new();
        for index in 0..keys {
            store.put(b0_key(index), b0_value(index, 0)).unwrap();
            expected.insert(index, b0_value(index, 0));
        }
        store.checkpoint().unwrap();
        for index in 0..keys {
            store.put(b0_key(index), b0_value(index, 1)).unwrap();
            expected.insert(index, b0_value(index, 1));
        }
        (store, expected)
    }

    fn assert_store_values(
        store: &mut BlinkStore<MemoryFile, MemoryFile>,
        expected: &BTreeMap<u64, Vec<u8>>,
    ) {
        for (index, value) in expected {
            assert_eq!(
                store.get(&b0_key(*index)).unwrap().value(),
                Some(value.as_slice()),
                "key {index}"
            );
        }
        store.check_invariants().unwrap();
    }

    fn reopen_memory_store(
        data: MemoryFile,
        wal: MemoryFile,
    ) -> BlinkStore<MemoryFile, MemoryFile> {
        let mut store = BlinkStore::open_with_wal(data, wal, DatabaseConfig::default()).unwrap();
        store.enable_planned_execution();
        store
    }

    fn leaf_of_key(store: &BlinkStore<MemoryFile, MemoryFile>, key: &DocumentKey) -> PageId {
        let encoded = key.encode();
        *store
            .state
            .pages
            .iter()
            .find(|(_, page)| match &***page {
                BlinkPage::Leaf { entries, .. } => entries
                    .iter()
                    .any(|entry| entry.key.as_ref() == encoded.as_slice()),
                _ => false,
            })
            .unwrap()
            .0
    }

    #[test]
    fn torn_checkpoint_data_page_is_rebuilt_from_wal_image_and_deltas() {
        let mut store = planned_store();
        for index in 0..400u64 {
            store.put(b0_key(index), b0_value(index, 0)).unwrap();
        }
        store.checkpoint().unwrap();
        let key_index = 123u64;
        let key = b0_key(key_index);
        let page_id = leaf_of_key(&store, &key);
        let checkpoint_image = {
            let mut data = MemoryFile(Vec::new());
            std::mem::swap(&mut data.0, &mut store.file.0);
            let image: [u8; PAGE_SIZE] = data.0
                [page_id.get() as usize * PAGE_SIZE..(page_id.get() as usize + 1) * PAGE_SIZE]
                .try_into()
                .unwrap();
            std::mem::swap(&mut data.0, &mut store.file.0);
            image
        };
        let mut committed_images = Vec::new();
        for round in 1..=5u64 {
            store.put(key.clone(), b0_value(key_index, round)).unwrap();
            store
                .put(
                    b0_key(key_index + 1 + round),
                    b0_value(key_index, round + 50),
                )
                .unwrap();
            committed_images.push(*store.dirty_pages[&page_id]);
        }
        let redo = store.wal_metrics().unwrap().unwrap().redo;
        assert!(redo.page_delta_records >= 8, "{redo:?}");
        let kinds = store
            .wal
            .as_mut()
            .unwrap()
            .committed_batches_on_disk()
            .iter()
            .flat_map(|batch| {
                batch
                    .pages
                    .iter()
                    .zip(&batch.redo_kinds)
                    .filter(|(page, _)| page.page_id == page_id)
                    .map(|(_, kind)| *kind)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(kinds[0], crate::wal::WalRedoKind::PageImage);
        assert!(
            kinds[1..]
                .iter()
                .all(|kind| *kind == crate::wal::WalRedoKind::PageDelta)
        );
        let latest = *committed_images.last().unwrap();
        let other_dirty = store
            .dirty_pages
            .iter()
            .filter(|(id, _)| **id != page_id)
            .map(|(id, image)| (*id, **image))
            .collect::<Vec<_>>();
        let (data, wal) = store.into_files();
        let wal = wal.unwrap();
        let mut torn = latest;
        torn[..PAGE_SIZE / 2].copy_from_slice(&checkpoint_image[..PAGE_SIZE / 2]);
        let mut garbage = [0u8; PAGE_SIZE];
        let mut state = 0x7042u64;
        garbage.copy_from_slice(&random_test_bytes(&mut state, PAGE_SIZE));
        for (name, variant) in [
            ("old checkpoint image", checkpoint_image),
            ("intermediate committed image", committed_images[1]),
            ("latest image", latest),
            ("torn image", torn),
            ("garbage image", garbage),
        ] {
            let mut data = MemoryFile(data.0.clone());
            for (other_id, other_image) in other_dirty.iter().take(1) {
                let offset = other_id.get() as usize * PAGE_SIZE;
                data.0[offset..offset + PAGE_SIZE].copy_from_slice(other_image);
            }
            let offset = page_id.get() as usize * PAGE_SIZE;
            data.0[offset..offset + PAGE_SIZE].copy_from_slice(&variant);
            let mut reopened = reopen_memory_store(data, MemoryFile(wal.0.clone()));
            assert_eq!(
                reopened.get(&key).unwrap().value(),
                Some(b0_value(key_index, 5).as_slice()),
                "{name}"
            );
            let (recovered_data, _) = reopened.into_files();
            assert_eq!(
                &recovered_data.0[offset..offset + PAGE_SIZE],
                &latest[..],
                "{name}"
            );
        }
    }

    struct FailAtOccurrence {
        point: &'static str,
        occurrence: usize,
        seen: usize,
        fired: Arc<std::sync::atomic::AtomicBool>,
    }

    impl FaultInjector for FailAtOccurrence {
        fn hit(&mut self, point: &str) -> Result<()> {
            if point == self.point {
                self.seen += 1;
                if self.seen == self.occurrence {
                    self.fired.store(true, Ordering::SeqCst);
                    return Err(Error::recovery(format!(
                        "injected failure at {point} #{}",
                        self.occurrence
                    )));
                }
            }
            Ok(())
        }
    }

    fn fail_at(
        point: &'static str,
        occurrence: usize,
    ) -> (FailAtOccurrence, Arc<std::sync::atomic::AtomicBool>) {
        let fired = Arc::new(std::sync::atomic::AtomicBool::new(false));
        (
            FailAtOccurrence {
                point,
                occurrence,
                seen: 0,
                fired: Arc::clone(&fired),
            },
            fired,
        )
    }

    fn width_sixteen_request(round: u64) -> (TransactionRequest, Vec<(u64, Vec<u8>)>) {
        let writes = (0..16u64)
            .map(|slot| {
                let index = (slot * 13 + round) % 200;
                (index, b0_value(index, 1_000 + round))
            })
            .collect::<Vec<_>>();
        (
            TransactionRequest::new(
                Vec::new(),
                writes
                    .iter()
                    .map(|(index, value)| TransactionMutation::Put {
                        key: b0_key(*index),
                        value: value.clone(),
                    })
                    .collect(),
            ),
            writes,
        )
    }

    #[test]
    fn page_delta_append_fault_matrix_keeps_transactions_atomic() {
        let points = [
            "before_wal_append",
            "during_wal_header_write",
            "during_page_delta_header_write",
            "during_wal_payload_write",
            "during_page_delta_payload_write",
            "during_wal_trailer_write",
            "after_page_delta_record",
            "after_page_images_written",
            "before_commit_record",
            "after_commit_record_write",
            "after_group_records_written",
            "before_wal_sync",
            "during_wal_sync",
            "after_wal_sync",
        ];
        let mut cases = 0;
        for point in points {
            for occurrence in 1..=40usize {
                let (mut store, mut expected) = page_delta_store(200);
                let before = store.wal_metrics().unwrap().unwrap().redo;
                let (injector, fired) = fail_at(point, occurrence);
                store.set_fault_injector(injector);
                let (request, writes) = width_sixteen_request(occurrence as u64);
                let result = store.apply_transaction_group(&[request]);
                if !fired.load(Ordering::SeqCst) {
                    assert!(occurrence > 1, "{point} never fired");
                    let redo = store.wal_metrics().unwrap().unwrap().redo;
                    assert!(
                        redo.page_delta_records > before.page_delta_records,
                        "{point}"
                    );
                    assert_eq!(
                        redo.page_image_records, before.page_image_records,
                        "{point}"
                    );
                    break;
                }
                assert!(result.is_err(), "{point} #{occurrence}");
                cases += 1;
                let (data, wal) = store.into_files();
                let mut reopened = reopen_memory_store(data, wal.unwrap());
                let applied = writes.iter().all(|(index, value)| {
                    reopened.get(&b0_key(*index)).unwrap().value() == Some(value.as_slice())
                });
                let untouched = writes.iter().all(|(index, _)| {
                    reopened.get(&b0_key(*index)).unwrap().value()
                        == Some(expected[index].as_slice())
                });
                assert!(
                    applied ^ untouched,
                    "{point} #{occurrence}: partial transaction"
                );
                let commit_written = matches!(
                    point,
                    "after_commit_record_write"
                        | "after_group_records_written"
                        | "before_wal_sync"
                        | "during_wal_sync"
                        | "after_wal_sync"
                );
                assert_eq!(applied, commit_written, "{point} #{occurrence}");
                if applied {
                    for (index, value) in &writes {
                        expected.insert(*index, value.clone());
                    }
                }
                assert_store_values(&mut reopened, &expected);
                let (request, writes) = width_sixteen_request(7_777);
                reopened.transact(request).unwrap();
                for (index, value) in writes {
                    expected.insert(index, value);
                }
                let (data, wal) = reopened.into_files();
                let mut again = reopen_memory_store(data, wal.unwrap());
                assert_store_values(&mut again, &expected);
            }
        }
        assert!(cases >= 40, "{cases}");
    }

    #[test]
    fn page_delta_torn_wal_tail_never_exposes_partial_transactions() {
        let (mut store, expected) = page_delta_store(200);
        let start = store.wal_metrics().unwrap().unwrap().wal_bytes;
        let (request, writes) = width_sixteen_request(3);
        store.transact(request).unwrap();
        let end = store.wal_metrics().unwrap().unwrap().wal_bytes;
        assert!(end - start < 4_000, "{}", end - start);
        let (data, wal) = store.into_files();
        let wal = wal.unwrap();
        for cut in start..=end {
            let mut torn_wal = MemoryFile(wal.0.clone());
            torn_wal.0.truncate(cut as usize);
            let mut reopened = reopen_memory_store(MemoryFile(data.0.clone()), torn_wal);
            let applied = writes.iter().all(|(index, value)| {
                reopened.get(&b0_key(*index)).unwrap().value() == Some(value.as_slice())
            });
            let untouched = writes.iter().all(|(index, _)| {
                reopened.get(&b0_key(*index)).unwrap().value() == Some(expected[index].as_slice())
            });
            assert!(applied ^ untouched, "cut {cut}");
            assert_eq!(applied, cut == end, "cut {cut}");
        }
    }

    #[test]
    fn page_delta_checkpoint_fault_matrix_preserves_acknowledged_writes() {
        let points = [
            "before_checkpoint_data_flush",
            "during_checkpoint_page_write",
            "before_checkpoint_data_sync",
            "after_checkpoint_data_sync",
            "before_checkpoint_superblock_write",
            "after_checkpoint_superblock_write",
            "before_checkpoint_metadata_sync",
            "after_checkpoint_metadata_sync",
            "before_wal_reset",
            "during_wal_truncate",
            "after_wal_truncate",
            "after_wal_reset_truncate_sync",
            "before_wal_reinitialization",
            "during_wal_reinitialization",
            "during_wal_header_write",
            "after_wal_reset_write",
            "during_wal_reset_sync",
            "after_wal_reset_sync",
        ];
        let mut cases = 0;
        for point in points {
            for occurrence in [1usize, 2, 7, 40] {
                let (mut store, mut expected) = page_delta_store(200);
                for round in 0..3u64 {
                    let (request, writes) = width_sixteen_request(round);
                    store.transact(request).unwrap();
                    for (index, value) in writes {
                        expected.insert(index, value);
                    }
                }
                let (injector, fired) = fail_at(point, occurrence);
                store.set_fault_injector(injector);
                let result = store.checkpoint();
                if !fired.load(Ordering::SeqCst) {
                    result.unwrap();
                    continue;
                }
                assert!(result.is_err(), "{point} #{occurrence}");
                assert!(store.checkpoint().is_err());
                cases += 1;
                let (data, wal) = store.into_files();
                let mut reopened = reopen_memory_store(data, wal.unwrap());
                assert_store_values(&mut reopened, &expected);
                for round in 10..13u64 {
                    let (request, writes) = width_sixteen_request(round);
                    reopened.transact(request).unwrap();
                    for (index, value) in writes {
                        expected.insert(index, value);
                    }
                }
                let (data, wal) = reopened.into_files();
                let mut again = reopen_memory_store(data, wal.unwrap());
                assert_store_values(&mut again, &expected);
                again.checkpoint().unwrap();
                let (data, wal) = again.into_files();
                let mut after_checkpoint = reopen_memory_store(data, wal.unwrap());
                assert_store_values(&mut after_checkpoint, &expected);
            }
        }
        assert!(cases >= 18, "{cases}");
    }

    #[test]
    fn page_deltas_after_flush_and_from_parallel_execution_recover() {
        for parallel in [false, true] {
            let mut store = if parallel {
                parallel_store()
            } else {
                planned_store()
            };
            let mut expected = BTreeMap::new();
            for index in 0..300u64 {
                store.put(b0_key(index), b0_value(index, 0)).unwrap();
                expected.insert(index, b0_value(index, 0));
            }
            store.flush().unwrap();
            assert!(store.dirty_pages.is_empty());
            let before = store.wal_metrics().unwrap().unwrap().redo;
            for round in 1..=3u64 {
                let requests = (0..16u64)
                    .map(|slot| {
                        let index = (slot * 17 + round) % 300;
                        expected.insert(index, b0_value(index, round + 10));
                        TransactionRequest::new(
                            Vec::new(),
                            vec![TransactionMutation::Put {
                                key: b0_key(index),
                                value: b0_value(index, round + 10),
                            }],
                        )
                    })
                    .collect::<Vec<_>>();
                for result in store.apply_transaction_group(&requests).unwrap() {
                    result.unwrap();
                }
            }
            let after = store.wal_metrics().unwrap().unwrap().redo;
            assert_eq!(
                after.page_image_records, before.page_image_records,
                "{parallel}"
            );
            assert!(
                after.page_delta_records >= before.page_delta_records + 48,
                "{parallel}"
            );
            let (data, wal) = store.into_files();
            let mut reopened = reopen_memory_store(data, wal.unwrap());
            assert_store_values(&mut reopened, &expected);
        }
    }

    fn splitmix_for_test(mut state: u64) -> u64 {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn phase_d_store(
        workers: usize,
        keys: u64,
    ) -> (BlinkStore<MemoryFile, MemoryFile>, BTreeMap<u64, Vec<u8>>) {
        let (mut store, expected) = page_delta_store(keys);
        if workers > 0 {
            store.enable_parallel_execution(workers).unwrap();
        }
        (store, expected)
    }

    #[test]
    fn group_scratch_reuse_keeps_varying_groups_isolated() {
        let (mut store, mut expected) = phase_d_store(2, 512);
        let mut key_cursor = 0u64;
        for (group_index, group_size) in [1usize, 64, 3, 128, 2, 32, 1, 256].into_iter().enumerate()
        {
            let requests = (0..group_size)
                .map(|_| {
                    let key_index = key_cursor % 512;
                    key_cursor += 1;
                    let value = b0_value(key_index, group_index as u64 + 100);
                    expected.insert(key_index, value.clone());
                    TransactionRequest::new(
                        Vec::new(),
                        vec![TransactionMutation::Put {
                            key: b0_key(key_index),
                            value,
                        }],
                    )
                })
                .collect::<Vec<_>>();
            for result in store.apply_transaction_group(&requests).unwrap() {
                result.unwrap();
            }
        }
        assert_store_values(&mut store, &expected);
        let (data_file, wal_file) = store.into_files();
        let mut reopened = reopen_memory_store(data_file, wal_file.unwrap());
        assert_store_values(&mut reopened, &expected);
    }

    struct PinnedPageSnapshot {
        page_id: PageId,
        page: Arc<BlinkPage>,
        contents: BlinkPage,
        image: [u8; PAGE_SIZE],
        payloads: Vec<(*const u8, Vec<u8>, Option<(*const u8, Vec<u8>)>)>,
    }

    fn snapshot_pinned_generation(pin: &GenerationPin) -> Vec<PinnedPageSnapshot> {
        (FIRST_DATA_PAGE..=pin.generation.high_water_page_id.get())
            .map(|raw_page_id| {
                let page_id = PageId::new(raw_page_id);
                let page = pin.page(page_id).unwrap();
                let payloads = match &*page {
                    BlinkPage::Leaf { entries, .. } => entries
                        .iter()
                        .map(|entry| {
                            (
                                entry.key.as_ptr(),
                                entry.key.to_vec(),
                                match &entry.value {
                                    Some(BlinkValueRef::Inline(value)) => {
                                        Some((value.as_ptr(), value.to_vec()))
                                    }
                                    _ => None,
                                },
                            )
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                PinnedPageSnapshot {
                    page_id,
                    contents: BlinkPage::clone(&page),
                    image: encode_blink_page(page_id, &page).unwrap(),
                    page,
                    payloads,
                }
            })
            .collect()
    }

    fn assert_pinned_generation_unchanged(
        pin: &GenerationPin,
        snapshots: &[PinnedPageSnapshot],
        documents: &[Document],
        label: &str,
    ) {
        for snapshot in snapshots {
            let page = pin.page(snapshot.page_id).unwrap();
            assert!(
                Arc::ptr_eq(&page, &snapshot.page),
                "{label}: pinned page {} was replaced",
                snapshot.page_id
            );
            assert_eq!(
                *page, snapshot.contents,
                "{label}: page {}",
                snapshot.page_id
            );
            assert_eq!(
                encode_blink_page(snapshot.page_id, &page).unwrap(),
                snapshot.image,
                "{label}: image {}",
                snapshot.page_id
            );
            if let BlinkPage::Leaf { entries, .. } = &*page {
                assert_eq!(entries.len(), snapshot.payloads.len());
                for (entry, (key_pointer, key, value)) in entries.iter().zip(&snapshot.payloads) {
                    assert_eq!(entry.key.as_ptr(), *key_pointer, "{label}: key moved");
                    assert_eq!(entry.key.as_ref(), key.as_slice(), "{label}: key bytes");
                    match (&entry.value, value) {
                        (Some(BlinkValueRef::Inline(bytes)), Some((pointer, expected))) => {
                            assert_eq!(bytes.as_ptr(), *pointer, "{label}: value moved");
                            assert_eq!(bytes.as_ref(), expected.as_slice(), "{label}: value bytes");
                        }
                        (Some(BlinkValueRef::Inline(_)), None) | (_, Some(_)) => {
                            panic!("{label}: entry value kind changed")
                        }
                        _ => {}
                    }
                }
            }
        }
        let mut corrections = 0;
        assert_eq!(
            scan_state(pin, None, usize::MAX, &mut corrections).unwrap(),
            documents,
            "{label}: pinned scan"
        );
    }

    #[test]
    fn pinned_generation_pages_entries_and_values_survive_later_writes() {
        for workers in [0usize, 2] {
            let (mut store, _) = phase_d_store(workers, 400);
            let pin = store.publisher.pin();
            let snapshots = snapshot_pinned_generation(&pin);
            let mut corrections = 0;
            let documents = scan_state(&pin, None, usize::MAX, &mut corrections).unwrap();
            assert_eq!(documents.len(), 400);
            let label = format!("workers {workers}");
            let distinct = keys_on_distinct_leaves(&store, 400, 16);
            for round in 2..6u64 {
                let wide = distinct
                    .iter()
                    .map(|(index, _)| (*index, b0_value(*index, round)))
                    .collect::<Vec<_>>();
                let chain = (0..3)
                    .map(|step| put_request(&[(distinct[0].0, b0_value(step, round + 10))]))
                    .collect::<Vec<_>>();
                let mut group = vec![put_request(&wide)];
                group.extend(chain);
                store.apply_transaction_group(&group).unwrap();
                assert_pinned_generation_unchanged(&pin, &snapshots, &documents, &label);
            }
            let inserts = (1_000..1_300u64)
                .map(|index| put_request(&[(index, b0_value(index, 7))]))
                .collect::<Vec<_>>();
            for chunk in inserts.chunks(32) {
                store.apply_transaction_group(chunk).unwrap();
            }
            store
                .apply_transaction_group(&[
                    TransactionRequest::new(
                        Vec::new(),
                        vec![TransactionMutation::Delete { key: b0_key(3) }],
                    ),
                    TransactionRequest::new(
                        Vec::new(),
                        vec![TransactionMutation::Put {
                            key: b0_key(5),
                            value: vec![0xa5; 2 * INLINE_VALUE_LIMIT],
                        }],
                    ),
                ])
                .unwrap();
            store.checkpoint().unwrap();
            assert!(store.split_metrics().leaf_splits > 0);
            assert_pinned_generation_unchanged(&pin, &snapshots, &documents, &label);
            if workers > 0 {
                assert!(store.batch_metrics().parallel_groups >= 4);
            }
            let current = store.publisher.pin();
            for (page_id, page) in &store.state.pages {
                assert!(
                    Arc::ptr_eq(page, &current.page(*page_id).unwrap()),
                    "{label}: committed page {page_id} is not the published page object"
                );
            }
            drop(current);
            drop(pin);
            store.check_invariants().unwrap();
        }
    }

    fn keys_on_distinct_leaves(
        store: &BlinkStore<MemoryFile, MemoryFile>,
        keys: u64,
        count: usize,
    ) -> Vec<(u64, PageId)> {
        let mut seen = BTreeSet::new();
        let mut chosen = Vec::new();
        for index in 0..keys {
            let leaf_id = leaf_of_key(store, &b0_key(index));
            if seen.insert(leaf_id) {
                chosen.push((index, leaf_id));
                if chosen.len() == count {
                    break;
                }
            }
        }
        assert_eq!(chosen.len(), count, "not enough distinct leaves");
        chosen
    }

    fn put_request(writes: &[(u64, Vec<u8>)]) -> TransactionRequest {
        TransactionRequest::new(
            Vec::new(),
            writes
                .iter()
                .map(|(index, value)| TransactionMutation::Put {
                    key: b0_key(*index),
                    value: value.clone(),
                })
                .collect(),
        )
    }

    fn assert_same_physical_state(
        left: &BlinkStore<MemoryFile, MemoryFile>,
        right: &BlinkStore<MemoryFile, MemoryFile>,
        label: &str,
    ) {
        assert_eq!(left.state.pages, right.state.pages, "{label}: pages");
        assert_eq!(left.state.root_page_id, right.state.root_page_id, "{label}");
        assert_eq!(
            left.state.high_water_page_id, right.state.high_water_page_id,
            "{label}"
        );
        assert_eq!(left.dirty_pages, right.dirty_pages, "{label}: dirty pages");
        assert_eq!(left.dirty_superblock, right.dirty_superblock, "{label}");
        assert_eq!(left.current_superblock, right.current_superblock, "{label}");
        assert_eq!(left.active_slot, right.active_slot, "{label}");
        assert_eq!(left.next_lsn, right.next_lsn, "{label}");
        assert_eq!(left.next_revision, right.next_revision, "{label}");
        assert_eq!(left.next_batch_id, right.next_batch_id, "{label}");
        assert_eq!(
            left.wal.as_ref().unwrap().next_lsn(),
            right.wal.as_ref().unwrap().next_lsn(),
            "{label}"
        );
    }

    fn result_signature(results: &[Result<TransactionResult>]) -> Vec<String> {
        results
            .iter()
            .map(|result| match result {
                Ok(result) => format!("ok {}", result.commit_lsn),
                Err(error) => format!("err {error:?}"),
            })
            .collect()
    }

    fn all_documents(store: &mut BlinkStore<MemoryFile, MemoryFile>) -> Vec<Document> {
        let mut documents = Vec::new();
        let mut cursor = None;
        loop {
            let page = store.scan(cursor.as_ref(), 97).unwrap();
            let Some(last) = page.last() else {
                break;
            };
            cursor = Some(last.key.clone());
            documents.extend(page);
        }
        documents
    }

    /// Runs the same groups on the serial planned executor and on the
    /// leaf-parallel executor, then checks results, in-memory pages, dirty
    /// images and the raw WAL bytes.
    fn run_serial_and_parallel(
        keys: u64,
        workers: usize,
        groups: &[Vec<TransactionRequest>],
    ) -> (
        BlinkStore<MemoryFile, MemoryFile>,
        BlinkStore<MemoryFile, MemoryFile>,
    ) {
        let (mut serial, _) = phase_d_store(0, keys);
        let (mut parallel, _) = phase_d_store(workers, keys);
        for (group_index, group) in groups.iter().enumerate() {
            let serial_results = serial.apply_transaction_group(group).unwrap();
            let parallel_results = parallel.apply_transaction_group(group).unwrap();
            assert_eq!(
                result_signature(&serial_results),
                result_signature(&parallel_results),
                "group {group_index}"
            );
            assert_same_physical_state(&serial, &parallel, &format!("group {group_index}"));
        }
        (serial, parallel)
    }

    fn assert_same_files(
        serial: BlinkStore<MemoryFile, MemoryFile>,
        parallel: BlinkStore<MemoryFile, MemoryFile>,
    ) {
        let (serial_data, serial_wal) = serial.into_files();
        let (parallel_data, parallel_wal) = parallel.into_files();
        assert!(serial_data.0 == parallel_data.0, "data files differ");
        assert!(
            serial_wal.unwrap().0 == parallel_wal.unwrap().0,
            "WAL bytes differ"
        );
    }

    #[test]
    fn phase_d_independent_sixteen_leaf_transaction_matches_serial() {
        let (probe, _) = phase_d_store(0, 600);
        let chosen = keys_on_distinct_leaves(&probe, 600, 16);
        drop(probe);
        let writes = chosen
            .iter()
            .map(|(index, _)| (*index, b0_value(*index, 70)))
            .collect::<Vec<_>>();
        let groups = vec![vec![put_request(&writes)]];
        for workers in [1, 2] {
            let (mut serial, _) = phase_d_store(0, 600);
            let (mut parallel, _) = phase_d_store(workers, 600);
            let metrics_before = parallel.batch_metrics();
            let serial_results = serial.apply_transaction_group(&groups[0]).unwrap();
            let parallel_results = parallel.apply_transaction_group(&groups[0]).unwrap();
            assert_eq!(
                result_signature(&serial_results),
                result_signature(&parallel_results)
            );
            assert_same_physical_state(&serial, &parallel, "sixteen leaves");
            let metrics = parallel.batch_metrics();
            assert_eq!(metrics.parallel_groups, 1, "{metrics:?}");
            assert_eq!(metrics.parallel_leaf_jobs, 16);
            assert_eq!(metrics.parallel_job_operations, 16);
            assert_eq!(metrics.parallel_fallback_groups, 0);
            assert_eq!(
                metrics.physical_mutation_nanos,
                metrics_before.physical_mutation_nanos
            );
            assert_eq!(
                metrics.physical_page_encode_nanos,
                metrics_before.physical_page_encode_nanos
            );
            assert!(metrics.parallel_worker_encode_nanos > 0);
            let batches = parallel.wal.as_mut().unwrap().committed_batches_on_disk();
            let last = batches.last().unwrap();
            assert_eq!(last.pages.len(), 16);
            assert!(
                last.redo_kinds
                    .iter()
                    .all(|kind| *kind == crate::wal::WalRedoKind::PageDelta)
            );
            for (index, value) in &writes {
                assert_eq!(
                    parallel.get(&b0_key(*index)).unwrap().value(),
                    Some(value.as_slice())
                );
            }
            parallel.check_invariants().unwrap();
            assert_same_files(serial, parallel);
        }
    }

    #[test]
    fn phase_d_same_leaf_chain_keeps_fifo_and_delta_bases() {
        let (probe, _) = phase_d_store(0, 600);
        let chosen = keys_on_distinct_leaves(&probe, 600, 2);
        let leaf_one = chosen[0].1;
        let same_leaf_keys = (0..600u64)
            .filter(|index| leaf_of_key(&probe, &b0_key(*index)) == leaf_one)
            .take(2)
            .collect::<Vec<_>>();
        drop(probe);
        let first_key = same_leaf_keys[0];
        let second_key = same_leaf_keys[1];
        let group = vec![
            put_request(&[(first_key, b0_value(first_key, 81))]),
            put_request(&[(second_key, b0_value(second_key, 82))]),
            put_request(&[(first_key, b0_value(first_key, 83))]),
            put_request(&[(chosen[1].0, b0_value(chosen[1].0, 84))]),
        ];
        let (serial, mut parallel) = run_serial_and_parallel(600, 2, &[group]);
        let metrics = parallel.batch_metrics();
        assert_eq!(metrics.parallel_groups, 1, "{metrics:?}");
        assert_eq!(metrics.parallel_leaf_jobs, 2);
        assert_eq!(metrics.parallel_job_operations, 4);
        let batches = parallel.wal.as_mut().unwrap().committed_batches_on_disk();
        let group_batches = &batches[batches.len() - 4..];
        let chain = group_batches[..3]
            .iter()
            .map(|batch| {
                assert_eq!(batch.pages.len(), 1);
                assert_eq!(batch.pages[0].page_id, leaf_one);
                assert_eq!(batch.redo_kinds[0], crate::wal::WalRedoKind::PageDelta);
                batch.commit_lsn
            })
            .collect::<Vec<_>>();
        assert!(chain.windows(2).all(|pair| pair[0] < pair[1]));
        let first_revision = parallel.get(&b0_key(first_key)).unwrap();
        assert_eq!(
            first_revision.value(),
            Some(b0_value(first_key, 83).as_slice())
        );
        assert_eq!(first_revision.revision(), Revision::from(chain[2]));
        assert_eq!(
            parallel.get(&b0_key(second_key)).unwrap().revision(),
            Revision::from(chain[1])
        );
        assert_same_files(serial, parallel);
    }

    #[test]
    fn phase_d_mixed_leaf_chains_match_serial() {
        let (probe, _) = phase_d_store(0, 600);
        let leaves = keys_on_distinct_leaves(&probe, 600, 4);
        drop(probe);
        let key = |slot: usize, round: u64| (leaves[slot].0, b0_value(leaves[slot].0, round));
        let group = vec![
            put_request(&[key(0, 91), key(1, 92)]),
            put_request(&[key(1, 93), key(2, 94)]),
            put_request(&[key(3, 95)]),
        ];
        let (serial, parallel) = run_serial_and_parallel(600, 2, &[group.clone(), group]);
        let metrics = parallel.batch_metrics();
        assert_eq!(metrics.parallel_groups, 2, "{metrics:?}");
        assert_eq!(metrics.parallel_leaf_jobs, 8);
        assert_eq!(metrics.parallel_job_operations, 10);
        assert_same_files(serial, parallel);
    }

    #[test]
    fn phase_d_worker_failure_is_atomic_and_leaves_store_usable() {
        for fault in [
            ParallelWorkerFault::Error {
                leaf_group_index: 5,
            },
            ParallelWorkerFault::Panic {
                leaf_group_index: 5,
            },
        ] {
            let (mut store, mut expected) = phase_d_store(2, 600);
            let chosen = keys_on_distinct_leaves(&store, 600, 16);
            let writes = chosen
                .iter()
                .map(|(index, _)| (*index, b0_value(*index, 101)))
                .collect::<Vec<_>>();
            let wal_before = store.wal_metrics().unwrap().unwrap();
            let lsn_before = store.wal.as_ref().unwrap().next_lsn();
            let pages_before = store.state.pages.clone();
            let dirty_before = store.dirty_pages.clone();
            store.parallel_worker_fault = Some(fault);
            let result = store.apply_transaction_group(&[put_request(&writes)]);
            assert!(result.is_err(), "{fault:?}");
            assert!(store.broken.is_none(), "{fault:?}");
            let wal_after = store.wal_metrics().unwrap().unwrap();
            assert_eq!(wal_after.wal_bytes, wal_before.wal_bytes);
            assert_eq!(wal_after.wal_syncs, wal_before.wal_syncs);
            assert_eq!(store.wal.as_ref().unwrap().next_lsn(), lsn_before);
            assert!(store.state.pages == pages_before);
            assert!(store.dirty_pages == dirty_before);
            assert_store_values(&mut store, &expected);
            let handle = store.versioned_read_handle();
            for (index, value) in &expected {
                assert_eq!(
                    handle.get(&b0_key(*index)).unwrap().value(),
                    Some(value.as_slice())
                );
            }
            store.parallel_worker_fault = None;
            let results = store
                .apply_transaction_group(&[put_request(&writes)])
                .unwrap();
            assert!(results[0].is_ok());
            for (index, value) in &writes {
                expected.insert(*index, value.clone());
            }
            assert_store_values(&mut store, &expected);
            let (data, wal) = store.into_files();
            let mut reopened = reopen_memory_store(data, wal.unwrap());
            assert_store_values(&mut reopened, &expected);
        }
    }

    #[test]
    fn phase_d_parallel_wal_recovers_like_serial() {
        let mut state = 0x5eed_d00du64;
        let mut groups = Vec::new();
        for _ in 0..40 {
            let transactions = 1 + (splitmix_for_test(state) % 20) as usize;
            state = state.wrapping_add(1);
            let mut group = Vec::new();
            for _ in 0..transactions {
                let width = 1 + (splitmix_for_test(state) % 16) as usize;
                state = state.wrapping_add(1);
                let mut indices = BTreeSet::new();
                while indices.len() < width {
                    indices.insert(splitmix_for_test(state) % 600);
                    state = state.wrapping_add(1);
                }
                let writes = indices
                    .into_iter()
                    .map(|index| (index, b0_value(index, state)))
                    .collect::<Vec<_>>();
                group.push(put_request(&writes));
            }
            groups.push(group);
        }
        let (mut serial, mut parallel) = run_serial_and_parallel(600, 2, &groups);
        assert!(parallel.batch_metrics().parallel_groups >= 30);
        let serial_documents = all_documents(&mut serial);
        assert_eq!(serial_documents, all_documents(&mut parallel));
        let (serial_data, serial_wal) = serial.into_files();
        let (parallel_data, parallel_wal) = parallel.into_files();
        let serial_wal = serial_wal.unwrap();
        let parallel_wal = parallel_wal.unwrap();
        assert!(serial_wal.0 == parallel_wal.0, "WAL bytes differ");
        let mut serial_reopened = reopen_memory_store(serial_data, serial_wal);
        let mut parallel_reopened = reopen_memory_store(parallel_data, parallel_wal);
        parallel_reopened.enable_parallel_execution(2).unwrap();
        assert_eq!(all_documents(&mut serial_reopened), serial_documents);
        assert_eq!(all_documents(&mut parallel_reopened), serial_documents);
        serial_reopened.check_invariants().unwrap();
        parallel_reopened.check_invariants().unwrap();
        assert!(serial_reopened.state.pages == parallel_reopened.state.pages);
        let more = groups[..5].to_vec();
        for group in &more {
            let serial_results = serial_reopened.apply_transaction_group(group).unwrap();
            let parallel_results = parallel_reopened.apply_transaction_group(group).unwrap();
            assert_eq!(
                result_signature(&serial_results),
                result_signature(&parallel_results)
            );
        }
        assert!(parallel_reopened.batch_metrics().parallel_groups >= 4);
        assert_same_files(serial_reopened, parallel_reopened);
    }

    #[test]
    fn phase_d_fault_matrix_keeps_acknowledged_durability() {
        let points = [
            "before_parallel_leaf_dispatch",
            "after_parallel_leaf_join",
            "before_wal_append",
            "during_wal_header_write",
            "during_page_delta_header_write",
            "during_wal_payload_write",
            "during_page_delta_payload_write",
            "during_wal_trailer_write",
            "after_page_delta_record",
            "after_page_images_written",
            "before_commit_record",
            "after_commit_record_write",
            "after_group_records_written",
            "before_wal_sync",
            "during_wal_sync",
            "after_wal_sync",
            "before_generation_publication",
        ];
        let (probe, _) = phase_d_store(0, 600);
        let leaves = keys_on_distinct_leaves(&probe, 600, 32);
        drop(probe);
        let transaction_writes = |round: u64| {
            [0usize, 16]
                .iter()
                .map(|offset| {
                    leaves[*offset..*offset + 16]
                        .iter()
                        .map(|(index, _)| (*index, b0_value(*index, round)))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };
        let mut cases = 0;
        for point in points {
            for occurrence in 1..=40usize {
                let (mut store, mut expected) = phase_d_store(2, 600);
                let parallel_before = store.batch_metrics().parallel_groups;
                let (injector, fired) = fail_at(point, occurrence);
                store.set_fault_injector(injector);
                let writes = transaction_writes(2_000 + occurrence as u64);
                let group = writes
                    .iter()
                    .map(|transaction| put_request(transaction))
                    .collect::<Vec<_>>();
                let result = store.apply_transaction_group(&group);
                if !fired.load(Ordering::SeqCst) {
                    assert!(occurrence > 1, "{point} never fired");
                    assert!(result.is_ok());
                    break;
                }
                assert!(result.is_err(), "{point} #{occurrence}");
                if !matches!(
                    point,
                    "before_parallel_leaf_dispatch" | "after_parallel_leaf_join"
                ) {
                    assert_eq!(
                        store.batch_metrics().parallel_groups,
                        parallel_before + 1,
                        "{point}"
                    );
                }
                cases += 1;
                let handle = store.versioned_read_handle();
                for (index, value) in &expected {
                    assert_eq!(
                        handle.get(&b0_key(*index)).unwrap().value(),
                        Some(value.as_slice()),
                        "{point} #{occurrence}: failed group became visible"
                    );
                }
                let (data, wal) = store.into_files();
                let mut reopened = reopen_memory_store(data, wal.unwrap());
                let applied = writes
                    .iter()
                    .map(|transaction| {
                        let applied = transaction.iter().all(|(index, value)| {
                            reopened.get(&b0_key(*index)).unwrap().value() == Some(value.as_slice())
                        });
                        let untouched = transaction.iter().all(|(index, _)| {
                            reopened.get(&b0_key(*index)).unwrap().value()
                                == Some(expected[index].as_slice())
                        });
                        assert!(
                            applied ^ untouched,
                            "{point} #{occurrence}: partial transaction"
                        );
                        applied
                    })
                    .collect::<Vec<_>>();
                assert!(
                    !(applied[1] && !applied[0]),
                    "{point} #{occurrence}: later transaction without earlier one"
                );
                if matches!(
                    point,
                    "before_parallel_leaf_dispatch"
                        | "after_parallel_leaf_join"
                        | "before_wal_append"
                ) {
                    assert_eq!(applied, vec![false, false], "{point}");
                }
                if matches!(
                    point,
                    "after_group_records_written"
                        | "before_wal_sync"
                        | "during_wal_sync"
                        | "after_wal_sync"
                        | "before_generation_publication"
                ) {
                    assert_eq!(applied, vec![true, true], "{point}");
                }
                for (transaction, transaction_applied) in writes.iter().zip(&applied) {
                    if *transaction_applied {
                        for (index, value) in transaction {
                            expected.insert(*index, value.clone());
                        }
                    }
                }
                assert_store_values(&mut reopened, &expected);
                reopened.enable_parallel_execution(2).unwrap();
                let followup = transaction_writes(9_999);
                reopened
                    .apply_transaction_group(
                        &followup
                            .iter()
                            .map(|transaction| put_request(transaction))
                            .collect::<Vec<_>>(),
                    )
                    .unwrap();
                assert!(reopened.batch_metrics().parallel_groups >= 1, "{point}");
                for transaction in followup {
                    for (index, value) in transaction {
                        expected.insert(index, value);
                    }
                }
                let (data, wal) = reopened.into_files();
                let mut again = reopen_memory_store(data, wal.unwrap());
                assert_store_values(&mut again, &expected);
            }
        }
        eprintln!(
            "phase-d fault matrix: {cases} injected failures over {} points",
            points.len()
        );
        assert!(cases >= 60, "{cases}");
    }

    fn differential_request(
        state: &mut u64,
        keys: u64,
        allow_overflow: bool,
    ) -> TransactionRequest {
        let mut next = || {
            *state = state.wrapping_add(1);
            splitmix_for_test(*state)
        };
        let width = 1 + (next() % 16) as usize;
        let mut indices = BTreeSet::new();
        while indices.len() < width {
            indices.insert(next() % (keys + keys / 4));
        }
        let mut mutations = Vec::new();
        for index in indices {
            let choice = next() % 100;
            let mutation = if choice < 8 {
                TransactionMutation::Delete { key: b0_key(index) }
            } else if allow_overflow && choice < 10 {
                TransactionMutation::Put {
                    key: b0_key(index),
                    value: vec![(index % 251) as u8; 700],
                }
            } else {
                TransactionMutation::Put {
                    key: b0_key(index),
                    value: b0_value(index, next()),
                }
            };
            mutations.push(mutation);
        }
        if next() % 100 < 2 {
            let duplicate = mutations[0].clone();
            mutations.push(duplicate);
        }
        let mut conditions = Vec::new();
        let condition_choice = next() % 100;
        if condition_choice < 6 {
            conditions.push(TransactionCondition::Exists {
                key: b0_key(next() % (keys + keys / 4)),
            });
        } else if condition_choice < 9 {
            conditions.push(TransactionCondition::RevisionEquals {
                key: b0_key(next() % keys),
                expected_revision: Revision::new(next() % 4),
            });
        }
        TransactionRequest::new(conditions, mutations)
    }

    #[test]
    fn phase_d_randomized_differential_serial_one_and_two_workers() {
        for seed in [11u64, 29, 47] {
            let keys = 500u64;
            let mut stores = [0usize, 1, 2].map(|workers| phase_d_store(workers, keys).0);
            let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15);
            for group_index in 0..80usize {
                let transactions = 1 + (splitmix_for_test(state ^ 0xabc) % 24) as usize;
                state = state.wrapping_add(7);
                let group = (0..transactions)
                    .map(|_| differential_request(&mut state, keys, group_index % 8 == 3))
                    .collect::<Vec<_>>();
                let signatures = stores
                    .iter_mut()
                    .map(|store| match store.apply_transaction_group(&group) {
                        Ok(results) => result_signature(&results),
                        Err(error) => vec![format!("group error {error:?}")],
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    signatures[0], signatures[1],
                    "seed {seed} group {group_index}"
                );
                assert_eq!(
                    signatures[0], signatures[2],
                    "seed {seed} group {group_index}"
                );
                for store in &stores[1..] {
                    assert_same_physical_state(
                        &stores[0],
                        store,
                        &format!("seed {seed} group {group_index}"),
                    );
                }
                if group_index == 30 {
                    for store in &mut stores {
                        store.flush().unwrap();
                    }
                }
                if group_index == 55 {
                    for store in &mut stores {
                        store.checkpoint().unwrap();
                    }
                }
            }
            for store in &stores[1..] {
                let metrics = store.batch_metrics();
                eprintln!(
                    "phase-d differential seed {seed} workers {}: parallel groups {}, fallback groups {} (overflow {}, structural {}, route {}, after dispatch {}), single-leaf {}, leaf jobs {}, operations {}",
                    store.parallel_workers,
                    metrics.parallel_groups,
                    metrics.parallel_fallback_groups,
                    metrics.parallel_fallback_overflow,
                    metrics.parallel_fallback_structural,
                    metrics.parallel_fallback_route,
                    metrics.parallel_fallback_after_dispatch,
                    metrics.parallel_skipped_single_leaf,
                    metrics.parallel_leaf_jobs,
                    metrics.parallel_job_operations
                );
                assert!(metrics.parallel_groups >= 20, "seed {seed}: {metrics:?}");
                assert!(
                    metrics.parallel_fallback_groups >= 1,
                    "seed {seed}: {metrics:?}"
                );
            }
            let documents = all_documents(&mut stores[0]);
            for store in &mut stores {
                assert_eq!(all_documents(store), documents, "seed {seed}");
                store.check_invariants().unwrap();
            }
            let [serial, one_worker, two_workers] = stores;
            let (serial_data, serial_wal) = serial.into_files();
            let serial_wal = serial_wal.unwrap();
            for store in [one_worker, two_workers] {
                let (data, wal) = store.into_files();
                let wal = wal.unwrap();
                assert!(data.0 == serial_data.0, "seed {seed}: data file differs");
                assert!(wal.0 == serial_wal.0, "seed {seed}: WAL bytes differ");
                let mut reopened = reopen_memory_store(data, wal);
                assert_eq!(all_documents(&mut reopened), documents, "seed {seed}");
                reopened.check_invariants().unwrap();
            }
        }
    }

    #[test]
    fn phase_d_prepared_redo_with_wrong_chain_base_writes_nothing() {
        let (mut store, _) = phase_d_store(0, 200);
        let key = b0_key(7);
        let leaf_id = leaf_of_key(&store, &key);
        let base = *store.dirty_pages[&leaf_id];
        let base_lsn = blink_image_lsn(&base);
        let base_crc = crc32c::crc32c(&base);
        let wal = store.wal.as_mut().unwrap();
        let commit_lsn = Lsn::new(wal.next_lsn().get() + 1);
        let mut page = decode_blink_page(&base, leaf_id).unwrap();
        if let BlinkPage::Leaf { lsn, .. } = &mut page {
            *lsn = commit_lsn;
        }
        let image = encode_blink_page(leaf_id, &page).unwrap();
        let payload = encode_page_delta(leaf_id, &base, &image).unwrap();
        let view = decode_page_delta(&payload).unwrap();
        let spans = view.spans.len() as u64;
        let changed_bytes = view.changed_bytes() as u64;
        drop(view);
        let wal_bytes = wal.metrics().unwrap().wal_bytes;
        let next_batch_id = wal.next_batch_id();
        for (wrong_lsn, wrong_crc) in [
            (base_lsn, base_crc ^ 1),
            (Lsn::new(base_lsn.get() + 1), base_crc),
        ] {
            let commit = PreparedWalCommit {
                batch_id: next_batch_id,
                commit_lsn,
                records: vec![PreparedWalRecord {
                    page_id: leaf_id,
                    page_lsn: commit_lsn,
                    image_crc: crc32c::crc32c(&image),
                    redo: PreparedWalRedo::Delta {
                        payload: &payload,
                        base_lsn: wrong_lsn,
                        base_crc: wrong_crc,
                        spans,
                        changed_bytes,
                    },
                }],
            };
            assert!(wal.append_group_prepared(&[commit], None).is_err());
            assert_eq!(wal.metrics().unwrap().wal_bytes, wal_bytes);
        }
        let commit = PreparedWalCommit {
            batch_id: next_batch_id,
            commit_lsn,
            records: vec![PreparedWalRecord {
                page_id: leaf_id,
                page_lsn: commit_lsn,
                image_crc: crc32c::crc32c(&image),
                redo: PreparedWalRedo::Delta {
                    payload: &payload,
                    base_lsn,
                    base_crc,
                    spans,
                    changed_bytes,
                },
            }],
        };
        wal.append_group_prepared(&[commit], None).unwrap();
        assert!(wal.metrics().unwrap().wal_bytes > wal_bytes);
    }
}
