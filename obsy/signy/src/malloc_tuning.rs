//! glibc allocator tuning, applied before the runtime spawns a thread.
//!
//! `docs/MEMORY_ATTRIBUTION.md` measured the gap this closes: at the OOM the
//! kernel decided on, 44–69% of the cgroup's anonymous memory was
//! already-freed heap glibc had not returned, and the `trimmed` variant —
//! `MALLOC_ARENA_MAX=1` with a fixed 128 KiB trim threshold — took anon/live
//! from 2.5–4.1 down to 1.34 on the same workload. Setting it here instead of
//! asking every operator to export two environment variables makes the
//! measured configuration the default one.
//!
//! This must run before any other thread exists: an arena is created the
//! first time a thread contends for one, and `M_ARENA_MAX` only bounds the
//! arenas not yet created. `main` calls this before building the tokio
//! runtime.
//!
//! `SIGNY_MALLOC_TUNING=off` restores glibc's defaults — the knob an
//! A/B measurement or an unexpected throughput regression reaches for.

/// Few arenas: freed memory consolidates where `M_TRIM_THRESHOLD` can
/// actually return it. One arena was measured first and rejected: anon fell
/// 3.6× but the flush path — allocation-heavy by design — halved its cadence
/// contending with the query and ingest threads for the single arena, and
/// the steady WAL backlog rose 8 → 50 MiB. The default is the measured
/// compromise; `SIGNY_MALLOC_ARENA_MAX` overrides it (0 leaves glibc's
/// own scaling in place, trim threshold still applied).
const DEFAULT_ARENA_MAX: libc_shim::c_int = 4;

/// Fixed 128 KiB, matching the measured `MALLOC_TRIM_THRESHOLD_=131072`.
/// Setting it also disables glibc's dynamic threshold, which only ever
/// ratchets upward and is how the high-water mark became permanent.
#[cfg(all(target_os = "linux", target_env = "gnu", feature = "memprof"))]
const TRIM_THRESHOLD: libc_shim::c_int = 131072;

/// Fixed 128 KiB, same number and same reasoning as the trim threshold: the
/// mmap threshold is the *other* glibc parameter that only ratchets upward —
/// every free of an mmapped chunk raises it toward 32 MiB, after which large
/// allocations land on the heap and their frees join the retained high-water
/// the trim threshold cannot reach. Fixing it sends every allocation over
/// 128 KiB through mmap, whose free is an munmap the kernel gets back
/// immediately. Measured on the soak rig (2 GiB, sustained 20 k eps):
/// anon/live 5.30 → 1.60 and time-to-OOM 150 → 502 s with arenas left at 4.
/// The earlier rejection of this knob dated from the pre-streaming-merge
/// build, whose kill was live spikes rather than retention.
/// `SIGNY_MALLOC_MMAP_THRESHOLD` overrides; 0 leaves glibc's dynamic
/// ratchet in place.
const MMAP_THRESHOLD: libc_shim::c_int = 131072;

mod libc_shim {
    #![allow(non_camel_case_types)]
    pub type c_int = i32;
    // glibc `malloc.h` values; stable ABI.
    #[cfg(all(target_os = "linux", target_env = "gnu", feature = "memprof"))]
    pub const M_TRIM_THRESHOLD: c_int = -1;
    #[cfg(all(target_os = "linux", target_env = "gnu", feature = "memprof"))]
    pub const M_MMAP_THRESHOLD: c_int = -3;
    #[cfg(all(target_os = "linux", target_env = "gnu", feature = "memprof"))]
    pub const M_ARENA_MAX: c_int = -8;
    #[cfg(all(target_os = "linux", target_env = "gnu", feature = "memprof"))]
    unsafe extern "C" {
        pub fn mallopt(param: c_int, value: c_int) -> c_int;
        pub fn malloc_trim(pad: usize) -> c_int;
    }
}

/// Apply the tuning unless `SIGNY_MALLOC_TUNING=off`.
///
/// Returns whether it was applied, purely so startup can log the fact — an
/// operator comparing memory numbers across builds needs to know which
/// allocator behavior each one ran with.
pub fn apply_from_env() -> bool {
    if std::env::var("SIGNY_MALLOC_TUNING")
        .map(|value| value.eq_ignore_ascii_case("off"))
        .unwrap_or(false)
    {
        return false;
    }
    let arena_max = std::env::var("SIGNY_MALLOC_ARENA_MAX")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(DEFAULT_ARENA_MAX);
    let mmap_threshold = std::env::var("SIGNY_MALLOC_MMAP_THRESHOLD")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(MMAP_THRESHOLD);
    apply(arena_max, mmap_threshold)
}

// The production binary's allocator is mimalloc (`src/main.rs`), so glibc
// tuning targets a heap nothing uses: it applies only in the memprof build,
// whose instrumented wrapper still allocates through glibc.
#[cfg(all(target_os = "linux", target_env = "gnu", feature = "memprof"))]
fn apply(arena_max: i32, mmap_threshold: i32) -> bool {
    // SAFETY: mallopt only writes allocator parameters; called before any
    // other thread exists.
    unsafe {
        if arena_max > 0 {
            libc_shim::mallopt(libc_shim::M_ARENA_MAX, arena_max);
        }
        libc_shim::mallopt(libc_shim::M_TRIM_THRESHOLD, TRIM_THRESHOLD);
        if mmap_threshold > 0 {
            libc_shim::mallopt(libc_shim::M_MMAP_THRESHOLD, mmap_threshold);
        }
    }
    true
}

#[cfg(not(all(target_os = "linux", target_env = "gnu", feature = "memprof")))]
fn apply(_arena_max: i32, _mmap_threshold: i32) -> bool {
    false
}

/// Return glibc's free-but-retained pages to the kernel, from the middle of
/// every arena and not just the top.
///
/// The trim threshold set above only releases the top of each heap; a chunk
/// freed *under* live allocations stays resident, and the second 24-hour soak
/// measured exactly that shape: every gauged resident flat — sidecars,
/// cache, parts, disk — while anon crept ~130 MiB/hour until the 2 GiB kill
/// at t≈8653 s (todo.md). `malloc_trim(0)` walks the arenas and
/// `MADV_DONTNEED`s whole free pages wherever they sit, which is the one
/// glibc call that reaches the middle-of-heap creep. Called periodically by
/// the loop `SIGNY_MALLOC_TRIM_INTERVAL` configures.
///
/// Returns whether anything could be released at all (false on non-glibc,
/// where the loop never starts).
pub fn trim() -> bool {
    trim_impl()
}

#[cfg(all(target_os = "linux", target_env = "gnu", feature = "memprof"))]
fn trim_impl() -> bool {
    // SAFETY: malloc_trim only consolidates and releases free memory; it is
    // documented as callable at any time and takes the arena locks itself.
    unsafe {
        libc_shim::malloc_trim(0);
    }
    true
}

#[cfg(not(all(target_os = "linux", target_env = "gnu", feature = "memprof")))]
fn trim_impl() -> bool {
    false
}
