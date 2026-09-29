//! Dropping a written part's pages from the OS cache, deliberately.
//!
//! The convoy-1h soak measured the whole process freezing for up to 52 s the
//! moment cgroup `memory.current` touched `memory.max`, and resuming the
//! instant retention's file deletions gave pages back (todo.md, 2026-08-10):
//! at the limit every page-touching thread enters direct reclaim, and the
//! flush/merge write stream keeps refilling what reclaim frees. The engine's
//! side of the fix is to not let freshly written part files sit in the cache
//! at all — they are fsynced (clean) by the time this is called, the WAL
//! covers their durability story, and a query that wants one re-reads it
//! from disk once and then holds it in the row-group cache, which is the
//! cache this engine actually budgets.
//!
//! The same no-new-dependency shim style as `malloc_tuning`: one libc call,
//! declared here, compiled out on non-Linux.

#[cfg(target_os = "linux")]
use std::fs;
use std::path::Path;

#[cfg(target_os = "linux")]
mod libc_shim {
    #![allow(non_camel_case_types)]
    pub type c_int = i32;
    /// Linux's value on every architecture this engine ships on.
    pub const POSIX_FADV_DONTNEED: c_int = 4;
    unsafe extern "C" {
        pub fn posix_fadvise(fd: c_int, offset: i64, len: i64, advice: c_int) -> c_int;
    }
}

/// Advise the kernel that a file's cached pages are not needed. Best-effort:
/// a failure changes nothing about correctness, so it is ignored.
#[cfg(target_os = "linux")]
pub fn drop_cache(path: &Path) {
    use std::os::unix::io::AsRawFd;
    if let Ok(file) = fs::File::open(path) {
        // len 0 means "to the end of the file".
        unsafe {
            libc_shim::posix_fadvise(file.as_raw_fd(), 0, 0, libc_shim::POSIX_FADV_DONTNEED);
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn drop_cache(_path: &Path) {}
