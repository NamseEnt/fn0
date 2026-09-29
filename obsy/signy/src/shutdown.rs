use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
// See the note in `flush.rs`: virtual time is what makes the force-flush
// backoff and its operator warning testable at all.
use tokio::time::Instant;

use tokio::sync::{mpsc, watch};

use crate::config::Config;
use crate::flush::{self, ForceFlush};
use crate::journal::Journal;
use crate::memtable::MemTable;
use crate::object_storage::RemoteCache;
use crate::part_registry::PartRegistry;
use crate::trace::TraceMemTable;
use crate::trace_registry::TraceRegistry;

/// Coordinates graceful shutdown for machine replacement. `draining` gates new
/// ingest and readiness; the watch channel fans the drain signal out to the
/// HTTP/gRPC servers and the background workers; the remaining fields expose
/// force-flush progress so an external controller can confirm durability before
/// the disk is discarded.
pub struct ShutdownState {
    draining: AtomicBool,
    fenced: AtomicBool,
    force_flush_complete: AtomicBool,
    pending_flush_bytes: AtomicU64,
    drain_tx: watch::Sender<bool>,
    _keep_rx: watch::Receiver<bool>,
}

impl ShutdownState {
    pub fn new() -> Self {
        let (drain_tx, _keep_rx) = watch::channel(false);
        Self {
            draining: AtomicBool::new(false),
            fenced: AtomicBool::new(false),
            force_flush_complete: AtomicBool::new(false),
            pending_flush_bytes: AtomicU64::new(0),
            drain_tx,
            _keep_rx,
        }
    }

    /// Start draining: reject new ingest and wake every drain subscriber. Safe
    /// to call more than once.
    pub fn begin_drain(&self) {
        self.draining.store(true, Ordering::Release);
        let _ = self.drain_tx.send(true);
    }

    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::Acquire)
    }

    /// Another process owns the prefix now. Drains, and marks the drain as one
    /// that cannot end in durability: everything this instance has not already
    /// published belongs to a manifest it no longer owns, so retrying the
    /// force-flush would loop until the orchestrator kills it.
    ///
    /// The unflushed data stays in the WAL on this disk. That is the honest
    /// outcome — the alternative is publishing over a writer that has been
    /// accepting traffic since, which loses whichever side is overwritten.
    pub fn mark_fenced(&self) {
        if !self.fenced.swap(true, Ordering::AcqRel) {
            tracing::error!(
                "fenced by a newer writer: this instance no longer owns the object-store prefix \
and is shutting down. Unflushed data remains in the local WAL; do not discard this disk until \
it has been reconciled."
            );
        }
        self.begin_drain();
    }

    pub fn is_fenced(&self) -> bool {
        self.fenced.load(Ordering::Acquire)
    }

    /// A receiver that resolves (via [`wait_for_drain`]) once draining starts.
    pub fn subscribe(&self) -> watch::Receiver<bool> {
        self.drain_tx.subscribe()
    }

    pub fn set_pending_flush_bytes(&self, bytes: u64) {
        self.pending_flush_bytes.store(bytes, Ordering::Relaxed);
    }

    pub fn pending_flush_bytes(&self) -> u64 {
        self.pending_flush_bytes.load(Ordering::Relaxed)
    }

    pub fn mark_flush_complete(&self) {
        self.force_flush_complete.store(true, Ordering::Release);
    }

    pub fn is_flush_complete(&self) -> bool {
        self.force_flush_complete.load(Ordering::Acquire)
    }
}

impl Default for ShutdownState {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve when the drain signal flips to `true`, including the case where it is
/// already `true`. The borrowed guard is dropped immediately so the caller does
/// not hold the watch lock. If every sender has been dropped the drain can never
/// arrive, so this never resolves rather than falsely triggering shutdown.
pub async fn wait_for_drain(rx: &mut watch::Receiver<bool>) {
    if rx.wait_for(|draining| *draining).await.is_err() {
        std::future::pending::<()>().await;
    }
}

/// Wait for a SIGTERM or SIGINT (or a Ctrl-C on non-Unix). The first signal
/// starts the drain sequence.
pub async fn wait_for_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = match signal(SignalKind::terminate()) {
            Ok(term) => term,
            Err(error) => {
                tracing::error!(%error, "failed to install SIGTERM handler");
                std::future::pending::<()>().await;
                return;
            }
        };
        let mut interrupt = match signal(SignalKind::interrupt()) {
            Ok(interrupt) => interrupt,
            Err(error) => {
                tracing::error!(%error, "failed to install SIGINT handler");
                std::future::pending::<()>().await;
                return;
            }
        };
        tokio::select! {
            _ = term.recv() => tracing::warn!("received SIGTERM"),
            _ = interrupt.recv() => tracing::warn!("received SIGINT"),
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
        tracing::warn!("received Ctrl-C");
    }
}

/// Everything the final force-flush needs to drive the MemTables to durable
/// object-store parts and advance the journal checkpoint.
pub struct FinalizeContext {
    pub shutdown: Arc<ShutdownState>,
    pub memtable: Arc<MemTable>,
    pub trace_memtable: Arc<TraceMemTable>,
    pub journal: Arc<Journal>,
    pub registry: Arc<PartRegistry>,
    pub trace_registry: Arc<TraceRegistry>,
    pub series_registry: Arc<crate::series_registry::SeriesRegistry>,
    pub remote_cache: Option<Arc<RemoteCache>>,
    pub config: Arc<Config>,
}

const FORCE_FLUSH_MIN_BACKOFF: Duration = Duration::from_millis(200);
const FORCE_FLUSH_MAX_BACKOFF: Duration = Duration::from_secs(10);

/// How the final force-flush ended, so the caller can distinguish a durable exit
/// from an operator-forced one. An operator abort leaves acknowledged data on
/// the WAL only; the process must not report a clean shutdown in that case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownOutcome {
    /// Every acknowledged record and the checkpoint reached durable storage.
    Durable,
    /// The operator forced termination before durability. The WAL is intact, so
    /// a restart on this same disk recovers without loss, but the disk must not
    /// be discarded (no machine replacement).
    AbortedByOperator,
    /// Another writer claimed the prefix. Retrying can never succeed, so the
    /// drain stops immediately rather than looping until it is killed. Like
    /// `AbortedByOperator` the WAL is intact and the disk must be kept.
    Fenced,
}

/// Force every pending MemTable record and checkpoint to durable object storage.
///
/// There is deliberately no hard timeout: a machine replacement discards the
/// disk, so giving up would lose data. Transient object-store failures are
/// retried with bounded backoff forever. If retries keep failing past
/// `shutdown_flush_warn_after`, a prominent stdout message repeats and an
/// operator can force termination with `SIGUSR1` (or `exit` on stdin when a
/// terminal is attached); absent that the process keeps retrying and exits on
/// its own the moment a flush succeeds.
///
/// The operator abort is only reachable *between* attempts, while backing off
/// after a failed pass. A hung (rather than erroring) object-store call would
/// leave neither the flush nor the abort able to make progress; this design
/// relies on `object_store`'s per-request and retry timeouts turning a stalled
/// backend into a retryable error. A backend configured without timeouts could
/// therefore block shutdown until the process is sent SIGKILL.
pub async fn finalize_flush(context: FinalizeContext) -> ShutdownOutcome {
    let warn_after = context.config.shutdown_flush_warn_after;
    finalize_flush_with_abort(context, warn_after, spawn_abort_watcher).await
}

/// Core force-flush retry loop with an injectable abort source so tests can
/// drive the operator-abort path without real stdin.
pub(crate) async fn finalize_flush_with_abort<F>(
    context: FinalizeContext,
    warn_after: Duration,
    make_abort: F,
) -> ShutdownOutcome
where
    F: FnOnce() -> mpsc::Receiver<()>,
{
    let started = Instant::now();
    let mut pending_checkpoint: Option<u64> = None;
    let mut attempt: u64 = 0;
    let mut backoff = FORCE_FLUSH_MIN_BACKOFF;
    let mut abort_rx: Option<mpsc::Receiver<()>> = None;
    let mut make_abort = Some(make_abort);

    loop {
        let pending_bytes = context
            .memtable
            .approximate_size()
            .saturating_add(context.trace_memtable.approximate_size())
            .saturating_add(context.journal.series_memtable().approximate_size());
        context
            .shutdown
            .set_pending_flush_bytes(pending_bytes as u64);

        let pass = ForceFlush {
            memtable: &context.memtable,
            trace_memtable: &context.trace_memtable,
            journal: &context.journal,
            registry: &context.registry,
            trace_registry: &context.trace_registry,
            series_registry: &context.series_registry,
            remote_cache: context.remote_cache.as_deref(),
            config: &context.config,
            pending_checkpoint: &mut pending_checkpoint,
        };

        if context.shutdown.is_fenced() {
            // Checked before the attempt rather than only after a failure: the
            // fence may have been observed by another worker, and a pass that
            // would succeed locally must still not publish.
            return ShutdownOutcome::Fenced;
        }

        match flush::force_flush_pass(pass).await {
            Ok(true) => {
                context.shutdown.set_pending_flush_bytes(0);
                context.shutdown.mark_flush_complete();
                tracing::info!(
                    attempts = attempt + 1,
                    "force-flush complete; all acknowledged data is durable"
                );
                return ShutdownOutcome::Durable;
            }
            Ok(false) => {
                // A pass succeeded but more remains (only possible if some data
                // was not yet visible to this pass). Retry promptly, but yield
                // so a hypothetical no-progress case cannot starve the runtime.
                backoff = FORCE_FLUSH_MIN_BACKOFF;
                tokio::task::yield_now().await;
                continue;
            }
            Err(error) if crate::object_storage::is_fenced_error(&error) => {
                context.shutdown.mark_fenced();
                return ShutdownOutcome::Fenced;
            }
            Err(error) => {
                attempt += 1;
                tracing::error!(%error, attempt, "force-flush attempt failed; retrying");

                if started.elapsed() >= warn_after {
                    println!(
                        "\n[signy] SHUTDOWN BLOCKED: force-flush to durable storage is still \
failing after {:?} ({attempt} attempts). Acknowledged data is NOT yet durable and this process \
will keep retrying — it will NOT exit on its own. It will exit automatically the moment storage \
recovers. To force termination now, type `exit` then Enter, or send SIGKILL (the WAL is preserved, \
so a restart on this same disk recovers without loss; do not discard the disk).",
                        started.elapsed()
                    );
                    if abort_rx.is_none()
                        && let Some(make_abort) = make_abort.take()
                    {
                        abort_rx = Some(make_abort());
                    }
                }

                if let Some(rx) = abort_rx.as_mut() {
                    tokio::select! {
                        _ = tokio::time::sleep(backoff) => {}
                        confirmation = rx.recv() => {
                            if confirmation.is_some() {
                                tracing::error!(
                                    "operator confirmed forced termination before durability; \
                    restart on this disk will recover unflushed data from the WAL"
                                );
                                return ShutdownOutcome::AbortedByOperator;
                            }
                        }
                    }
                } else {
                    tokio::time::sleep(backoff).await;
                }
                backoff = (backoff * 2).min(FORCE_FLUSH_MAX_BACKOFF);
            }
        }
    }
}

/// Read stdin on a blocking thread and signal once the operator types `exit`.
/// Two ways for an operator to abandon a force-flush that will not finish.
///
/// `SIGUSR1` is the one that matters. Under systemd or in a container stdin is
/// not a TTY, so the typed command was unreachable exactly where this decision
/// gets made — and the alternative there is the orchestrator's grace period
/// expiring into a `SIGKILL`, which is the same abort with no exit code, no log
/// line, and no chance to record that the data is still in the WAL on that
/// disk. A signal reaches the process the same way `SIGTERM` already does:
/// `kill -USR1 1`.
///
/// stdin is kept because it is the convenient one when a person is watching a
/// terminal, and neither costs anything when unused.
fn spawn_abort_watcher() -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel(1);

    #[cfg(unix)]
    {
        let signal_tx = tx.clone();
        tokio::spawn(async move {
            use tokio::signal::unix::{SignalKind, signal};
            let mut abort = match signal(SignalKind::user_defined1()) {
                Ok(abort) => abort,
                Err(error) => {
                    tracing::error!(%error, "failed to install the SIGUSR1 abort handler");
                    return;
                }
            };
            if abort.recv().await.is_some() {
                tracing::warn!("SIGUSR1 received; abandoning the force-flush");
                // The channel holds one slot and one abort ends the loop, so a
                // full channel means an abort is already in flight.
                let _ = signal_tx.try_send(());
            }
        });
    }

    std::thread::spawn(move || {
        use std::io::BufRead;
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    let command = line.trim().to_ascii_lowercase();
                    if command == "exit" || command == "quit" {
                        let _ = tx.blocking_send(());
                        return;
                    }
                }
                Err(_) => return,
            }
        }
        // stdin closed, which is the normal case in a container. The signal
        // path above is the one that works there.
        tracing::info!("stdin is unavailable; abandon a stuck force-flush with SIGUSR1");
    });
    rx
}

#[cfg(all(test, unix))]
mod abort_signal_tests {
    use super::*;

    /// The abort that works where the decision is actually made.
    ///
    /// Under systemd or in a container stdin is not a TTY, so the typed
    /// command was unreachable exactly there, and the only remaining way out
    /// was the orchestrator's grace period expiring into a `SIGKILL` — the
    /// same abort with no exit code and no log line.
    #[tokio::test]
    async fn sigusr1_abandons_a_force_flush_that_cannot_finish() {
        use tokio::signal::unix::{SignalKind, signal};

        // Installed here first so the process disposition is already a handler
        // when the signal is raised. Without it a scheduling race between
        // spawning the watcher and raising the signal would kill the test
        // binary, since SIGUSR1 terminates by default.
        let mut guard = signal(SignalKind::user_defined1()).expect("install SIGUSR1");

        let mut abort = spawn_abort_watcher();
        // Give the watcher's task a chance to register its own stream.
        tokio::task::yield_now().await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let status = std::process::Command::new("kill")
            .args(["-USR1", &std::process::id().to_string()])
            .status()
            .expect("raise SIGUSR1");
        assert!(status.success());

        let received = tokio::time::timeout(Duration::from_secs(5), abort.recv())
            .await
            .expect("the abort must arrive rather than hang");
        assert!(received.is_some(), "the watcher forwards the signal");

        // Drain the test's own stream so the signal does not leak into a later
        // test that installs one.
        let _ = tokio::time::timeout(Duration::from_millis(50), guard.recv()).await;
    }
}
