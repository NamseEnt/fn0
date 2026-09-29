    use super::*;
    use crate::config::Config;
    use crate::journal::Journal;
    use crate::memtable::MemTable;
    use crate::object_storage::ObjectStorage;
    use crate::part_registry::PartRegistry;
    use crate::tenant_policy::TenantPolicy;
    use tower::ServiceExt;

    const RETENTION_URI: &str = "/signy/api/v1/admin/tenants/acme/retention";

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "signy-admin-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn tenant(raw: &str) -> TenantId {
        TenantId::parse(raw).expect("valid tenant id")
    }

    fn state_with(policy: Arc<TenantPolicy>) -> Arc<AppState> {
        let config = Config {
            data_dir: temp_dir(),
            ..Config::default()
        };
        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
        let parts = Arc::new(PartRegistry::new());
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            parts.operation_lock(),
        ));
        crate::test_support::state_with_tenant_policy(
            config,
            memtable,
            journal,
            parts,
            trace_parts,
            None,
            policy,
        )
    }

    async fn call(
        state: &Arc<AppState>,
        method: &str,
        uri: &str,
        body: &str,
    ) -> (StatusCode, String) {
        let request = axum::http::Request::builder()
            .method(method)
            .uri(uri)
            .body(axum::body::Body::from(body.to_string()))
            .unwrap();
        let response = crate::build_router(state.clone())
            .oneshot(request)
            .await
            .unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .unwrap();
        (status, String::from_utf8_lossy(&bytes).to_string())
    }

    #[tokio::test]
    async fn a_push_is_stored_then_readable_and_deletable() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = Arc::new(TenantPolicy::for_test_with_store(storage.clone()));
        let state = state_with(policy.clone());

        let (status, _) = call(&state, "GET", RETENTION_URI, "").await;
        assert_eq!(status, StatusCode::NOT_FOUND, "no policy pushed yet");

        let (status, body) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":1,"retention":"30d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["tenant"], "acme");
        assert_eq!(json["revision"], 1);
        assert_eq!(json["retention"], "30d");
        assert_eq!(json["result"], "applied");
        assert!(json["updated_at"].as_str().is_some());

        let (status, body) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":1,"retention":"30d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["result"], "duplicate");

        let (status, body) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":0,"retention":"7d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["result"], "stale");

        let (status, _) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":1,"retention":"7d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);

        // A 200 is a promise that the policy survives a restart.
        assert_eq!(storage.load_tenant_policies().await.unwrap().len(), 1);
        assert!(policy.query_floor_ns(&tenant("acme"), crate::tenant_policy::Signal::Logs).is_some());

        let (status, body) = call(&state, "GET", RETENTION_URI, "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap()["retention"],
            "30d"
        );

        let (status, _) = call(&state, "DELETE", RETENTION_URI, "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            policy.query_floor_ns(&tenant("acme"), crate::tenant_policy::Signal::Logs),
            None,
            "DELETE returns the tenant to unknown, which keeps its data"
        );
        assert!(storage.load_tenant_policies().await.unwrap().is_empty());
    }

    /// A limit rides on the same push as retention, survives a restart with
    /// it, and is reported as sent. The last part matters most: a body
    /// without `max_stored_bytes` clears the limit rather than keeping it,
    /// because the body is the policy and not a patch of it.
    #[tokio::test]
    async fn a_limit_rides_the_same_push_and_a_body_without_one_clears_it() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = Arc::new(TenantPolicy::for_test_with_store(storage.clone()));
        let state = state_with(policy.clone());

        let (status, body) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":1,"retention":"30d","max_stored_bytes":"10GiB"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["max_stored_bytes"], "10GiB");
        assert_eq!(
            policy.max_stored_bytes(&tenant("acme")),
            Some(crate::tenant_policy::TenantStorageLimit::Bytes(
                10 * 1024 * 1024 * 1024
            ))
        );

        // Durable, and readable back through a restart.
        let reload_config = Config::default();
        let reloaded = TenantPolicy::load(&reload_config, Some(storage.clone()))
            .await
            .unwrap();
        assert_eq!(
            reloaded.max_stored_bytes(&tenant("acme")),
            Some(crate::tenant_policy::TenantStorageLimit::Bytes(
                10 * 1024 * 1024 * 1024
            )),
            "a limit that does not survive a restart is not a policy"
        );

        let (status, body) = call(&state, "PUT", RETENTION_URI, r#"{"revision":2,"retention":"30d"}"#).await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            serde_json::from_str::<serde_json::Value>(&body)
                .unwrap()
                .get("max_stored_bytes")
                .is_none()
        );
        assert_eq!(
            policy.max_stored_bytes(&tenant("acme")),
            None,
            "omitting the field clears it; the body is the whole policy"
        );

        // A value that cannot be parsed stores nothing at all, retention
        // included: a partially applied push is worse than a rejected one.
        let (status, _) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":3,"retention":"7d","max_stored_bytes":"plenty"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            policy.view(&tenant("acme")).unwrap().retention,
            "30d",
            "the rejected push must not have changed retention either"
        );
    }

    #[tokio::test]
    async fn a_failing_store_returns_503_and_applies_nothing() {
        let policy = Arc::new(TenantPolicy::for_test_with_store(Arc::new(
            ObjectStorage::in_memory_with_failing_writes(),
        )));
        let state = state_with(policy.clone());

        let (status, _) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":1,"retention":"30d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            policy.query_floor_ns(&tenant("acme"), crate::tenant_policy::Signal::Logs),
            None,
            "a policy that is not durable is not applied either"
        );
        assert_eq!(
            policy
                .metrics
                .push_persist_errors
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert_eq!(
            policy
                .metrics
                .push_accepted
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
    }

    #[tokio::test]
    async fn a_malformed_tenant_or_value_is_a_400_and_stores_nothing() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = Arc::new(TenantPolicy::for_test_with_store(storage.clone()));
        let state = state_with(policy);

        let (status, _) = call(
            &state,
            "PUT",
            "/signy/api/v1/admin/tenants/..%2Fetc/retention",
            r#"{"revision":1,"retention":"30d"}"#,
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "a tenant id is never propagated into a path"
        );

        for body in [r#"{"revision":1,"retention":"soon"}"#, r#"{"revision":1,"retention":7}"#, "{}", "not json"] {
            let (status, _) = call(&state, "PUT", RETENTION_URI, body).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        }
        assert!(storage.load_tenant_policies().await.unwrap().is_empty());
    }

    /// A pushed value is applied exactly as sent. The instance has no setting
    /// that caps or rewrites it, so what a `GET` reports and what the query
    /// floor enforces can never disagree.
    #[tokio::test]
    async fn a_pushed_value_is_reported_and_enforced_as_sent() {
        let policy = Arc::new(TenantPolicy::for_test_with_store(Arc::new(
            ObjectStorage::in_memory(),
        )));
        let state = state_with(policy.clone());

        let (status, body) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":1,"retention":"30d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap()["retention"],
            "30d"
        );
        let floor_ns = policy.query_floor_ns(&tenant("acme"), crate::tenant_policy::Signal::Logs).unwrap();
        let thirty_days_ns = 30 * 24 * 60 * 60 * 1_000_000_000i64;
        let elapsed = crate::tenant_policy::now_ns() - floor_ns;
        assert!(
            (elapsed - thirty_days_ns).abs() < 1_000_000_000,
            "the pushed thirty days is what the query floor enforces"
        );
    }

    /// Asking for preservation must never cost data that staying silent would
    /// have kept: an explicit `infinite` and a tenant the control plane has
    /// never mentioned both leave queries unclamped.
    #[tokio::test]
    async fn an_explicit_infinite_keeps_as_much_as_never_being_pushed() {
        let policy = Arc::new(TenantPolicy::for_test_with_store(Arc::new(
            ObjectStorage::in_memory(),
        )));
        let state = state_with(policy.clone());

        let (status, _) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":1,"retention":"infinite"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(policy.query_floor_ns(&tenant("acme"), crate::tenant_policy::Signal::Logs), None);
        assert_eq!(policy.query_floor_ns(&tenant("never-pushed"), crate::tenant_policy::Signal::Logs), None);
    }

    /// `/metrics` is one positional `format!` over roughly forty-five counters,
    /// so a name and a value can drift apart without the compiler noticing. Give
    /// three of them distinct values and check each lands on its own line.
    #[tokio::test]
    async fn every_metric_name_carries_its_own_value() {
        let policy = Arc::new(TenantPolicy::for_test_with_store(Arc::new(
            ObjectStorage::in_memory(),
        )));
        let state = state_with(policy.clone());
        use std::sync::atomic::Ordering;
        state.metrics.merge_errors.store(11, Ordering::Relaxed);
        state
            .metrics
            .merge_inputs_changed
            .store(22, Ordering::Relaxed);
        state.metrics.retention_success.store(33, Ordering::Relaxed);
        policy.record_rejected_push();

        let body = crate::query::metrics(axum::extract::State(state.clone())).await;
        for (name, value) in [
            ("signy_merge_errors_total", "11"),
            ("signy_merge_inputs_changed_total", "22"),
            ("signy_retention_success_total", "33"),
            ("signy_tenant_policy_push_rejected_total", "1"),
        ] {
            assert!(
                body.lines().any(|line| line == format!("{name} {value}")),
                "{name} should report {value}:\n{body}"
            );
        }
    }


    /// Per-tenant numbers live here, not as labels on `/metrics`: a label per
    /// tenant would multiply every series by the tenant count — on a workload
    /// whose whole point is many small tenants, that is the cardinality
    /// problem this engine bounds everywhere else. The reader that needs
    /// per-tenant numbers is the control plane, which already asks per tenant.
    #[tokio::test]
    async fn per_tenant_usage_is_scoped_to_one_tenant() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = Arc::new(TenantPolicy::for_test_with_store(storage));
        let state = state_with(policy);
        let uri = "/signy/api/v1/admin/tenants/acme/usage";

        state.memtable.insert(
            tenant("acme"),
            vec![crate::memtable::LogEntry {
                timestamp_ns: 1_700_000_000_000_000_000,
                line: "counted".to_string(),
                structured_metadata: vec![("app".to_string(), "usage".to_string())],
            }],
        );

        let (status, body) = call(&state, "GET", uri, "").await;
        assert_eq!(status, StatusCode::OK);
        let usage: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(usage["tenant"], "acme");
        assert_eq!(usage["entries"], 1);

        // A different tenant sees its own zeroes, not acme's numbers.
        let (status, body) = call(
            &state,
            "GET",
            "/signy/api/v1/admin/tenants/other/usage",
            "",
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let other: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(other["entries"], 0);
    }

    /// `/metrics` stays free of tenant labels, which is the decision the usage
    /// endpoint exists to preserve rather than an oversight.
    #[tokio::test]
    async fn the_operator_scrape_carries_no_tenant_labels() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = Arc::new(TenantPolicy::for_test_with_store(storage));
        let state = state_with(policy);
        state.memtable.insert(
            tenant("acme"),
            vec![crate::memtable::LogEntry {
                timestamp_ns: 1,
                line: "x".to_string(),
                structured_metadata: vec![("app".to_string(), "scrape".to_string())],
            }],
        );

        let rendered = crate::query::metrics(axum::extract::State(state)).await;
        assert!(
            !rendered.contains("acme"),
            "a tenant id in the operator scrape is a series per tenant"
        );
        assert!(rendered.contains("signy_storage_limit_rejected_total"));
    }

    async fn read_labels_as(state: &Arc<AppState>, tenant: &str) -> StatusCode {
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/signy/api/v1/logs/attributes")
            .header(crate::tenant::TENANT_HEADER, tenant)
            .body(axum::body::Body::empty())
            .unwrap();
        crate::build_router(state.clone())
            .oneshot(request)
            .await
            .unwrap()
            .status()
    }

    /// The pushed policies are the tenant registry: a push onboards the tenant
    /// the moment it answers 200, and a delete returns it to unknown, which
    /// every request path refuses.
    #[tokio::test]
    async fn a_push_onboards_the_tenant_and_a_delete_offboards_it() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = Arc::new(TenantPolicy::for_test_with_store(storage));
        let state = state_with(policy);

        assert_eq!(
            read_labels_as(&state, "acme").await,
            StatusCode::FORBIDDEN,
            "a tenant nothing was pushed for is not served"
        );

        let (status, _) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":1,"retention":"30d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            read_labels_as(&state, "acme").await,
            StatusCode::OK,
            "the push onboarded the tenant"
        );
        assert_eq!(
            read_labels_as(&state, "stranger").await,
            StatusCode::FORBIDDEN,
            "onboarding one tenant does not open the door for another"
        );

        let (status, _) = call(&state, "DELETE", RETENTION_URI, "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            read_labels_as(&state, "acme").await,
            StatusCode::FORBIDDEN,
            "the delete offboarded the tenant"
        );
    }

    /// `GET …/admin/tenants` is the control plane's reconciliation read: every
    /// pushed tenant with its policy, and nothing else.
    #[tokio::test]
    async fn the_tenant_list_reports_every_pushed_policy() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = Arc::new(TenantPolicy::for_test_with_store(storage));
        let state = state_with(policy);
        const LIST_URI: &str = "/signy/api/v1/admin/tenants";

        let (status, body) = call(&state, "GET", LIST_URI, "").await;
        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["tenants"].as_array().unwrap().len(), 0);

        for (uri, request_body) in [
            (
                "/signy/api/v1/admin/tenants/zeta/retention",
                r#"{"revision":1,"retention":"7d","max_stored_bytes":"1GiB"}"#,
            ),
            (
                "/signy/api/v1/admin/tenants/acme/retention",
                r#"{"revision":1,"retention":"30d"}"#,
            ),
        ] {
            let (status, _) = call(&state, "PUT", uri, request_body).await;
            assert_eq!(status, StatusCode::OK);
        }

        let (_, body) = call(&state, "GET", LIST_URI, "").await;
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        let tenants = json["tenants"].as_array().unwrap();
        assert_eq!(
            tenants
                .iter()
                .map(|entry| entry["tenant"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["acme", "zeta"],
            "listed in name order"
        );
        assert_eq!(tenants[0]["retention"], "30d");
        assert_eq!(tenants[1]["retention"], "7d");
        assert_eq!(tenants[1]["max_stored_bytes"], "1GiB");
        assert!(
            tenants[0].get("max_stored_bytes").is_none(),
            "a field the control plane never pushed is absent, not defaulted"
        );
        assert!(tenants[0]["updated_at"].as_str().is_some());
    }

    /// fn0 is the source of truth for both the policy and its revision
    /// counter; this one endpoint fences a push against whatever revision it
    /// carries: newer applies, an exact retry is idempotent, the same
    /// revision with a different body conflicts, and an older delivery is a
    /// stale no-op that reports the policy actually in force.
    #[tokio::test]
    async fn revision_fencing_applies_stale_duplicate_and_conflict_through_one_endpoint() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = Arc::new(TenantPolicy::for_test_with_store(storage));
        let state = state_with(policy.clone());

        let (status, body) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":1,"retention":"30d","log_retention":"30d","trace_retention":"30d","metric_retention":"30d","max_stored_bytes":"512MiB"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["revision"], 1);
        assert_eq!(json["result"], "applied");

        let (status, _) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":3,"retention":"30d","log_retention":"30d","trace_retention":"30d","metric_retention":"30d","max_stored_bytes":"512MiB"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "a revision may skip ahead");

        let (status, body) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":2,"retention":"7d","log_retention":"7d","trace_retention":"7d","metric_retention":"7d","max_stored_bytes":"1GiB"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "an older delivery is a stale no-op");
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["revision"], 3, "the stale reply names the policy actually in force");
        assert_eq!(json["result"], "stale");

        let (status, body) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":3,"retention":"30d","log_retention":"30d","trace_retention":"30d","metric_retention":"30d","max_stored_bytes":"512MiB"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "the exact retry is idempotent");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&body).unwrap()["result"],
            "duplicate"
        );

        let (status, _) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":3,"retention":"7d","log_retention":"7d","trace_retention":"7d","metric_retention":"7d","max_stored_bytes":"1GiB"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "a revision cannot carry new values");

        assert_eq!(policy.view(&tenant("acme")).unwrap().revision, 3);
    }

    #[tokio::test]
    async fn project_policy_ownership_and_revoke_survive_retries_and_restart() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = Arc::new(TenantPolicy::for_test_with_store(storage.clone()));
        let state = state_with(policy.clone());
        let project_retention =
            "/signy/api/v1/admin/project-tenants/acme/retention";
        let project_revoke =
            "/signy/api/v1/admin/project-tenants/acme/access/revoke";

        let (status, body) = call(
            &state,
            "PUT",
            project_retention,
            r#"{"revision":1,"retention":"30d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["result"], "applied");
        assert_eq!(read_labels_as(&state, "acme").await, StatusCode::OK);

        let (status, _) = call(
            &state,
            "PUT",
            RETENTION_URI,
            r#"{"revision":2,"retention":"7d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);

        let (status, body) = call(
            &state,
            "PUT",
            project_retention,
            r#"{"revision":1,"retention":"30d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["result"], "duplicate");

        let (status, _) = call(&state, "POST", project_revoke, r#"{"fence":1}"#).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(read_labels_as(&state, "acme").await, StatusCode::FORBIDDEN);

        let (status, _) = call(
            &state,
            "PUT",
            project_retention,
            r#"{"revision":2,"retention":"7d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::GONE);

        let (status, _) = call(&state, "POST", project_revoke, r#"{"fence":1}"#).await;
        assert_eq!(status, StatusCode::OK);

        let reloaded = TenantPolicy::load(&Config::default(), Some(storage)).await.unwrap();
        assert!(!reloaded.is_tenant_allowed(&tenant("acme")));
        assert_eq!(reloaded.view(&tenant("acme")).unwrap().retention, "30d");
    }

    #[tokio::test]
    async fn revoke_creates_a_durable_tombstone_before_registration() {
        let storage = Arc::new(ObjectStorage::in_memory());
        let policy = Arc::new(TenantPolicy::for_test_with_store(storage.clone()));
        let state = state_with(policy);
        let revoke = "/signy/api/v1/admin/project-tenants/new/access/revoke";
        let retention = "/signy/api/v1/admin/project-tenants/new/retention";

        let (status, _) = call(&state, "POST", revoke, r#"{"fence":4}"#).await;
        assert_eq!(status, StatusCode::OK);
        let (status, _) = call(
            &state,
            "PUT",
            retention,
            r#"{"revision":1,"retention":"30d"}"#,
        )
        .await;
        assert_eq!(status, StatusCode::GONE);
    }
