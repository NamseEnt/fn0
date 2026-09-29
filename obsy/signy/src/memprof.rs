//! Which part of the engine the resident bytes belong to.
//!
//! `docs/VISION.md` invariant I divides a declared budget into arenas, and the
//! shares it proposes were written before anything measured them. M10 phase A
//! is the measurement, and this is its instrument: a global allocator that
//! tags every allocation with the arena that was current when it was made, so
//! **live** bytes — not merely allocated bytes — can be attributed. Peak
//! anonymous memory is what an OOM kill is decided on, and cumulative
//! allocation says nothing about it.
//!
//! Off unless the `memprof` feature is on. With the feature off, [`enter`] is
//! an inlined function returning a zero-sized value and the process keeps the
//! system allocator, so a release build is unchanged. With it on, every
//! allocation grows by `max(align, 16)` bytes to carry its tag, which is
//! itself reported (`live_allocations`) so a reader can subtract it.
//!
//! The label is a thread-local, so a guard must never be held across an
//! `.await`: a suspended task leaves its label on a worker thread that then
//! runs somebody else's future. Every call site here wraps a synchronous
//! region — the `spawn_blocking` closures the flush, merge and query paths
//! already run inside, and the straight-line decode in the push handler.
//! Attribution is therefore precise where a guard exists and lands in
//! [`Arena::Other`] where none does, which is a reported bucket rather than a
//! silent one.

/// Owner of an allocation, in the vocabulary `docs/VISION.md` invariant I uses
/// for its arena table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Arena {
    /// Everything with no guard around it: the HTTP stack, the runtime,
    /// response serialization, and any path this instrument does not name.
    Other = 0,
    /// Push body decompression, protobuf/JSON decode and the `LogEntry`
    /// vectors that decode produces — which are the bytes the memtable then
    /// holds, since they are moved into it rather than copied.
    Ingest = 1,
    /// The journal record framing and the writer's batch buffer.
    Wal = 2,
    /// Row materialization out of a memtable snapshot and the Parquet writer.
    Flush = 3,
    /// One merge group's read, filter and rewrite.
    Merge = 4,
    /// Every scan, pipeline stage and metric evaluation.
    Query = 5,
    /// Blooms and the stream index a `PartReader` holds for as long as it
    /// lives.
    Sidecar = 6,
    /// `meta.json` parsing and the `PartMeta` a registry entry keeps.
    PartMeta = 7,
    /// Decoded row groups a `PartReader` keeps for later scans, bounded by
    /// `row_group_cache_max_bytes`.
    RowGroupCache = 8,
    /// The metric memtable's live series: the buffer struct, the canonical
    /// label bytes it keys on, and the open Gorilla stream each one holds
    /// between flushes.
    SeriesMemtable = 9,
    /// The catalog and bloom a `SeriesPartReader` holds for as long as it
    /// lives — one entry per series per part, each carrying that series'
    /// canonical labels. Kept even when the part's body is offloaded, so this
    /// is resident per stored part rather than per active series.
    SeriesCatalog = 10,
}

impl Arena {
    pub const ALL: [Arena; 11] = [
        Arena::Other,
        Arena::Ingest,
        Arena::Wal,
        Arena::Flush,
        Arena::Merge,
        Arena::Query,
        Arena::Sidecar,
        Arena::PartMeta,
        Arena::RowGroupCache,
        Arena::SeriesMemtable,
        Arena::SeriesCatalog,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Arena::Other => "other",
            Arena::Ingest => "ingest",
            Arena::Wal => "wal",
            Arena::Flush => "flush",
            Arena::Merge => "merge",
            Arena::Query => "query",
            Arena::Sidecar => "sidecar",
            Arena::PartMeta => "part_meta",
            Arena::RowGroupCache => "row_group_cache",
            Arena::SeriesMemtable => "series_memtable",
            Arena::SeriesCatalog => "series_catalog",
        }
    }
}

#[cfg(not(feature = "memprof"))]
mod imp {
    use super::Arena;

    /// The no-op guard. Constructing and dropping it emits no code.
    pub struct Guard;

    // Empty, and present only so that call sites may write `drop(guard)` to
    // end an arena before an `.await` without the feature-off build tripping
    // `clippy::drop_non_drop`.
    impl Drop for Guard {
        fn drop(&mut self) {}
    }

    #[inline(always)]
    pub fn enter(_arena: Arena) -> Guard {
        Guard
    }

    #[inline(always)]
    pub fn render() -> String {
        String::new()
    }
}

#[cfg(feature = "memprof")]
mod imp {
    use super::Arena;
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;
    use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

    const ARENAS: usize = Arena::ALL.len();

    /// Bytes prepended to every allocation to carry its arena tag. Sixteen
    /// keeps the payload on the alignment `malloc` would have given it, so the
    /// instrumented build's allocation size classes stay comparable.
    const HEADER: usize = 16;

    static LIVE_BYTES: [AtomicI64; ARENAS] = [const { AtomicI64::new(0) }; ARENAS];
    static PEAK_BYTES: [AtomicI64; ARENAS] = [const { AtomicI64::new(0) }; ARENAS];
    static TOTAL_BYTES: [AtomicU64; ARENAS] = [const { AtomicU64::new(0) }; ARENAS];
    static LIVE_ALLOCS: [AtomicI64; ARENAS] = [const { AtomicI64::new(0) }; ARENAS];
    static TOTAL_ALLOCS: [AtomicU64; ARENAS] = [const { AtomicU64::new(0) }; ARENAS];

    // `const`-initialized and holding a type with no destructor, so the access
    // registers no TLS destructor and cannot allocate — which it must not,
    // being on the allocator's own path.
    thread_local! {
        static CURRENT: Cell<u8> = const { Cell::new(0) };
    }

    pub struct Guard(u8);

    impl Drop for Guard {
        fn drop(&mut self) {
            let previous = self.0;
            let _ = CURRENT.try_with(|cell| cell.set(previous));
        }
    }

    /// Tag allocations made on this thread until the guard drops.
    ///
    /// Nesting restores the enclosing arena, so a sidecar load inside a query
    /// is charged to the sidecar — which is what residency attribution wants,
    /// the sidecar outliving the query that faulted it in.
    pub fn enter(arena: Arena) -> Guard {
        let previous = CURRENT
            .try_with(|cell| cell.replace(arena as u8))
            .unwrap_or(0);
        Guard(previous)
    }

    fn current() -> usize {
        CURRENT.try_with(|cell| cell.get()).unwrap_or(0) as usize
    }

    fn charge(index: usize, size: usize) {
        TOTAL_BYTES[index].fetch_add(size as u64, Ordering::Relaxed);
        TOTAL_ALLOCS[index].fetch_add(1, Ordering::Relaxed);
        LIVE_ALLOCS[index].fetch_add(1, Ordering::Relaxed);
        let live = LIVE_BYTES[index].fetch_add(size as i64, Ordering::Relaxed) + size as i64;
        PEAK_BYTES[index].fetch_max(live, Ordering::Relaxed);
    }

    fn refund(index: usize, size: usize) {
        LIVE_BYTES[index].fetch_sub(size as i64, Ordering::Relaxed);
        LIVE_ALLOCS[index].fetch_sub(1, Ordering::Relaxed);
    }

    fn wrapped(layout: Layout) -> (Layout, usize) {
        let offset = layout.align().max(HEADER);
        let size = layout.size().saturating_add(offset);
        // `align` is unchanged and `size` only grew, so this cannot fail for
        // any layout the caller could already have allocated.
        (
            unsafe { Layout::from_size_align_unchecked(size, layout.align()) },
            offset,
        )
    }

    /// The tag slot sits in the last 8 bytes of the header, which is at least
    /// 16 bytes, so it never overlaps the payload.
    unsafe fn tag_slot(user: *mut u8) -> *mut u64 {
        unsafe { user.sub(8) as *mut u64 }
    }

    pub struct ProfilingAllocator;

    unsafe impl GlobalAlloc for ProfilingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let (wrapped_layout, offset) = wrapped(layout);
            let base = unsafe { System.alloc(wrapped_layout) };
            if base.is_null() {
                return base;
            }
            let index = current();
            let user = unsafe { base.add(offset) };
            unsafe { tag_slot(user).write(index as u64) };
            charge(index, layout.size());
            user
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            let (wrapped_layout, offset) = wrapped(layout);
            let base = unsafe { System.alloc_zeroed(wrapped_layout) };
            if base.is_null() {
                return base;
            }
            let index = current();
            let user = unsafe { base.add(offset) };
            unsafe { tag_slot(user).write(index as u64) };
            charge(index, layout.size());
            user
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            let (wrapped_layout, offset) = wrapped(layout);
            let index = (unsafe { tag_slot(ptr).read() } as usize).min(ARENAS - 1);
            refund(index, layout.size());
            unsafe { System.dealloc(ptr.sub(offset), wrapped_layout) }
        }

        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            let (wrapped_layout, offset) = wrapped(layout);
            // The block keeps the arena it was born in. A `Vec` that grows
            // inside a query but was filled by ingest is still ingest's
            // residency, and re-tagging it here would make the two arenas
            // disagree with their own deallocations.
            let index = (unsafe { tag_slot(ptr).read() } as usize).min(ARENAS - 1);
            let base = unsafe {
                System.realloc(
                    ptr.sub(offset),
                    wrapped_layout,
                    new_size.saturating_add(offset),
                )
            };
            if base.is_null() {
                return base;
            }
            let user = unsafe { base.add(offset) };
            unsafe { tag_slot(user).write(index as u64) };
            refund(index, layout.size());
            charge(index, new_size);
            user
        }
    }

    pub fn render() -> String {
        let mut out = String::new();
        out.push_str(
            "# HELP signy_memprof_live_bytes Live heap bytes attributed to an arena.\n\
             # TYPE signy_memprof_live_bytes gauge\n",
        );
        for arena in Arena::ALL {
            let index = arena as usize;
            out.push_str(&format!(
                "signy_memprof_live_bytes{{arena=\"{}\"}} {}\n",
                arena.name(),
                LIVE_BYTES[index].load(Ordering::Relaxed)
            ));
        }
        out.push_str(
            "# HELP signy_memprof_peak_bytes High-water live heap bytes for an arena.\n\
             # TYPE signy_memprof_peak_bytes gauge\n",
        );
        for arena in Arena::ALL {
            let index = arena as usize;
            out.push_str(&format!(
                "signy_memprof_peak_bytes{{arena=\"{}\"}} {}\n",
                arena.name(),
                PEAK_BYTES[index].load(Ordering::Relaxed)
            ));
        }
        out.push_str(
            "# HELP signy_memprof_live_allocations Live allocation count for an arena.\n\
             # TYPE signy_memprof_live_allocations gauge\n",
        );
        for arena in Arena::ALL {
            let index = arena as usize;
            out.push_str(&format!(
                "signy_memprof_live_allocations{{arena=\"{}\"}} {}\n",
                arena.name(),
                LIVE_ALLOCS[index].load(Ordering::Relaxed)
            ));
        }
        out.push_str(
            "# HELP signy_memprof_allocated_bytes_total Cumulative bytes allocated by an arena.\n\
             # TYPE signy_memprof_allocated_bytes_total counter\n",
        );
        for arena in Arena::ALL {
            let index = arena as usize;
            out.push_str(&format!(
                "signy_memprof_allocated_bytes_total{{arena=\"{}\"}} {}\n",
                arena.name(),
                TOTAL_BYTES[index].load(Ordering::Relaxed)
            ));
        }
        out.push_str(
            "# HELP signy_memprof_allocations_total Cumulative allocation count for an arena.\n\
             # TYPE signy_memprof_allocations_total counter\n",
        );
        for arena in Arena::ALL {
            let index = arena as usize;
            out.push_str(&format!(
                "signy_memprof_allocations_total{{arena=\"{}\"}} {}\n",
                arena.name(),
                TOTAL_ALLOCS[index].load(Ordering::Relaxed)
            ));
        }
        out.push_str(
            "# HELP signy_memprof_header_bytes Bytes the tag header itself adds.\n\
             # TYPE signy_memprof_header_bytes gauge\n",
        );
        let live_allocations: i64 = Arena::ALL
            .iter()
            .map(|arena| LIVE_ALLOCS[*arena as usize].load(Ordering::Relaxed))
            .sum();
        out.push_str(&format!(
            "signy_memprof_header_bytes {}\n",
            live_allocations * HEADER as i64
        ));
        out.push_str(&mallinfo());
        out
    }

    /// glibc's own view of the heap, which is the only thing that can separate
    /// "the engine is holding this" from "the allocator has not given it
    /// back". It also counts what C dependencies allocate directly — zstd's
    /// compression contexts go through `malloc`, not through the Rust global
    /// allocator, and are therefore invisible to the arena counters above.
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    fn mallinfo() -> String {
        #[repr(C)]
        struct MallInfo2 {
            arena: usize,
            ordblks: usize,
            smblks: usize,
            hblks: usize,
            hblkhd: usize,
            usmblks: usize,
            fsmblks: usize,
            uordblks: usize,
            fordblks: usize,
            keepcost: usize,
        }
        unsafe extern "C" {
            fn mallinfo2() -> MallInfo2;
        }
        let info = unsafe { mallinfo2() };
        format!(
            "# HELP signy_memprof_malloc_bytes glibc mallinfo2: heap the allocator took from the kernel, split into in-use and retained-free.\n\
             # TYPE signy_memprof_malloc_bytes gauge\n\
             signy_memprof_malloc_bytes{{state=\"arena\"}} {}\n\
             signy_memprof_malloc_bytes{{state=\"mmapped\"}} {}\n\
             signy_memprof_malloc_bytes{{state=\"in_use\"}} {}\n\
             signy_memprof_malloc_bytes{{state=\"free\"}} {}\n\
             signy_memprof_malloc_bytes{{state=\"releasable\"}} {}\n",
            info.arena, info.hblkhd, info.uordblks, info.fordblks, info.keepcost
        )
    }

    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    fn mallinfo() -> String {
        String::new()
    }
}

pub use imp::{Guard, enter, render};

#[cfg(feature = "memprof")]
pub use imp::ProfilingAllocator;
