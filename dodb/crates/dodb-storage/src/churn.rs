#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ChurnSite {
    Harness = 0,
    OtherStorage,
    Admission,
    Planner,
    LeafJobConstruction,
    LeafLaneExecution,
    JobResultCollection,
    PackedLeafMutationCow,
    PageDeltaGeneration,
    WalPreparation,
    WalAppend,
    StateInstall,
    CatalogPublication,
    DirtyTracking,
}

pub const CHURN_SITES: [ChurnSite; 14] = [
    ChurnSite::Harness,
    ChurnSite::OtherStorage,
    ChurnSite::Admission,
    ChurnSite::Planner,
    ChurnSite::LeafJobConstruction,
    ChurnSite::LeafLaneExecution,
    ChurnSite::JobResultCollection,
    ChurnSite::PackedLeafMutationCow,
    ChurnSite::PageDeltaGeneration,
    ChurnSite::WalPreparation,
    ChurnSite::WalAppend,
    ChurnSite::StateInstall,
    ChurnSite::CatalogPublication,
    ChurnSite::DirtyTracking,
];

impl ChurnSite {
    pub fn name(self) -> &'static str {
        match self {
            Self::Harness => "harness",
            Self::OtherStorage => "other_storage",
            Self::Admission => "admission",
            Self::Planner => "planner",
            Self::LeafJobConstruction => "leaf_job_construction",
            Self::LeafLaneExecution => "leaf_lane_execution",
            Self::JobResultCollection => "job_result_collection",
            Self::PackedLeafMutationCow => "packed_leaf_mutation_cow",
            Self::PageDeltaGeneration => "page_delta_generation",
            Self::WalPreparation => "wal_preparation",
            Self::WalAppend => "wal_append",
            Self::StateInstall => "state_install",
            Self::CatalogPublication => "catalog_publication",
            Self::DirtyTracking => "dirty_tracking",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ChurnCounter {
    AllocCalls = 0,
    AllocBytes,
    FreeCalls,
    FreeBytes,
    ReallocCalls,
    ReallocBytes,
    LeafEntryClones,
    LeafEntryDrops,
    ArcKeyClones,
    ArcValueClones,
    ArcKeyDrops,
    ArcValueDrops,
    PayloadArcsCreated,
    LeafPageClones,
    LeafVecCapacityBytes,
    InternalPageClones,
    BlinkPageArcClones,
    PageImageCopies,
    PageImageBytesCopied,
    PageImageBuffers,
    PageEncodes,
    DirtyPageInserts,
    DirtyPageReplaces,
    DirtyPageBytesCopied,
    DeltaPayloadBuffers,
    DeltaPayloadBytes,
    DeltaVerifyImages,
    PlannerMapInserts,
    PlannerKeyCopies,
    PlannerMutationClones,
    JobsBuilt,
    LeafEntriesCopied,
    LeafSlotBytesCopied,
    LeafPayloadBytesCopied,
    LeafCompactions,
    LeafKeyComparisons,
    AllocLe16,
    AllocLe32,
    AllocLe64,
    AllocLe128,
    AllocLe256,
    AllocLe1024,
    AllocLe4096,
    AllocGt4096,
    ArenaAllocCalls,
    ArenaAllocBytes,
}

pub const CHURN_COUNTERS: [ChurnCounter; 46] = [
    ChurnCounter::AllocCalls,
    ChurnCounter::AllocBytes,
    ChurnCounter::FreeCalls,
    ChurnCounter::FreeBytes,
    ChurnCounter::ReallocCalls,
    ChurnCounter::ReallocBytes,
    ChurnCounter::LeafEntryClones,
    ChurnCounter::LeafEntryDrops,
    ChurnCounter::ArcKeyClones,
    ChurnCounter::ArcValueClones,
    ChurnCounter::ArcKeyDrops,
    ChurnCounter::ArcValueDrops,
    ChurnCounter::PayloadArcsCreated,
    ChurnCounter::LeafPageClones,
    ChurnCounter::LeafVecCapacityBytes,
    ChurnCounter::InternalPageClones,
    ChurnCounter::BlinkPageArcClones,
    ChurnCounter::PageImageCopies,
    ChurnCounter::PageImageBytesCopied,
    ChurnCounter::PageImageBuffers,
    ChurnCounter::PageEncodes,
    ChurnCounter::DirtyPageInserts,
    ChurnCounter::DirtyPageReplaces,
    ChurnCounter::DirtyPageBytesCopied,
    ChurnCounter::DeltaPayloadBuffers,
    ChurnCounter::DeltaPayloadBytes,
    ChurnCounter::DeltaVerifyImages,
    ChurnCounter::PlannerMapInserts,
    ChurnCounter::PlannerKeyCopies,
    ChurnCounter::PlannerMutationClones,
    ChurnCounter::JobsBuilt,
    ChurnCounter::LeafEntriesCopied,
    ChurnCounter::LeafSlotBytesCopied,
    ChurnCounter::LeafPayloadBytesCopied,
    ChurnCounter::LeafCompactions,
    ChurnCounter::LeafKeyComparisons,
    ChurnCounter::AllocLe16,
    ChurnCounter::AllocLe32,
    ChurnCounter::AllocLe64,
    ChurnCounter::AllocLe128,
    ChurnCounter::AllocLe256,
    ChurnCounter::AllocLe1024,
    ChurnCounter::AllocLe4096,
    ChurnCounter::AllocGt4096,
    ChurnCounter::ArenaAllocCalls,
    ChurnCounter::ArenaAllocBytes,
];

impl ChurnCounter {
    pub fn name(self) -> &'static str {
        match self {
            Self::AllocCalls => "alloc_calls",
            Self::AllocBytes => "alloc_bytes",
            Self::FreeCalls => "free_calls",
            Self::FreeBytes => "free_bytes",
            Self::ReallocCalls => "realloc_calls",
            Self::ReallocBytes => "realloc_bytes",
            Self::LeafEntryClones => "leaf_entry_clones",
            Self::LeafEntryDrops => "leaf_entry_drops",
            Self::ArcKeyClones => "arc_key_clones",
            Self::ArcValueClones => "arc_value_clones",
            Self::ArcKeyDrops => "arc_key_drops",
            Self::ArcValueDrops => "arc_value_drops",
            Self::PayloadArcsCreated => "payload_arcs_created",
            Self::LeafPageClones => "leaf_page_clones",
            Self::LeafVecCapacityBytes => "leaf_vec_capacity_bytes",
            Self::InternalPageClones => "internal_page_clones",
            Self::BlinkPageArcClones => "blink_page_arc_clones",
            Self::PageImageCopies => "page_image_copies",
            Self::PageImageBytesCopied => "page_image_bytes_copied",
            Self::PageImageBuffers => "page_image_buffers",
            Self::PageEncodes => "page_encodes",
            Self::DirtyPageInserts => "dirty_page_inserts",
            Self::DirtyPageReplaces => "dirty_page_replaces",
            Self::DirtyPageBytesCopied => "dirty_page_bytes_copied",
            Self::DeltaPayloadBuffers => "delta_payload_buffers",
            Self::DeltaPayloadBytes => "delta_payload_bytes",
            Self::DeltaVerifyImages => "delta_verify_images",
            Self::PlannerMapInserts => "planner_map_inserts",
            Self::PlannerKeyCopies => "planner_key_copies",
            Self::PlannerMutationClones => "planner_mutation_clones",
            Self::JobsBuilt => "jobs_built",
            Self::LeafEntriesCopied => "leaf_entries_copied",
            Self::LeafSlotBytesCopied => "leaf_slot_bytes_copied",
            Self::LeafPayloadBytesCopied => "leaf_payload_bytes_copied",
            Self::LeafCompactions => "leaf_compactions",
            Self::LeafKeyComparisons => "leaf_key_comparisons",
            Self::AllocLe16 => "alloc_le_16",
            Self::AllocLe32 => "alloc_le_32",
            Self::AllocLe64 => "alloc_le_64",
            Self::AllocLe128 => "alloc_le_128",
            Self::AllocLe256 => "alloc_le_256",
            Self::AllocLe1024 => "alloc_le_1024",
            Self::AllocLe4096 => "alloc_le_4096",
            Self::AllocGt4096 => "alloc_gt_4096",
            Self::ArenaAllocCalls => "arena_alloc_calls",
            Self::ArenaAllocBytes => "arena_alloc_bytes",
        }
    }
}

#[cfg(feature = "churn-counters")]
mod enabled {
    use std::cell::Cell;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{CHURN_COUNTERS, CHURN_SITES, ChurnCounter, ChurnSite};

    const SITE_COUNT: usize = CHURN_SITES.len();
    const COUNTER_COUNT: usize = CHURN_COUNTERS.len();

    #[allow(clippy::declare_interior_mutable_const)]
    const ZERO: AtomicU64 = AtomicU64::new(0);
    #[allow(clippy::declare_interior_mutable_const)]
    const ROW: [AtomicU64; COUNTER_COUNT] = [ZERO; COUNTER_COUNT];
    static COUNTS: [[AtomicU64; COUNTER_COUNT]; SITE_COUNT] = [ROW; SITE_COUNT];

    thread_local! {
        static CURRENT_SITE: Cell<u8> = const { Cell::new(0) };
    }

    pub struct SiteGuard {
        previous: u8,
    }

    impl Drop for SiteGuard {
        fn drop(&mut self) {
            let previous = self.previous;
            let _ = CURRENT_SITE.try_with(|site| site.set(previous));
        }
    }

    pub fn enter(site: ChurnSite) -> SiteGuard {
        let previous = CURRENT_SITE
            .try_with(|current| current.replace(site as u8))
            .unwrap_or(0);
        SiteGuard { previous }
    }

    fn current_site() -> usize {
        CURRENT_SITE
            .try_with(|site| site.get() as usize)
            .unwrap_or(0)
    }

    pub fn add(counter: ChurnCounter, amount: u64) {
        COUNTS[current_site()][counter as usize].fetch_add(amount, Ordering::Relaxed);
    }

    pub fn record_alloc(bytes: usize) {
        let row = &COUNTS[current_site()];
        row[ChurnCounter::AllocCalls as usize].fetch_add(1, Ordering::Relaxed);
        row[ChurnCounter::AllocBytes as usize].fetch_add(bytes as u64, Ordering::Relaxed);
        let bucket = match bytes {
            0..=16 => ChurnCounter::AllocLe16,
            17..=32 => ChurnCounter::AllocLe32,
            33..=64 => ChurnCounter::AllocLe64,
            65..=128 => ChurnCounter::AllocLe128,
            129..=256 => ChurnCounter::AllocLe256,
            257..=1024 => ChurnCounter::AllocLe1024,
            1025..=4096 => ChurnCounter::AllocLe4096,
            _ => ChurnCounter::AllocGt4096,
        };
        row[bucket as usize].fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_free(bytes: usize) {
        let row = &COUNTS[current_site()];
        row[ChurnCounter::FreeCalls as usize].fetch_add(1, Ordering::Relaxed);
        row[ChurnCounter::FreeBytes as usize].fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn record_realloc(bytes: usize) {
        let row = &COUNTS[current_site()];
        row[ChurnCounter::ReallocCalls as usize].fetch_add(1, Ordering::Relaxed);
        row[ChurnCounter::ReallocBytes as usize].fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn record_arena_alloc(bytes: usize) {
        let row = &COUNTS[current_site()];
        row[ChurnCounter::ArenaAllocCalls as usize].fetch_add(1, Ordering::Relaxed);
        row[ChurnCounter::ArenaAllocBytes as usize].fetch_add(bytes as u64, Ordering::Relaxed);
    }

    static LEAF_SAMPLES: std::sync::Mutex<Vec<[u32; 3]>> = std::sync::Mutex::new(Vec::new());

    pub fn record_leaf_sample(entries: usize, key_bytes: usize, value_bytes: usize) {
        if let Ok(mut samples) = LEAF_SAMPLES.lock() {
            samples.push([entries as u32, key_bytes as u32, value_bytes as u32]);
        }
    }

    pub fn leaf_samples_since(start: usize) -> (usize, Vec<[u32; 3]>) {
        match LEAF_SAMPLES.lock() {
            Ok(samples) => (samples.len(), samples.get(start..).unwrap_or(&[]).to_vec()),
            Err(_) => (0, Vec::new()),
        }
    }

    pub fn snapshot() -> Vec<(ChurnSite, ChurnCounter, u64)> {
        let mut values = Vec::with_capacity(SITE_COUNT * COUNTER_COUNT);
        for site in CHURN_SITES {
            for counter in CHURN_COUNTERS {
                values.push((
                    site,
                    counter,
                    COUNTS[site as usize][counter as usize].load(Ordering::Relaxed),
                ));
            }
        }
        values
    }
}

#[cfg(feature = "churn-counters")]
pub use enabled::{
    SiteGuard, add, enter, leaf_samples_since, record_alloc, record_free, record_leaf_sample,
    record_realloc, snapshot,
};

#[cfg(feature = "churn-counters")]
pub use enabled::record_arena_alloc;

#[cfg(not(feature = "churn-counters"))]
pub struct SiteGuard;

#[cfg(not(feature = "churn-counters"))]
impl Drop for SiteGuard {
    #[inline(always)]
    fn drop(&mut self) {}
}

#[cfg(not(feature = "churn-counters"))]
#[inline(always)]
pub fn enter(_site: ChurnSite) -> SiteGuard {
    SiteGuard
}

#[cfg(not(feature = "churn-counters"))]
#[inline(always)]
pub fn add(_counter: ChurnCounter, _amount: u64) {}

#[cfg(not(feature = "churn-counters"))]
#[inline(always)]
pub fn record_leaf_sample(_entries: usize, _key_bytes: usize, _value_bytes: usize) {}

#[cfg(not(feature = "churn-counters"))]
#[inline(always)]
pub fn record_arena_alloc(_bytes: usize) {}

#[cfg(not(feature = "churn-counters"))]
pub fn leaf_samples_since(_start: usize) -> (usize, Vec<[u32; 3]>) {
    (0, Vec::new())
}

#[cfg(not(feature = "churn-counters"))]
pub fn snapshot() -> Vec<(ChurnSite, ChurnCounter, u64)> {
    Vec::new()
}

pub const ENABLED: bool = cfg!(feature = "churn-counters");
