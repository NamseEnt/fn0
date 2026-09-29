/// The bloom half of a part's sidecar, made evictable.
///
/// The 24-hour soak died at t≈1834 s with `part_sidecar_resident_bytes` at
/// 630 MiB and climbing (todo.md, the soak section): every open part kept its
/// line and exact-field window blooms resident forever — ~2 MiB per part
/// since the 0.1% FPP widening — and the live part count under a real
/// retention window is proportional to ingest rate times that window, so the
/// one term the declared budget did not cover grew with data held rather
/// than with work done. The blooms are durable in `index.bin`
/// (`docs/VISION.md` scoped exactly this fix), so the resident copy is a
/// cache: eviction is a re-read, not a loss.
///
/// What stays resident is the stream index — tens of kilobytes per part
/// against the blooms' megabytes — because the infallible metadata paths
/// (`label_names`, `label_values`, `series`) read it, and turning those into
/// I/O-fallible answers for the small half of the bytes buys nothing.
///
/// Shape: one process-wide byte total and LRU registry over every reader's
/// slot, unlike the row-group cache's per-reader maps — a reader owns
/// exactly one bloom entry, so "evict your own least-recently-used" would
/// never evict anything. The byte total is authoritative in the registry:
/// every add and subtract happens under its lock, and a slot's data and its
/// registry entry are reconciled by re-checking the entry before a deferred
/// clear, so an eviction racing a reinstall skips rather than double-counts.
///
/// 0 = unbounded: the pre-eviction behaviour, and the default when no memory
/// budget is declared. The startup wiring sets it before any registry loads
/// parts.
static BLOOM_CACHE_BUDGET: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static BLOOM_CACHE_TOTAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static BLOOM_SLOT_NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// What the budget above actually costs, which the resident gauge cannot say.
///
/// `signy_part_sidecar_resident_bytes` reports how much is held; these report
/// how hard it was to hold it. A miss is not free: it re-reads the whole of
/// `index.bin` into an owned buffer, decodes it, and drops the buffer — so a
/// cache riding its ceiling turns every pruning query into a large
/// allocate-and-free, and the bytes counter is what that costs per second.
/// Without them a soak can see the resident half sitting flat at its cap and
/// read it as "the cache is working", which is the same picture a cache that
/// evicts and re-reads on every query paints.
static BLOOM_CACHE_HITS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static BLOOM_CACHE_MISSES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static BLOOM_CACHE_READ_BYTES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// A pruning query found the blooms already resident.
pub fn record_bloom_cache_hit() {
    BLOOM_CACHE_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// A pruning query had to re-read `index.bin`; `bytes` is the whole file.
pub fn record_bloom_cache_miss(bytes: u64) {
    BLOOM_CACHE_MISSES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    BLOOM_CACHE_READ_BYTES.fetch_add(bytes, std::sync::atomic::Ordering::Relaxed);
}

/// The cache's lifecycle, which the hit/miss pair alone cannot describe.
///
/// A 90% hit rate reads as a healthy cache. It is also what a cache produces
/// when it holds a part's blooms for a moment, evicts them under pressure, and
/// decodes the same part again shortly after — the hits are the queries that
/// land inside those moments. These separate the two: how long an entry
/// survives after it is installed, and how soon the part that lost it wants it
/// back. A short life beside a short gap is thrashing, whatever the hit rate
/// says, and it means the working set does not fit rather than that the cache
/// is behaving badly.
///
/// `INSTALL_BYTES` is the decoded size, against `READ_BYTES`' raw file size:
/// what a miss puts into memory, as against what it reads.
static BLOOM_INSTALLS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static BLOOM_INSTALL_BYTES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static BLOOM_EVICTIONS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static BLOOM_RESIDENT_NANOS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static BLOOM_REDECODES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static BLOOM_REDECODE_GAP_NANOS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Nanoseconds since the first call, so a slot can hold two instants in two
/// atomics without a lock. Zero is "never", which is why the epoch is read
/// once rather than being the process start.
fn bloom_clock_nanos() -> u64 {
    static EPOCH: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let epoch = EPOCH.get_or_init(std::time::Instant::now);
    epoch.elapsed().as_nanos() as u64 + 1
}

pub fn bloom_cache_lifecycle() -> (u64, u64, u64, u64, u64, u64) {
    use std::sync::atomic::Ordering::Relaxed;
    (
        BLOOM_INSTALLS.load(Relaxed),
        BLOOM_INSTALL_BYTES.load(Relaxed),
        BLOOM_EVICTIONS.load(Relaxed),
        BLOOM_RESIDENT_NANOS.load(Relaxed),
        BLOOM_REDECODES.load(Relaxed),
        BLOOM_REDECODE_GAP_NANOS.load(Relaxed),
    )
}

pub fn bloom_cache_counters() -> (u64, u64, u64) {
    use std::sync::atomic::Ordering::Relaxed;
    (
        BLOOM_CACHE_HITS.load(Relaxed),
        BLOOM_CACHE_MISSES.load(Relaxed),
        BLOOM_CACHE_READ_BYTES.load(Relaxed),
    )
}

struct BloomCacheEntry {
    slot: std::sync::Weak<BloomSlot>,
    bytes: u64,
    last_touch: u64,
}

#[derive(Default)]
struct BloomCacheRegistry {
    entries: HashMap<u64, BloomCacheEntry>,
    clock: u64,
}

static BLOOM_CACHE_REGISTRY: std::sync::OnceLock<std::sync::Mutex<BloomCacheRegistry>> =
    std::sync::OnceLock::new();

fn bloom_cache_registry() -> &'static std::sync::Mutex<BloomCacheRegistry> {
    BLOOM_CACHE_REGISTRY.get_or_init(|| std::sync::Mutex::new(BloomCacheRegistry::default()))
}

/// Startup wiring, called before the registries load their parts.
pub fn configure_bloom_cache(budget: Option<u64>) {
    BLOOM_CACHE_BUDGET.store(
        budget.unwrap_or(0),
        std::sync::atomic::Ordering::Release,
    );
}

/// The process-wide resident bloom total, for the metrics endpoint.
pub fn bloom_cache_bytes() -> u64 {
    BLOOM_CACHE_TOTAL.load(std::sync::atomic::Ordering::Acquire)
}

/// One reader's bloom residency. The reader keeps the `Arc`; the registry
/// keeps a `Weak` so a dropped reader cannot be chosen as a victim into a
/// leak.
pub(crate) struct BloomSlot {
    id: u64,
    data: std::sync::Mutex<Option<Arc<BloomIndex>>>,
    /// When the current entry was installed, and when the last one was
    /// evicted. Both on [`bloom_clock_nanos`], both zero for "never".
    installed_at: std::sync::atomic::AtomicU64,
    evicted_at: std::sync::atomic::AtomicU64,
}

impl BloomSlot {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            id: BLOOM_SLOT_NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            data: std::sync::Mutex::new(None),
            installed_at: std::sync::atomic::AtomicU64::new(0),
            evicted_at: std::sync::atomic::AtomicU64::new(0),
        })
    }

    /// The resident blooms, touched for LRU — or `None`, meaning the caller
    /// pays the re-read.
    pub(crate) fn get(&self) -> Option<Arc<BloomIndex>> {
        let data = self.data.lock().expect("bloom slot poisoned").clone()?;
        let mut reg = bloom_cache_registry()
            .lock()
            .expect("bloom registry poisoned");
        reg.clock += 1;
        let clock = reg.clock;
        if let Some(entry) = reg.entries.get_mut(&self.id) {
            entry.last_touch = clock;
        }
        Some(data)
    }

    /// Install freshly decoded blooms and settle the budget: while the total
    /// is over it, the least-recently-used *other* slots are cleared. The
    /// installing slot is never its own victim — a part being queried right
    /// now is by definition the most recently used.
    pub(crate) fn install(self: &Arc<Self>, blooms: Arc<BloomIndex>, bytes: u64) {
        use std::sync::atomic::Ordering::Relaxed;
        let now = bloom_clock_nanos();
        BLOOM_INSTALLS.fetch_add(1, Relaxed);
        BLOOM_INSTALL_BYTES.fetch_add(bytes, Relaxed);
        // This part lost its blooms and has come back for them: the gap is
        // how long the eviction bought, and a short one is the definition of
        // thrashing.
        let evicted = self.evicted_at.swap(0, Relaxed);
        if evicted != 0 {
            BLOOM_REDECODES.fetch_add(1, Relaxed);
            BLOOM_REDECODE_GAP_NANOS.fetch_add(now.saturating_sub(evicted), Relaxed);
        }
        self.installed_at.store(now, Relaxed);
        *self.data.lock().expect("bloom slot poisoned") = Some(blooms);
        let mut victims: Vec<Arc<BloomSlot>> = Vec::new();
        {
            let mut reg = bloom_cache_registry()
                .lock()
                .expect("bloom registry poisoned");
            reg.clock += 1;
            let clock = reg.clock;
            // A concurrent duplicate decode or an eviction racing this
            // reinstall both land here: the old entry's bytes leave the
            // total before the new ones join it, so the total never counts
            // one slot twice and never goes negative.
            if let Some(old) = reg.entries.remove(&self.id) {
                BLOOM_CACHE_TOTAL.fetch_sub(old.bytes, std::sync::atomic::Ordering::AcqRel);
            }
            BLOOM_CACHE_TOTAL.fetch_add(bytes, std::sync::atomic::Ordering::AcqRel);
            reg.entries.insert(
                self.id,
                BloomCacheEntry {
                    slot: Arc::downgrade(self),
                    bytes,
                    last_touch: clock,
                },
            );
            let budget = BLOOM_CACHE_BUDGET.load(std::sync::atomic::Ordering::Acquire);
            if budget > 0 {
                while BLOOM_CACHE_TOTAL.load(std::sync::atomic::Ordering::Acquire) > budget
                    && reg.entries.len() > 1
                {
                    let Some(victim_id) = reg
                        .entries
                        .iter()
                        .filter(|(id, _)| **id != self.id)
                        .min_by_key(|(_, entry)| entry.last_touch)
                        .map(|(id, _)| *id)
                    else {
                        break;
                    };
                    let entry = reg.entries.remove(&victim_id).expect("chosen from the map");
                    BLOOM_CACHE_TOTAL.fetch_sub(entry.bytes, std::sync::atomic::Ordering::AcqRel);
                    if let Some(slot) = entry.slot.upgrade() {
                        victims.push(slot);
                    }
                }
            }
        }
        // Outside the registry lock, and re-checked under it: a victim that
        // reinstalled between selection and here has a fresh entry and keeps
        // its data. The Arc a running query may still hold keeps its memory
        // until that query ends — the same bounded transient the row-group
        // cache accepts.
        for slot in victims {
            let mut data = slot.data.lock().expect("bloom slot poisoned");
            let reg = bloom_cache_registry()
                .lock()
                .expect("bloom registry poisoned");
            if !reg.entries.contains_key(&slot.id) {
                slot.note_evicted();
                *data = None;
            }
        }
    }

    /// Deterministic removal on reader drop — a merged-away or
    /// retention-deleted part gives its bytes back immediately rather than
    /// waiting to be chosen as a victim.
    /// Records this slot losing its entry, whichever path took it.
    fn note_evicted(&self) {
        use std::sync::atomic::Ordering::Relaxed;
        let now = bloom_clock_nanos();
        let installed = self.installed_at.swap(0, Relaxed);
        if installed != 0 {
            BLOOM_EVICTIONS.fetch_add(1, Relaxed);
            BLOOM_RESIDENT_NANOS.fetch_add(now.saturating_sub(installed), Relaxed);
            self.evicted_at.store(now, Relaxed);
        }
    }

    pub(crate) fn remove(&self) {
        self.note_evicted();
        let mut data = self.data.lock().expect("bloom slot poisoned");
        let mut reg = bloom_cache_registry()
            .lock()
            .expect("bloom registry poisoned");
        if let Some(entry) = reg.entries.remove(&self.id) {
            BLOOM_CACHE_TOTAL.fetch_sub(entry.bytes, std::sync::atomic::Ordering::AcqRel);
        }
        *data = None;
    }
}
