//! Object-store collection that no writer owns: part objects no manifest
//! names any more, and catalog history no startup reads any more.
//!
//! It runs on its own cadence rather than inside retention, because retention
//! only reaches the store when it retires a part, and merges and compactions
//! leave orphans on every pass whether or not anything has expired.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tokio::sync::watch;
use tokio::time::interval;

use crate::config::Config;
use crate::metrics::RuntimeMetrics;
use crate::object_storage::RemoteCache;
use crate::shutdown::wait_for_drain;

pub async fn object_store_gc_loop(
    cache: Arc<RemoteCache>,
    config: Arc<Config>,
    metrics: Arc<RuntimeMetrics>,
    mut drain_rx: watch::Receiver<bool>,
) {
    let mut ticker = interval(config.orphan_gc_interval.max(Duration::from_secs(1)));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = wait_for_drain(&mut drain_rx) => return,
        }
        let (orphans, catalog) = object_store_gc_pass(&cache, &config, &metrics).await;
        if orphans.is_ok() && catalog.is_ok() {
            cache.record_remote_success();
            metrics
                .object_store_gc_success
                .fetch_add(1, Ordering::Relaxed);
        } else {
            cache.record_remote_failure();
            metrics
                .object_store_gc_errors
                .fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// One pass of each collector, reported separately.
///
/// Catalog history and part orphans share nothing but this loop. Sequencing
/// pruning behind collection meant a store too large to sweep inside one pass
/// also stopped the catalog from ever being pruned.
pub(crate) async fn object_store_gc_pass(
    cache: &RemoteCache,
    config: &Config,
    metrics: &RuntimeMetrics,
) -> (Result<(), String>, Result<(), String>) {
    let orphans = collect_orphans_once(cache, config, metrics).await;
    let catalog = prune_catalog_once(cache, config, metrics).await;
    (orphans, catalog)
}

async fn collect_orphans_once(
    cache: &RemoteCache,
    config: &Config,
    metrics: &RuntimeMetrics,
) -> Result<(), String> {
    let options = config.orphan_collection_options();
    // The pass bounds itself; this only catches a store that never answers.
    let ceiling = options.max_runtime * 2;
    let collection = match tokio::time::timeout(ceiling, cache.storage.collect_orphans(&options))
        .await
        .map_err(|_| "orphan collection stopped answering".to_string())
    {
        Ok(Ok(collection)) => collection,
        Ok(Err(error)) | Err(error) => {
            metrics
                .orphan_collect_errors
                .fetch_add(1, Ordering::Relaxed);
            tracing::error!(%error, "orphan collection failed");
            return Err(error);
        }
    };
    metrics
        .orphan_objects_removed
        .fetch_add(collection.deleted_objects as u64, Ordering::Relaxed);
    metrics
        .orphan_bytes_removed
        .fetch_add(collection.deleted_bytes, Ordering::Relaxed);
    metrics
        .orphan_delete_errors
        .fetch_add(collection.delete_errors as u64, Ordering::Relaxed);
    metrics
        .orphan_scan_cycles
        .fetch_add(collection.scan_cycles_completed as u64, Ordering::Relaxed);
    metrics
        .orphan_candidate_objects
        .store(collection.candidate_objects as u64, Ordering::Relaxed);
    metrics
        .orphan_candidate_bytes
        .store(collection.candidate_bytes, Ordering::Relaxed);
    metrics
        .orphan_ledger_entries
        .store(collection.ledger_entries as u64, Ordering::Relaxed);
    metrics
        .orphan_collect_success
        .fetch_add(1, Ordering::Relaxed);
    if collection.candidate_objects > 0 || collection.deleted_objects > 0 {
        tracing::info!(
            scanned_objects = collection.scanned_objects,
            scan_cycles_completed = collection.scan_cycles_completed,
            candidate_objects = collection.candidate_objects,
            candidate_bytes = collection.candidate_bytes,
            deleted_objects = collection.deleted_objects,
            deleted_bytes = collection.deleted_bytes,
            delete_errors = collection.delete_errors,
            dry_run = options.dry_run,
            "orphan collection pass completed"
        );
    }
    Ok(())
}

async fn prune_catalog_once(
    cache: &RemoteCache,
    config: &Config,
    metrics: &RuntimeMetrics,
) -> Result<(), String> {
    let Some(min_age) = config.catalog_prune_min_age else {
        return Ok(());
    };
    let pruned = match tokio::time::timeout(
        config.max_retention_runtime,
        cache.storage.prune_catalog(min_age),
    )
    .await
    .map_err(|_| "catalog pruning timed out".to_string())
    {
        Ok(Ok(pruned)) => pruned,
        Ok(Err(error)) | Err(error) => {
            metrics.catalog_prune_errors.fetch_add(1, Ordering::Relaxed);
            tracing::error!(%error, "catalog pruning failed");
            return Err(error);
        }
    };
    metrics
        .catalog_objects_pruned
        .fetch_add(pruned as u64, Ordering::Relaxed);
    metrics
        .catalog_prune_success
        .fetch_add(1, Ordering::Relaxed);
    if pruned > 0 {
        tracing::info!(pruned, "catalog pruning removed objects");
    }
    Ok(())
}
