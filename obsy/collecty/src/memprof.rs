//! Which part of the collector the resident bytes belong to.
//!
//! [`docs/MEMORY.md`](../docs/MEMORY.md) declares a budget out of terms nobody
//! had measured. This is the instrument that says which of those terms the
//! process is actually holding: a global allocator that tags every allocation
//! with the arena current when it was made, so **live** bytes can be
//! attributed rather than merely counted.
//!
//! Off unless the `memprof` feature is on, and with it off [`enter`] is an
//! inlined function returning a zero-sized value.
//!
//! **Tagging and the allocator underneath it are separate choices.** The
//! wrapper is generic, so `TaggedAllocator<System>` and
//! `TaggedAllocator<MiMalloc>` are both buildable and an attribution run is
//! not forced onto a different memory system from the shipped one. signy's
//! equivalent is hardwired to `System`, and the consequence there was that its
//! instrumented build died at a rate its shipped build survived, so the day it
//! most wanted attribution it had to give it up.
//!
//! **What the arenas cannot see.** The tag is a thread-local, so a guard held
//! across an `.await` would tag another task's work, and every guard here
//! therefore wraps a synchronous region. Two consequences are reported rather
//! than hidden:
//!
//! * **A request body is allocated inside hyper**, on the connection's own
//!   future, with no synchronous region to guard. It lands in [`Arena::Other`].
//!   The exact figure for admitted-and-not-yet-appended bytes does not need the
//!   allocator anyway — the in-flight semaphore knows it — and the sampler
//!   reports it as its own column.
//! * **zstd allocates through C `malloc`**, not through the Rust global
//!   allocator, so a compression context is invisible to every arena below.
//!   It shows up only in `libc_in_use` minus the tagged total, which is the
//!   column that answers whether reusing a context moved anything.

/// Owner of an allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Arena {
    /// Everything with no guard around it: hyper, the runtime, request bodies
    /// on their way in, and any path this instrument does not name.
    Other = 0,
    /// The framing an accepted export goes through before it is appended.
    Intake = 1,
    /// The queue's own structures: the segment writers, the held-segment
    /// bookkeeping, and whatever an append allocates on the Rust side.
    Queue = 2,
    /// A segment read back off disk for delivery, and the request body it
    /// becomes. This is the term a drained backlog is suspected of leaving
    /// behind.
    Send = 3,
}

impl Arena {
    pub const ALL: [Arena; 4] = [Arena::Other, Arena::Intake, Arena::Queue, Arena::Send];

    pub fn name(self) -> &'static str {
        match self {
            Arena::Other => "other",
            Arena::Intake => "intake",
            Arena::Queue => "queue",
            Arena::Send => "send",
        }
    }
}

#[cfg(not(feature = "memprof"))]
mod imp {
    use super::Arena;

    pub struct Guard;

    impl Drop for Guard {
        fn drop(&mut self) {}
    }

    #[inline(always)]
    pub fn enter(_arena: Arena) -> Guard {
        Guard
    }

    #[inline(always)]
    pub fn start_sampler(_inflight: super::InflightGauge) {}
}

#[cfg(feature = "memprof")]
mod imp {
    use super::{Arena, InflightGauge};
    use std::alloc::{GlobalAlloc, Layout};
    use std::cell::Cell;
    use std::io::Write;
    use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    const ARENAS: usize = Arena::ALL.len();

    /// Bytes prepended to every allocation to carry its tag. Sixteen keeps the
    /// payload on the alignment the allocator would have given it, so size
    /// classes stay comparable with an uninstrumented build. It is reported as
    /// its own column so a reader can subtract it.
    const HEADER: usize = 16;

    static LIVE_BYTES: [AtomicI64; ARENAS] = [const { AtomicI64::new(0) }; ARENAS];
    static PEAK_BYTES: [AtomicI64; ARENAS] = [const { AtomicI64::new(0) }; ARENAS];
    static LIVE_ALLOCS: [AtomicI64; ARENAS] = [const { AtomicI64::new(0) }; ARENAS];
    static TOTAL_BYTES: [AtomicU64; ARENAS] = [const { AtomicU64::new(0) }; ARENAS];
    static TOTAL_ALLOCS: [AtomicU64; ARENAS] = [const { AtomicU64::new(0) }; ARENAS];

    // `const`-initialized and holding a type with no destructor, so the access
    // registers no TLS destructor and cannot allocate -- which it must not,
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

    /// Tag allocations made on this thread until the guard drops. Nesting
    /// restores the enclosing arena, so an append inside an intake region is
    /// charged to the queue and the intake resumes after it.
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
        // `align` is unchanged and `size` only grew, so this cannot fail for a
        // layout the caller could already have allocated.
        (
            unsafe { Layout::from_size_align_unchecked(size, layout.align()) },
            offset,
        )
    }

    /// The tag sits in the last eight bytes of the header, which is at least
    /// sixteen, so it never overlaps the payload.
    unsafe fn tag_slot(user: *mut u8) -> *mut u64 {
        unsafe { user.sub(8) as *mut u64 }
    }

    /// The tagging wrapper, over whichever allocator it is given.
    pub struct TaggedAllocator<A> {
        inner: A,
    }

    impl<A> TaggedAllocator<A> {
        pub const fn new(inner: A) -> TaggedAllocator<A> {
            TaggedAllocator { inner }
        }
    }

    unsafe impl<A: GlobalAlloc> GlobalAlloc for TaggedAllocator<A> {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let (wrapped_layout, offset) = wrapped(layout);
            let base = unsafe { self.inner.alloc(wrapped_layout) };
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
            let base = unsafe { self.inner.alloc_zeroed(wrapped_layout) };
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
            unsafe { self.inner.dealloc(ptr.sub(offset), wrapped_layout) }
        }

        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            let (wrapped_layout, offset) = wrapped(layout);
            // The block keeps the arena it was born in: a buffer that grows
            // inside one region but was filled by another is still the other's
            // residency, and re-tagging here would make an arena's refunds
            // disagree with its charges.
            let index = (unsafe { tag_slot(ptr).read() } as usize).min(ARENAS - 1);
            let base = unsafe {
                self.inner.realloc(
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

    /// glibc's own view of the heap, which is the only thing that separates
    /// "the collector is holding this" from "the allocator has not given it
    /// back". It counts what C dependencies allocate directly too, so under
    /// glibc `in_use` minus the tagged total is C-side allocation plus
    /// whatever Rust allocation no guard covered.
    #[derive(Default, Clone, Copy)]
    struct MallocView {
        arena: u64,
        mmapped: u64,
        in_use: u64,
        free: u64,
        keepcost: u64,
    }

    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    fn malloc_view() -> MallocView {
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
        MallocView {
            arena: info.arena as u64,
            mmapped: info.hblkhd as u64,
            in_use: info.uordblks as u64,
            free: info.fordblks as u64,
            keepcost: info.keepcost as u64,
        }
    }

    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    fn malloc_view() -> MallocView {
        MallocView::default()
    }

    /// One field out of `/proc/self/status`, in bytes for the `kB` ones.
    fn proc_status(field: &str, scale: u64) -> u64 {
        let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
            return 0;
        };
        status
            .lines()
            .find_map(|line| line.strip_prefix(field))
            .and_then(|rest| rest.split_whitespace().next()?.parse::<u64>().ok())
            .unwrap_or(0)
            * scale
    }

    const COLUMNS: &str = "t,rss,threads,inflight_bytes,tagged_live,tagged_live_other,\
tagged_live_intake,tagged_live_queue,tagged_live_send,tagged_peak_other,tagged_peak_intake,\
tagged_peak_queue,tagged_peak_send,tagged_allocs,header_bytes,total_bytes,total_allocs,\
libc_arena,libc_mmap,libc_in_use,libc_free,libc_keepcost\n";

    fn row(started: Instant, inflight: &InflightGauge) -> String {
        let live: Vec<i64> = (0..ARENAS)
            .map(|i| LIVE_BYTES[i].load(Ordering::Relaxed))
            .collect();
        let peak: Vec<i64> = (0..ARENAS)
            .map(|i| PEAK_BYTES[i].load(Ordering::Relaxed))
            .collect();
        let allocs: i64 = (0..ARENAS)
            .map(|i| LIVE_ALLOCS[i].load(Ordering::Relaxed))
            .sum();
        let total_bytes: u64 = (0..ARENAS)
            .map(|i| TOTAL_BYTES[i].load(Ordering::Relaxed))
            .sum();
        let total_allocs: u64 = (0..ARENAS)
            .map(|i| TOTAL_ALLOCS[i].load(Ordering::Relaxed))
            .sum();
        let malloc = malloc_view();
        format!(
            "{:.2},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            started.elapsed().as_secs_f64(),
            proc_status("VmRSS:", 1024),
            proc_status("Threads:", 1),
            inflight.occupied(),
            live.iter().sum::<i64>(),
            live[Arena::Other as usize],
            live[Arena::Intake as usize],
            live[Arena::Queue as usize],
            live[Arena::Send as usize],
            peak[Arena::Other as usize],
            peak[Arena::Intake as usize],
            peak[Arena::Queue as usize],
            peak[Arena::Send as usize],
            allocs,
            allocs * HEADER as i64,
            total_bytes,
            total_allocs,
            malloc.arena,
            malloc.mmapped,
            malloc.in_use,
            malloc.free,
            malloc.keepcost,
        )
    }

    /// Sample into the CSV named by `COLLECTY_MEMPROF_CSV`, at
    /// `COLLECTY_MEMPROF_INTERVAL_MS` (default 250 ms, the 4 Hz signy's rig
    /// samples at, so the two miss the same spikes).
    ///
    /// A file rather than a route: collecty publishes no metrics port, and
    /// giving it one for this would put a measurement surface into the
    /// product.
    pub fn start_sampler(inflight: InflightGauge) {
        let Ok(path) = std::env::var("COLLECTY_MEMPROF_CSV") else {
            return;
        };
        let interval = std::env::var("COLLECTY_MEMPROF_INTERVAL_MS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(250);
        std::thread::Builder::new()
            .name("memprof".to_string())
            .spawn(move || {
                let Ok(mut file) = std::fs::File::create(&path) else {
                    return;
                };
                let _ = file.write_all(COLUMNS.as_bytes());
                let started = Instant::now();
                loop {
                    let _ = file.write_all(row(started, &inflight).as_bytes());
                    let _ = file.flush();
                    std::thread::sleep(Duration::from_millis(interval));
                }
            })
            .ok();
    }
    /// The wrapper's own accounting, exercised directly.
    ///
    /// The tagged allocator is installed by the binary, not by the library, so
    /// a test binary's own allocations do not touch these counters and a delta
    /// measured here is exactly what the calls below did. One test rather than
    /// several: the counters are process-wide statics, and two tests moving
    /// them at once would each see the other's work.
    #[cfg(test)]
    mod tests {
        use super::*;
        use std::alloc::System;

        fn live(arena: Arena) -> i64 {
            LIVE_BYTES[arena as usize].load(Ordering::Relaxed)
        }

        #[test]
        fn an_arena_is_charged_what_it_allocates_and_refunded_all_of_it() {
            let allocator = TaggedAllocator::new(System);
            let layout = Layout::from_size_align(4096, 8).unwrap();
            let before = Arena::ALL.map(live);

            // Charged to the arena current at the moment of the allocation.
            let queued = {
                let _tag = enter(Arena::Queue);
                unsafe { allocator.alloc(layout) }
            };
            assert_eq!(live(Arena::Queue) - before[Arena::Queue as usize], 4096);
            assert_eq!(live(Arena::Send) - before[Arena::Send as usize], 0);

            // Nesting restores the enclosing arena rather than clearing it.
            {
                let _outer = enter(Arena::Intake);
                {
                    let _inner = enter(Arena::Send);
                    assert_eq!(current(), Arena::Send as usize);
                }
                assert_eq!(current(), Arena::Intake as usize);
            }
            assert_eq!(current(), Arena::Other as usize);

            // A block keeps the arena it was born in, whoever grows it.
            let grown = {
                let _tag = enter(Arena::Send);
                unsafe { allocator.realloc(queued, layout, 8192) }
            };
            assert_eq!(live(Arena::Queue) - before[Arena::Queue as usize], 8192);
            assert_eq!(live(Arena::Send) - before[Arena::Send as usize], 0);

            let grown_layout = Layout::from_size_align(8192, 8).unwrap();
            unsafe { allocator.dealloc(grown, grown_layout) };

            // Every arena is back where it started: a refund goes to the arena
            // that was charged, not to the one that happened to be current.
            assert_eq!(Arena::ALL.map(live), before);
        }
    }
}

/// What the in-flight gate is holding, which the allocator cannot see because
/// a request body is allocated on a future rather than in a guarded region.
#[derive(Clone)]
pub struct InflightGauge {
    inner: std::sync::Arc<dyn Fn() -> u64 + Send + Sync>,
}

impl InflightGauge {
    pub fn new(read: impl Fn() -> u64 + Send + Sync + 'static) -> InflightGauge {
        InflightGauge {
            inner: std::sync::Arc::new(read),
        }
    }

    /// A gauge for a process that has no intake, used before one exists.
    pub fn zero() -> InflightGauge {
        InflightGauge::new(|| 0)
    }

    pub fn occupied(&self) -> u64 {
        (self.inner)()
    }
}

pub use imp::{Guard, enter, start_sampler};

#[cfg(feature = "memprof")]
pub use imp::TaggedAllocator;
