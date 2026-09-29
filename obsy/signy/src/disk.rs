use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::watch;

use crate::shutdown::wait_for_drain;

/// How much room is left on the filesystem holding the data directory.
///
/// Sampled by a task rather than read where it is used. `statvfs` is a syscall
/// that can block on a stalled filesystem, and the two readers are the ingest
/// path and a `/metrics` scrape — neither is somewhere to put a blocking call
/// on an unbounded number of requests. What they read instead is a number a
/// task refreshed, whose staleness is bounded by the sample interval.
///
/// This is deliberately the one thing the engine measures about the machine it
/// is on. Everything else on `/metrics` describes the engine's own structures,
/// and a node exporter describes the machine — but a full disk is the failure
/// this process meets first and reports worst, as a flush that cannot write.
pub struct DiskSpace {
    free_bytes: AtomicU64,
    total_bytes: AtomicU64,
}

impl DiskSpace {
    /// Not yet sampled, and therefore not a reason to refuse anything.
    ///
    /// `u64::MAX` rather than zero: a guard that fails closed before it has
    /// measured anything would refuse every write between process start and the
    /// first sample.
    pub fn unknown() -> Self {
        Self {
            free_bytes: AtomicU64::new(u64::MAX),
            total_bytes: AtomicU64::new(0),
        }
    }

    /// Sampled once, so the value is real from the moment the gate can read it.
    /// A failure leaves it unknown, which is the same disposition as a stalled
    /// sampler: the engine does not stop accepting because it cannot see.
    pub fn sampled(path: &Path) -> Self {
        let space = Self::unknown();
        space.refresh(path);
        space
    }

    pub fn free_bytes(&self) -> u64 {
        self.free_bytes.load(Ordering::Relaxed)
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes.load(Ordering::Relaxed)
    }

    /// Take one reading. A failure is logged and leaves the previous value in
    /// place, because a filesystem that cannot answer once usually answers the
    /// next time and the alternative is a self-inflicted refusal.
    pub fn refresh(&self, path: &Path) {
        match statvfs(path) {
            Ok(reading) => {
                self.free_bytes.store(reading.free_bytes, Ordering::Relaxed);
                self.total_bytes
                    .store(reading.total_bytes, Ordering::Relaxed);
            }
            Err(error) => {
                tracing::warn!(
                    path = %path.display(),
                    %error,
                    "failed to read free space on the data directory"
                );
            }
        }
    }

    #[cfg(test)]
    pub fn with_free_bytes(free_bytes: u64) -> Self {
        Self {
            free_bytes: AtomicU64::new(free_bytes),
            total_bytes: AtomicU64::new(free_bytes),
        }
    }
}

pub struct Reading {
    pub free_bytes: u64,
    pub total_bytes: u64,
}

/// Free space as the unprivileged user sees it.
///
/// `f_bavail` rather than `f_bfree`: filesystems reserve a slice for root, and
/// counting it as available means the guard lets writes through right up to the
/// point where they start failing.
pub fn statvfs(path: &Path) -> Result<Reading, String> {
    let raw = std::ffi::CString::new(path.as_os_str().as_encoded_bytes())
        .map_err(|error| format!("data directory path is not a valid C string: {error}"))?;
    // SAFETY: `raw` is a NUL-terminated path and `stats` is the out parameter
    // the call is documented to fill, read only after a zero return.
    let stats = unsafe {
        let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        if libc::statvfs(raw.as_ptr(), stats.as_mut_ptr()) != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        stats.assume_init()
    };
    let block_size = stats.f_frsize as u64;
    Ok(Reading {
        free_bytes: (stats.f_bavail as u64).saturating_mul(block_size),
        total_bytes: (stats.f_blocks as u64).saturating_mul(block_size),
    })
}

/// Refresh the reading until the process drains.
pub async fn disk_sampler_loop(
    space: Arc<DiskSpace>,
    data_dir: PathBuf,
    interval: std::time::Duration,
    mut drain_rx: watch::Receiver<bool>,
) {
    let mut ticker = tokio::time::interval(interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = wait_for_drain(&mut drain_rx) => return,
        }
        let space = space.clone();
        let data_dir = data_dir.clone();
        // On a blocking pool: the point of sampling out of band is that a
        // stalled filesystem stalls one task instead of the runtime.
        if tokio::task::spawn_blocking(move || space.refresh(&data_dir))
            .await
            .is_err()
        {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reading_of_the_data_directory_is_plausible() {
        let reading =
            statvfs(&std::env::temp_dir()).expect("the temp directory is on a filesystem");
        assert!(reading.total_bytes > 0, "a filesystem has a size");
        assert!(
            reading.free_bytes <= reading.total_bytes,
            "free space cannot exceed the filesystem"
        );
    }

    #[test]
    fn an_unreadable_path_leaves_the_previous_reading_alone() {
        let space = DiskSpace::sampled(&std::env::temp_dir());
        let sampled = space.free_bytes();
        assert_ne!(sampled, u64::MAX, "the temp directory is readable");

        space.refresh(Path::new("/definitely/not/a/path/on/this/machine"));
        assert_eq!(
            space.free_bytes(),
            sampled,
            "a failed sample must not be read as a full disk"
        );
    }
}
