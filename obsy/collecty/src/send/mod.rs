#[cfg(test)]
mod tests;
mod transport;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bytes::Bytes;
use tokio::sync::watch;

use crate::queue::{Queue, SealedSegment, SenderId, Spool};
use crate::signal::Signal;

pub use transport::HttpTransport;

pub type DeliverFuture<'a> = Pin<Box<dyn Future<Output = Outcome> + Send + 'a>>;

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Taken, and the highest segment signy says it now holds whole.
    ///
    /// Normally the segment that was sent. It can be higher — signy answered
    /// an earlier attempt collecty never heard — and then everything up to it
    /// can go at once.
    Accepted(u64),
    Retry(String),
    Refused(String),
}

/// One segment on its way to signy.
///
/// The body is the segment file, byte for byte: one zstd stream over every
/// record the segment took. The sender, the signal and the segment number are
/// what let signy skip what it already stored: each signal is a stream of its
/// own, numbered from one, and a segment is sent from its first record every
/// time, so signy counts as it reads and knows exactly which records it has
/// seen before.
pub struct Shipment {
    pub body: Bytes,
    pub sender: SenderId,
    pub signal: Signal,
    pub segment: u64,
}

pub trait Transport: Send + Sync + 'static {
    fn deliver<'a>(&'a self, shipment: Shipment) -> DeliverFuture<'a>;
}

#[derive(Clone, Copy, Debug)]
pub struct SenderConfig {
    pub retry_initial: Duration,
    pub retry_max: Duration,
}

impl Default for SenderConfig {
    fn default() -> Self {
        Self {
            retry_initial: Duration::from_millis(100),
            retry_max: Duration::from_secs(30),
        }
    }
}

/// Segments and bytes, never records.
///
/// What a segment holds is inside its compression, and reading it back would
/// mean decompressing every segment on the way out to learn a number nobody
/// acts on. `collecty_records_appended_total` is where the record count lives,
/// counted where it is free.
#[derive(Default, Debug)]
pub struct SenderStats {
    pub sent_segments: AtomicU64,
    pub sent_bytes: AtomicU64,
    pub refused_segments: AtomicU64,
    pub refused_bytes: AtomicU64,
    pub retries: AtomicU64,
}

pub struct Sender<T> {
    queue: Arc<Queue>,
    /// Every file this sender touches, it touches through here. Closing a
    /// segment, opening one and unlinking one are all syscalls that return
    /// when the device says so, and none of them belongs on a runtime that is
    /// also accepting connections.
    spool: Spool,
    transport: Arc<T>,
    config: SenderConfig,
    stats: Arc<SenderStats>,
}

impl<T: Transport> Sender<T> {
    pub fn new(
        queue: Arc<Queue>,
        spool: Spool,
        transport: Arc<T>,
        config: SenderConfig,
    ) -> Sender<T> {
        Sender {
            queue,
            spool,
            transport,
            config,
            stats: Arc::new(SenderStats::default()),
        }
    }

    pub fn stats(&self) -> Arc<SenderStats> {
        self.stats.clone()
    }

    pub async fn run(&self, mut shutdown: watch::Receiver<bool>) {
        while !*shutdown.borrow() {
            // Before looking for work: a quiet host would otherwise hold its
            // records until a segment filled.
            match self.spool.seal_if_due().await {
                Ok(Err(error)) => {
                    tracing::error!(%error, "an open segment could not be closed");
                }
                Err(gone) => {
                    tracing::error!(%gone, "an open segment could not be closed");
                    return;
                }
                Ok(Ok(())) => {}
            }

            let Some((signal, seq)) = self.queue.oldest_sealed() else {
                tokio::select! {
                    _ = self.queue.wait_for_sealed() => {}
                    _ = tokio::time::sleep(self.config.retry_initial) => {}
                    _ = shutdown.changed() => {}
                }
                continue;
            };

            let segment = match self.spool.read_segment(signal, seq).await {
                Ok(Ok(segment)) => segment,
                Ok(Err(error)) => {
                    tracing::error!(
                        %error,
                        signal = signal.as_str(),
                        segment = seq,
                        "the segment could not be read"
                    );
                    tokio::time::sleep(self.config.retry_initial).await;
                    continue;
                }
                Err(gone) => {
                    tracing::error!(%gone, "the segment could not be read");
                    return;
                }
            };

            self.deliver(segment, &mut shutdown).await;
        }
    }

    pub(crate) async fn deliver(
        &self,
        segment: SealedSegment,
        shutdown: &mut watch::Receiver<bool>,
    ) {
        let SealedSegment { signal, seq, body } = segment;
        let bytes = body.len() as u64;

        let mut backoff = self.config.retry_initial;
        loop {
            if *shutdown.borrow() {
                return;
            }
            let shipment = Shipment {
                body: body.clone(),
                sender: self.queue.sender_id(),
                signal,
                segment: seq,
            };
            match self.transport.deliver(shipment).await {
                Outcome::Accepted(stored) => {
                    self.stats.sent_segments.fetch_add(1, Ordering::Relaxed);
                    self.stats.sent_bytes.fetch_add(bytes, Ordering::Relaxed);
                    self.commit(signal, stored.max(seq)).await;
                    return;
                }
                // Permanent for this segment's shape rather than its content:
                // signy drops a record it cannot decode on its own side and
                // answers 200, so what reaches here is a stream or a framing
                // that will fail the same way however many times it is sent.
                Outcome::Refused(reason) => {
                    tracing::error!(
                        reason,
                        signal = signal.as_str(),
                        segment = seq,
                        bytes,
                        "dropping a segment signy refuses to accept"
                    );
                    self.stats.refused_segments.fetch_add(1, Ordering::Relaxed);
                    self.stats.refused_bytes.fetch_add(bytes, Ordering::Relaxed);
                    self.commit(signal, seq).await;
                    return;
                }
                Outcome::Retry(reason) => {
                    self.stats.retries.fetch_add(1, Ordering::Relaxed);
                    tracing::warn!(
                        reason,
                        signal = signal.as_str(),
                        segment = seq,
                        backoff_ms = backoff.as_millis() as u64,
                        "signy did not take the segment"
                    );
                    tokio::select! {
                        _ = tokio::time::sleep(jittered(backoff)) => {}
                        _ = shutdown.changed() => {}
                    }
                    backoff = (backoff * 2).min(self.config.retry_max);
                }
            }
        }
    }

    async fn commit(&self, signal: Signal, acked: u64) {
        match self.spool.commit(signal, acked).await {
            Ok(Err(error)) => tracing::error!(%error, "the cursor could not be advanced"),
            Err(gone) => tracing::error!(%gone, "the cursor could not be advanced"),
            Ok(Ok(())) => {}
        }
    }
}

fn jittered(backoff: Duration) -> Duration {
    let spread = backoff.as_millis() as u64 / 4;
    if spread == 0 {
        return backoff;
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.subsec_nanos() as u64)
        .unwrap_or(0);
    backoff + Duration::from_millis(nanos % spread)
}
