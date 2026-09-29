//! Bytes allocated and peak live bytes, for the benches where the number that
//! has to move is memory rather than time.
//!
//! Invariants I and II are about memory: `entries_bytes` is 1.4-2.8x under,
//! and `Row::from_entry` clones a whole `BTreeMap` per row. Wall-clock will
//! shift a little when `Arc<Labels>` lands; bytes-allocated-per-row is the
//! number that has to collapse, so it is measured directly.
//!
//! Each bench binary is its own crate, so a binary that wants this declares
//! the allocator itself:
//!
//! ```ignore
//! #[global_allocator]
//! static ALLOCATOR: corpus::alloc::CountingAllocator = corpus::alloc::CountingAllocator;
//! ```
//!
//! Recording is off by default and gated on one relaxed load, so the timed
//! benches in the same binary pay a predicate rather than a counter.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};

static RECORDING: AtomicBool = AtomicBool::new(false);
static BYTES_ALLOCATED: AtomicU64 = AtomicU64::new(0);
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static LIVE: AtomicI64 = AtomicI64::new(0);
static PEAK_LIVE: AtomicI64 = AtomicI64::new(0);

pub struct CountingAllocator;

#[inline]
fn record_alloc(size: usize) {
    if !RECORDING.load(Ordering::Relaxed) {
        return;
    }
    BYTES_ALLOCATED.fetch_add(size as u64, Ordering::Relaxed);
    ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    let live = LIVE.fetch_add(size as i64, Ordering::Relaxed) + size as i64;
    PEAK_LIVE.fetch_max(live, Ordering::Relaxed);
}

#[inline]
fn record_dealloc(size: usize) {
    if !RECORDING.load(Ordering::Relaxed) {
        return;
    }
    // Signed, and allowed to go negative: memory allocated before recording
    // started is freed inside it, and clamping that at zero would understate
    // every later peak by however much was released.
    LIVE.fetch_sub(size as i64, Ordering::Relaxed);
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            record_alloc(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            record_alloc(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        record_dealloc(layout.size());
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            record_dealloc(layout.size());
            record_alloc(new_size);
        }
        new_ptr
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AllocStats {
    pub bytes_allocated: u64,
    pub allocations: u64,
    pub peak_live_bytes: i64,
    pub retained_bytes: i64,
}

/// Run `body` with accounting on and report what it allocated.
///
/// Single-threaded by construction: the counters are process-wide, so two
/// concurrent measurements would each report the other's allocations.
pub fn measure<T>(body: impl FnOnce() -> T) -> (T, AllocStats) {
    BYTES_ALLOCATED.store(0, Ordering::Relaxed);
    ALLOCATIONS.store(0, Ordering::Relaxed);
    LIVE.store(0, Ordering::Relaxed);
    PEAK_LIVE.store(0, Ordering::Relaxed);
    RECORDING.store(true, Ordering::SeqCst);
    let value = body();
    RECORDING.store(false, Ordering::SeqCst);
    let stats = AllocStats {
        bytes_allocated: BYTES_ALLOCATED.load(Ordering::Relaxed),
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        peak_live_bytes: PEAK_LIVE.load(Ordering::Relaxed),
        retained_bytes: LIVE.load(Ordering::Relaxed),
    };
    (value, stats)
}

pub fn header(title: &str) {
    eprintln!();
    eprintln!("=== allocation accounting: {title} ===");
    eprintln!(
        "{:<30} {:>8} {:>13} {:>10} {:>12} {:>11}  note",
        "case", "rows", "bytes alloc", "bytes/row", "peak live", "allocs/row"
    );
}

pub fn row(case: &str, rows: usize, stats: &AllocStats, note: &str) {
    let rows_f = rows.max(1) as f64;
    eprintln!(
        "{:<30} {:>8} {:>13} {:>10.1} {:>12} {:>11.2}  {}",
        case,
        rows,
        stats.bytes_allocated,
        stats.bytes_allocated as f64 / rows_f,
        stats.peak_live_bytes,
        stats.allocations as f64 / rows_f,
        note,
    );
}

pub fn footer() {
    eprintln!();
}
