use std::sync::Arc;

use collecty::config::Config;
use collecty::host_metrics::HostMetrics;
use collecty::journal::{JournalRuntime, JournalSource};
use collecty::observe::{Reporter, SourceStats};
use collecty::queue::{Queue, Spool};
use collecty::receive::{self, Intake};
use collecty::send::{HttpTransport, Sender};
use collecty::signal::Signal;
use tokio::sync::watch;

/// The allocator, and the tagging that may sit over it.
///
/// Four builds out of two independent features, which is the point: an
/// attribution run and the shipped build can be the same memory system.
/// signy's instrument is hardwired over the system allocator, and the day it
/// most wanted attribution it had to give it up because the instrumented build
/// could not hold the rate the shipped one held.
#[cfg(all(feature = "memprof", feature = "mimalloc"))]
#[global_allocator]
static ALLOCATOR: collecty::memprof::TaggedAllocator<mimalloc::MiMalloc> =
    collecty::memprof::TaggedAllocator::new(mimalloc::MiMalloc);

#[cfg(all(feature = "memprof", not(feature = "mimalloc")))]
#[global_allocator]
static ALLOCATOR: collecty::memprof::TaggedAllocator<std::alloc::System> =
    collecty::memprof::TaggedAllocator::new(std::alloc::System);

#[cfg(all(not(feature = "memprof"), feature = "mimalloc"))]
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Two, and not one per core.
///
/// Nothing on this runtime is CPU-heavy: compression and every write to the
/// queue belong to the spool thread, and what is left is accepting
/// connections, reading bodies and shipping segments. A worker per core was
/// therefore a thread per core doing almost nothing, and under glibc a thread
/// that allocates is given an arena of its own -- which `docs/MEMORY.md`
/// measured at three quarters of the footprint before the spool thread took
/// the appends off the blocking pool.
///
/// Not one, yet. The sender still calls `Queue::seal_if_due` on this runtime
/// and that closes a segment and `fsync`s it, so a single worker would stop
/// accepting, polling and timing out for as long as the disk took. One worker
/// is what this should become once no blocking work is left here.
#[tokio::main(worker_threads = 2)]
async fn main() {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("collecty: {error}");
            std::process::exit(2);
        }
    };
    init_tracing(config.log_json);

    if let Err(error) = run(config).await {
        tracing::error!(%error, "collecty stopped");
        std::process::exit(1);
    }
}

fn init_tracing(json: bool) {
    let filter = tracing_subscriber::EnvFilter::try_from_env("COLLECTY_LOG")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    if json {
        builder.json().init();
    } else {
        builder.init();
    }
}

async fn run(config: Config) -> Result<(), String> {
    let dir = config.queue_dir();
    let queue = Arc::new(
        Queue::open(&dir, config.queue, config.zstd_level)
            .map_err(|error| format!("cannot open the queue at {dir:?}: {error}"))?,
    );

    let spool = Spool::new(queue.clone());

    let listener =
        receive::bind(config.listen_addr).map_err(|error| format!("cannot serve OTLP: {error}"))?;
    let intake = Intake::new(
        spool.clone(),
        config.max_request_bytes,
        config.max_inflight_bytes,
    );

    {
        let gauge = intake.clone();
        collecty::memprof::start_sampler(collecty::memprof::InflightGauge::new(move || {
            gauge.inflight_occupied()
        }));
    }

    let (shutdown, watcher) = watch::channel(false);
    let transport = Arc::new(HttpTransport::with_access(
        config.signy_url.clone(),
        config.send_timeout,
        config.signy_access_client_id.clone(),
        config.signy_access_client_secret.clone(),
    ));

    let sender = Sender::new(queue.clone(), spool.clone(), transport, config.sender);
    let source_stats = Arc::new(SourceStats::default());
    let reporter = Reporter::new(
        queue.clone(),
        sender.stats(),
        spool.clone(),
        config.generated_telemetry_tenant.clone(),
        source_stats.clone(),
    );
    let host_metrics = match config.host_metrics_interval {
        Some(_) => {
            let tenant_id = config.generated_telemetry_tenant.clone().ok_or_else(|| {
                "COLLECTY_GENERATED_TELEMETRY_TENANT_ID is required for host metrics".to_string()
            })?;
            Some(
                HostMetrics::new(config.host_metrics_root.clone(), tenant_id)
                    .map_err(|error| format!("cannot initialize host metrics: {error}"))?,
            )
        }
        None => None,
    };
    let journal = match config.journal.clone() {
        Some(journal_config) => {
            let tenant_id = config.generated_telemetry_tenant.clone().ok_or_else(|| {
                "COLLECTY_GENERATED_TELEMETRY_TENANT_ID is required for journald".to_string()
            })?;
            Some(
                JournalSource::new(
                    journal_config,
                    config.host_metrics_root.clone(),
                    config.data_dir.clone(),
                    tenant_id,
                    queue.sender_id(),
                    JournalRuntime {
                        intake: intake.clone(),
                        spool: spool.clone(),
                        source_stats: source_stats.clone(),
                    },
                )
                .map_err(|error| format!("cannot initialize journald: {error}"))?,
            )
        }
        None => None,
    };
    let sending = {
        let watcher = watcher.clone();
        tokio::spawn(async move { sender.run(watcher).await })
    };

    let reporting = tokio::spawn(report_loop(
        reporter,
        intake.clone(),
        config.report_interval,
        watcher.clone(),
    ));
    let host_reporting = config.host_metrics_interval.map(|interval| {
        let source = host_metrics.expect("host metrics source exists when interval is set");
        let intake = intake.clone();
        let watcher = watcher.clone();
        tokio::spawn(host_metrics_loop(
            source,
            intake,
            interval,
            watcher,
            source_stats.clone(),
        ))
    });
    let journal_reporting = journal.map(|source| {
        let watcher = watcher.clone();
        tokio::spawn(source.run(watcher))
    });

    tracing::info!(
        listen = %config.listen_addr,
        data_dir = %config.data_dir.display(),
        signy = %config.signy_url,
        queue_max_bytes = config.queue.max_bytes,
        "collecty is accepting OTLP"
    );

    let stopping = shutdown.clone();
    let served = receive::serve(intake, listener, async move {
        wait_for_a_stop_signal().await;
        tracing::info!("collecty is shutting down");
        let _ = stopping.send(true);
    })
    .await;

    let _ = shutdown.send(true);
    let _ = sending.await;
    let _ = reporting.await;
    if let Some(host_reporting) = host_reporting {
        let _ = host_reporting.await;
    }
    if let Some(journal_reporting) = journal_reporting {
        let _ = journal_reporting.await;
    }

    // The only `fsync` the queue has is the one that closes a segment, so
    // leaving without closing the open one would leave its records to be
    // recovered rather than simply read.
    //
    // Through the spool like everything else, and awaited: the reply arrives
    // after the `fsync` has returned, so the records are on the device before
    // this function does. Nothing joins the thread afterwards because nothing
    // needs to -- durability is the reply, not the thread ending.
    match spool.seal().await {
        Ok(Err(error)) => {
            tracing::error!(%error, "the open segment could not be closed on the way out")
        }
        Err(gone) => tracing::error!(%gone, "the open segment could not be closed on the way out"),
        Ok(Ok(())) => {}
    }
    served.map_err(|error| format!("the OTLP listener stopped: {error}"))
}

async fn host_metrics_loop(
    source: HostMetrics,
    intake: Arc<Intake>,
    interval: std::time::Duration,
    mut shutdown: watch::Receiver<bool>,
    source_stats: Arc<SourceStats>,
) {
    let mut ticker = tokio::time::interval(interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = shutdown.changed() => return,
        }
        let export = match source.collect() {
            Ok(export) => export,
            Err(error) => {
                source_stats
                    .host_metrics_errors
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                tracing::warn!(%error, host = source.host_name(), "collecty could not read host metrics");
                continue;
            }
        };
        if let Err(refusal) = intake
            .accept(Signal::Metrics, vec![bytes::Bytes::from(export)])
            .await
        {
            source_stats
                .host_metrics_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            tracing::warn!(%refusal, "collecty could not queue host metrics");
        } else {
            source_stats
                .host_metrics_exports
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

async fn wait_for_a_stop_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate = match signal(SignalKind::terminate()) {
        Ok(stream) => stream,
        Err(error) => {
            tracing::error!(%error, "cannot listen for SIGTERM");
            return;
        }
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate.recv() => {}
    }
}

async fn report_loop(
    reporter: Reporter,
    intake: Arc<Intake>,
    interval: std::time::Duration,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut ticker = tokio::time::interval(interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = shutdown.changed() => return,
        }
        let observed = reporter.observe();
        reporter.log(&observed);
        let Some(export) = reporter.export(&observed) else {
            continue;
        };
        if let Err(refusal) = intake
            .accept(Signal::Metrics, vec![bytes::Bytes::from(export)])
            .await
        {
            tracing::warn!(%refusal, "collecty could not queue its own metrics");
        }
    }
}
