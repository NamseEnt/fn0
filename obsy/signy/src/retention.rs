use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::watch;
use tokio::time::interval;

use crate::config::Config;
use crate::metrics::RuntimeMetrics;
use crate::object_storage::{ManifestPart, MetricManifestPart, RemoteCache, TraceManifestPart};
use crate::part;
use crate::part_registry::PartRegistry;
use crate::series_registry::SeriesRegistry;
use crate::shutdown::wait_for_drain;
use crate::tenant_policy::TenantPolicy;
use crate::trace_registry::TraceRegistry;

#[allow(clippy::too_many_arguments)]
pub async fn retention_loop(
    registry: Arc<PartRegistry>,
    trace_registry: Arc<TraceRegistry>,
    series_registry: Arc<SeriesRegistry>,
    remote_cache: Option<Arc<RemoteCache>>,
    config: Arc<Config>,
    tenant_policy: Arc<TenantPolicy>,
    journal: Arc<crate::journal::Journal>,
    metrics: Arc<RuntimeMetrics>,
    healthy: Arc<AtomicBool>,
    mut drain_rx: watch::Receiver<bool>,
) {
    healthy.store(true, Ordering::Release);
    let mut ticker = interval(config.retention_interval.max(Duration::from_secs(1)));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    // Nothing here fetches a policy: the control plane pushes them and this
    // loop only applies what is already in memory. signy never calls out.
    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = wait_for_drain(&mut drain_rx) => return,
        }
        metrics.unknown_tenants.store(
            unknown_tenant_count(
                &registry,
                &trace_registry,
                &series_registry,
                &journal,
                &tenant_policy,
            ) as u64,
            Ordering::Relaxed,
        );
        if let Err(error) = retention_once(
            &registry,
            &trace_registry,
            &series_registry,
            remote_cache.as_deref(),
            &config,
            &tenant_policy,
        )
        .await
        {
            metrics
                .retention_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            healthy.store(false, Ordering::Release);
            if let Some(cache) = remote_cache.as_deref() {
                cache.record_remote_failure();
            }
            tracing::error!(%error, "retention iteration failed");
        } else {
            metrics
                .retention_success
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            healthy.store(true, Ordering::Release);
        }
    }
}

/// Tenants holding data that the control plane has never mentioned.
///
/// *Absent* and `"infinite"` keep the same data, so a control plane that
/// silently dropped a tenant is only visible as this number rising. Computed on
/// the retention tick rather than per scrape: it walks every part's tenant
/// index, which is not work an unauthenticated endpoint should be able to ask
/// for at scrape frequency.
///
/// Both memtables are counted alongside the parts. A tenant that has just
/// started pushing owns no `meta.json` segment until its first flush, and that
/// is exactly the window in which the control plane is most likely to have
/// missed it.
fn unknown_tenant_count(
    registry: &PartRegistry,
    trace_registry: &TraceRegistry,
    series_registry: &SeriesRegistry,
    journal: &crate::journal::Journal,
    tenant_policy: &TenantPolicy,
) -> usize {
    let Some(policies) = tenant_policy.snapshot() else {
        return 0;
    };
    let mut unknown: std::collections::BTreeSet<crate::tenant::TenantId> =
        std::collections::BTreeSet::new();
    let mut note = |tenant: &crate::tenant::TenantId| {
        if policies.retention(tenant).is_none() {
            unknown.insert(tenant.clone());
        }
    };
    registry.visit_tenants(&mut note);
    trace_registry.visit_tenants(&mut note);
    series_registry.visit_tenants(&mut note);
    for tenant in journal.log_memtable().tenants() {
        note(&tenant);
    }
    for tenant in journal.trace_memtable().tenants() {
        note(&tenant);
    }
    for tenant in journal.series_memtable().tenants() {
        note(&tenant);
    }
    unknown.len()
}

async fn retention_once(
    registry: &PartRegistry,
    trace_registry: &TraceRegistry,
    series_registry: &SeriesRegistry,
    remote_cache: Option<&RemoteCache>,
    config: &Config,
    tenant_policy: &TenantPolicy,
) -> Result<(), String> {
    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock is before UNIX epoch: {error}"))?
        .as_nanos();
    retention_once_at(
        registry,
        trace_registry,
        series_registry,
        remote_cache,
        config,
        tenant_policy,
        now_ns,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
/// Retire expired log parts: first from the registry readers plan against,
/// then from the manifest a restore validates against.
///
/// The order is the contract. A reader plans under the lifecycle read guard
/// and restores under it again, so the registry is the side that is serialised
/// against it -- the write guard cannot be taken while a reader holds the read
/// guard, and a reader that re-plans afterwards no longer asks for the part.
/// Writing the manifest first left a window where a reader still planning from
/// the registry asked the restore for a part the manifest had already dropped,
/// and that fails the whole query rather than returning less.
///
/// Failing on the manifest write now leaves the part retired locally and still
/// in the manifest -- retention lagging rather than data vanishing under a
/// reader -- and a restart rebuilds the registry from the manifest, so the
/// part returns and the next tick retires it again.
async fn retire_log_parts(
    registry: &PartRegistry,
    cache: &RemoteCache,
    config: &Config,
    ids: &[String],
) -> Result<(), String> {
    unregister_log_parts(registry, ids).await;
    drop_log_parts_from_manifest(cache, config, ids).await
}

/// Stop planning reads against these parts, under the lifecycle write guard.
async fn unregister_log_parts(registry: &PartRegistry, ids: &[String]) {
    let _guard =
        crate::part_registry::PartRegistry::write_without_convoy(registry.operation_lock()).await;
    registry.unregister(ids);
}

/// Drop these parts from the manifest, outside the guard: this is a network
/// round trip and holding lifecycle writers for it is what the read/re-plan
/// protocol exists to avoid.
async fn drop_log_parts_from_manifest(
    cache: &RemoteCache,
    config: &Config,
    ids: &[String],
) -> Result<(), String> {
    match tokio::time::timeout(
        config.max_retention_runtime,
        cache.storage.publish(&[], ids),
    )
    .await
    {
        Ok(Ok(_)) => cache.record_remote_success(),
        Ok(Err(error)) => {
            cache.record_remote_failure();
            return Err(error);
        }
        Err(_) => {
            cache.record_remote_failure();
            return Err("object-store retention timed out".to_string());
        }
    }
    Ok(())
}

async fn retention_once_at(
    registry: &PartRegistry,
    trace_registry: &TraceRegistry,
    series_registry: &SeriesRegistry,
    remote_cache: Option<&RemoteCache>,
    config: &Config,
    tenant_policy: &TenantPolicy,
    now_ns: u128,
) -> Result<(), String> {
    // `None` means "delete nothing this tick" — the storeless test fixture.
    // A loaded policy always resolves, even before the control plane has
    // pushed anything: an empty map makes every tenant unknown, and unknown
    // already means keep.
    let Some(cutoffs) = tenant_policy.cutoffs_at(now_ns.min(i64::MAX as u128) as i64) else {
        return Ok(());
    };
    let batch_size = config.retention_batch_size.max(1);

    // Snapshot under a shared lifecycle guard. Remote manifest I/O happens
    // after releasing it, so retention cannot stop flush/merge/eviction for a
    // network round trip and queries can continue concurrently.
    let guard = registry.operation_lock().read_owned().await;
    let mut log_parts: Vec<_> = registry
        .snapshot()
        .into_iter()
        .filter(|reader| cutoffs.log_part_fully_expired(reader.meta()))
        .map(|reader| {
            (
                ManifestPart {
                    id: reader.meta().id.clone(),
                    partition: reader.meta().partition.clone(),
                },
                reader.part().dir.clone(),
            )
        })
        .collect();
    let mut trace_parts: Vec<_> = trace_registry
        .snapshot()
        .into_iter()
        .filter(|reader| cutoffs.trace_part_fully_expired(&reader.part().meta))
        .map(|reader| {
            (
                TraceManifestPart {
                    id: reader.part().meta.id.clone(),
                    partition: reader.part().meta.partition.clone(),
                },
                reader.part().dir.clone(),
            )
        })
        .collect();
    let mut metric_parts: Vec<_> = series_registry
        .snapshot()
        .into_iter()
        .filter(|reader| cutoffs.metric_part_fully_expired(&reader.part().meta))
        .map(|reader| {
            (
                MetricManifestPart {
                    id: reader.part().meta.id.clone(),
                    partition: reader.part().meta.partition.clone(),
                },
                reader.part().dir.clone(),
            )
        })
        .collect();
    log_parts.sort_by_key(|(part, _)| part.id.clone());
    trace_parts.sort_by_key(|(part, _)| part.id.clone());
    metric_parts.sort_by_key(|(part, _)| part.id.clone());
    log_parts.truncate(batch_size);
    trace_parts.truncate(batch_size);
    metric_parts.truncate(batch_size);

    if log_parts.is_empty() && trace_parts.is_empty() && metric_parts.is_empty() {
        return Ok(());
    }
    drop(guard);

    // Retire local bodies before the remote manifest CAS. If the process
    // crashes after this point, startup restores still-active descriptors from
    // the old manifest; if the CAS already happened, the descriptor is absent
    // and cannot be resurrected by local-cache reconciliation.
    let mut removed = 0usize;
    let mut removed_log_ids = Vec::new();
    let mut removed_trace_ids = Vec::new();
    let mut removed_metric_ids = Vec::new();
    {
        // Deletion lock first, and the wait for it happens here rather than one
        // line later. This pass has to wait: a merge rewrite holds the deletion
        // lock's read half for as long as its group takes, deliberately, so its
        // inputs cannot be deleted under it. What the wait must not do is hold
        // `operation_lock` while it happens — that is the lock a query holds
        // for its whole scan and a flush takes to commit, and the measured cost
        // of taking it first was the whole server stopping for the rest of the
        // rewrite: 39 of 39 soak freezes began on a retention tick, the longest
        // 52 s, one of them to delete a single part. Worse, merge's own commit
        // wants `operation_lock` after taking the deletion lock, so the old
        // order had the two spinning against each other until retention's
        // `try_write` won the gap between merge's rewrite guard and its commit
        // guard, which is why the freezes were unpredictable as well as long.
        //
        // Deletion-then-operation is also merge's order (`merge/scheduler.rs`
        // reads the deletion lock, then takes the operation lock to install the
        // replacement), so this is now the one order both double acquisitions
        // use, and the cycle is gone rather than survived by spinning.
        let _deletion_guard =
            crate::part_registry::PartRegistry::write_without_convoy(registry.deletion_lock())
                .await;
        // Then the operation lock, for the deletes and the retirement only. A
        // query must not be able to observe a part that is registered but whose
        // files are already gone, so these stay atomic together — it is the
        // waiting that moved out, not the work.
        let _guard =
            crate::part_registry::PartRegistry::write_without_convoy(registry.operation_lock())
                .await;
        // One snapshot per registry, not one per candidate. This runs under the
        // exclusive lifecycle lock, so a scan per candidate stalls flush, merge
        // and queries for as long as the whole batch takes.
        let active_log_dirs: std::collections::HashMap<String, std::path::PathBuf> = registry
            .snapshot()
            .into_iter()
            .map(|reader| (reader.meta().id.clone(), reader.part().dir.clone()))
            .collect();
        let active_trace_dirs: std::collections::HashMap<String, std::path::PathBuf> =
            trace_registry
                .snapshot()
                .into_iter()
                .map(|reader| (reader.part().meta.id.clone(), reader.part().dir.clone()))
                .collect();
        let active_metric_dirs: std::collections::HashMap<String, std::path::PathBuf> =
            series_registry
                .snapshot()
                .into_iter()
                .map(|reader| (reader.part().meta.id.clone(), reader.part().dir.clone()))
                .collect();
        for (descriptor, dir) in &log_parts {
            if active_log_dirs.get(&descriptor.id) == Some(dir) {
                part::remove_part_dirs(std::slice::from_ref(dir))?;
                removed_log_ids.push(descriptor.id.clone());
                removed += 1;
            }
        }
        for (descriptor, dir) in &trace_parts {
            if active_trace_dirs.get(&descriptor.id) == Some(dir) {
                part::remove_part_dirs(std::slice::from_ref(dir))?;
                removed_trace_ids.push(descriptor.id.clone());
                removed += 1;
            }
        }
        for (descriptor, dir) in &metric_parts {
            if active_metric_dirs.get(&descriptor.id) == Some(dir) {
                part::remove_part_dirs(std::slice::from_ref(dir))?;
                removed_metric_ids.push(descriptor.id.clone());
                removed += 1;
            }
        }
        if remote_cache.is_none() {
            registry.unregister(&removed_log_ids);
            trace_registry.unregister(&removed_trace_ids);
            series_registry.unregister(&removed_metric_ids);
        }
    }

    if let Some(cache) = remote_cache {
        if !removed_log_ids.is_empty() {
            retire_log_parts(registry, cache, config, &removed_log_ids).await?;
        }
        if !removed_trace_ids.is_empty() {
            let descriptors: Vec<_> = trace_parts
                .iter()
                .filter(|(part, _)| removed_trace_ids.iter().any(|id| id == &part.id))
                .map(|(part, _)| part.clone())
                .collect();
            // Registry first, then the manifest -- see `retire_log_parts`.
            {
                let _guard = crate::part_registry::PartRegistry::write_without_convoy(
                    registry.operation_lock(),
                )
                .await;
                trace_registry.unregister(&removed_trace_ids);
            }
            match tokio::time::timeout(
                config.max_retention_runtime,
                cache.storage.remove_trace_parts(&descriptors),
            )
            .await
            {
                Ok(Ok(_)) => cache.record_remote_success(),
                Ok(Err(error)) => {
                    cache.record_remote_failure();
                    return Err(error);
                }
                Err(_) => {
                    cache.record_remote_failure();
                    return Err("trace object-store retention timed out".to_string());
                }
            }
        }
        // The metric manifest last, extending the log-then-trace ordering:
        // each signal's descriptors retire as soon as their own manifest write
        // lands, so a failure in a later signal cannot leave an earlier one
        // registered but unservable.
        if !removed_metric_ids.is_empty() {
            let descriptors: Vec<_> = metric_parts
                .iter()
                .filter(|(part, _)| removed_metric_ids.iter().any(|id| id == &part.id))
                .map(|(part, _)| part.clone())
                .collect();
            // Registry first, then the manifest -- see `retire_log_parts`.
            {
                let _guard = crate::part_registry::PartRegistry::write_without_convoy(
                    registry.operation_lock(),
                )
                .await;
                series_registry.unregister(&removed_metric_ids);
            }
            match tokio::time::timeout(
                config.max_retention_runtime,
                cache.storage.remove_metric_parts(&descriptors),
            )
            .await
            {
                Ok(Ok(_)) => cache.record_remote_success(),
                Ok(Err(error)) => {
                    cache.record_remote_failure();
                    return Err(error);
                }
                Err(_) => {
                    cache.record_remote_failure();
                    return Err("metric object-store retention timed out".to_string());
                }
            }
        }
    }

    tracing::info!(removed, "retention removed expired parts");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::memtable::Labels;
    use crate::object_storage::ObjectStorage;
    use crate::part::{self, Row};
    use crate::tenant::TenantId;
    use crate::tenant_policy::TenantRetention;
    use crate::trace::TraceSpan;
    use std::collections::HashSet;

    fn tenant(raw: &str) -> TenantId {
        TenantId::parse(raw).expect("valid tenant id")
    }

    fn row_for(owner: &str, timestamp_ns: i64) -> Row {
        Row {
            tenant: tenant(owner),
            timestamp_ns,
            line: format!("{owner} line"),
            structured_metadata: vec![],
        }
    }

    fn span_for(owner: &str, trace_id: &str, timestamp_ns: i64) -> TraceSpan {
        TraceSpan {
            tenant: tenant(owner),
            trace_id: trace_id.to_string(),
            span_id: format!("{owner}-span"),
            start_time_ns: timestamp_ns,
            end_time_ns: timestamp_ns + 1,
            span: Default::default(),
            resource: None,
            resource_schema_url: String::new(),
            scope: None,
            scope_schema_url: String::new(),
        }
    }

    fn policy_with(entries: &[(&str, TenantRetention)]) -> TenantPolicy {
        let policy = TenantPolicy::enabled_for_test();
        policy.install_for_test(
            entries
                .iter()
                .map(|(name, retention)| (tenant(name), *retention))
                .collect(),
        );
        policy
    }

    fn temp_root(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("signy-{label}-{}", uuid::Uuid::new_v4()))
    }

    fn per_tenant_config(root: std::path::PathBuf) -> Config {
        Config {
            data_dir: root,
            ..Config::default()
        }
    }

    /// The count reads the memtables as well as the parts. A tenant that has
    /// only just started pushing owns no `meta.json` segment yet, and that is
    /// precisely when a control-plane omission is newest and most worth
    /// reporting.
    #[tokio::test]
    async fn the_unknown_tenant_count_sees_tenants_that_have_only_pushed() {
        let root = temp_root("retention-unknown-count");
        let config = Config {
            data_dir: root.clone(),
            ..Config::default()
        };
        std::fs::create_dir_all(&root).unwrap();
        let memtable = Arc::new(crate::memtable::MemTable::new());
        let journal =
            Arc::new(crate::journal::Journal::spawn(&config, memtable.clone()).expect("journal"));
        let registry = Arc::new(PartRegistry::new());
        let trace_registry = Arc::new(TraceRegistry::standalone());
        let policy = policy_with(&[("acme", TenantRetention::Finite(Duration::from_secs(60)))]);

        assert_eq!(
            unknown_tenant_count(
                &registry,
                &trace_registry,
                &SeriesRegistry::standalone(),
                &journal,
                &policy
            ),
            0
        );

        memtable.insert(
            tenant("brand-new"),
            vec![crate::memtable::LogEntry {
                timestamp_ns: 1_000,
                line: "never flushed".to_string(),
                structured_metadata: Vec::new(),
            }],
        );
        assert_eq!(
            unknown_tenant_count(
                &registry,
                &trace_registry,
                &SeriesRegistry::standalone(),
                &journal,
                &policy
            ),
            1
        );

        journal.trace_memtable().insert(vec![span_for(
            "brand-new-traces",
            &"dd".repeat(16),
            1_000,
        )]);
        assert_eq!(
            unknown_tenant_count(
                &registry,
                &trace_registry,
                &SeriesRegistry::standalone(),
                &journal,
                &policy
            ),
            2
        );

        // A flushed part counts the same tenant once, not twice.
        let parts =
            part::flush_rows(vec![row_for("brand-new", 1_000)], &root.join("parts"), 100).unwrap();
        registry.register(parts).unwrap();
        assert_eq!(
            unknown_tenant_count(
                &registry,
                &trace_registry,
                &SeriesRegistry::standalone(),
                &journal,
                &policy
            ),
            2
        );

        // With the policy switched off there is nothing to be unknown against.
        assert_eq!(
            unknown_tenant_count(
                &registry,
                &trace_registry,
                &SeriesRegistry::standalone(),
                &journal,
                &TenantPolicy::disabled()
            ),
            0
        );
    }

    #[tokio::test]
    async fn a_part_survives_while_any_tenant_in_it_is_unknown() {
        let root = temp_root("retention-unknown-tenant");
        let parts_root = root.join("parts");
        let parts = part::flush_rows(
            vec![row_for("alpha", 1_000), row_for("beta", 1_000)],
            &parts_root,
            100,
        )
        .unwrap();
        let registry = Arc::new(PartRegistry::new());
        registry.register(parts.clone()).unwrap();
        let trace_registry = Arc::new(TraceRegistry::standalone());
        // alpha has expired many times over; beta was never mentioned by the
        // control plane, and unknown means keep.
        let policy = policy_with(&[("alpha", TenantRetention::Finite(Duration::from_nanos(1)))]);

        retention_once_at(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &per_tenant_config(root),
            &policy,
            1_000_000,
        )
        .await
        .unwrap();

        assert_eq!(registry.part_count(), 1);
        assert!(parts[0].dir.exists());
    }

    /// Unknown means keep, in whichever state the data happens to be. Nothing
    /// deletes memtable entries for retention — the query floor makes expired
    /// ones invisible and the flush carries them to a part — so an unknown
    /// tenant that has only pushed must come through a tick untouched, and so
    /// must the part it eventually becomes.
    #[tokio::test]
    async fn an_unknown_tenant_survives_retention_in_the_memtable_and_in_a_part() {
        let root = temp_root("retention-unknown-memtable");
        let parts_root = root.join("parts");
        let config = per_tenant_config(root);
        let registry = Arc::new(PartRegistry::new());
        let trace_registry = Arc::new(TraceRegistry::standalone());
        // Everything the control plane knows about has expired many times
        // over; `unmentioned` is absent from it entirely.
        let policy = policy_with(&[("alpha", TenantRetention::Finite(Duration::from_nanos(1)))]);

        let memtable = crate::memtable::MemTable::new();
        memtable.insert(
            tenant("unmentioned"),
            vec![crate::memtable::LogEntry {
                timestamp_ns: 1_000,
                line: "still in the memtable".to_string(),
                structured_metadata: Vec::new(),
            }],
        );

        retention_once_at(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &config,
            &policy,
            1_000_000,
        )
        .await
        .unwrap();

        let in_memory = memtable.query(
            &tenant("unmentioned"),
            &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            10,
            true,
        );
        assert_eq!(
            in_memory.iter().flat_map(|stream| &stream.entries).count(),
            1
        );

        // The same rows once they have been flushed, in a part of their own.
        let parts =
            part::flush_rows(vec![row_for("unmentioned", 1_000)], &parts_root, 100).unwrap();
        registry.register(parts.clone()).unwrap();

        retention_once_at(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &config,
            &policy,
            1_000_000,
        )
        .await
        .unwrap();

        assert_eq!(registry.part_count(), 1);
        assert!(parts[0].dir.exists());
    }

    #[tokio::test]
    async fn a_part_whose_every_tenant_expired_takes_the_free_whole_delete_path() {
        let root = temp_root("retention-all-expired");
        let parts_root = root.join("parts");
        let parts = part::flush_rows(
            vec![row_for("alpha", 1_000), row_for("beta", 1_000)],
            &parts_root,
            100,
        )
        .unwrap();
        let registry = Arc::new(PartRegistry::new());
        registry.register(parts.clone()).unwrap();
        let trace_registry = Arc::new(TraceRegistry::standalone());
        let policy = policy_with(&[
            ("alpha", TenantRetention::Finite(Duration::from_nanos(1))),
            ("beta", TenantRetention::Finite(Duration::from_nanos(1))),
        ]);

        retention_once_at(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &per_tenant_config(root),
            &policy,
            1_000_000,
        )
        .await
        .unwrap();

        assert_eq!(registry.part_count(), 0);
        assert!(!parts[0].dir.exists());
    }

    #[tokio::test]
    async fn an_infinite_tenant_keeps_its_part_forever() {
        let root = temp_root("retention-infinite");
        let parts_root = root.join("parts");
        let parts = part::flush_rows(vec![row_for("intern", 1_000)], &parts_root, 100).unwrap();
        let registry = Arc::new(PartRegistry::new());
        registry.register(parts.clone()).unwrap();
        let trace_registry = Arc::new(TraceRegistry::standalone());
        let policy = policy_with(&[("intern", TenantRetention::Infinite)]);

        retention_once_at(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &per_tenant_config(root),
            &policy,
            i64::MAX as u128,
        )
        .await
        .unwrap();

        assert_eq!(registry.part_count(), 1);
    }

    #[tokio::test]
    async fn an_endpoint_that_never_answered_deletes_nothing() {
        let root = temp_root("retention-no-snapshot");
        let parts_root = root.join("parts");
        let parts = part::flush_rows(vec![row_for("alpha", 1_000)], &parts_root, 100).unwrap();
        let registry = Arc::new(PartRegistry::new());
        registry.register(parts.clone()).unwrap();
        let trace_registry = Arc::new(TraceRegistry::standalone());
        // Enabled, but no successful fetch has ever happened.
        let policy = TenantPolicy::enabled_for_test();

        retention_once_at(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &per_tenant_config(root),
            &policy,
            i64::MAX as u128,
        )
        .await
        .unwrap();

        assert_eq!(registry.part_count(), 1);
        assert!(parts[0].dir.exists());
    }

    #[tokio::test]
    async fn traces_follow_the_same_per_tenant_rules() {
        let root = temp_root("retention-traces");
        let traces_root = root.join("traces");
        let spans = vec![
            span_for("alpha", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 1_000),
            span_for("beta", "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", 1_000),
        ];
        let parts = crate::trace_part::flush_trace_spans(&spans, &traces_root, 100).unwrap();
        let registry = Arc::new(PartRegistry::new());
        let trace_registry = Arc::new(TraceRegistry::standalone());
        trace_registry.register(parts.clone()).unwrap();

        // beta is unknown, so the shared trace part stays.
        retention_once_at(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &per_tenant_config(root.clone()),
            &policy_with(&[("alpha", TenantRetention::Finite(Duration::from_nanos(1)))]),
            1_000_000,
        )
        .await
        .unwrap();
        assert_eq!(trace_registry.part_count(), 1);

        // Once beta has a policy too, the whole part goes.
        retention_once_at(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &per_tenant_config(root),
            &policy_with(&[
                ("alpha", TenantRetention::Finite(Duration::from_nanos(1))),
                ("beta", TenantRetention::Finite(Duration::from_nanos(1))),
            ]),
            1_000_000,
        )
        .await
        .unwrap();
        assert_eq!(trace_registry.part_count(), 0);
        assert!(!parts[0].dir.exists());
    }

    fn metric_part_for(
        root: &std::path::Path,
        owner: &str,
        ts: i64,
    ) -> crate::series_part::SeriesPart {
        use crate::series::{
            METRIC_NAME_LABEL, MetricSample, MetricValue, SampleKind, SeriesLabels,
        };
        let memtable = crate::series::SeriesMemTable::new();
        memtable.insert(vec![MetricSample {
            tenant: tenant(owner),
            labels: SeriesLabels::from_pairs(vec![(
                METRIC_NAME_LABEL.to_string(),
                "queue_depth".to_string(),
            )]),
            ts_ns: ts,
            value: MetricValue::Scalar(1.0),
            kind: SampleKind::Gauge,
            datapoint_index: 0,
        }]);
        let snapshot = memtable.begin_flush();
        let parts = crate::series_part::flush_series_snapshot(&snapshot, root).unwrap();
        memtable.commit_flush();
        parts.into_iter().next().unwrap()
    }

    #[tokio::test]
    async fn metrics_follow_the_same_per_tenant_rules_and_leave_the_remote_manifest() {
        let root = temp_root("retention-metrics");
        let metrics_root = root.join("metrics");
        let part = metric_part_for(&metrics_root, "alpha", 1_000);
        let registry = Arc::new(PartRegistry::new());
        let trace_registry = Arc::new(TraceRegistry::standalone());
        let series_registry = Arc::new(SeriesRegistry::standalone());
        series_registry.register(vec![part.clone()]).unwrap();
        let storage = Arc::new(crate::object_storage::ObjectStorage::in_memory());
        storage
            .publish_metric_parts(std::slice::from_ref(&part), &[])
            .await
            .unwrap();
        let remote = RemoteCache::new(storage.clone(), root.join("parts"));

        // Unknown means keep, for metrics exactly as for the others.
        retention_once_at(
            &registry,
            &trace_registry,
            &series_registry,
            Some(&remote),
            &per_tenant_config(root.clone()),
            &policy_with(&[(
                "someone-else",
                TenantRetention::Finite(Duration::from_nanos(1)),
            )]),
            1_000_000,
        )
        .await
        .unwrap();
        assert_eq!(series_registry.part_count(), 1);

        retention_once_at(
            &registry,
            &trace_registry,
            &series_registry,
            Some(&remote),
            &per_tenant_config(root),
            &policy_with(&[("alpha", TenantRetention::Finite(Duration::from_nanos(1)))]),
            1_000_000,
        )
        .await
        .unwrap();
        assert_eq!(series_registry.part_count(), 0, "the registry let it go");
        assert!(!part.dir.exists(), "the local files are gone");
        assert!(
            storage
                .load_metric_manifest()
                .await
                .unwrap()
                .parts
                .is_empty(),
            "the manifest no longer exposes it"
        );
    }

    #[tokio::test]
    async fn an_upgrade_keeps_data_alive_past_the_old_cutoff() {
        let root = temp_root("retention-upgrade");
        let parts_root = root.join("parts");
        let parts = part::flush_rows(vec![row_for("alpha", 1_000)], &parts_root, 100).unwrap();
        let registry = Arc::new(PartRegistry::new());
        registry.register(parts.clone()).unwrap();
        let trace_registry = Arc::new(TraceRegistry::standalone());
        let config = per_tenant_config(root);

        // The upgraded plan is applied at deletion time, not at write time, so
        // data written under the old plan is covered by the new one.
        let upgraded = policy_with(&[(
            "alpha",
            TenantRetention::Finite(Duration::from_nanos(1_000_000)),
        )]);
        retention_once_at(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &config,
            &upgraded,
            1_000_500,
        )
        .await
        .unwrap();
        assert_eq!(registry.part_count(), 1);

        let downgraded =
            policy_with(&[("alpha", TenantRetention::Finite(Duration::from_nanos(100)))]);
        retention_once_at(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &config,
            &downgraded,
            1_000_500,
        )
        .await
        .unwrap();
        assert_eq!(registry.part_count(), 0);
    }

    #[tokio::test]
    async fn the_storeless_fixture_deletes_nothing() {
        let root = std::env::temp_dir().join(format!("signy-retention-{}", uuid::Uuid::new_v4()));
        let registry = Arc::new(PartRegistry::new());
        let trace_registry = Arc::new(TraceRegistry::standalone());
        let config = Config {
            data_dir: root,
            ..Config::default()
        };
        retention_once(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &config,
            &TenantPolicy::disabled(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn retention_removes_an_expired_local_part() {
        let root = std::env::temp_dir().join(format!("signy-retention-{}", uuid::Uuid::new_v4()));
        let parts_root = root.join("parts");
        let _labels: Labels = [("app".to_string(), "retention".to_string())]
            .into_iter()
            .collect();
        let parts = part::flush_rows(
            vec![Row {
                tenant: crate::tenant::test_tenant(),
                timestamp_ns: 1_000,
                line: "expired".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            100,
        )
        .unwrap();
        let registry = Arc::new(PartRegistry::new());
        registry.register(parts.clone()).unwrap();
        let trace_registry = Arc::new(TraceRegistry::standalone());
        let config = Config {
            data_dir: root,
            ..Config::default()
        };
        let policy = policy_with(&[(
            crate::tenant::test_tenant().as_str(),
            TenantRetention::Finite(Duration::from_secs(1)),
        )]);
        retention_once(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &config,
            &policy,
        )
        .await
        .unwrap();
        assert_eq!(registry.part_count(), 0);
        assert!(!parts[0].dir.exists());
    }

    /// A retention pass waiting for a merge rewrite must not stop the queries.
    ///
    /// The rewrite is represented by the only thing it actually holds — the
    /// deletion lock's read half — and the query by the only thing it needs, the
    /// operation lock's read half. With the acquisition order reversed, the pass
    /// holds the query's lock for the whole length of the rewrite; the soak
    /// measured that as freezes up to 52 s, every one starting on a retention
    /// tick.
    #[tokio::test]
    async fn a_retention_pass_waiting_for_a_merge_does_not_stop_queries() {
        let root =
            std::env::temp_dir().join(format!("signy-retention-wait-{}", uuid::Uuid::new_v4()));
        let parts_root = root.join("parts");
        let _labels: Labels = [("app".to_string(), "retention".to_string())]
            .into_iter()
            .collect();
        let parts = part::flush_rows(
            vec![Row {
                tenant: crate::tenant::test_tenant(),
                timestamp_ns: 1_000,
                line: "expired".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            100,
        )
        .unwrap();
        let registry = Arc::new(PartRegistry::new());
        registry.register(parts.clone()).unwrap();
        let trace_registry = Arc::new(TraceRegistry::standalone());
        let config = Config {
            data_dir: root,
            ..Config::default()
        };
        let policy = policy_with(&[(
            crate::tenant::test_tenant().as_str(),
            TenantRetention::Finite(Duration::from_secs(1)),
        )]);

        // The merge rewrite, holding its inputs against deletion.
        let rewrite_guard = registry.deletion_lock().read_owned().await;

        let series_registry = SeriesRegistry::standalone();
        let (pass, query_served) = tokio::join!(
            retention_once(
                &registry,
                &trace_registry,
                &series_registry,
                None,
                &config,
                &policy,
            ),
            async {
                // Long enough for the pass to reach its wait, short enough that
                // the test stays fast.
                tokio::time::sleep(Duration::from_millis(100)).await;
                let served = tokio::time::timeout(
                    Duration::from_millis(500),
                    registry.operation_lock().read_owned(),
                )
                .await
                .is_ok();
                // Release the rewrite so the pass finishes either way, and the
                // assertions below read the same on both branches.
                drop(rewrite_guard);
                served
            }
        );
        pass.unwrap();
        assert!(
            query_served,
            "a query could not take the operation lock while retention waited for the deletion lock"
        );
        assert_eq!(registry.part_count(), 0);
        assert!(!parts[0].dir.exists());
    }

    #[tokio::test]
    async fn retention_removes_expired_parts_from_a_remote_manifest() {
        let root = std::env::temp_dir().join(format!("signy-retention-{}", uuid::Uuid::new_v4()));
        let parts_root = root.join("parts");
        let _labels: Labels = [("app".to_string(), "remote-retention".to_string())]
            .into_iter()
            .collect();
        let parts = part::flush_rows(
            vec![Row {
                tenant: crate::tenant::test_tenant(),
                timestamp_ns: 1_000,
                line: "expired remotely".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            100,
        )
        .unwrap();
        let storage = Arc::new(crate::object_storage::ObjectStorage::in_memory());
        storage.publish(&parts, &[]).await.unwrap();
        let registry = Arc::new(PartRegistry::new());
        registry.register(parts.clone()).unwrap();
        let trace_registry = Arc::new(TraceRegistry::standalone());
        let remote = RemoteCache::new(storage.clone(), parts_root.clone());
        let config = Config {
            data_dir: root,
            ..Config::default()
        };
        let policy = policy_with(&[(
            crate::tenant::test_tenant().as_str(),
            TenantRetention::Finite(Duration::from_secs(1)),
        )]);

        retention_once(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            Some(&remote),
            &config,
            &policy,
        )
        .await
        .unwrap();

        assert!(storage.load_manifest().await.unwrap().parts.is_empty());
        assert_eq!(registry.part_count(), 0);
        assert!(!parts[0].dir.exists());
    }

    #[tokio::test]
    async fn retention_clock_keeps_the_cutoff_boundary() {
        let root =
            std::env::temp_dir().join(format!("signy-retention-boundary-{}", uuid::Uuid::new_v4()));
        let parts_root = root.join("parts");
        let parts = part::flush_rows(
            vec![Row {
                tenant: crate::tenant::test_tenant(),
                timestamp_ns: 90,
                line: "boundary".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            16,
        )
        .unwrap();
        let registry = Arc::new(PartRegistry::new());
        registry.register(parts.clone()).unwrap();
        let trace_registry = Arc::new(TraceRegistry::standalone());
        let config = Config {
            data_dir: root,
            ..Config::default()
        };
        let policy = policy_with(&[(
            crate::tenant::test_tenant().as_str(),
            TenantRetention::Finite(Duration::from_nanos(10)),
        )]);

        retention_once_at(
            &registry,
            &trace_registry,
            &SeriesRegistry::standalone(),
            None,
            &config,
            &policy,
            100,
        )
        .await
        .unwrap();

        assert_eq!(registry.part_count(), 1);
        assert!(parts[0].dir.exists());
    }

    /// The lifecycle contract at the seam the reader was built around: it
    /// plans from the registry under the read guard, retention runs in the
    /// deliberate re-plan gap, and the reader must not then be asked to
    /// restore a part the manifest no longer has.
    ///
    /// The two retirement steps are driven separately because the hazard lives
    /// between them: run back to back, either order leaves the reader nothing
    /// to trip over.
    async fn reader_across_retirement(registry_first: bool) -> Result<(), String> {
        let storage = Arc::new(ObjectStorage::in_memory());
        let parts_root = temp_root("replan-gap").join("parts");
        let parts = part::flush_rows(
            vec![row_for("t", 1_700_000_000_000_000_000)],
            &parts_root,
            100,
        )
        .unwrap();
        storage.publish(&parts, &[]).await.unwrap();
        let ids = vec![parts[0].meta.id.clone()];

        let registry = Arc::new(PartRegistry::new());
        registry.register(parts).unwrap();
        let cache = Arc::new(RemoteCache::new(storage, parts_root.clone()));
        let config = Config::default();

        let planning = registry.clone();
        let required = move || -> HashSet<String> {
            planning
                .snapshot()
                .into_iter()
                .map(|reader| reader.meta().id.clone())
                .collect()
        };
        // Nothing is local, so everything planned is asked of the restore.
        let missing = |required: &HashSet<String>| required.clone();

        let retiring = registry.clone();
        let retiring_cache = cache.clone();
        let retiring_config = config.clone();
        crate::remote_lifecycle::pin_remote_parts(
            registry.operation_lock(),
            Some(cache.clone()),
            required,
            missing,
            crate::remote_lifecycle::RemoteDomain::Logs,
            Duration::from_secs(30),
            move || {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        if registry_first {
                            unregister_log_parts(&retiring, &ids).await;
                        } else {
                            drop_log_parts_from_manifest(&retiring_cache, &retiring_config, &ids)
                                .await
                                .unwrap();
                        }
                    })
                });
                Ok(())
            },
            None,
        )
        .await
        .map(|_| ())
        .map_err(|error| format!("{error:?}"))
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn retiring_from_the_registry_first_leaves_the_reader_whole() {
        reader_across_retirement(true)
            .await
            .expect("a re-planned reader must simply drop the retired part");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn retiring_from_the_manifest_first_breaks_the_reader() {
        // Why `retire_log_parts` unregisters before it writes the manifest:
        // the other order leaves the reader planning a part the restore then
        // refuses, which fails the query rather than returning less.
        let error = reader_across_retirement(false)
            .await
            .expect_err("the manifest-first order must break the reader");
        assert!(
            error.contains("no longer present in the object-store manifest"),
            "expected the restore to refuse the retired part, got: {error}"
        );
    }
}
