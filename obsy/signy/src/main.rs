use std::sync::Arc;

use signy::config::Config;

#[cfg(feature = "memprof")]
#[global_allocator]
static ALLOCATOR: signy::memprof::ProfilingAllocator = signy::memprof::ProfilingAllocator;

/// mimalloc. glibc was measured out first (todo.md, 2026-08-09): with every
/// gauged resident flat, glibc-retained free crept until the 2 GiB kill — 4.05
/// hours even with the fixed thresholds, the arena cap and a `malloc_trim`
/// timer, because a free chunk on a page it shares with a live one can never
/// be returned. What replaced it has to return freed pages on its own, which
/// is the property the competitors get from the Go runtime for free; jemalloc
/// held that role from 2026-08-09 and mimalloc takes it on 2026-08-31.
///
/// The memprof build keeps its instrumented wrapper over glibc — its job is
/// live-byte attribution, and the glibc tuning stays active there, which is
/// why an arena split and an absolute footprint must not be read off the same
/// run ([`docs/MEMORY_ATTRIBUTION.md`](../docs/MEMORY_ATTRIBUTION.md)).
#[cfg(all(not(feature = "memprof"), not(feature = "jemalloc")))]
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// The alternative, built with `--features jemalloc`. It is here because
/// mimalloc's defaults were never put through a long run, jemalloc's were, and
/// which one this engine should ship is a measurement rather than a
/// preference. It also answers a question mimalloc cannot: `stats.allocated`
/// is live bytes, and a release-built mimalloc compiles that counter out, so
/// the shipped binary can say what it holds but not what of that is in use.
#[cfg(all(not(feature = "memprof"), feature = "jemalloc"))]
#[global_allocator]
static ALLOCATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

// mimalloc runs at its own defaults. The jemalloc sweep it replaces found no
// setting that beat the defaults (a five-way 600 s sweep on the soak rig,
// todo.md 2026-08-09), and nothing here has yet measured mimalloc's own knobs,
// so the defaults are what this build claims — not what it has proven best.
// An operator A/B reaches them through `MIMALLOC_*` environment variables;
// `purge_delay` is the one that decides how fast freed pages go back.

fn main() {
    // Before the runtime exists: M_ARENA_MAX only bounds arenas not yet
    // created, and tokio's workers each create one on first contention.
    // Under mimalloc this is a no-op and logs `applied = false`.
    let malloc_tuned = signy::malloc_tuning::apply_from_env();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build the tokio runtime")
        .block_on(run(malloc_tuned));
}

async fn run(malloc_tuned: bool) {
    let config = Arc::new(
        Config::from_env().unwrap_or_else(|error| panic!("invalid configuration: {error}")),
    );
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("signy=info,warn"));
    match config.log_format {
        signy::config::LogFormat::Text => tracing_subscriber::fmt().with_env_filter(filter).init(),
        signy::config::LogFormat::Json => tracing_subscriber::fmt()
            .json()
            .with_current_span(false)
            .with_env_filter(filter)
            .init(),
    }
    tracing::info!(
        applied = malloc_tuned,
        "glibc malloc tuning (SIGNY_MALLOC_TUNING=off to disable)"
    );
    if config.capacity_probe {
        tracing::warn!(
            capacity_probe = true,
            "DANGEROUS CAPACITY PROBE ENABLED: metric/cardinality and ingest backpressure limits are bypassed; use only in an isolated cgroup test"
        );
    } else {
        tracing::info!(capacity_probe = false, "capacity probe disabled");
    }
    config.log_memory_budget();
    signy::run(config).await;
}
