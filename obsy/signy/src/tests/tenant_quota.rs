    use super::*;
    use crate::clock::Clock;
    use crate::tenant::TenantId;

    fn tenant(name: &str) -> TenantId {
        TenantId::parse(name).unwrap()
    }

    fn quota(config: Config, policy: Arc<TenantPolicy>) -> TenantQuota {
        quota_over(
            config,
            policy,
            Arc::new(crate::part_registry::PartRegistry::new()),
        )
    }

    fn quota_over(
        config: Config,
        policy: Arc<TenantPolicy>,
        parts: Arc<crate::part_registry::PartRegistry>,
    ) -> TenantQuota {
        quota_over_all(
            config,
            policy,
            parts,
            Arc::new(crate::series_registry::SeriesRegistry::standalone()),
        )
    }

    fn quota_over_all(
        config: Config,
        policy: Arc<TenantPolicy>,
        parts: Arc<crate::part_registry::PartRegistry>,
        series_parts: Arc<crate::series_registry::SeriesRegistry>,
    ) -> TenantQuota {
        TenantQuota::new(
            Arc::new(config),
            Arc::new(RuntimeMetrics::new()),
            policy,
            parts,
            Arc::new(crate::trace_registry::TraceRegistry::standalone()),
            series_parts,
        )
    }

    /// A metric registry holding one flushed part for the named tenant, the
    /// series counterpart of `registry_holding`.
    fn series_registry_holding(name: &str) -> Arc<crate::series_registry::SeriesRegistry> {
        use crate::series::{METRIC_NAME_LABEL, MetricSample, SampleKind, SeriesLabels};

        let root = crate::test_support::temp_dir("storage-quota-series");
        let labels = SeriesLabels::from_pairs(vec![
            (METRIC_NAME_LABEL.to_string(), "queue_depth".to_string()),
            ("instance".to_string(), "a".to_string()),
        ]);
        let memtable = crate::series::SeriesMemTable::new();
        memtable.insert(
            (0..64)
                .map(|index| MetricSample {
                    tenant: TenantId::parse(name).unwrap(),
                    labels: labels.clone(),
                    ts_ns: 1_772_000_000_000_000_000 + index * 1_000_000_000,
                    value: crate::series::MetricValue::Scalar(index as f64),
                    kind: SampleKind::Gauge,
                    datapoint_index: 0,
                })
                .collect(),
        );
        let snapshot = memtable.begin_flush();
        crate::series_part::flush_series_snapshot(&snapshot, &root).unwrap();
        memtable.commit_flush();

        let registry = Arc::new(
            crate::series_registry::SeriesRegistry::load_from_disk(
                &root,
                Arc::new(tokio::sync::RwLock::new(())),
            )
            .unwrap(),
        );
        assert!(registry.tenant_stored_bytes(&TenantId::parse(name).unwrap()) > 0);
        registry
    }

    /// A registry holding one part with rows for each named tenant, so a
    /// storage limit has something real to measure.
    fn registry_holding(tenants: &[&str]) -> Arc<crate::part_registry::PartRegistry> {
        use crate::memtable::Labels;
        use crate::part::Row;
        use std::collections::BTreeMap;

        let root = crate::test_support::temp_dir("storage-quota");
        let mut labels: Labels = BTreeMap::new();
        labels.insert("app".to_string(), "test".to_string());
        let rows: Vec<Row> = tenants
            .iter()
            .flat_map(|name| {
                let _labels = std::sync::Arc::new(labels.clone());
                (0..64).map(move |index| Row {
                    tenant: TenantId::parse(name).unwrap(),
                    timestamp_ns: index,
                    line: format!("line {index}"),
                    structured_metadata: vec![],
                })
            })
            .collect();
        let registry = Arc::new(crate::part_registry::PartRegistry::new());
        registry
            .register(crate::part::flush_rows(rows, &root, 16).unwrap())
            .unwrap();
        for name in tenants {
            assert!(registry.tenant_stored_bytes(&TenantId::parse(name).unwrap()) > 0);
        }
        registry
    }

    /// The limit is a stock, not a flow: being under it admits regardless of
    /// how fast the tenant is writing, and being at it refuses regardless of
    /// how slowly.
    #[tokio::test]
    async fn a_tenant_at_its_storage_limit_is_refused_and_told_to_wait() {
        let parts = registry_holding(&["acme"]);
        let stored = parts.tenant_stored_bytes(&tenant("acme"));
        let policy = Arc::new(TenantPolicy::enabled_for_test());
        policy
            .push(&tenant("acme"), 1, "30d", Some(&(stored + 1).to_string()))
            .await
            .unwrap();
        quota_over(Config::default(), policy.clone(), parts.clone())
            .admit_storage(&tenant("acme"))
            .expect("a tenant below its limit still writes");

        policy
            .push(&tenant("acme"), 2, "30d", Some(&stored.to_string()))
            .await
            .unwrap();
        let error = quota_over(Config::default(), policy, parts)
            .admit_storage(&tenant("acme"))
            .expect_err("a tenant at its limit is refused");
        assert_eq!(error.status, axum::http::StatusCode::TOO_MANY_REQUESTS);
        assert!(
            error.retry_after.is_some_and(|wait| wait.as_secs() >= 60),
            "a storage refusal clears when retention runs, not in a second"
        );
        assert!(error.message.contains("at its limit"), "{}", error.message);
    }

    /// Metrics are the third signal and they occupy the same disk, so they are
    /// charged like the other two. M14 added metric parts without teaching the
    /// usage endpoint about them, which left a tenant refused on a total larger
    /// than the one it was shown; both readers ask `tenant_stored_bytes` now.
    #[tokio::test]
    async fn a_tenant_holding_only_metrics_is_charged_for_them() {
        let series_parts = series_registry_holding("acme");
        let stored = series_parts.tenant_stored_bytes(&tenant("acme"));
        let empty_logs = Arc::new(crate::part_registry::PartRegistry::new());
        let policy = Arc::new(TenantPolicy::enabled_for_test());
        policy
            .push(&tenant("acme"), 1, "30d", Some(&stored.to_string()))
            .await
            .unwrap();

        let quota = quota_over_all(
            Config::default(),
            policy.clone(),
            empty_logs.clone(),
            series_parts.clone(),
        );
        assert_eq!(
            quota.tenant_stored_bytes(&tenant("acme")),
            stored,
            "a tenant with no logs and no traces still stores its metric parts"
        );

        quota_over_all(Config::default(), policy, empty_logs, series_parts)
            .admit_storage(&tenant("acme"))
            .expect_err("metric bytes alone reach the limit");
    }

    /// One tenant's storage says nothing about its neighbour's, even though
    /// they share the object.
    #[tokio::test]
    async fn a_storage_limit_does_not_reach_the_tenant_beside_it() {
        let parts = registry_holding(&["acme"]);
        let stored = parts.tenant_stored_bytes(&tenant("acme"));
        let policy = Arc::new(TenantPolicy::enabled_for_test());
        policy
            .push(&tenant("acme"), 1, "30d", Some(&stored.to_string()))
            .await
            .unwrap();
        let quota = quota_over(Config::default(), policy, parts);
        quota
            .admit_storage(&tenant("acme"))
            .expect_err("the tenant holding the part is at its limit");
        quota
            .admit_storage(&tenant("globex"))
            .expect("a tenant holding nothing is not");
    }

    /// A pushed limit applies only to the tenant that named it.
    #[tokio::test]
    async fn a_pushed_storage_limit_applies_only_to_the_named_tenant() {
        let parts = registry_holding(&["acme", "globex"]);
        let stored = parts.tenant_stored_bytes(&tenant("acme"));
        let clock = Clock::fixed(0);
        let policy = Arc::new(TenantPolicy::enabled_with_clock(clock));
        policy
            .push(&tenant("acme"), 1, "7d", Some(&format!("{}", stored * 4)))
            .await
            .unwrap();
        let quota = quota_over(Config::default(), policy, parts);
        quota
            .admit_storage(&tenant("acme"))
            .expect("the pushed limit is four times what the tenant holds");
        quota
            .admit_storage(&tenant("globex"))
            .expect("a tenant with no explicit limit has no policy default");
    }

    /// A tenant issuing many concurrent scans would otherwise hold every permit
    /// of the shared query semaphore and queue everyone else behind it.
    #[test]
    fn one_tenant_cannot_hold_every_query_slot() {
        let config = Config {
            max_concurrent_queries_per_tenant: 2,
            ..Config::default()
        };
        let quota = Arc::new(quota(config, Arc::new(TenantPolicy::disabled())));
        let loud = tenant("loud");

        let first = quota.begin_query(&loud).unwrap();
        let second = quota.begin_query(&loud).unwrap();
        let refused = quota
            .begin_query(&loud)
            .expect_err("a third concurrent query is over the limit");
        assert_eq!(refused.status, StatusCode::TOO_MANY_REQUESTS);

        // The limit is per tenant, not global.
        let quiet = quota
            .begin_query(&tenant("quiet"))
            .expect("another tenant has its own slots");

        // Slots are released by dropping, including on a cancelled query.
        drop(first);
        quota
            .begin_query(&loud)
            .expect("finishing a query frees its slot");
        drop(second);
        drop(quiet);
    }
