//! What the process is holding, as the kernel and the allocator each see it.
//!
//! [`crate::memprof`] answers "which part of the engine owns these bytes", and
//! only in the instrumented build, whose wrapper allocates through the system
//! allocator. The shipped binary allocates through mimalloc (`src/main.rs`),
//! and about that build this process could publish nothing at all: a run that
//! died at its limit left behind one number, the cgroup's anonymous total,
//! which is live bytes and retained bytes added together with no way to tell
//! them apart. Those are different problems with different fixes, and "it used
//! 2 GiB" does not say which one happened.
//!
//! Two sources, kept apart because they answer different questions:
//!
//! * **The kernel**, through `/proc/self`. `rss` is what the cgroup limit is
//!   actually spent on and what an OOM kill is decided on; `peak_rss` is the
//!   kernel's own high-water mark, so a peak between two scrapes is not lost.
//!   Published by every build, because it is a fact about the process rather
//!   than about an allocator.
//! * **mimalloc**, through `mi_process_info`. `committed` is what it has asked
//!   the operating system for and not handed back. Published only when
//!   mimalloc is the allocator, which is exactly when `memprof` is off:
//!   publishing it from the instrumented build would attribute glibc's
//!   behavior to mimalloc's counters, and the two must never be read off the
//!   same run.
//!
//! **What is deliberately not here, and why.** mimalloc's in-use bytes
//! (`malloc_normal`, `malloc_requested`) are maintained only when the C
//! library is compiled with `MI_STAT > 0`, and a release build compiles them
//! out — so the production binary can say what it holds from the operating
//! system but not what of that is in use. `committed` and `rss` both count
//! retained free memory, so neither alone separates live from retained. Until
//! that gap is closed, the live side has to come from the engine's own gauges
//! (memtable, sidecars, part metadata, caches) and the difference against
//! `committed` is what the allocator is sitting on. `mi_stats_get_json` would
//! add `reserved` and `purged` — how much has ever been given back — and is
//! the cheap next step if that question comes up.
//!
//! One trap, found by reading mimalloc's source rather than its header, and
//! recorded so nobody re-derives it: **`mi_process_info` does not fill
//! `current_rss` on Linux.** It seeds that field from the commit counter and
//! then lets the platform overwrite it, and the Linux implementation sets only
//! `utime`, `stime`, `page_faults` and `peak_rss` (from `ru_maxrss`). A gauge
//! named `rss` fed from that call would report committed bytes under the name
//! of resident ones. That is why the resident numbers below come from `/proc`.

/// What the allocator holds, in the vocabulary both allocators can answer in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Allocator {
    /// Bytes the allocator holds that cost resident memory: mimalloc's
    /// `committed`, jemalloc's `resident`. Both count retained free memory,
    /// which is the point -- the gap against `live_bytes` is the retention.
    pub committed_bytes: u64,
    /// High-water mark for the above, where the allocator keeps one.
    pub peak_committed_bytes: Option<u64>,
    /// Bytes actually in use by the program. jemalloc reports this
    /// (`stats.allocated`); a release-built mimalloc compiles the counter out,
    /// so it is `None` there and the engine's own gauges are the live side.
    pub live_bytes: Option<u64>,
    /// Bytes in pages the allocator is currently serving allocations from.
    /// jemalloc only, and the number that makes the two reasons separable:
    /// `active - live` is slack inside pages that are in use, which is
    /// fragmentation and cannot be returned while one live object sits on the
    /// page; `committed - active` is memory already freed and not yet handed
    /// back, which is the decay policy and is a setting rather than a defect.
    pub active_bytes: Option<u64>,
    /// The allocator's own bookkeeping, which is resident too and belongs to
    /// neither of the two above.
    pub metadata_bytes: Option<u64>,
    /// Total address space mapped. Costs no resident memory by itself; it is
    /// here so a large `retained` can be read against something.
    pub mapped_bytes: Option<u64>,
    /// Address space the allocator has not returned to the operating system.
    /// jemalloc only (`stats.retained`).
    pub retained_bytes: Option<u64>,
    /// Major faults, where the allocator reports them (mimalloc).
    pub major_page_faults: Option<u64>,
}

/// What the process holds, at one instant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    /// Resident bytes, from the kernel. `None` where `/proc` cannot be read.
    pub rss_bytes: Option<u64>,
    /// The kernel's own high-water mark for the above.
    pub peak_rss_bytes: Option<u64>,
    /// `None` in a build whose allocator publishes nothing this can read,
    /// which is the instrumented build.
    pub allocator: Option<Allocator>,
}

/// Read both sources.
pub fn snapshot() -> Snapshot {
    let (rss_bytes, peak_rss_bytes) = kernel_resident();
    Snapshot {
        rss_bytes,
        peak_rss_bytes,
        allocator: self::allocator::read(),
    }
}

pub fn render() -> String {
    let stats = snapshot();
    let mut out = String::new();
    if let Some(rss) = stats.rss_bytes {
        out.push_str(&format!(
            "# HELP signy_process_rss_bytes Resident bytes of this process, from the kernel. \
This is what a memory limit is spent on and what an OOM kill is decided on.\n\
# TYPE signy_process_rss_bytes gauge\n\
signy_process_rss_bytes {rss}\n"
        ));
    }
    if let Some(peak) = stats.peak_rss_bytes {
        out.push_str(&format!(
            "# HELP signy_process_peak_rss_bytes The kernel's own high-water mark for the above, \
so a peak between two scrapes is not lost.\n\
# TYPE signy_process_peak_rss_bytes gauge\n\
signy_process_peak_rss_bytes {peak}\n"
        ));
    }
    let Some(allocator) = stats.allocator else {
        return out;
    };
    out.push_str(&format!(
        "# HELP signy_allocator_committed_bytes Bytes the allocator holds that cost resident \
memory -- mimalloc's committed, jemalloc's resident. It counts retained free memory, so the gap \
against what is live is what the allocator is sitting on, and that is what kills a process whose \
own residents are flat.\n\
# TYPE signy_allocator_committed_bytes gauge\n\
signy_allocator_committed_bytes {}\n",
        allocator.committed_bytes
    ));
    if let Some(peak) = allocator.peak_committed_bytes {
        out.push_str(&format!(
            "# TYPE signy_allocator_peak_committed_bytes gauge\n\
signy_allocator_peak_committed_bytes {peak}\n"
        ));
    }
    if let Some(live) = allocator.live_bytes {
        out.push_str(&format!(
            "# HELP signy_allocator_live_bytes Bytes in use by the program, from the allocator \
itself. Absent under mimalloc, whose in-use counters a release build compiles out.\n\
# TYPE signy_allocator_live_bytes gauge\n\
signy_allocator_live_bytes {live}\n"
        ));
    }
    if let Some(active) = allocator.active_bytes {
        out.push_str(&format!(
            "# HELP signy_allocator_active_bytes Bytes in pages the allocator is serving \
allocations from. Against live bytes it separates the two reasons memory is not returned: \
active minus live is slack inside pages that are in use, which one live object per page is enough \
to pin, and committed minus active is freed memory the decay policy has not handed back yet.\n\
# TYPE signy_allocator_active_bytes gauge\n\
signy_allocator_active_bytes {active}\n"
        ));
    }
    if let Some(metadata) = allocator.metadata_bytes {
        out.push_str(&format!(
            "# HELP signy_allocator_metadata_bytes The allocator's own bookkeeping. Resident, and \
belonging to neither the live nor the retained side.\n\
# TYPE signy_allocator_metadata_bytes gauge\n\
signy_allocator_metadata_bytes {metadata}\n"
        ));
    }
    if let Some(mapped) = allocator.mapped_bytes {
        out.push_str(&format!(
            "# TYPE signy_allocator_mapped_bytes gauge\n\
signy_allocator_mapped_bytes {mapped}\n"
        ));
    }
    if let Some(retained) = allocator.retained_bytes {
        out.push_str(&format!(
            "# HELP signy_allocator_retained_bytes Address space the allocator has not returned \
to the operating system. It costs no resident memory itself; a large value beside a large \
committed one says the decay is running and the resident half is still in use.\n\
# TYPE signy_allocator_retained_bytes gauge\n\
signy_allocator_retained_bytes {retained}\n"
        ));
    }
    if let Some(faults) = allocator.major_page_faults {
        out.push_str(&format!(
            "# HELP signy_process_major_page_faults_total Faults that went to disk: reclaim took \
back a page this process wanted again.\n\
# TYPE signy_process_major_page_faults_total counter\n\
signy_process_major_page_faults_total {faults}\n"
        ));
    }
    out
}

/// Resident and peak-resident bytes from `/proc/self/status`.
///
/// `VmRSS` and `VmHWM` rather than `statm`, because the peak is only in
/// `status` and reading one file for both keeps them from disagreeing about
/// the instant they describe.
fn kernel_resident() -> (Option<u64>, Option<u64>) {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return (None, None);
    };
    let field = |name: &str| {
        status
            .lines()
            .find_map(|line| {
                line.strip_prefix(name)?
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()
            })
            .map(|kib| kib * 1024)
    };
    (field("VmRSS:"), field("VmHWM:"))
}

/// mimalloc: the default shipped allocator.
#[cfg(all(not(feature = "memprof"), not(feature = "jemalloc")))]
mod allocator {
    use super::Allocator;

    pub fn dump_profile(_dir: &std::path::Path) -> Result<String, String> {
        Err("mimalloc has no heap profiler; build with --features jemalloc-prof".to_string())
    }

    /// mimalloc's text dump exists (`mi_stats_print`) but a release build
    /// compiles out everything that would make it worth reading, so this
    /// says that rather than serving a page of zeroes.
    pub fn report() -> String {
        String::from(
            "mimalloc: no statistics in a release build (MI_STAT=0). Build with \
--features jemalloc for a per-size-class report.\n",
        )
    }

    pub fn read() -> Option<Allocator> {
        let mut elapsed_msecs = 0usize;
        let mut user_msecs = 0usize;
        let mut system_msecs = 0usize;
        let mut current_rss = 0usize;
        let mut peak_rss = 0usize;
        let mut current_commit = 0usize;
        let mut peak_commit = 0usize;
        let mut page_faults = 0usize;
        // SAFETY: every argument is a live, initialized `usize` this call only
        // writes to. `mi_process_info` reads counters and allocates nothing.
        unsafe {
            libmimalloc_sys::mi_process_info(
                &mut elapsed_msecs,
                &mut user_msecs,
                &mut system_msecs,
                &mut current_rss,
                &mut peak_rss,
                &mut current_commit,
                &mut peak_commit,
                &mut page_faults,
            );
        }
        // `current_rss` and `peak_rss` are deliberately dropped on the floor:
        // see this module's header for what Linux does and does not fill in.
        Some(Allocator {
            committed_bytes: current_commit as u64,
            peak_committed_bytes: Some(peak_commit as u64),
            // All four need `MI_STAT > 0`, which a release build compiles out.
            live_bytes: None,
            active_bytes: None,
            metadata_bytes: None,
            mapped_bytes: None,
            retained_bytes: None,
            major_page_faults: Some(page_faults as u64),
        })
    }
}

/// jemalloc, built with `--features jemalloc`.
#[cfg(all(not(feature = "memprof"), feature = "jemalloc"))]
mod allocator {
    use super::Allocator;

    #[cfg(not(feature = "jemalloc-prof"))]
    pub fn dump_profile(_dir: &std::path::Path) -> Result<String, String> {
        Err("built without the jemalloc-prof feature".to_string())
    }

    #[cfg(feature = "jemalloc-prof")]
    pub fn dump_profile(dir: &std::path::Path) -> Result<String, String> {
        if !tikv_jemalloc_ctl::profiling::prof::read().unwrap_or(false) {
            return Err(
                "profiling is compiled in but off; start with _RJEM_MALLOC_CONF=prof:true"
                    .to_string(),
            );
        }
        std::fs::create_dir_all(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
        let path = dir.join(format!(
            "heap-{}.prof",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_secs())
                .unwrap_or(0)
        ));
        let c_path = std::ffi::CString::new(path.as_os_str().as_encoded_bytes())
            .map_err(|error| format!("path is not a C string: {error}"))?;
        // SAFETY: `prof.dump` reads one `*const c_char` from the value it is
        // given, which is what `write` passes when `T` is that pointer type --
        // a reference to it would be one level too many, and jemalloc reports
        // that as a side-effecting failure rather than a type error. The
        // string outlives the call and the key is NUL-terminated.
        unsafe {
            tikv_jemalloc_ctl::raw::write(b"prof.dump\0", c_path.as_ptr())
                .map_err(|error| format!("prof.dump failed: {error}"))?;
        }
        Ok(path.display().to_string())
    }

    /// jemalloc's own text dump, including the per-bin table.
    pub fn report() -> String {
        let mut buffer: Vec<u8> = Vec::new();
        // SAFETY: the callback is called with a pointer to a NUL-terminated
        // string that is valid for the duration of the call, and `arg` is the
        // buffer we pass in. Nothing here unwinds.
        unsafe extern "C" fn write_cb(arg: *mut std::ffi::c_void, text: *const std::ffi::c_char) {
            if arg.is_null() || text.is_null() {
                return;
            }
            // SAFETY: `arg` is the `Vec<u8>` handed to `malloc_stats_print`,
            // and jemalloc calls this synchronously from the same thread.
            let buffer = unsafe { &mut *(arg as *mut Vec<u8>) };
            // SAFETY: jemalloc passes a NUL-terminated C string.
            let text = unsafe { std::ffi::CStr::from_ptr(text) };
            buffer.extend_from_slice(text.to_bytes());
        }
        // SAFETY: `write_cb` matches the expected signature, the buffer
        // outlives the call, and the options string is NUL-terminated.
        unsafe {
            tikv_jemalloc_sys::malloc_stats_print(
                Some(write_cb),
                (&raw mut buffer).cast::<std::ffi::c_void>(),
                c"".as_ptr(),
            );
        }
        String::from_utf8_lossy(&buffer).into_owned()
    }

    pub fn read() -> Option<Allocator> {
        // jemalloc's statistics are cached and refreshed by advancing the
        // epoch; without this every scrape after the first reads the same
        // numbers, which would look exactly like a heap that stopped moving.
        let _ = tikv_jemalloc_ctl::epoch::advance();
        let resident = tikv_jemalloc_ctl::stats::resident::read().ok()?;
        Some(Allocator {
            committed_bytes: resident as u64,
            // jemalloc keeps no high-water mark of its own; the kernel's
            // `VmHWM` is beside it and is the peak that matters.
            peak_committed_bytes: None,
            live_bytes: tikv_jemalloc_ctl::stats::allocated::read()
                .ok()
                .map(|bytes| bytes as u64),
            active_bytes: tikv_jemalloc_ctl::stats::active::read()
                .ok()
                .map(|bytes| bytes as u64),
            metadata_bytes: tikv_jemalloc_ctl::stats::metadata::read()
                .ok()
                .map(|bytes| bytes as u64),
            mapped_bytes: tikv_jemalloc_ctl::stats::mapped::read()
                .ok()
                .map(|bytes| bytes as u64),
            retained_bytes: tikv_jemalloc_ctl::stats::retained::read()
                .ok()
                .map(|bytes| bytes as u64),
            // Not jemalloc's to report; the kernel's own counter is what the
            // soak reads for this.
            major_page_faults: None,
        })
    }
}

/// The instrumented build: its heap is glibc's, and mimalloc's or jemalloc's
/// counters would describe one nothing in the process allocates from.
#[cfg(feature = "memprof")]
mod allocator {
    use super::Allocator;

    pub fn read() -> Option<Allocator> {
        None
    }

    /// The instrumented build's own report is `/metrics`' memprof families,
    /// which attribute live bytes by arena.
    pub fn report() -> String {
        String::from("memprof build: see the signy_memprof_* families on /metrics\n")
    }

    pub fn dump_profile(_dir: &std::path::Path) -> Result<String, String> {
        Err("the memprof build has no heap profiler; its attribution is on /metrics".to_string())
    }
}

/// The allocator's own full report, as text, for the questions a handful of
/// gauges cannot answer.
///
/// The gauges say how much is slack; this says **which size classes** hold it.
/// jemalloc prints a per-bin table -- regions in use, slabs, utilization -- and
/// a size class whose utilization is low while its slab count is high is the
/// shape that pins pages. That is the evidence an argument for arena
/// allocation needs, and without it the argument is a guess.
///
/// Empty in builds that have no such report. Not on the metrics path: it is a
/// page of text, pulled deliberately, not scraped every fifteen seconds.
pub fn report() -> String {
    self::allocator::report()
}

/// Write a heap profile -- live bytes by allocation stack -- into `dir`.
///
/// Returns the path written, or why it could not be. Needs the
/// `jemalloc-prof` build *and* `_RJEM_MALLOC_CONF=prof:true` at startup; the
/// error says which is missing rather than failing silently, because a
/// profiler that quietly does nothing is worse than one that is absent.
///
/// The profile is jemalloc's own format, read with `jeprof`.
pub fn dump_profile(dir: &std::path::Path) -> Result<String, String> {
    self::allocator::dump_profile(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_running_process_reports_a_resident_size_and_a_peak_at_least_as_large() {
        let stats = snapshot();
        let Some(rss) = stats.rss_bytes else {
            return;
        };
        let Some(peak) = stats.peak_rss_bytes else {
            return;
        };
        assert!(rss > 0, "{stats:?}");
        assert!(peak >= rss, "{stats:?}");
        let body = render();
        assert!(body.contains("signy_process_rss_bytes "), "{body}");
    }

    /// The allocator half is published by exactly one build, and it is the one
    /// that ships.
    #[test]
    fn only_the_production_build_publishes_the_allocator_gauges() {
        let stats = snapshot();
        let body = render();
        if cfg!(feature = "memprof") {
            assert!(stats.allocator.is_none(), "{stats:?}");
            assert!(!body.contains("signy_allocator_committed_bytes"), "{body}");
        } else {
            assert!(stats.allocator.is_some(), "{stats:?}");
            assert!(body.contains("signy_allocator_committed_bytes "), "{body}");
            // Only jemalloc can say what is live; mimalloc's counter is
            // compiled out of a release build, and a gauge that is absent is
            // the honest report of that.
            assert_eq!(
                stats.allocator.and_then(|a| a.live_bytes).is_some(),
                cfg!(feature = "jemalloc"),
                "{stats:?}"
            );
        }
    }
}
