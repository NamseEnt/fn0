//! Tier B load gate: an in-process object store that adds deterministic,
//! seeded latency, jitter, and error injection around any real backend. It runs
//! inside the production binary over the real HTTP/gRPC paths, needs no external
//! process, and replays identically for a fixed seed. Error injection is applied
//! to write operations only, so it exercises the flush/publish retry and
//! backpressure paths without breaking the restore reads a query depends on.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use futures_util::stream::BoxStream;
use object_store::path::Path;
use object_store::{
    GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta, ObjectStore,
    PutMultipartOptions, PutOptions, PutPayload, PutResult,
};

/// Load knobs read from the process environment. `None` means no knob was set
/// and `from_url` must leave the constructed store untouched (zero overhead).
#[derive(Clone, Copy, Debug)]
pub(crate) struct FaultConfig {
    write_latency: Duration,
    read_latency: Duration,
    jitter: Duration,
    error_rate: f64,
    seed: u64,
}

impl FaultConfig {
    /// Reads the `SIGNY_OBJECT_STORE_*` load knobs. Returns `None` when
    /// none of the latency/error knobs are present, so the caller can skip the
    /// wrapper entirely.
    pub(crate) fn from_env() -> Result<Option<Self>, String> {
        Self::from_vars(std::env::vars().collect())
    }

    fn from_vars(vars: Vec<(String, String)>) -> Result<Option<Self>, String> {
        let lookup = |name: &str| {
            vars.iter()
                .find(|(key, _)| key == name)
                .map(|(_, v)| v.clone())
        };
        let write_ms = parse_u64(
            "SIGNY_OBJECT_STORE_LATENCY_MS",
            lookup("SIGNY_OBJECT_STORE_LATENCY_MS"),
        )?;
        let read_ms = parse_u64(
            "SIGNY_OBJECT_STORE_READ_LATENCY_MS",
            lookup("SIGNY_OBJECT_STORE_READ_LATENCY_MS"),
        )?;
        let jitter_ms = parse_u64(
            "SIGNY_OBJECT_STORE_LATENCY_JITTER_MS",
            lookup("SIGNY_OBJECT_STORE_LATENCY_JITTER_MS"),
        )?;
        let error_rate = parse_f64(
            "SIGNY_OBJECT_STORE_ERROR_RATE",
            lookup("SIGNY_OBJECT_STORE_ERROR_RATE"),
        )?;
        let seed = parse_u64(
            "SIGNY_OBJECT_STORE_FAULT_SEED",
            lookup("SIGNY_OBJECT_STORE_FAULT_SEED"),
        )?;

        if write_ms.is_none() && read_ms.is_none() && jitter_ms.is_none() && error_rate.is_none() {
            return Ok(None);
        }
        let error_rate = error_rate.unwrap_or(0.0);
        if !(0.0..=1.0).contains(&error_rate) {
            return Err("SIGNY_OBJECT_STORE_ERROR_RATE must be between 0.0 and 1.0".to_string());
        }
        let write_latency = Duration::from_millis(write_ms.unwrap_or(0));
        Ok(Some(Self {
            write_latency,
            // Restore latency shapes reads independently; it defaults to the
            // write latency when the read knob is absent.
            read_latency: read_ms.map(Duration::from_millis).unwrap_or(write_latency),
            jitter: Duration::from_millis(jitter_ms.unwrap_or(0)),
            error_rate,
            seed: seed.unwrap_or(0x5eed_2026),
        }))
    }
}

#[cfg(test)]
impl FaultConfig {
    pub(crate) fn for_test(
        write_ms: u64,
        read_ms: u64,
        jitter_ms: u64,
        error_rate: f64,
        seed: u64,
    ) -> Self {
        Self {
            write_latency: Duration::from_millis(write_ms),
            read_latency: Duration::from_millis(read_ms),
            jitter: Duration::from_millis(jitter_ms),
            error_rate,
            seed,
        }
    }
}

fn parse_u64(name: &str, value: Option<String>) -> Result<Option<u64>, String> {
    match value {
        Some(raw) => raw
            .trim()
            .parse::<u64>()
            .map(Some)
            .map_err(|error| format!("invalid {name} {raw:?}: {error}")),
        None => Ok(None),
    }
}

fn parse_f64(name: &str, value: Option<String>) -> Result<Option<f64>, String> {
    match value {
        Some(raw) => raw
            .trim()
            .parse::<f64>()
            .map(Some)
            .map_err(|error| format!("invalid {name} {raw:?}: {error}")),
        None => Ok(None),
    }
}

/// splitmix64 finalizer: turns a monotonic counter into a well-distributed draw
/// so jitter and error decisions are deterministic yet not correlated across
/// consecutive operations.
fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = value;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Decrements the in-flight count however the read leaves, including a
/// cancelled future, so the peak cannot drift upward across a restore that
/// abandons its remaining downloads on the first error.
struct InFlightGuard<'a> {
    counter: &'a AtomicU64,
}

impl Drop for InFlightGuard<'_> {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Wraps `inner` with seeded latency and write-error injection.
#[derive(Debug)]
pub(crate) struct LatencyFaultStore {
    inner: Arc<dyn ObjectStore>,
    config: FaultConfig,
    operation_counter: AtomicU64,
    /// Reads currently inside `shape_read`, and the high-water mark of that.
    ///
    /// Concurrency measured where it happens. Inferring it from elapsed time
    /// works only if nothing else can stretch the clock, and under virtual time
    /// a pending blocking task is enough to make that untrue.
    reads_in_flight: AtomicU64,
    peak_reads_in_flight: AtomicU64,
    /// Every read this store has served. Lets a test count the work a startup
    /// phase does without running one at a scale where the cost is visible.
    total_reads: AtomicU64,
}

impl LatencyFaultStore {
    pub(crate) fn new(inner: Arc<dyn ObjectStore>, config: FaultConfig) -> Self {
        Self {
            inner,
            config,
            operation_counter: AtomicU64::new(0),
            reads_in_flight: AtomicU64::new(0),
            peak_reads_in_flight: AtomicU64::new(0),
            total_reads: AtomicU64::new(0),
        }
    }

    /// The most reads this store ever had open at once.
    #[cfg(test)]
    pub(crate) fn peak_reads_in_flight(&self) -> u64 {
        self.peak_reads_in_flight.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn total_reads(&self) -> u64 {
        self.total_reads.load(Ordering::Relaxed)
    }

    /// One deterministic draw keyed by the seed and the operation index. The
    /// same seed replays the same sequence of jitter and error decisions.
    fn next_draw(&self) -> u64 {
        let index = self.operation_counter.fetch_add(1, Ordering::Relaxed);
        splitmix64(self.config.seed ^ index.wrapping_mul(0x9E3779B97F4A7C15))
    }

    fn jitter_from(&self, draw: u64) -> Duration {
        if self.config.jitter.is_zero() {
            return Duration::ZERO;
        }
        // Top 53 bits give a fraction in [0, 1).
        let fraction = (draw >> 11) as f64 / (1u64 << 53) as f64;
        self.config.jitter.mul_f64(fraction)
    }

    fn error_from(&self, draw: u64) -> bool {
        if self.config.error_rate <= 0.0 {
            return false;
        }
        // Low 32 bits give an independent fraction in [0, 1).
        let fraction = (draw & 0xFFFF_FFFF) as f64 / (u32::MAX as f64 + 1.0);
        fraction < self.config.error_rate
    }

    async fn sleep(&self, base: Duration, draw: u64) {
        let total = base.saturating_add(self.jitter_from(draw));
        if !total.is_zero() {
            tokio::time::sleep(total).await;
        }
    }

    /// Shapes a write: applies latency, then optionally fails with a retriable
    /// error so the engine's flush/publish retry path is exercised. The latency
    /// is applied even when the operation is about to fail, matching a real
    /// backend that spends time before rejecting a request.
    async fn shape_write(&self) -> object_store::Result<()> {
        let draw = self.next_draw();
        self.sleep(self.config.write_latency, draw).await;
        if self.error_from(draw) {
            return Err(object_store::Error::Generic {
                store: "fault-injection",
                source: "injected object-store write failure".into(),
            });
        }
        Ok(())
    }

    async fn shape_read(&self) {
        self.total_reads.fetch_add(1, Ordering::Relaxed);
        let in_flight = self.reads_in_flight.fetch_add(1, Ordering::Relaxed) + 1;
        self.peak_reads_in_flight
            .fetch_max(in_flight, Ordering::Relaxed);
        let _guard = InFlightGuard {
            counter: &self.reads_in_flight,
        };
        let draw = self.next_draw();
        self.sleep(self.config.read_latency, draw).await;
    }
}

impl std::fmt::Display for LatencyFaultStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "LatencyFaultStore({})", self.inner)
    }
}

#[async_trait]
impl ObjectStore for LatencyFaultStore {
    async fn put_opts(
        &self,
        location: &Path,
        payload: PutPayload,
        opts: PutOptions,
    ) -> object_store::Result<PutResult> {
        self.shape_write().await?;
        self.inner.put_opts(location, payload, opts).await
    }

    async fn put_multipart_opts(
        &self,
        location: &Path,
        opts: PutMultipartOptions,
    ) -> object_store::Result<Box<dyn MultipartUpload>> {
        // Errors are injected on the single-shot write path; the multipart
        // handle only carries latency so large-part uploads still see delay.
        let draw = self.next_draw();
        self.sleep(self.config.write_latency, draw).await;
        self.inner.put_multipart_opts(location, opts).await
    }

    async fn get_opts(
        &self,
        location: &Path,
        options: GetOptions,
    ) -> object_store::Result<GetResult> {
        self.shape_read().await;
        self.inner.get_opts(location, options).await
    }

    async fn delete(&self, location: &Path) -> object_store::Result<()> {
        self.shape_write().await?;
        self.inner.delete(location).await
    }

    fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, object_store::Result<ObjectMeta>> {
        // Listing is a control-plane read used by GC and recovery, not the hot
        // path; it is delegated without added latency.
        self.inner.list(prefix)
    }

    fn list_with_offset(
        &self,
        prefix: Option<&Path>,
        offset: &Path,
    ) -> BoxStream<'static, object_store::Result<ObjectMeta>> {
        self.inner.list_with_offset(prefix, offset)
    }

    async fn list_with_delimiter(&self, prefix: Option<&Path>) -> object_store::Result<ListResult> {
        self.shape_read().await;
        self.inner.list_with_delimiter(prefix).await
    }

    async fn copy(&self, from: &Path, to: &Path) -> object_store::Result<()> {
        self.shape_write().await?;
        self.inner.copy(from, to).await
    }

    async fn copy_if_not_exists(&self, from: &Path, to: &Path) -> object_store::Result<()> {
        self.shape_write().await?;
        self.inner.copy_if_not_exists(from, to).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_vars_returns_none_when_no_knobs_set() {
        assert!(FaultConfig::from_vars(vec![]).unwrap().is_none());
    }

    #[test]
    fn read_latency_defaults_to_write_latency() {
        let config = FaultConfig::from_vars(vec![(
            "SIGNY_OBJECT_STORE_LATENCY_MS".to_string(),
            "20".to_string(),
        )])
        .unwrap()
        .unwrap();
        assert_eq!(config.write_latency, Duration::from_millis(20));
        assert_eq!(config.read_latency, Duration::from_millis(20));
    }

    #[test]
    fn rejects_out_of_range_error_rate() {
        let error = FaultConfig::from_vars(vec![(
            "SIGNY_OBJECT_STORE_ERROR_RATE".to_string(),
            "1.5".to_string(),
        )])
        .unwrap_err();
        assert!(error.contains("ERROR_RATE"));
    }

    #[test]
    fn draws_replay_identically_for_a_fixed_seed() {
        let config = FaultConfig {
            write_latency: Duration::ZERO,
            read_latency: Duration::ZERO,
            jitter: Duration::from_millis(10),
            error_rate: 0.5,
            seed: 42,
        };
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        let first = LatencyFaultStore::new(inner.clone(), config);
        let second = LatencyFaultStore::new(inner, config);
        let first_draws: Vec<_> = (0..16).map(|_| first.next_draw()).collect();
        let second_draws: Vec<_> = (0..16).map(|_| second.next_draw()).collect();
        assert_eq!(first_draws, second_draws);
        // Error injection is a deterministic function of the draw.
        let errors: Vec<_> = first_draws.iter().map(|&d| second.error_from(d)).collect();
        assert!(errors.iter().any(|&e| e));
        assert!(errors.iter().any(|&e| !e));
    }

    #[tokio::test]
    async fn injects_write_errors_at_high_rate_but_never_on_reads() {
        let config = FaultConfig {
            write_latency: Duration::ZERO,
            read_latency: Duration::ZERO,
            jitter: Duration::ZERO,
            error_rate: 1.0,
            seed: 7,
        };
        let store = LatencyFaultStore::new(Arc::new(object_store::memory::InMemory::new()), config);
        let location = Path::from("probe");
        let error = store
            .put_opts(
                &location,
                PutPayload::from_static(b"body"),
                PutOptions::default(),
            )
            .await
            .unwrap_err();
        assert!(matches!(error, object_store::Error::Generic { .. }));
        // A read of a missing key returns NotFound, never an injected error.
        let read = store.get(&location).await.unwrap_err();
        assert!(matches!(read, object_store::Error::NotFound { .. }));
    }
}
