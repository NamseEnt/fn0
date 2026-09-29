    use super::*;
    use crate::tenant::test_tenant;
    use crate::memtable::Labels;
    use crate::part::Row;
    use std::path::{Path, PathBuf};

    fn temp_dir(label: &str) -> PathBuf {
        crate::test_support::temp_dir(&format!("object-store-{label}"))
    }

    fn row(line: &str) -> Row {
        let _labels: Labels = [("app".to_string(), "remote".to_string())]
            .into_iter()
            .collect();
        Row {
            tenant: test_tenant(),
            timestamp_ns: 1_700_000_000_000_000_000,
            line: line.to_string(),
            structured_metadata: Vec::new(),
        }
    }

    #[tokio::test]
    async fn flush_transaction_rolls_back_partial_cross_domain_publication() {
        let root = temp_dir("flush-transaction");
        let parts_root = root.join("parts");
        let parts = crate::part::flush_rows(vec![row("transaction")], &parts_root, 16).unwrap();
        let storage = ObjectStorage::in_memory();
        storage.publish(&parts, &[]).await.unwrap();
        let transaction = FlushTransaction {
            offset: 10,
            log_parts: parts.iter().map(ManifestPart::from).collect(),
            trace_parts: Vec::new(),
            metric_parts: Vec::new(),
        };
        write_flush_transaction(&root, &transaction).unwrap();

        storage.reconcile_flush_transaction(&root, 0).await.unwrap();

        assert!(storage.load_manifest().await.unwrap().parts.is_empty());
        assert!(!parts[0].dir.exists());
        assert!(!root.join(FLUSH_TRANSACTION_FILE).exists());
    }

    #[tokio::test]
    async fn committed_flush_transaction_is_only_cleared_after_checkpoint() {
        let root = temp_dir("flush-committed");
        let parts_root = root.join("parts");
        let parts = crate::part::flush_rows(vec![row("committed")], &parts_root, 16).unwrap();
        let storage = ObjectStorage::in_memory();
        storage.publish(&parts, &[]).await.unwrap();
        let transaction = FlushTransaction {
            offset: 10,
            log_parts: parts.iter().map(ManifestPart::from).collect(),
            trace_parts: Vec::new(),
            metric_parts: Vec::new(),
        };
        write_flush_transaction(&root, &transaction).unwrap();

        storage.reconcile_flush_transaction(&root, 10).await.unwrap();

        assert_eq!(storage.load_manifest().await.unwrap().parts.len(), 1);
        assert!(parts[0].dir.exists());
        assert!(!root.join(FLUSH_TRANSACTION_FILE).exists());
    }

    fn metric_parts_for(root: &Path, base_ts: i64) -> Vec<crate::series_part::SeriesPart> {
        use crate::series::{METRIC_NAME_LABEL, MetricSample, MetricValue, SampleKind, SeriesLabels};
        let memtable = crate::series::SeriesMemTable::new();
        let labels = SeriesLabels::from_pairs(vec![
            (METRIC_NAME_LABEL.to_string(), "queue_depth".to_string()),
            ("instance".to_string(), "a".to_string()),
        ]);
        memtable.insert(
            (0..3)
                .map(|index| MetricSample {
                    tenant: test_tenant(),
                    labels: labels.clone(),
                    ts_ns: base_ts + index * 1_000_000_000,
                    value: MetricValue::Scalar(index as f64),
                    kind: SampleKind::Gauge,
                    datapoint_index: 0,
                })
                .collect(),
        );
        let snapshot = memtable.begin_flush();
        let parts = crate::series_part::flush_series_snapshot(&snapshot, root).unwrap();
        memtable.commit_flush();
        parts
    }

    #[tokio::test]
    async fn metric_parts_offload_restore_and_evict_like_the_other_signals() {
        let root = temp_dir("metric-offload");
        let storage = ObjectStorage::in_memory();
        let parts = metric_parts_for(&root, 1_700_000_000_000_000_000);
        let manifest = storage.publish_metric_parts(&parts, &[]).await.unwrap();
        assert_eq!(manifest.parts.len(), 1);
        let id = manifest.parts[0].id.clone();

        // The body leaves; the catalog restore plans without it, the body
        // restore brings the samples back byte-identical.
        std::fs::remove_file(parts[0].data_path()).unwrap();
        storage.restore_metric_catalog(&root).await.unwrap();
        storage
            .restore_metric_parts(&root, &std::iter::once(id).collect())
            .await
            .unwrap();
        let reader = crate::series_part::SeriesPartReader::open(
            crate::series_part::load_series_part(&parts[0].dir).unwrap(),
        )
        .unwrap();
        let catalog = reader.tenant_catalog(&test_tenant());
        assert_eq!(reader.read_series(catalog.get(0).unwrap().chunk).unwrap().len(), 3);

        storage
            .evict_metric_cache(&root, 0, &[parts[0].dir.clone()])
            .unwrap();
        assert!(!parts[0].data_path().exists());
        assert!(parts[0].index_path().exists(), "the catalog stays local");
    }

    #[tokio::test]
    async fn flush_transaction_rolls_back_uncommitted_metric_additions() {
        let root = temp_dir("metric-flush-transaction");
        let metrics_root = root.join("metrics");
        let storage = ObjectStorage::in_memory();
        let parts = metric_parts_for(&metrics_root, 1_700_000_000_000_000_000);
        storage.publish_metric_parts(&parts, &[]).await.unwrap();
        let transaction = FlushTransaction {
            offset: 10,
            log_parts: Vec::new(),
            trace_parts: Vec::new(),
            metric_parts: parts.iter().map(MetricManifestPart::from).collect(),
        };
        write_flush_transaction(&root, &transaction).unwrap();

        storage.reconcile_flush_transaction(&root, 0).await.unwrap();

        assert!(storage.load_metric_manifest().await.unwrap().parts.is_empty());
        assert!(!parts[0].dir.exists());
        assert!(!root.join(FLUSH_TRANSACTION_FILE).exists());
    }

    /// An intent file written before the metrics signal existed carries no
    /// `metric_parts` key, and must parse as the empty list it meant.
    #[test]
    fn a_pre_metrics_flush_transaction_still_parses() {
        let parsed: FlushTransaction = serde_json::from_str(
            r#"{"offset":7,"log_parts":[],"trace_parts":[]}"#,
        )
        .unwrap();
        assert!(parsed.metric_parts.is_empty());
    }

    // ---------------------------------------------------------------------
    // The crate boundary.
    //
    // Whether `object_store` implements S3 correctly is its problem. Ours is
    // everything on this side of the call: what we hand it, and how we read
    // what it hands back. A mistake here is silent — every write still
    // succeeds — so this seam is tested more finely than anything around it.
    // ---------------------------------------------------------------------

    /// The single most dangerous line in the seam. `Overwrite` where `Update`
    /// belongs disables compare-and-swap outright, and nothing downstream can
    /// tell: the write succeeds either way. The `file://` branch is a
    /// deliberate opt-out for a single-process development backend; every
    /// other scheme must condition.
    #[test]
    fn put_mode_conditions_on_every_backend_except_the_local_one() {
        let version = UpdateVersion {
            e_tag: Some("etag-1".to_string()),
            version: None,
        };

        let remote = ObjectStorage::in_memory();
        assert!(
            matches!(remote.put_mode(Some(version.clone())), PutMode::Update(_)),
            "a known version on a remote store must produce a conditional update"
        );
        assert!(
            matches!(remote.put_mode(None), PutMode::Create),
            "no known version means the object must not exist yet"
        );

        let dir = temp_dir("put-mode-local");
        let local = ObjectStorage::from_url(&format!("file://{}", dir.display())).unwrap();
        assert!(
            matches!(local.put_mode(Some(version)), PutMode::Overwrite),
            "file:// opts out of CAS deliberately"
        );
        assert!(matches!(local.put_mode(None), PutMode::Create));
    }

    /// `local_manifest_overwrite` is what `put_mode` above branches on, so the
    /// scheme test and the mode test are the same guarantee seen from two
    /// sides. A scheme wrongly classified as local silently drops CAS.
    #[test]
    fn only_the_file_scheme_opts_out_of_conditional_writes() {
        let dir = temp_dir("scheme-local");
        let local = ObjectStorage::from_url(&format!("file://{}", dir.display())).unwrap();
        assert!(local.local_manifest_overwrite);

        for url in [
            "s3://bucket/prefix",
            "s3://bucket",
            "memory:///",
        ] {
            let store = ObjectStorage::from_url(url)
                .unwrap_or_else(|error| panic!("{url} should build: {error}"));
            assert!(
                !store.local_manifest_overwrite,
                "{url} must keep compare-and-swap"
            );
        }
    }

    #[test]
    fn from_url_rejects_what_it_cannot_address() {
        assert!(ObjectStorage::from_url("not a url").is_err());
        assert!(ObjectStorage::from_url("").is_err());
        // A scheme object_store does not know must fail loudly rather than
        // fall back to something that happens to work locally.
        assert!(ObjectStorage::from_url("ftp://host/path").is_err());
    }

    /// The prefix comes from the URL path and every key is built by joining
    /// onto it. Getting the join wrong puts this instance's objects somewhere
    /// other than where its own reads look, or — worse — on top of another
    /// deployment sharing the bucket.
    #[test]
    fn keys_are_built_under_the_url_prefix() {
        let nested = ObjectStorage::from_url("s3://bucket/team/signy").unwrap();
        assert_eq!(
            nested.manifest_path().as_ref(),
            format!("team/signy/{MANIFEST_FILE}")
        );
        assert_eq!(
            nested
                .part_path(
                    &ManifestPart {
                        id: "part-1".to_string(),
                        partition: "2026-01-01".to_string(),
                    },
                    DATA_FILE
                )
                .as_ref(),
            format!("team/signy/parts/2026-01-01/part-1/{DATA_FILE}")
        );
        assert_eq!(
            nested.tenant_policy_path("acme").as_ref(),
            format!("team/signy/{TENANT_POLICY_PREFIX}/acme.json")
        );

        // A bucket-root URL has an empty prefix, and the join must not leave a
        // leading separator behind.
        let root = ObjectStorage::from_url("s3://bucket").unwrap();
        assert_eq!(root.manifest_path().as_ref(), MANIFEST_FILE);
        assert!(!root.manifest_path().as_ref().starts_with('/'));
    }

    /// `object_store` reports "no such object" as an error, and for the
    /// manifest that is not an error at all — it is the first boot. Reading it
    /// as a failure would make an empty prefix unstartable; reading any *other*
    /// error as an empty manifest would silently discard every registered part.
    #[tokio::test]
    async fn a_missing_manifest_is_the_first_boot_and_nothing_else_is() {
        let storage = ObjectStorage::in_memory();
        let manifest = storage
            .load_manifest()
            .await
            .expect("an absent manifest is a valid empty state");
        assert!(manifest.parts.is_empty());
        assert_eq!(manifest.generation, 0);

        // A present-but-unreadable manifest is the opposite case.
        storage
            .store
            .put(&storage.manifest_path(), Bytes::from_static(b"{ not json").into())
            .await
            .unwrap();
        assert!(
            storage.load_manifest().await.is_err(),
            "a corrupt manifest must never read as an empty one"
        );
    }

    #[tokio::test]
    async fn catalog_reconstructs_a_missing_replica() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("catalog-replica").join("parts");
        let parts = part::flush_rows(vec![row("replicated")], &root, 100).unwrap();
        storage.publish(&parts, &[]).await.unwrap();
        let state = storage.load_catalog_state().await.unwrap();
        let missing = storage.catalog_commit_path("b", state.generation);
        storage.store.delete(&missing).await.unwrap();

        let recovered = ObjectStorage::sharing_store_for_test(storage.store.clone());
        let manifest = recovered.load_manifest().await.unwrap();

        assert_eq!(manifest.parts.len(), 1);
        assert!(recovered.store.get(&missing).await.is_ok());
    }

    #[tokio::test]
    async fn catalog_refuses_divergent_valid_replicas() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("catalog-divergence").join("parts");
        let parts = part::flush_rows(vec![row("divergent")], &root, 100).unwrap();
        storage.publish(&parts, &[]).await.unwrap();
        let state = storage.load_catalog_state().await.unwrap();
        let path = storage.catalog_commit_path("b", state.generation);
        let bytes = storage.store.get(&path).await.unwrap().bytes().await.unwrap();
        let mut commit: CatalogCommit = serde_json::from_slice(&bytes).unwrap();
        commit.mutation.transaction_id = "different-transaction".to_string();
        let divergent = serde_json::to_vec(&seal_catalog_commit(commit).unwrap()).unwrap();
        storage.store.put(&path, divergent.into()).await.unwrap();

        let recovered = ObjectStorage::sharing_store_for_test(storage.store.clone());
        assert!(recovered.load_manifest().await.is_err());
    }

    async fn commit_alternating_trace_generations(storage: &ObjectStorage, count: u64) {
        let descriptor = TraceManifestPart {
            id: "catalog-prune-part".to_string(),
            partition: "2026-01-01".to_string(),
        };
        for generation in 1..=count {
            let mutation = if generation % 2 == 1 {
                CatalogMutation {
                    trace_added: vec![descriptor.clone()],
                    ..Default::default()
                }
            } else {
                CatalogMutation {
                    trace_removed: vec![descriptor.id.clone()],
                    ..Default::default()
                }
            };
            storage.commit_catalog_mutation(mutation).await.unwrap();
        }
    }

    /// Pruning has to leave a catalog a startup can still read when the newest
    /// snapshot is the one that went bad, which is what the second verified
    /// snapshot and the commits after it are kept for.
    #[tokio::test]
    async fn catalog_pruning_keeps_the_fallback_snapshot_and_its_commits() {
        let storage = ObjectStorage::in_memory();
        let newest_generation = CATALOG_SNAPSHOT_INTERVAL * 3 + 5;
        commit_alternating_trace_generations(&storage, newest_generation).await;
        let state_before_pruning = storage.load_catalog_state().await.unwrap();
        let min_age = std::time::Duration::from_secs(8 * 24 * 60 * 60);

        assert_eq!(
            storage
                .prune_catalog_at(min_age, chrono::Utc::now())
                .await
                .unwrap(),
            0,
            "nothing younger than the minimum age is deleted"
        );

        let removed = storage
            .prune_catalog_at(min_age, chrono::Utc::now() + chrono::Duration::days(30))
            .await
            .unwrap();
        let oldest_kept_snapshot = CATALOG_SNAPSHOT_INTERVAL * 2;
        assert_eq!(
            removed as u64,
            2 + (oldest_kept_snapshot - 1) * 2,
            "one older snapshot and every commit before the fallback snapshot, on both replicas"
        );

        let newest_snapshot = CATALOG_SNAPSHOT_INTERVAL * 3;
        for replica in ["a", "b"] {
            storage
                .store
                .put(
                    &storage.catalog_snapshot_path(replica, newest_snapshot),
                    Bytes::from_static(b"damaged").into(),
                )
                .await
                .unwrap();
        }
        let restarted = ObjectStorage::sharing_store_for_test(storage.store.clone());
        assert_eq!(
            restarted.load_catalog_state().await.unwrap(),
            state_before_pruning,
            "a startup falls back to the older snapshot and replays the kept commits"
        );
    }

    #[tokio::test]
    async fn catalog_pruning_needs_two_verified_snapshots() {
        let storage = ObjectStorage::in_memory();
        commit_alternating_trace_generations(&storage, CATALOG_SNAPSHOT_INTERVAL * 2 + 1).await;
        storage
            .store
            .put(
                &storage.catalog_snapshot_path("b", CATALOG_SNAPSHOT_INTERVAL),
                Bytes::from_static(b"damaged").into(),
            )
            .await
            .unwrap();

        let removed = storage
            .prune_catalog_at(
                std::time::Duration::from_secs(60),
                chrono::Utc::now() + chrono::Duration::days(30),
            )
            .await
            .unwrap();

        assert_eq!(removed, 0, "one intact snapshot is not enough to prune behind");
    }

    #[tokio::test]
    async fn one_catalog_generation_carries_all_signal_mutations() {
        let storage = ObjectStorage::in_memory();
        let state = storage
            .commit_catalog_mutation(CatalogMutation {
                transaction_id: "all-signals".to_string(),
                writer_epoch: 0,
                log_added: vec![ManifestPart {
                    id: "log-part".to_string(),
                    partition: "2026-01-01".to_string(),
                }],
                trace_added: vec![TraceManifestPart {
                    id: "trace-part".to_string(),
                    partition: "2026-01-01".to_string(),
                }],
                metric_added: vec![MetricManifestPart {
                    id: "metric-part".to_string(),
                    partition: "2026-01-01".to_string(),
                }],
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(state.generation, 1);
        assert_eq!(state.manifest.generation, 1);
        assert_eq!(state.trace_manifest.generation, 1);
        assert_eq!(state.metric_manifest.generation, 1);
        assert_eq!(
            storage
                .catalog_generations(CATALOG_COMMITS_PREFIX, None)
                .await
                .unwrap(),
            vec![1]
        );
    }

    #[tokio::test]
    #[ignore = "requires a dedicated R2 test bucket and credentials"]
    async fn r2_catalog_backend_smoke() {
        let url = std::env::var("SIGNY_R2_TEST_URL")
            .expect("SIGNY_R2_TEST_URL must point at a disposable R2 test prefix");
        let storage = ObjectStorage::from_url(&url).unwrap();
        storage.verify_catalog_protection().unwrap();
        storage.verify_conditional_put().await.unwrap();
        storage.claim_writer_epoch().await.unwrap();
        storage.verify_catalog_listing().await.unwrap();
    }

    #[test]
    fn object_store_environment_keys_are_normalized_and_explicit_values_win() {
        let options: BTreeMap<_, _> = normalized_object_store_options([
            ("AWS_ACCESS_KEY_ID".to_string(), "aws-key".to_string()),
            ("AWS_REGION".to_string(), "ap-northeast-2".to_string()),
            (
                "OBJECT_STORE_AWS_ACCESS_KEY_ID".to_string(),
                "explicit-key".to_string(),
            ),
            (
                "OBJECT_STORE_ENDPOINT".to_string(),
                "http://minio:9000".to_string(),
            ),
            ("UNRELATED".to_string(), "ignored".to_string()),
        ])
        .into_iter()
        .collect();

        assert_eq!(options.get("aws_access_key_id").unwrap(), "explicit-key");
        assert_eq!(options.get("aws_region").unwrap(), "ap-northeast-2");
        assert_eq!(options.get("endpoint").unwrap(), "http://minio:9000");
        assert!(!options.contains_key("unrelated"));
    }

    #[tokio::test]
    async fn restores_queryable_part_after_cache_is_deleted() {
        let storage = ObjectStorage::in_memory();
        let source = temp_dir("source").join("parts");
        let parts = part::flush_rows(vec![row("from object store")], &source, 100).unwrap();
        storage.publish(&parts, &[]).await.unwrap();

        std::fs::remove_dir_all(&source).unwrap();
        let manifest = storage.restore_catalog(&source).await.unwrap();
        assert!(
            !source
                .join(&manifest.parts[0].partition)
                .join(&manifest.parts[0].id)
                .join(DATA_FILE)
                .exists()
        );
        let ids = manifest.parts.iter().map(|part| part.id.clone()).collect();
        storage.restore_parts(&source, &ids).await.unwrap();
        let registry =
            crate::part_registry::PartRegistry::load_from_manifest(&source, &manifest).unwrap();
        let result = registry
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
            .unwrap();
        assert_eq!(result[0].entries[0].line, "from object store");
    }

    #[tokio::test]
    async fn manifest_replace_is_atomic_and_increments_generation() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("replace").join("parts");
        let old = part::flush_rows(vec![row("old")], &root, 100).unwrap();
        let first = storage.publish(&old, &[]).await.unwrap();
        assert_eq!(first.generation, 1);

        let old_id = old[0].meta.id.clone();
        let mut replacement_row = row("new");
        replacement_row.timestamp_ns += 1;
        let new = part::flush_rows(vec![replacement_row], &root, 100).unwrap();
        let second = storage.publish(&new, &[old_id]).await.unwrap();
        assert_eq!(second.generation, 2);
        assert_eq!(second.parts.len(), 1);
        assert_eq!(second.parts[0].id, new[0].meta.id);
    }

    #[tokio::test]
    async fn competing_replacement_cannot_publish_duplicate_rows() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("replacement-conflict").join("parts");
        let old = part::flush_rows(vec![row("old")], &root, 100).unwrap();
        storage.publish(&old, &[]).await.unwrap();
        let old_id = old[0].meta.id.clone();

        let mut first_row = row("first replacement");
        first_row.timestamp_ns += 1;
        let first = part::flush_rows(vec![first_row], &root, 100).unwrap();
        storage
            .publish(&first, std::slice::from_ref(&old_id))
            .await
            .unwrap();

        // Repeating the exact operation is idempotent, but a different output
        // for the already-consumed input generation is a conflict.
        storage
            .publish(&first, std::slice::from_ref(&old_id))
            .await
            .unwrap();
        let mut second_row = row("second replacement");
        second_row.timestamp_ns += 2;
        let second = part::flush_rows(vec![second_row], &root, 100).unwrap();
        let error = storage
            .publish(&second, std::slice::from_ref(&old_id))
            .await
            .unwrap_err();
        assert!(error.contains("manifest replacement conflict"));

        let manifest = storage.load_manifest().await.unwrap();
        assert_eq!(manifest.parts.len(), 1);
        assert_eq!(manifest.parts[0].id, first[0].meta.id);
    }

    #[tokio::test]
    async fn reconciliation_publishes_local_only_parts_without_resurrecting_stale_parts() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile").join("parts");
        let old = part::flush_rows(vec![row("old")], &root, 100).unwrap();
        storage.publish(&old, &[]).await.unwrap();
        let old_id = old[0].meta.id.clone();

        let mut replacement_row = row("replacement");
        replacement_row.timestamp_ns += 1;
        let replacement = part::flush_rows(vec![replacement_row], &root, 100).unwrap();
        storage
            .publish(&replacement, std::slice::from_ref(&old_id))
            .await
            .unwrap();

        let mut local_row = row("written while object store was disabled");
        local_row.timestamp_ns += 2;
        let local_only = part::flush_rows(vec![local_row], &root, 100).unwrap();
        let reconciled = storage.reconcile_local_cache(&root).await.unwrap();

        let ids: HashSet<_> = reconciled
            .parts
            .iter()
            .map(|part| part.id.as_str())
            .collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(replacement[0].meta.id.as_str()));
        assert!(ids.contains(local_only[0].meta.id.as_str()));
        assert!(!ids.contains(old_id.as_str()));
    }

    #[tokio::test]
    async fn reconciliation_finishes_an_interrupted_merge_replacement() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile-uncommitted-merge").join("parts");
        let old = part::flush_rows(vec![row("old")], &root, 100).unwrap();
        storage.publish(&old, &[]).await.unwrap();
        let old_dirs: Vec<_> = old.iter().map(|part| part.dir.clone()).collect();

        let mut replacement_row = row("uncommitted replacement");
        replacement_row.timestamp_ns += 1;
        let replacement =
            part::flush_rows_with_merge_tombstone(vec![replacement_row], &root, 100, &old_dirs)
                .unwrap();
        assert!(replacement[0].dir.join(part::MERGE_TOMBSTONE_FILE).exists());

        let reconciled = storage.reconcile_local_cache(&root).await.unwrap();
        assert_eq!(reconciled.parts.len(), 1);
        assert_eq!(reconciled.parts[0].id, replacement[0].meta.id);
        assert!(!old[0].dir.exists());
        assert!(!replacement[0].dir.join(part::MERGE_TOMBSTONE_FILE).exists());
    }

    #[tokio::test]
    async fn reconciliation_migrates_a_local_only_merge_into_an_empty_manifest() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile-local-only-merge").join("parts");
        let old = part::flush_rows(vec![row("old local")], &root, 100).unwrap();
        let mut replacement_row = row("merged while local only");
        replacement_row.timestamp_ns += 1;
        let replacement = part::flush_rows_with_merge_tombstone(
            vec![replacement_row],
            &root,
            100,
            &[old[0].dir.clone()],
        )
        .unwrap();

        let manifest = storage.reconcile_local_cache(&root).await.unwrap();
        assert_eq!(manifest.generation, 1);
        assert_eq!(manifest.parts.len(), 1);
        assert_eq!(manifest.parts[0].id, replacement[0].meta.id);
        assert!(!old[0].dir.exists());
    }

    #[tokio::test]
    async fn reconciliation_migrates_an_unbalanced_local_only_merge_tree() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile-unbalanced-local-merge").join("parts");
        let first = part::flush_rows(vec![row("first")], &root, 100).unwrap();
        let second = part::flush_rows(vec![row("second")], &root, 100).unwrap();
        let third = part::flush_rows(vec![row("third")], &root, 100).unwrap();
        let middle = part::flush_rows_with_merge_tombstone(
            vec![row("middle")],
            &root,
            100,
            &[first[0].dir.clone(), second[0].dir.clone()],
        )
        .unwrap();
        let newest = part::flush_rows_with_merge_tombstone(
            vec![row("newest")],
            &root,
            100,
            &[middle[0].dir.clone(), third[0].dir.clone()],
        )
        .unwrap();

        let manifest = storage.reconcile_local_cache(&root).await.unwrap();
        assert_eq!(manifest.generation, 1);
        assert_eq!(manifest.parts.len(), 1);
        assert_eq!(manifest.parts[0].id, newest[0].meta.id);
        assert!(!first[0].dir.exists());
        assert!(!second[0].dir.exists());
        assert!(!third[0].dir.exists());
        assert!(!middle[0].dir.exists());
    }

    #[tokio::test]
    async fn reconciliation_migrates_independent_local_only_merge_trees() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile-independent-local-merges").join("parts");
        let first_old = part::flush_rows(vec![row("first old")], &root, 100).unwrap();
        let second_old = part::flush_rows(vec![row("second old")], &root, 100).unwrap();

        let first = part::flush_rows_with_merge_tombstone(
            vec![row("first replacement")],
            &root,
            100,
            &[first_old[0].dir.clone()],
        )
        .unwrap();
        let second = part::flush_rows_with_merge_tombstone(
            vec![row("second replacement")],
            &root,
            100,
            &[second_old[0].dir.clone()],
        )
        .unwrap();

        let manifest = storage.reconcile_local_cache(&root).await.unwrap();
        let active: HashSet<_> = manifest.parts.iter().map(|part| part.id.as_str()).collect();
        assert_eq!(
            manifest.generation, 1,
            "all roots must be seeded by one CAS"
        );
        assert_eq!(active.len(), 2);
        assert!(active.contains(first[0].meta.id.as_str()));
        assert!(active.contains(second[0].meta.id.as_str()));
        assert!(!first_old[0].dir.exists());
        assert!(!second_old[0].dir.exists());
    }

    #[tokio::test]
    async fn reconciliation_resumes_after_atomic_local_only_root_seed() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile-after-root-seed").join("parts");
        let first_old = part::flush_rows(vec![row("first old")], &root, 100).unwrap();
        let second_old = part::flush_rows(vec![row("second old")], &root, 100).unwrap();
        let first = part::flush_rows_with_merge_tombstone(
            vec![row("first replacement")],
            &root,
            100,
            &[first_old[0].dir.clone()],
        )
        .unwrap();
        let second = part::flush_rows_with_merge_tombstone(
            vec![row("second replacement")],
            &root,
            100,
            &[second_old[0].dir.clone()],
        )
        .unwrap();

        // State after the atomic root manifest CAS but before local tombstone
        // cleanup, equivalent to a crash at that reconciliation boundary.
        let seeded = storage
            .publish(&[first[0].clone(), second[0].clone()], &[])
            .await
            .unwrap();
        assert_eq!(seeded.generation, 1);
        assert!(first_old[0].dir.exists());
        assert!(second_old[0].dir.exists());

        let reconciled = storage.reconcile_local_cache(&root).await.unwrap();
        let active: HashSet<_> = reconciled
            .parts
            .iter()
            .map(|part| part.id.as_str())
            .collect();
        assert_eq!(active.len(), 2);
        assert!(active.contains(first[0].meta.id.as_str()));
        assert!(active.contains(second[0].meta.id.as_str()));
        assert!(!first_old[0].dir.exists());
        assert!(!second_old[0].dir.exists());
    }

    #[tokio::test]
    async fn reconciliation_rejects_overlapping_local_only_merge_roots_before_publish() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile-overlapping-local-merges").join("parts");
        let first_old = part::flush_rows(vec![row("first old")], &root, 100).unwrap();
        let shared_old = part::flush_rows(vec![row("shared old")], &root, 100).unwrap();
        let second_old = part::flush_rows(vec![row("second old")], &root, 100).unwrap();

        part::flush_rows_with_merge_tombstone(
            vec![row("first replacement")],
            &root,
            100,
            &[first_old[0].dir.clone(), shared_old[0].dir.clone()],
        )
        .unwrap();
        part::flush_rows_with_merge_tombstone(
            vec![row("second replacement")],
            &root,
            100,
            &[shared_old[0].dir.clone(), second_old[0].dir.clone()],
        )
        .unwrap();

        let error = storage.reconcile_local_cache(&root).await.unwrap_err();
        assert!(error.contains("overlapping local-only merge roots"));
        assert!(storage.load_manifest().await.unwrap().parts.is_empty());
    }

    #[tokio::test]
    async fn reconciliation_rejects_a_stale_competing_merge_replacement() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile-competing-merge").join("parts");
        let old = part::flush_rows(vec![row("old")], &root, 100).unwrap();
        storage.publish(&old, &[]).await.unwrap();
        let old_id = old[0].meta.id.clone();

        let mut winner_row = row("remote winner");
        winner_row.timestamp_ns += 1;
        let winner = part::flush_rows(vec![winner_row], &root, 100).unwrap();
        storage
            .publish(&winner, std::slice::from_ref(&old_id))
            .await
            .unwrap();

        let mut loser_row = row("stale local loser");
        loser_row.timestamp_ns += 2;
        let old_dirs = vec![old[0].dir.clone()];
        let loser =
            part::flush_rows_with_merge_tombstone(vec![loser_row], &root, 100, &old_dirs).unwrap();

        let error = storage.reconcile_local_cache(&root).await.unwrap_err();
        assert!(error.contains("local merge replacement conflict"));

        let manifest = storage.load_manifest().await.unwrap();
        assert_eq!(manifest.parts.len(), 1);
        assert_eq!(manifest.parts[0].id, winner[0].meta.id);
        assert!(loser[0].dir.exists(), "ambiguous local data was deleted");
    }

    #[tokio::test]
    async fn reconciliation_accepts_an_already_active_merge_chain_descendant() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile-active-descendant").join("parts");
        let oldest = part::flush_rows(vec![row("oldest")], &root, 100).unwrap();
        storage.publish(&oldest, &[]).await.unwrap();

        let mut middle_row = row("middle");
        middle_row.timestamp_ns += 1;
        let middle = part::flush_rows_with_merge_tombstone(
            vec![middle_row],
            &root,
            100,
            &[oldest[0].dir.clone()],
        )
        .unwrap();
        storage
            .publish(&middle, &[oldest[0].meta.id.clone()])
            .await
            .unwrap();

        let mut newest_row = row("newest");
        newest_row.timestamp_ns += 2;
        let newest = part::flush_rows_with_merge_tombstone(
            vec![newest_row],
            &root,
            100,
            &[middle[0].dir.clone()],
        )
        .unwrap();
        storage
            .publish(&newest, &[middle[0].meta.id.clone()])
            .await
            .unwrap();

        let manifest = storage.reconcile_local_cache(&root).await.unwrap();
        assert_eq!(manifest.parts.len(), 1);
        assert_eq!(manifest.parts[0].id, newest[0].meta.id);
        assert!(!oldest[0].dir.exists());
        assert!(!middle[0].dir.exists());
    }

    #[tokio::test]
    async fn reconciliation_resumes_a_partial_local_part_upload() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile-partial-upload").join("parts");
        let local = part::flush_rows(vec![row("partial upload")], &root, 100).unwrap();
        let descriptor = ManifestPart::from(&local[0]);
        let data = tokio::fs::read(local[0].data_path()).await.unwrap();
        storage
            .store
            .put_opts(
                &storage.part_path(&descriptor, DATA_FILE),
                data.into(),
                PutOptions {
                    mode: PutMode::Create,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let manifest = storage.reconcile_local_cache(&root).await.unwrap();

        assert_eq!(manifest.parts.len(), 1);
        assert_eq!(manifest.parts[0].id, local[0].meta.id);
        assert!(!local[0].dir.join(UPLOAD_MARKER_FILE).exists());
    }

    #[tokio::test]
    async fn reconciliation_resumes_a_fully_uploaded_uncommitted_part() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile-complete-upload").join("parts");
        let local = part::flush_rows(vec![row("uploaded before crash")], &root, 100).unwrap();

        // State after upload_part completed but before the manifest CAS.
        write_upload_marker(&local[0]).unwrap();
        storage.upload_part(&local[0]).await.unwrap();
        assert!(storage.load_manifest().await.unwrap().parts.is_empty());

        let manifest = storage.reconcile_local_cache(&root).await.unwrap();
        assert_eq!(manifest.parts.len(), 1);
        assert_eq!(manifest.parts[0].id, local[0].meta.id);
        assert!(!local[0].dir.join(UPLOAD_MARKER_FILE).exists());
    }

    #[tokio::test]
    async fn reconciliation_repairs_corrupt_active_catalog_before_local_scan() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("reconcile-corrupt-catalog").join("parts");
        let local = part::flush_rows(vec![row("repair catalog")], &root, 100).unwrap();
        storage.publish(&local, &[]).await.unwrap();
        std::fs::write(local[0].meta_path(), b"not json").unwrap();

        let manifest = storage.reconcile_local_cache(&root).await.unwrap();
        let registry =
            crate::part_registry::PartRegistry::load_from_manifest(&root, &manifest).unwrap();

        assert_eq!(registry.part_count(), 1);
    }

    #[tokio::test]
    async fn publish_rejects_different_bytes_under_an_existing_part_key() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("immutable-collision").join("parts");
        let local = part::flush_rows(vec![row("original")], &root, 100).unwrap();
        storage.publish(&local, &[]).await.unwrap();
        let descriptor = ManifestPart::from(&local[0]);
        storage
            .store
            .put_opts(
                &storage.part_path(&descriptor, DATA_FILE),
                bytes::Bytes::from_static(b"different").into(),
                PutOptions {
                    mode: PutMode::Overwrite,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let error = storage.publish(&local, &[]).await.unwrap_err();

        assert!(error.contains("immutable object collision"));
        assert!(
            local[0].dir.join(UPLOAD_MARKER_FILE).exists(),
            "publication intent must be durable before object upload"
        );
    }

    #[tokio::test]
    async fn a_file_past_the_chunk_streams_up_whole_and_compares_in_ranges() {
        let storage = ObjectStorage::in_memory();
        let dir = temp_dir("streamed-upload");
        std::fs::create_dir_all(&dir).unwrap();
        let local = dir.join("big.bin");
        // Past UPLOAD_CHUNK_BYTES and not a multiple of it, so the last chunk
        // is a short one.
        let len = UPLOAD_CHUNK_BYTES as usize + 7 * 1024 * 1024 + 13;
        let body: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
        std::fs::write(&local, &body).unwrap();
        let key = ObjectPath::from("streamed/big.bin");

        storage
            .upload_object(&local, len as u64, &key, "part", "part big file big.bin")
            .await
            .unwrap();

        let stored = storage.store.get(&key).await.unwrap().bytes().await.unwrap();
        assert_eq!(stored.len(), len, "a streamed upload must store every byte");
        assert_eq!(stored.as_ref(), body.as_slice(), "and store them unchanged");

        // Publishing the same part again is the resume path: it must accept
        // the identical object rather than fail on the existing key.
        storage
            .upload_object(&local, len as u64, &key, "part", "part big file big.bin")
            .await
            .unwrap();

        let mut different = body.clone();
        *different.last_mut().unwrap() ^= 0xff;
        std::fs::write(&local, &different).unwrap();
        let error = storage
            .upload_object(&local, len as u64, &key, "part", "part big file big.bin")
            .await
            .unwrap_err();
        assert!(
            error.contains("immutable object collision"),
            "a differing byte past the first chunk must still be caught: {error}"
        );
    }

    #[tokio::test]
    async fn a_failed_restore_leaves_no_part_directory_behind() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("failed-restore").join("parts");
        let local = part::flush_rows(vec![row("gone")], &root, 100).unwrap();
        storage.publish(&local, &[]).await.unwrap();
        let descriptor = ManifestPart::from(&local[0]);
        let dir = local[0].dir.clone();

        // What retention leaves behind: the manifest still names the part, the
        // objects under it are gone, and the local copy has been evicted.
        for file in PART_FILES {
            storage
                .store
                .delete(&storage.part_path(&descriptor, file))
                .await
                .unwrap();
        }
        std::fs::remove_dir_all(&dir).unwrap();

        let ids: HashSet<String> = [descriptor.id.clone()].into_iter().collect();
        let error = storage.restore_parts(&root, &ids).await.unwrap_err();
        assert!(
            error.contains("failed to download part"),
            "the restore must report the missing object: {error}"
        );
        assert!(
            !dir.exists(),
            "a restore that could not fetch anything must not leave {} behind: startup reconciliation refuses a part directory with no metadata",
            dir.display()
        );
    }

    #[tokio::test]
    async fn a_restore_commits_a_whole_part_or_nothing_at_all() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("restore-whole").join("parts");
        let local = part::flush_rows(vec![row("kept")], &root, 100).unwrap();
        storage.publish(&local, &[]).await.unwrap();
        let descriptor = ManifestPart::from(&local[0]);
        let dir = local[0].dir.clone();
        let ids: HashSet<String> = [descriptor.id.clone()].into_iter().collect();

        // A local part that is already valid opens without being touched.
        storage.restore_parts(&root, &ids).await.unwrap();
        assert!(dir.join(META_FILE).exists());

        // Evicted and restored: the directory comes back whole.
        std::fs::remove_dir_all(&dir).unwrap();
        storage.restore_parts(&root, &ids).await.unwrap();
        for file in PART_FILES {
            assert!(
                dir.join(file).exists(),
                "a committed part directory holds {file}"
            );
        }

        // The objects go away under it. Every retry must fail and leave the
        // final namespace exactly as it found it -- no directory, and so no
        // debris to accumulate.
        std::fs::remove_dir_all(&dir).unwrap();
        for file in PART_FILES {
            storage
                .store
                .delete(&storage.part_path(&descriptor, file))
                .await
                .unwrap();
        }
        for attempt in 0..3 {
            storage.restore_parts(&root, &ids).await.unwrap_err();
            assert!(
                !dir.exists(),
                "attempt {attempt} left a part directory behind"
            );
        }
        let partition = dir.parent().unwrap();
        let leftovers: Vec<_> = std::fs::read_dir(partition)
            .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
            .unwrap_or_default();
        assert!(
            leftovers.is_empty(),
            "failed restores must not accumulate anything: {leftovers:?}"
        );

        // The invariant startup depends on, stated the way startup reads it:
        // a part directory that exists holds its metadata. Reconciliation
        // refuses one that does not and retries until its budget is gone.
        for partition in std::fs::read_dir(&root).unwrap().flatten() {
            if !partition.path().is_dir() || partition.file_name() == ".tmp" {
                continue;
            }
            for part_dir in std::fs::read_dir(partition.path()).unwrap().flatten() {
                assert!(
                    part_dir.path().join(META_FILE).exists(),
                    "a part directory with no metadata is observable at {}",
                    part_dir.path().display()
                );
            }
        }
    }

    /// The contract the orphan collector's own comment states: an object that
    /// has left the active set survives `grace_period` from the moment it left
    /// it. Not from when it was written -- every part retention retires is
    /// older than the grace by construction, so measuring from the write time
    /// gives a retired object no grace at all.
    ///
    /// Time is injected rather than waited on, and `now` is pushed far past
    /// the object's write time so that the write-time check cannot be what
    /// keeps it alive.
    #[tokio::test]
    async fn a_retired_object_survives_its_grace_however_old_the_object_is() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("gc-grace").join("parts");
        let local = part::flush_rows(vec![row("retired")], &root, 100).unwrap();
        storage.publish(&local, &[]).await.unwrap();
        let descriptor = ManifestPart::from(&local[0]);
        let key = storage.part_path(&descriptor, DATA_FILE);
        let grace = std::time::Duration::from_secs(300);

        // Long enough after the write that the object counts as old.
        let retired_at = chrono::Utc::now() + chrono::Duration::hours(1);
        storage
            .publish(&[], std::slice::from_ref(&descriptor.id))
            .await
            .unwrap();

        storage
            .garbage_collect_orphans_at(grace, retired_at)
            .await
            .unwrap();
        assert!(
            storage.store.head(&key).await.is_ok(),
            "an object retired this instant must not be collected in the same pass"
        );

        storage
            .garbage_collect_orphans_at(grace, retired_at + chrono::Duration::seconds(299))
            .await
            .unwrap();
        assert!(
            storage.store.head(&key).await.is_ok(),
            "it must still be there a second before its grace is up"
        );

        storage
            .garbage_collect_orphans_at(grace, retired_at + chrono::Duration::seconds(301))
            .await
            .unwrap();
        assert!(
            storage.store.head(&key).await.is_err(),
            "and it must be collected once the grace has passed"
        );
    }

    /// The grace is a property of the store, not of the process that noticed
    /// the retirement: restarting must not hand a retired object a fresh
    /// grace, and must not collect it early either.
    #[tokio::test]
    async fn the_grace_outlives_the_process_that_started_it() {
        let dir = temp_dir("gc-grace-restart");
        let url = format!("file://{}", dir.display());
        let root = dir.join("cache").join("parts");
        let grace = std::time::Duration::from_secs(300);

        let storage = ObjectStorage::from_url(&url).unwrap();
        let local = part::flush_rows(vec![row("retired")], &root, 100).unwrap();
        storage.publish(&local, &[]).await.unwrap();
        let descriptor = ManifestPart::from(&local[0]);
        let key = storage.part_path(&descriptor, DATA_FILE);

        let retired_at = chrono::Utc::now() + chrono::Duration::hours(1);
        storage
            .publish(&[], std::slice::from_ref(&descriptor.id))
            .await
            .unwrap();
        storage
            .garbage_collect_orphans_at(grace, retired_at)
            .await
            .unwrap();

        // A new instance over the same store is what a restart looks like.
        let restarted = ObjectStorage::from_url(&url).unwrap();
        restarted
            .garbage_collect_orphans_at(grace, retired_at + chrono::Duration::seconds(60))
            .await
            .unwrap();
        assert!(
            restarted.store.head(&key).await.is_ok(),
            "a restart must not collect an object still inside its grace"
        );

        restarted
            .garbage_collect_orphans_at(grace, retired_at + chrono::Duration::seconds(301))
            .await
            .unwrap();
        assert!(
            restarted.store.head(&key).await.is_err(),
            "and must still collect it once the grace it started with has passed"
        );
    }

    #[tokio::test]
    async fn file_backend_can_update_an_existing_manifest() {
        let remote = temp_dir("file-backend");
        let url = url::Url::from_directory_path(&remote).unwrap();
        let storage = ObjectStorage::from_url(url.as_str()).unwrap();
        let root = temp_dir("file-backend-parts").join("parts");

        let old = part::flush_rows(vec![row("old")], &root, 100).unwrap();
        assert_eq!(storage.publish(&old, &[]).await.unwrap().generation, 1);

        let old_id = old[0].meta.id.clone();
        let mut replacement_row = row("new");
        replacement_row.timestamp_ns += 1;
        let new = part::flush_rows(vec![replacement_row], &root, 100).unwrap();
        let manifest = storage.publish(&new, &[old_id]).await.unwrap();

        assert_eq!(manifest.generation, 2);
        assert_eq!(storage.load_manifest().await.unwrap().generation, 2);
        assert_eq!(manifest.parts[0].id, new[0].meta.id);
    }

    #[tokio::test]
    async fn invalid_part_is_rejected_before_manifest_update() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("invalid-publish").join("parts");
        let parts = part::flush_rows(vec![row("corrupt me")], &root, 100).unwrap();
        std::fs::write(parts[0].data_path(), b"not parquet").unwrap();

        let error = storage.publish(&parts, &[]).await.unwrap_err();

        assert!(error.contains("refusing to publish invalid part"));
        let manifest = storage.load_manifest().await.unwrap();
        assert_eq!(manifest.generation, 0);
        assert!(manifest.parts.is_empty());
    }

    #[test]
    fn eviction_respects_cache_limit() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("evict").join("parts");
        let parts = part::flush_rows(vec![row("evict me")], &root, 100).unwrap();
        let eligible = vec![parts[0].dir.clone()];
        assert_eq!(storage.evict_cache(&root, 0, &eligible).unwrap(), 0);
        assert!(!parts[0].data_path().exists());
        assert!(parts[0].meta_path().exists());
        assert!(parts[0].index_path().exists());
    }

    #[test]
    fn eviction_preserves_parts_absent_from_the_registry() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("evict-stale").join("parts");
        let active = part::flush_rows(vec![row("active")], &root, 100).unwrap();
        let mut stale_row = row("stale");
        stale_row.timestamp_ns += 1;
        let stale = part::flush_rows(vec![stale_row], &root, 100).unwrap();
        let active_bytes = std::fs::metadata(active[0].data_path()).unwrap().len();
        let eligible = vec![active[0].dir.clone()];

        assert_eq!(
            storage.evict_cache(&root, u64::MAX, &eligible).unwrap(),
            active_bytes
        );
        assert!(active[0].dir.exists());
        assert!(stale[0].dir.exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn cache_restore_and_eviction_reject_symlinked_partitions() {
        use std::os::unix::fs::symlink;

        let storage = ObjectStorage::in_memory();
        let source = temp_dir("symlink-source").join("parts");
        let parts = part::flush_rows(vec![row("remote")], &source, 100).unwrap();
        let manifest = storage.publish(&parts, &[]).await.unwrap();
        let descriptor = &manifest.parts[0];

        let cache = temp_dir("symlink-cache").join("parts");
        std::fs::create_dir_all(&cache).unwrap();
        let outside = temp_dir("symlink-outside");
        let outside_part = outside.join(&descriptor.id);
        std::fs::create_dir_all(&outside_part).unwrap();
        let outside_data = outside_part.join(DATA_FILE);
        std::fs::write(&outside_data, b"must survive").unwrap();
        symlink(&outside, cache.join(&descriptor.partition)).unwrap();

        let restore_error = storage.restore_catalog(&cache).await.unwrap_err();
        assert!(restore_error.contains("symlinked cache directory"));
        assert_eq!(std::fs::read(&outside_data).unwrap(), b"must survive");

        // Eviction is driven by the registry's part directories now, so it
        // never enumerates partitions. A symlinked partition is still refused,
        // by the containment check on the part directory itself: canonicalizing
        // it resolves through the symlink and lands outside the cache root.
        let eligible = vec![cache.join(&descriptor.partition).join(&descriptor.id)];
        let eviction_error = storage.evict_cache(&cache, 0, &eligible).unwrap_err();
        assert!(
            eviction_error.contains("escapes root")
                || eviction_error.contains("refusing unsafe cache directory"),
            "{eviction_error}"
        );
        assert_eq!(std::fs::read(&outside_data).unwrap(), b"must survive");

        let empty_storage = ObjectStorage::in_memory();
        let migration_error = empty_storage
            .reconcile_local_cache(&cache)
            .await
            .unwrap_err();
        assert!(migration_error.contains("symlinked cache partition"));
        assert_eq!(std::fs::read(&outside_data).unwrap(), b"must survive");

        std::fs::remove_file(cache.join(&descriptor.partition)).unwrap();
        let cache_part = cache.join(&descriptor.partition).join(&descriptor.id);
        std::fs::create_dir_all(&cache_part).unwrap();
        symlink(&outside_data, cache_part.join(DATA_FILE)).unwrap();
        let file_error = storage.restore_catalog(&cache).await.unwrap_err();
        assert!(file_error.contains("symlinked cache file"));
        let eviction_error = storage.evict_cache(&cache, 0, &eligible).unwrap_err();
        assert!(eviction_error.contains("refusing symlinked cache file"));
        assert_eq!(std::fs::read(&outside_data).unwrap(), b"must survive");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn reconciliation_rejects_a_symlinked_upload_marker() {
        use std::os::unix::fs::symlink;

        let storage = ObjectStorage::in_memory();
        let root = temp_dir("symlink-upload-marker").join("parts");
        let parts = part::flush_rows(vec![row("local")], &root, 100).unwrap();
        let outside = temp_dir("symlink-marker-target").join("outside.txt");
        std::fs::write(&outside, b"must survive").unwrap();
        symlink(&outside, parts[0].dir.join(UPLOAD_MARKER_FILE)).unwrap();

        let error = storage.reconcile_local_cache(&root).await.unwrap_err();
        assert!(error.contains("symlinked cache file"));
        let publish_error = storage.publish(&parts, &[]).await.unwrap_err();
        assert!(publish_error.contains("symlinked upload marker"));
        assert_eq!(std::fs::read(&outside).unwrap(), b"must survive");
        assert!(storage.load_manifest().await.unwrap().parts.is_empty());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn migration_and_upload_reject_symlinked_immutable_part_files() {
        use std::os::unix::fs::symlink;

        let storage = ObjectStorage::in_memory();
        let root = temp_dir("symlink-immutable-part").join("parts");
        let parts = part::flush_rows(vec![row("local")], &root, 100).unwrap();
        let outside = temp_dir("symlink-immutable-target").join(DATA_FILE);
        std::fs::rename(parts[0].data_path(), &outside).unwrap();
        let expected = std::fs::read(&outside).unwrap();
        symlink(&outside, parts[0].data_path()).unwrap();

        let migration_error = storage.reconcile_local_cache(&root).await.unwrap_err();
        assert!(migration_error.contains("symlinked cache file"));
        assert_eq!(std::fs::read(&outside).unwrap(), expected);
        assert!(storage.load_manifest().await.unwrap().parts.is_empty());

        let upload_error = storage.publish(&parts, &[]).await.unwrap_err();
        assert!(upload_error.contains("unsafe part file"));
        assert_eq!(std::fs::read(&outside).unwrap(), expected);
        assert!(storage.load_manifest().await.unwrap().parts.is_empty());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn restore_reconcile_and_eviction_reject_a_symlinked_cache_root() {
        use std::os::unix::fs::symlink;

        let storage = ObjectStorage::in_memory();
        let outside_root = temp_dir("symlink-cache-root-outside").join("parts");
        let parts = part::flush_rows(vec![row("outside")], &outside_root, 100).unwrap();
        storage.publish(&parts, &[]).await.unwrap();
        let outside_data = parts[0].data_path();
        let link_parent = temp_dir("symlink-cache-root-link");
        let cache_link = link_parent.join("parts");
        symlink(&outside_root, &cache_link).unwrap();

        let restore_error = storage.restore_catalog(&cache_link).await.unwrap_err();
        assert!(restore_error.contains("unsafe cache root"));
        assert!(outside_data.exists());

        let eligible = vec![parts[0].dir.clone()];
        let eviction_error = storage.evict_cache(&cache_link, 0, &eligible).unwrap_err();
        assert!(eviction_error.contains("unsafe cache root"));
        assert!(outside_data.exists());

        let empty_storage = ObjectStorage::in_memory();
        let reconcile_error = empty_storage
            .reconcile_local_cache(&cache_link)
            .await
            .unwrap_err();
        assert!(reconcile_error.contains("unsafe cache root"));
        assert!(outside_data.exists());
    }

    #[tokio::test]
    async fn trace_parts_publish_restore_and_eviction_round_trip() {
        use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
        use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};

        let storage = ObjectStorage::in_memory();
        let root = temp_dir("trace-round-trip").join("traces");
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: vec![1; 16],
                        span_id: vec![2; 8],
                        start_time_unix_nano: 1_700_000_000_000_000_000,
                        end_time_unix_nano: 1_700_000_000_000_000_100,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let spans = crate::trace::normalize_request(&test_tenant(), request).unwrap();
        let parts = crate::trace_part::flush_trace_spans(&spans, &root, 100).unwrap();
        let manifest = storage.publish_trace_parts(&parts).await.unwrap();
        assert_eq!(manifest.parts.len(), 1);

        let id = manifest.parts[0].id.clone();
        std::fs::remove_file(parts[0].data_path()).unwrap();
        storage.restore_trace_catalog(&root).await.unwrap();
        assert!(
            storage
                .restore_trace_parts(&root, &std::iter::once(id.clone()).collect())
                .await
                .is_ok()
        );
        assert!(parts[0].data_path().exists());

        storage
            .evict_trace_cache(&root, 0, &[parts[0].dir.clone()])
            .unwrap();
        assert!(!parts[0].data_path().exists());
        storage
            .restore_trace_parts(&root, &std::iter::once(id).collect())
            .await
            .unwrap();
        let reader = crate::trace_part::TracePartReader::open(
            crate::trace_part::load_trace_part(&parts[0].dir).unwrap(),
        )
        .unwrap();
        assert_eq!(reader.query_trace_id(&test_tenant(), &"01".repeat(16)).unwrap().len(), 1);
    }

    #[tokio::test]
    async fn trace_reconciliation_publishes_local_only_parts() {
        use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
        use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};

        let storage = ObjectStorage::in_memory();
        let root = temp_dir("trace-local-migration").join("traces");
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: vec![7; 16],
                        span_id: vec![8; 8],
                        start_time_unix_nano: 1_700_000_000_000_000_000,
                        end_time_unix_nano: 1_700_000_000_000_000_100,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let spans = crate::trace::normalize_request(&test_tenant(), request).unwrap();
        let parts = crate::trace_part::flush_trace_spans(&spans, &root, 100).unwrap();

        let manifest = storage.reconcile_trace_local_cache(&root).await.unwrap();

        assert_eq!(manifest.parts.len(), 1);
        assert_eq!(manifest.parts[0].id, parts[0].meta.id);
        assert_eq!(storage.load_trace_manifest().await.unwrap().parts.len(), 1);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn trace_cache_rejects_symlinked_immutable_files() {
        use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
        use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};
        use std::os::unix::fs::symlink;

        let storage = ObjectStorage::in_memory();
        let source = temp_dir("trace-symlink-source").join("traces");
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![Span {
                        trace_id: vec![9; 16],
                        span_id: vec![1; 8],
                        start_time_unix_nano: 1,
                        end_time_unix_nano: 2,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let spans = crate::trace::normalize_request(&test_tenant(), request).unwrap();
        let parts = crate::trace_part::flush_trace_spans(&spans, &source, 100).unwrap();
        let manifest = storage.publish_trace_parts(&parts).await.unwrap();
        let root = temp_dir("trace-symlink-cache").join("traces");
        let descriptor = &manifest.parts[0];
        let cache_part = root.join(&descriptor.partition).join(&descriptor.id);
        std::fs::create_dir_all(&cache_part).unwrap();
        let outside = temp_dir("trace-symlink-target").join(TRACE_DATA_FILE);
        std::fs::write(&outside, b"must survive").unwrap();
        symlink(&outside, cache_part.join(TRACE_DATA_FILE)).unwrap();

        let restore_error = storage.restore_trace_catalog(&root).await.unwrap_err();
        assert!(restore_error.contains("symlinked trace cache file"));
        let eligible = vec![root.join(&descriptor.partition).join(&descriptor.id)];
        let eviction_error = storage.evict_trace_cache(&root, 0, &eligible).unwrap_err();
        assert!(eviction_error.contains("symlinked trace cache file"));
        assert_eq!(std::fs::read(&outside).unwrap(), b"must survive");
    }

    #[tokio::test]
    async fn seeded_fault_store_publishes_losslessly_after_injected_errors() {
        // Tier B recovery gate: a high write-error rate makes most publish
        // attempts fail, but retrying the same acknowledged parts must
        // eventually land every one in the manifest with no loss and no
        // duplication. A fixed seed makes the injected-error sequence
        // reproducible.
        let root = temp_dir("fault-recovery");
        let parts_root = root.join("parts");
        let parts = crate::part::flush_rows(
            vec![row("first"), row("second"), row("third")],
            &parts_root,
            16,
        )
        .unwrap();
        let config = super::fault_store::FaultConfig::for_test(0, 0, 0, 0.6, 0x00c0_ffee_5eed);
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        let store: Arc<dyn ObjectStore> =
            Arc::new(super::fault_store::LatencyFaultStore::new(inner, config));
        let storage = ObjectStorage::from_store(store, "signy-fault");

        let mut attempts = 0;
        loop {
            attempts += 1;
            match storage.publish(&parts, &[]).await {
                Ok(_) => break,
                Err(error) => {
                    assert!(
                        error.contains("injected object-store write failure")
                            || error.contains("failed to"),
                        "unexpected error: {error}"
                    );
                    assert!(attempts < 1000, "publish never recovered");
                }
            }
        }
        assert!(attempts > 1, "test seed did not inject any failure");

        let manifest = storage.load_manifest().await.unwrap();
        let mut ids: Vec<_> = manifest.parts.iter().map(|part| part.id.clone()).collect();
        ids.sort();
        let mut expected: Vec<_> = parts.iter().map(|part| part.meta.id.clone()).collect();
        expected.sort();
        assert_eq!(ids, expected, "every acknowledged part must be present exactly once");
    }

    /// An `ObjectStore` that accepts every write regardless of the condition
    /// attached to it — the failure mode a misconfigured S3-compatible store
    /// presents, expressed directly so the guard can be tested against it.
    #[derive(Debug)]
    struct IgnoresConditions {
        inner: Arc<dyn object_store::ObjectStore>,
    }

    impl std::fmt::Display for IgnoresConditions {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "IgnoresConditions")
        }
    }

    #[async_trait::async_trait]
    impl object_store::ObjectStore for IgnoresConditions {
        async fn put_opts(
            &self,
            location: &object_store::path::Path,
            payload: object_store::PutPayload,
            _opts: object_store::PutOptions,
        ) -> object_store::Result<object_store::PutResult> {
            // The whole point: the mode is thrown away.
            self.inner
                .put_opts(
                    location,
                    payload,
                    object_store::PutOptions {
                        mode: object_store::PutMode::Overwrite,
                        ..Default::default()
                    },
                )
                .await
        }

        async fn put_multipart_opts(
            &self,
            location: &object_store::path::Path,
            opts: object_store::PutMultipartOptions,
        ) -> object_store::Result<Box<dyn object_store::MultipartUpload>> {
            self.inner.put_multipart_opts(location, opts).await
        }

        async fn get_opts(
            &self,
            location: &object_store::path::Path,
            opts: object_store::GetOptions,
        ) -> object_store::Result<object_store::GetResult> {
            self.inner.get_opts(location, opts).await
        }

        async fn delete(&self, location: &object_store::path::Path) -> object_store::Result<()> {
            self.inner.delete(location).await
        }

        fn list(
            &self,
            prefix: Option<&object_store::path::Path>,
        ) -> futures_util::stream::BoxStream<'static, object_store::Result<object_store::ObjectMeta>>
        {
            self.inner.list(prefix)
        }

        async fn list_with_delimiter(
            &self,
            prefix: Option<&object_store::path::Path>,
        ) -> object_store::Result<object_store::ListResult> {
            self.inner.list_with_delimiter(prefix).await
        }

        async fn copy(
            &self,
            from: &object_store::path::Path,
            to: &object_store::path::Path,
        ) -> object_store::Result<()> {
            self.inner.copy(from, to).await
        }

        async fn copy_if_not_exists(
            &self,
            from: &object_store::path::Path,
            to: &object_store::path::Path,
        ) -> object_store::Result<()> {
            self.inner.copy_if_not_exists(from, to).await
        }
    }

    /// `from_url` hands the environment straight to `object_store` and nothing
    /// else checks what came back. Get `OBJECT_STORE_CONDITIONAL_PUT` wrong and
    /// every manifest guarantee here rests on nothing — silently, because the
    /// writes all succeed.
    ///
    /// This is not a test of `object_store`. It is a test that *our* startup
    /// refuses a store whose conditional writes do not condition on anything.
    #[tokio::test]
    async fn the_preflight_refuses_a_store_that_ignores_conditions() {
        let honest = ObjectStorage::sharing_store_for_test(Arc::new(
            object_store::memory::InMemory::new(),
        ));
        honest
            .verify_conditional_put()
            .await
            .expect("a store that honours conditions must pass");

        let dishonest = ObjectStorage::sharing_store_for_test(Arc::new(IgnoresConditions {
            inner: Arc::new(object_store::memory::InMemory::new()),
        }));
        let error = dishonest
            .verify_conditional_put()
            .await
            .expect_err("a store that ignores conditions must not start");
        assert!(
            error.contains("does not enforce conditional writes"),
            "{error}"
        );
        assert!(
            error.contains("OBJECT_STORE_CONDITIONAL_PUT"),
            "the message must say what to set: {error}"
        );
    }

    /// The probe leaves nothing behind, so a restart does not accumulate them
    /// and the check is repeatable within one process.
    #[tokio::test]
    async fn the_preflight_cleans_up_after_itself_and_can_run_twice() {
        let storage = ObjectStorage::sharing_store_for_test(Arc::new(
            object_store::memory::InMemory::new(),
        ));
        storage.verify_conditional_put().await.unwrap();
        storage.verify_conditional_put().await.unwrap();

        let leftovers: Vec<_> = futures_util::StreamExt::collect::<Vec<_>>(storage.store.list(None))
            .await
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|meta| meta.location.as_ref().contains("_preflight"))
            .collect();
        assert!(leftovers.is_empty(), "probe objects survived: {leftovers:?}");
    }

    /// `file://` opts out of CAS on purpose, so the preflight must not turn a
    /// documented development backend into a boot failure.
    #[tokio::test]
    async fn the_preflight_skips_the_local_development_backend() {
        let dir = temp_dir("preflight-local");
        let storage = ObjectStorage::from_url(&format!("file://{}", dir.display()))
            .expect("local store builds");
        storage
            .verify_conditional_put()
            .await
            .expect("file:// deliberately opts out of conditional writes");
    }

    /// The M6 replacement procedure says drain the old instance before
    /// starting the new one. Orchestrators break that ordering routinely, and
    /// until now both processes would have kept writing, each believing it was
    /// the only one.
    #[tokio::test]
    async fn a_second_claim_fences_the_first_writer() {
        let store: Arc<dyn object_store::ObjectStore> =
            Arc::new(object_store::memory::InMemory::new());
        let old = ObjectStorage::sharing_store_for_test(store.clone());
        let new = ObjectStorage::sharing_store_for_test(store);

        assert_eq!(old.claim_writer_epoch().await.unwrap(), 1);

        // The original still owns the prefix and can publish.
        let dir = temp_dir("fenced-writer").join("parts");
        let first = crate::part::flush_rows(vec![row("before takeover")], &dir, 100).unwrap();
        old.publish(&first, &[]).await.expect("the owner publishes");

        // The replacement takes over.
        assert_eq!(new.claim_writer_epoch().await.unwrap(), 2);

        let second = crate::part::flush_rows(vec![row("after takeover")], &dir, 100).unwrap();
        let error = old
            .publish(&second, &[])
            .await
            .expect_err("a fenced writer must not publish");
        assert!(crate::object_storage::is_fenced_error(&error), "{error}");

        // And the replacement is unaffected.
        new.publish(&second, &[])
            .await
            .expect("the new owner publishes");
        let manifest = new.load_manifest().await.unwrap();
        assert_eq!(manifest.writer_epoch, 2);
        assert_eq!(manifest.parts.len(), 2);
    }

    /// Trace writes go through a manifest of their own, so the claim has to
    /// reach both or a takeover would fence only half of the old writer.
    #[tokio::test]
    async fn the_claim_fences_trace_writes_too() {
        let store: Arc<dyn object_store::ObjectStore> =
            Arc::new(object_store::memory::InMemory::new());
        let old = ObjectStorage::sharing_store_for_test(store.clone());
        let new = ObjectStorage::sharing_store_for_test(store);
        old.claim_writer_epoch().await.unwrap();
        new.claim_writer_epoch().await.unwrap();

        let dir = temp_dir("fenced-traces").join("traces");
        let spans = vec![crate::trace::TraceSpan {
            tenant: crate::tenant::test_tenant(),
            trace_id: "ab".repeat(16),
            span_id: "fenced".to_string(),
            start_time_ns: 1_000,
            end_time_ns: 2_000,
            span: Default::default(),
            resource: None,
            resource_schema_url: String::new(),
            scope: None,
            scope_schema_url: String::new(),
        }];
        let parts = crate::trace_part::flush_trace_spans(&spans, &dir, 100).unwrap();

        let error = old
            .publish_trace_parts(&parts)
            .await
            .expect_err("a fenced writer must not publish traces");
        assert!(crate::object_storage::is_fenced_error(&error), "{error}");
        assert!(new.publish_trace_parts(&parts).await.is_ok());
    }

    /// An unclaimed store keeps working. Fencing is only meaningful once
    /// someone has taken ownership, and the local development backend never
    /// does.
    #[tokio::test]
    async fn an_unclaimed_store_is_never_fenced() {
        let storage = ObjectStorage::in_memory();
        let dir = temp_dir("unclaimed").join("parts");
        let parts = crate::part::flush_rows(vec![row("unclaimed")], &dir, 100).unwrap();
        assert_eq!(storage.writer_epoch(), 0);
        assert!(storage.publish(&parts, &[]).await.is_ok());
    }

    #[test]
    fn manifest_rejects_relative_path_components() {
        for value in [".", "..", "a/b", ""] {
            let manifest = Manifest {
                generation: 1,
                writer_epoch: 1,
                parts: vec![ManifestPart {
                    id: value.to_string(),
                    partition: "2026-01-01".to_string(),
                }],
            };
            assert!(validate_manifest(&manifest).is_err(), "accepted {value:?}");
        }
    }

    /// A retention batch is not a replacement. Once a tick has removed some of
    /// its ids and then failed before retiring them from the registry, the
    /// next tick's batch mixes those ids with newly expired ones. Requiring an
    /// intact set there wedged retention permanently: the batch could only ever
    /// grow, so every later tick failed too.
    #[tokio::test]
    async fn a_pure_removal_tolerates_ids_an_earlier_tick_already_removed() {
        let root = temp_dir("retention-partial-removal");
        let storage = ObjectStorage::in_memory();

        let mut first_row = row("already removed");
        first_row.timestamp_ns += 1;
        let first = crate::part::flush_rows(vec![first_row], &root, 100).unwrap();
        let mut second_row = row("newly expired");
        second_row.timestamp_ns += 2;
        let second = crate::part::flush_rows(vec![second_row], &root, 100).unwrap();
        storage.publish(&first, &[]).await.unwrap();
        storage.publish(&second, &[]).await.unwrap();

        let first_id = first[0].meta.id.clone();
        let second_id = second[0].meta.id.clone();
        storage
            .publish(&[], std::slice::from_ref(&first_id))
            .await
            .unwrap();

        storage
            .publish(&[], &[first_id, second_id])
            .await
            .expect("a mixed removal batch removes what is left");
        assert!(storage.load_manifest().await.unwrap().parts.is_empty());
    }

    #[tokio::test]
    async fn a_trace_removal_tolerates_ids_an_earlier_tick_already_removed() {
        let root = temp_dir("trace-retention-partial-removal");
        let storage = ObjectStorage::in_memory();

        let mut spans = Vec::new();
        for (index, trace_id) in ["aa", "bb"].iter().enumerate() {
            spans.push(crate::trace::TraceSpan {
                tenant: test_tenant(),
                trace_id: trace_id.repeat(16),
                span_id: format!("span-{index}"),
                start_time_ns: 1_700_000_000_000_000_000 + index as i64,
                end_time_ns: 1_700_000_000_000_000_001 + index as i64,
                span: Default::default(),
                resource: None,
                resource_schema_url: String::new(),
                scope: None,
                scope_schema_url: String::new(),
            });
        }
        let first = crate::trace_part::flush_trace_spans(&[spans[0].clone()], &root, 100).unwrap();
        let second =
            crate::trace_part::flush_trace_spans(&[spans[1].clone()], &root, 100).unwrap();
        storage.publish_trace_parts(&first).await.unwrap();
        storage.publish_trace_parts(&second).await.unwrap();

        let descriptor = |part: &crate::trace_part::TracePart| TraceManifestPart {
            id: part.meta.id.clone(),
            partition: part.meta.partition.clone(),
        };
        storage
            .remove_trace_parts(&[descriptor(&first[0])])
            .await
            .unwrap();

        storage
            .remove_trace_parts(&[descriptor(&first[0]), descriptor(&second[0])])
            .await
            .expect("a mixed removal batch removes what is left");
        assert!(
            storage
                .load_trace_manifest()
                .await
                .unwrap()
                .parts
                .is_empty()
        );
    }

    /// Restore was one round trip at a time, so its cost was `parts × round
    /// trip` against a store where latency is the dominant term.
    ///
    /// Concurrency is measured at the store rather than inferred from elapsed
    /// time. A clock-based proxy needs nothing else to be able to stretch the
    /// clock, and that is not true here: under virtual time a pending blocking
    /// task defers the auto-advance, which serializes the sleeps and makes the
    /// measurement report the opposite of what happened.
    #[tokio::test]
    async fn restoring_a_catalog_overlaps_its_downloads() {
        const PARTS: usize = 32;

        let root = temp_dir("restore-concurrency");
        let parts_root = root.join("parts");
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        // Enough latency that the downloads are in flight together rather than
        // finishing one before the next is polled.
        let config = super::fault_store::FaultConfig::for_test(0, 20, 0, 0.0, 7);
        let faulty = Arc::new(super::fault_store::LatencyFaultStore::new(inner, config));
        let store: Arc<dyn ObjectStore> = faulty.clone();
        let storage = ObjectStorage::from_store(store, "restore-concurrency");

        for index in 0..PARTS {
            let parts =
                crate::part::flush_rows(vec![row(&format!("part-{index}"))], &parts_root, 16)
                    .unwrap();
            storage.publish(&parts, &[]).await.unwrap();
        }
        // Drop the local catalog so every part has to come back from the store.
        std::fs::remove_dir_all(&parts_root).unwrap();

        let manifest = storage.restore_catalog(&parts_root).await.unwrap();
        assert_eq!(manifest.parts.len(), PARTS);

        let peak = faulty.peak_reads_in_flight();
        assert!(
            peak > 1,
            "restore ran one download at a time (peak in flight {peak})"
        );
        assert!(
            peak <= super::RESTORE_CONCURRENCY as u64,
            "restore exceeded its own bound (peak in flight {peak}); an unbounded fan-out \
opens a connection per part"
        );
    }

    /// Concurrent restores of the same part do not destroy each other.
    ///
    /// Restores run under a *shared* lifecycle guard on purpose — object-store
    /// latency must not block writers exclusively — so several readers that miss
    /// the same evicted part all enter `download_part` at once. They used to
    /// stage into a directory named for the part alone and `remove_dir_all` it
    /// on the way in, so each one tore down whatever the others had downloaded:
    /// the losers failed with `File exists` or `No such file or directory`,
    /// which the restore path counts as an object-store failure and which drags
    /// `remote_healthy` down over a race the store had no part in. A
    /// query-heavy load run with fault injection *disabled* failed 266 of 358
    /// restores this way.
    #[tokio::test]
    async fn concurrent_restores_of_one_part_all_succeed() {
        const RESTORERS: usize = 12;

        let root = temp_dir("restore-same-part").join("parts");
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        // Enough read latency that the downloads overlap rather than finishing
        // one before the next is polled. Zero injected errors: whatever fails
        // here is this code, not the store.
        let config = super::fault_store::FaultConfig::for_test(0, 20, 0, 0.0, 11);
        let store: Arc<dyn ObjectStore> =
            Arc::new(super::fault_store::LatencyFaultStore::new(inner, config));
        let storage = Arc::new(ObjectStorage::from_store(store, "restore-same-part"));

        let parts = crate::part::flush_rows(vec![row("contended")], &root, 100).unwrap();
        let manifest = storage.publish(&parts, &[]).await.unwrap();
        // Take the whole directory, so every restorer runs the full three-file
        // path — the one that replaces the part directory wholesale.
        std::fs::remove_dir_all(&root).unwrap();

        let ids: HashSet<String> = manifest.parts.iter().map(|part| part.id.clone()).collect();
        let restores = (0..RESTORERS).map(|_| {
            let storage = storage.clone();
            let root = root.clone();
            let ids = ids.clone();
            async move { storage.restore_parts(&root, &ids).await }
        });
        let outcomes = futures_util::future::join_all(restores).await;
        let failures: Vec<_> = outcomes.iter().filter_map(|out| out.as_ref().err()).collect();
        assert!(
            failures.is_empty(),
            "{} of {RESTORERS} concurrent restores failed: {failures:?}",
            failures.len()
        );

        let registry =
            crate::part_registry::PartRegistry::load_from_manifest(&root, &manifest).unwrap();
        let result = registry
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
            .unwrap();
        assert_eq!(result[0].entries[0].line, "contended");
    }

    /// Eviction visits the registry's parts, not the tree. Directories on disk
    /// that no registered part points at are neither read nor removed — which
    /// is what the walk already did by skipping them, at the cost of walking
    /// them first.
    #[test]
    fn eviction_touches_only_the_parts_it_was_given() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("evict-scope").join("parts");
        let evictable = part::flush_rows(vec![row("evictable")], &root, 100).unwrap();
        let mut other = row("untouched");
        other.timestamp_ns += 1;
        let untouched = part::flush_rows(vec![other], &root, 100).unwrap();

        // A directory the registry knows nothing about: local-only data, or an
        // interrupted transaction awaiting operator recovery.
        let stray = root.join("2020-01-01").join("stray-part");
        std::fs::create_dir_all(&stray).unwrap();
        std::fs::write(stray.join(DATA_FILE), b"not eviction's business").unwrap();

        storage
            .evict_cache(&root, 0, &[evictable[0].dir.clone()])
            .unwrap();

        assert!(!evictable[0].data_path().exists(), "the given part is evicted");
        assert!(
            evictable[0].meta_path().exists(),
            "its catalog survives: only bodies are evictable"
        );
        assert!(
            untouched[0].data_path().exists(),
            "a registered part that was not passed is left alone"
        );
        assert!(
            stray.join(DATA_FILE).exists(),
            "and a directory nothing points at is not eviction's to remove"
        );
    }

    /// Reconciling a clean cache reads the catalog once, not twice.
    ///
    /// `reconcile_local_cache` restored the catalog at the start and again at
    /// the end. The second pass exists to fetch catalog files for parts the
    /// reconcile itself published, and a clean restart publishes nothing — so
    /// it re-validated every part's checksums for no result. Measured at 10,099
    /// parts, that pass was about eighteen seconds of a sixty-four second
    /// startup.
    ///
    /// Counted here rather than timed, so the regression is caught without a
    /// run at the scale where it hurts.
    #[tokio::test]
    async fn reconciling_a_clean_cache_reads_the_catalog_once() {
        let root = temp_dir("reconcile-passes");
        let parts_root = root.join("parts");
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        let config = super::fault_store::FaultConfig::for_test(0, 0, 0, 0.0, 3);
        let faulty = Arc::new(super::fault_store::LatencyFaultStore::new(inner, config));
        let store: Arc<dyn ObjectStore> = faulty.clone();
        let storage = ObjectStorage::from_store(store, "reconcile-passes");

        for index in 0..6 {
            let mut sample = row(&format!("part-{index}"));
            sample.timestamp_ns += index as i64;
            let parts = crate::part::flush_rows(vec![sample], &parts_root, 16).unwrap();
            storage.publish(&parts, &[]).await.unwrap();
        }
        // Local catalog intact and nothing unpublished: the case a restart of a
        // cleanly stopped instance presents.
        let before = storage.catalog_validations();
        let manifest = storage.reconcile_local_cache(&parts_root).await.unwrap();
        let validations = storage.catalog_validations() - before;

        assert_eq!(manifest.parts.len(), 6);
        assert_eq!(
            validations, 6,
            "reconcile checksummed {validations} catalogs for six parts; a second pass is back"
        );
        // The store itself is barely touched, which is the point of the
        // catalog being local: the cost is checksums, not round trips.
        assert!(faulty.total_reads() > 0);
    }

    /// The cost unit of this whole design is object-store requests, and the
    /// number of them per publication is a property of this code rather than of
    /// the backend, so it is pinned here rather than estimated in a document.
    ///
    /// A publication of one part costs three PUTs for the immutable files and
    /// two PUTs for the replicated catalog generation. Once the catalog is
    /// loaded, later publications do not reread the append-only history.
    #[tokio::test]
    async fn publishing_a_part_costs_a_fixed_number_of_requests() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("op-counts").join("parts");

        let mut first_row = row("first");
        first_row.timestamp_ns += 1;
        let first = part::flush_rows(vec![first_row], &root, 100).unwrap();
        let before = storage.operation_counts();
        storage.publish(&first, &[]).await.unwrap();
        let first_publish = delta(before, storage.operation_counts());

        assert_eq!(first_publish.puts, PART_FILES.len() as u64 + 2);
        assert_eq!(
            PART_FILES.len(),
            3,
            "every file per part is a billed request per flush, so the count is \
             load-bearing rather than incidental"
        );
        assert_eq!(first_publish.gets, 6);
        assert_eq!(first_publish.lists, 8);
        assert_eq!(first_publish.copies, 0);

        for index in 2..=9u64 {
            let mut filler = row("filler");
            filler.timestamp_ns += index as i64;
            let part = part::flush_rows(vec![filler], &root, 100).unwrap();
            storage.publish(&part, &[]).await.unwrap();
        }

        let mut tenth_row = row("tenth");
        tenth_row.timestamp_ns += 10;
        let tenth = part::flush_rows(vec![tenth_row], &root, 100).unwrap();
        let before = storage.operation_counts();
        storage.publish(&tenth, &[]).await.unwrap();
        let tenth_publish = delta(before, storage.operation_counts());

        assert_eq!(tenth_publish.puts, first_publish.puts);
        assert_eq!(tenth_publish.multipart_puts, first_publish.multipart_puts);
        assert_eq!(tenth_publish.copies, first_publish.copies);

        // The catalog record remains bounded by the mutation, rather than
        // growing with every descriptor already committed.
        assert!(tenth_publish.put_bytes >= first_publish.put_bytes);
        assert_eq!(
            tenth_publish.ranged_gets, 0,
            "no read path asks for a byte range yet; when one does, this is the \
             number that stops being zero"
        );

        // The split has to add up, or a class silently swallows another's
        // bytes and the conclusion drawn from it is wrong in the direction
        // nobody checks. `other` is a real class here, not a leak: the upload
        // marker is neither a part nor a manifest.
        let by_kind = tenth_publish.put_bytes_by_kind;
        assert_eq!(
            by_kind.manifest + by_kind.part + by_kind.trace_part + by_kind.other,
            tenth_publish.put_bytes,
            "the per-kind byte split must account for every byte the total counted"
        );
        assert!(
            by_kind.part > 0 && by_kind.manifest > 0,
            "publishing a part writes both part files and a manifest, so neither \
             class may be zero: {by_kind:?}"
        );
        let read_kinds = tenth_publish.get_bytes_by_kind;
        assert_eq!(
            read_kinds.manifest + read_kinds.part + read_kinds.trace_part + read_kinds.other,
            tenth_publish.get_bytes,
            "the per-kind byte split must account for every byte the total counted"
        );
    }

    /// Restoring an evicted body fetches the body and nothing else.
    ///
    /// Eviction removes the Parquet body and deliberately leaves `index.bin`
    /// and `meta.json` behind, so the catalog a restore needs is already local
    /// and already checksummed against the manifest descriptor. Refetching it
    /// cost two extra GETs on every cache miss, and deleted a healthy catalog
    /// in order to replace it with an identical copy of itself.
    #[tokio::test]
    async fn restoring_an_evicted_body_costs_one_get() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("restore-op-counts").join("parts");
        let parts = part::flush_rows(vec![row("evicted then restored")], &root, 100).unwrap();
        let manifest = storage.publish(&parts, &[]).await.unwrap();
        storage
            .evict_cache(&root, 0, &[parts[0].dir.clone()])
            .unwrap();
        assert!(!parts[0].data_path().exists());
        assert!(parts[0].index_path().exists() && parts[0].meta_path().exists());

        let ids: HashSet<String> = manifest.parts.iter().map(|part| part.id.clone()).collect();
        let before = storage.operation_counts();
        storage.restore_parts(&root, &ids).await.unwrap();
        let restore = delta(before, storage.operation_counts());

        assert_eq!(
            restore.gets, 1,
            "the catalog is cached locally, so only the evicted body is fetched"
        );
        assert_eq!(restore.puts, 0);
        assert_eq!(restore.lists, 0);

        let registry =
            crate::part_registry::PartRegistry::load_from_manifest(&root, &manifest).unwrap();
        let result = registry
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
            .unwrap();
        assert_eq!(result[0].entries[0].line, "evicted then restored");
    }

    /// The one-GET path is an optimisation of a *valid* local catalog, not an
    /// assumption that one exists. A part whose sidecar no longer matches its
    /// descriptor falls back to fetching every file, because that is the only
    /// way to get a catalog that does match.
    #[tokio::test]
    async fn restoring_a_part_with_a_damaged_catalog_fetches_every_file() {
        let storage = ObjectStorage::in_memory();
        let root = temp_dir("restore-damaged-catalog").join("parts");
        let parts = part::flush_rows(vec![row("damaged then restored")], &root, 100).unwrap();
        let manifest = storage.publish(&parts, &[]).await.unwrap();
        storage
            .evict_cache(&root, 0, &[parts[0].dir.clone()])
            .unwrap();
        std::fs::write(parts[0].index_path(), b"not the index that was published").unwrap();

        let ids: HashSet<String> = manifest.parts.iter().map(|part| part.id.clone()).collect();
        let before = storage.operation_counts();
        storage.restore_parts(&root, &ids).await.unwrap();
        let restore = delta(before, storage.operation_counts());

        assert_eq!(
            restore.gets,
            PART_FILES.len() as u64,
            "a damaged local catalog fetches every part file"
        );

        let registry =
            crate::part_registry::PartRegistry::load_from_manifest(&root, &manifest).unwrap();
        let result = registry
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
            .unwrap();
        assert_eq!(result[0].entries[0].line, "damaged then restored");
    }

    fn delta(before: ObjectStoreOpCounts, after: ObjectStoreOpCounts) -> ObjectStoreOpCounts {
        ObjectStoreOpCounts {
            puts: after.puts - before.puts,
            multipart_puts: after.multipart_puts - before.multipart_puts,
            gets: after.gets - before.gets,
            ranged_gets: after.ranged_gets - before.ranged_gets,
            deletes: after.deletes - before.deletes,
            lists: after.lists - before.lists,
            listed_objects: after.listed_objects - before.listed_objects,
            copies: after.copies - before.copies,
            get_bytes: after.get_bytes - before.get_bytes,
            put_bytes: after.put_bytes - before.put_bytes,
            get_bytes_by_kind: kind_delta(before.get_bytes_by_kind, after.get_bytes_by_kind),
            put_bytes_by_kind: kind_delta(before.put_bytes_by_kind, after.put_bytes_by_kind),
        }
    }

    fn kind_delta(before: PathByteCounts, after: PathByteCounts) -> PathByteCounts {
        PathByteCounts {
            manifest: after.manifest - before.manifest,
            part: after.part - before.part,
            trace_part: after.trace_part - before.trace_part,
            other: after.other - before.other,
        }
    }

    /// A request that hides rows but does not survive a restart is the one
    /// failure this must not have: the instance would come back serving lines
    /// a tenant was told were gone.
    #[tokio::test]
    async fn a_deletion_request_outlives_the_process_that_accepted_it() {
        let store: Arc<dyn object_store::ObjectStore> =
            Arc::new(object_store::memory::InMemory::new());
        let storage = ObjectStorage::sharing_store_for_test(store.clone());
        let tenant = crate::tenant::test_tenant();

        let accepted = crate::delete_requests::DeleteRequests::new(Some(storage));
        let request = accepted
            .submit(&tenant, "attr=app%3Ddrop", 0, 1_000, 500)
            .await
            .expect("a valid request");

        let restarted = crate::delete_requests::DeleteRequests::new(Some(
            ObjectStorage::sharing_store_for_test(store),
        ));
        assert_eq!(restarted.load().await.unwrap(), 1);
        let restored = restarted.list(&tenant);
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].request_id, request.request_id);
        assert!(!restarted.mask_for(&tenant).is_empty());
    }

    /// An object store whose listing can be slowed, whose reads can be made to
    /// fail for one path, and whose deletes can be refused -- the three things
    /// the collector's budgets, its fail-closed rule and an R2 Bucket Lock
    /// look like from inside a test.
    #[derive(Debug)]
    struct ScriptedStore {
        inner: Arc<dyn ObjectStore>,
        list_delay: std::time::Duration,
        unreadable: Option<String>,
        unlistable: Option<String>,
        undeletable: Option<String>,
        deletes_attempted: AtomicU64,
    }

    impl ScriptedStore {
        fn wrapping(inner: Arc<dyn ObjectStore>) -> Self {
            Self {
                inner,
                list_delay: std::time::Duration::ZERO,
                unreadable: None,
                unlistable: None,
                undeletable: None,
                deletes_attempted: AtomicU64::new(0),
            }
        }

        fn with_list_delay(mut self, delay: std::time::Duration) -> Self {
            self.list_delay = delay;
            self
        }

        fn refusing_reads_of(mut self, path_fragment: &str) -> Self {
            self.unreadable = Some(path_fragment.to_string());
            self
        }

        fn refusing_listing_of(mut self, path_fragment: &str) -> Self {
            self.unlistable = Some(path_fragment.to_string());
            self
        }

        fn refusing_deletes_of(mut self, path_fragment: &str) -> Self {
            self.undeletable = Some(path_fragment.to_string());
            self
        }
    }

    impl std::fmt::Display for ScriptedStore {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "ScriptedStore({})", self.inner)
        }
    }

    #[async_trait::async_trait]
    impl ObjectStore for ScriptedStore {
        async fn put_opts(
            &self,
            location: &ObjectPath,
            payload: object_store::PutPayload,
            options: PutOptions,
        ) -> object_store::Result<PutResult> {
            self.inner.put_opts(location, payload, options).await
        }

        async fn put_multipart_opts(
            &self,
            location: &ObjectPath,
            options: object_store::PutMultipartOptions,
        ) -> object_store::Result<Box<dyn MultipartUpload>> {
            self.inner.put_multipart_opts(location, options).await
        }

        async fn get_opts(
            &self,
            location: &ObjectPath,
            options: GetOptions,
        ) -> object_store::Result<object_store::GetResult> {
            if let Some(fragment) = &self.unreadable
                && location.as_ref().contains(fragment.as_str())
            {
                return Err(object_store::Error::Generic {
                    store: "scripted",
                    source: "read refused".into(),
                });
            }
            self.inner.get_opts(location, options).await
        }

        async fn delete(&self, location: &ObjectPath) -> object_store::Result<()> {
            self.deletes_attempted.fetch_add(1, Ordering::Relaxed);
            if let Some(fragment) = &self.undeletable
                && location.as_ref().contains(fragment.as_str())
            {
                return Err(object_store::Error::Generic {
                    store: "scripted",
                    source: "delete refused".into(),
                });
            }
            self.inner.delete(location).await
        }

        fn list(
            &self,
            prefix: Option<&ObjectPath>,
        ) -> futures_util::stream::BoxStream<'static, object_store::Result<object_store::ObjectMeta>>
        {
            use futures_util::StreamExt;
            if let Some(fragment) = &self.unlistable
                && prefix.is_some_and(|prefix| prefix.as_ref().contains(fragment.as_str()))
            {
                return futures_util::stream::once(async {
                    Err(object_store::Error::Generic {
                        store: "scripted",
                        source: "listing refused".into(),
                    })
                })
                .boxed();
            }
            let delay = self.list_delay;
            self.inner
                .list(prefix)
                .then(move |item| async move {
                    tokio::time::sleep(delay).await;
                    item
                })
                .boxed()
        }

        fn list_with_offset(
            &self,
            prefix: Option<&ObjectPath>,
            offset: &ObjectPath,
        ) -> futures_util::stream::BoxStream<'static, object_store::Result<object_store::ObjectMeta>>
        {
            use futures_util::StreamExt;
            if let Some(fragment) = &self.unlistable
                && prefix.is_some_and(|prefix| prefix.as_ref().contains(fragment.as_str()))
            {
                return futures_util::stream::once(async {
                    Err(object_store::Error::Generic {
                        store: "scripted",
                        source: "listing refused".into(),
                    })
                })
                .boxed();
            }
            let delay = self.list_delay;
            self.inner
                .list_with_offset(prefix, offset)
                .then(move |item| async move {
                    tokio::time::sleep(delay).await;
                    item
                })
                .boxed()
        }

        async fn list_with_delimiter(
            &self,
            prefix: Option<&ObjectPath>,
        ) -> object_store::Result<object_store::ListResult> {
            self.inner.list_with_delimiter(prefix).await
        }

        async fn copy(&self, from: &ObjectPath, to: &ObjectPath) -> object_store::Result<()> {
            self.inner.copy(from, to).await
        }

        async fn copy_if_not_exists(
            &self,
            from: &ObjectPath,
            to: &ObjectPath,
        ) -> object_store::Result<()> {
            self.inner.copy_if_not_exists(from, to).await
        }
    }

    /// Publishes `count` parts and retires all but the first, so the store
    /// holds one active part and `count - 1` orphaned ones.
    async fn store_with_orphans(
        storage: &ObjectStorage,
        label: &str,
        count: usize,
    ) -> (Vec<ManifestPart>, Vec<ManifestPart>) {
        let root = temp_dir(label).join("parts");
        let mut published = Vec::new();
        for index in 0..count {
            let parts =
                part::flush_rows(vec![row(&format!("{label}-{index}"))], &root, 100 + index)
                    .unwrap();
            storage.publish(&parts, &[]).await.unwrap();
            published.push(ManifestPart::from(&parts[0]));
        }
        let (active, retired) = published.split_at(1);
        let retired_ids: Vec<String> = retired.iter().map(|part| part.id.clone()).collect();
        storage.publish(&[], &retired_ids).await.unwrap();
        (active.to_vec(), retired.to_vec())
    }

    fn bounded_options(
        grace_period: std::time::Duration,
        max_scanned_objects: usize,
        max_deleted_objects: usize,
    ) -> OrphanCollectionOptions {
        OrphanCollectionOptions {
            grace_period,
            max_runtime: std::time::Duration::from_secs(60),
            max_scanned_objects,
            max_deleted_objects,
            max_deleted_bytes: u64::MAX,
            dry_run: false,
        }
    }

    /// The rule that makes the collector safe to point at a large store: an
    /// object it sees for the first time in this pass is never also deleted by
    /// it, however long ago the object was retired. Two passes have to agree.
    #[tokio::test]
    async fn an_object_seen_for_the_first_time_is_never_deleted_by_the_pass_that_saw_it() {
        let storage = ObjectStorage::in_memory();
        let (_, retired) = store_with_orphans(&storage, "gc-two-sightings", 2).await;
        let key = storage.part_path(&retired[0], DATA_FILE);
        // No grace at all, so only the two-sightings rule can hold it.
        let options = bounded_options(std::time::Duration::ZERO, usize::MAX, usize::MAX);
        let now = chrono::Utc::now() + chrono::Duration::hours(1);

        let first = storage
            .collect_orphans_at(&options, now, tokio::time::Instant::now())
            .await
            .unwrap();
        assert_eq!(first.deleted_objects, 0, "the first sighting deletes nothing");
        assert!(storage.store.head(&key).await.is_ok());

        let second = storage
            .collect_orphans_at(
                &options,
                now + chrono::Duration::seconds(1),
                tokio::time::Instant::now(),
            )
            .await
            .unwrap();
        assert!(second.deleted_objects > 0, "the second pass deletes what the first recorded");
        assert!(storage.store.head(&key).await.is_err());
    }

    /// A pass whose scan budget runs out has to leave the sightings it made
    /// behind it. The collector this replaced dropped them, so a store it
    /// could not finish scanning never reached a second sighting at all.
    #[tokio::test]
    async fn a_scan_that_runs_out_of_budget_keeps_what_it_learned_and_resumes() {
        let storage = ObjectStorage::in_memory();
        let (active, retired) = store_with_orphans(&storage, "gc-resume", 4).await;
        let grace = std::time::Duration::from_secs(300);
        let retired_at = chrono::Utc::now() + chrono::Duration::hours(1);
        // Two objects a pass, against nine of them (three per part).
        let options = bounded_options(grace, 2, usize::MAX);

        let mut passes = 0;
        loop {
            let outcome = storage
                .collect_orphans_at(&options, retired_at, tokio::time::Instant::now())
                .await
                .unwrap();
            passes += 1;
            assert_eq!(outcome.deleted_objects, 0, "nothing is deletable inside the grace");
            if outcome.scan_cycles_completed > 0 {
                break;
            }
            assert!(passes < 20, "the scan is not making progress");
        }
        assert!(passes > 1, "the budget has to have cut at least one pass short");

        let removed = storage
            .collect_orphans_at(
                &options,
                retired_at + chrono::Duration::seconds(301),
                tokio::time::Instant::now(),
            )
            .await
            .unwrap();
        assert!(
            removed.deleted_objects > 0,
            "sightings made by the cut-short passes are what let this one delete"
        );
        for part in &active {
            assert!(
                storage.store.head(&storage.part_path(part, DATA_FILE)).await.is_ok(),
                "an object the manifest still names is never a candidate"
            );
        }
        let mut remaining = 0;
        for part in &retired {
            if storage.store.head(&storage.part_path(part, DATA_FILE)).await.is_ok() {
                remaining += 1;
            }
        }
        assert!(remaining < retired.len(), "retired objects are being collected");
    }

    /// Time is the other budget, and it has to end the pass the same way: save
    /// the ledger, report, and let the next pass carry on.
    #[tokio::test(start_paused = true)]
    async fn a_slow_listing_stops_at_the_deadline_and_the_next_pass_continues() {
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        let store = Arc::new(ScriptedStore::wrapping(inner).with_list_delay(std::time::Duration::from_millis(100)));
        let storage = ObjectStorage::from_store(store, "signy-test");
        let (_, retired) = store_with_orphans(&storage, "gc-slow-list", 4).await;
        let grace = std::time::Duration::from_secs(300);
        let retired_at = chrono::Utc::now() + chrono::Duration::hours(1);
        let options = OrphanCollectionOptions {
            grace_period: grace,
            // Two listed objects' worth of delay.
            max_runtime: std::time::Duration::from_millis(250),
            max_scanned_objects: usize::MAX,
            max_deleted_objects: usize::MAX,
            max_deleted_bytes: u64::MAX,
            dry_run: false,
        };

        let first = storage
            .collect_orphans_at(&options, retired_at, tokio::time::Instant::now())
            .await
            .unwrap();
        assert!(first.scanned_objects > 0, "the pass makes progress before its deadline");
        assert!(
            first.scanned_objects < retired.len() * PART_FILES.len(),
            "and the deadline cuts it short"
        );
        assert_eq!(first.scan_cycles_completed, 0);

        let second = storage
            .collect_orphans_at(&options, retired_at, tokio::time::Instant::now())
            .await
            .unwrap();
        assert!(
            second.scanned_objects > 0,
            "the next pass resumes from the stored cursor rather than starting over"
        );
    }

    /// An unreadable catalog must stop the pass, not read as an empty active
    /// set: that difference is every live object in the store.
    #[tokio::test]
    async fn collection_deletes_nothing_when_the_catalog_cannot_be_read() {
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        let store = Arc::new(ScriptedStore::wrapping(inner.clone()));
        let storage = ObjectStorage::from_store(store, "signy-test");
        let (active, retired) = store_with_orphans(&storage, "gc-fail-closed", 2).await;
        let options = bounded_options(std::time::Duration::ZERO, usize::MAX, usize::MAX);
        let now = chrono::Utc::now() + chrono::Duration::hours(1);
        storage
            .collect_orphans_at(&options, now, tokio::time::Instant::now())
            .await
            .unwrap();

        let blinded = ObjectStorage::from_store(
            Arc::new(ScriptedStore::wrapping(inner).refusing_reads_of(MANIFEST_FILE)),
            "signy-test",
        );
        assert!(
            blinded
                .collect_orphans_at(&options, now, tokio::time::Instant::now())
                .await
                .is_err(),
            "a catalog read failure ends the pass"
        );
        for part in active.iter().chain(retired.iter()) {
            assert!(
                storage.store.head(&storage.part_path(part, DATA_FILE)).await.is_ok(),
                "and it deletes nothing on the way out"
            );
        }
    }

    /// What a Bucket Lock, or any store that refuses a delete, looks like: the
    /// object stays, the pass reports the refusal, and its ledger entry
    /// survives so the next pass tries again.
    #[tokio::test]
    async fn a_refused_deletion_is_reported_and_retried() {
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        let store = Arc::new(ScriptedStore::wrapping(inner).refusing_deletes_of(DATA_FILE));
        let storage = ObjectStorage::from_store(store, "signy-test");
        let (_, retired) = store_with_orphans(&storage, "gc-refused", 2).await;
        let key = storage.part_path(&retired[0], DATA_FILE);
        let options = bounded_options(std::time::Duration::ZERO, usize::MAX, usize::MAX);
        let now = chrono::Utc::now() + chrono::Duration::hours(1);

        storage
            .collect_orphans_at(&options, now, tokio::time::Instant::now())
            .await
            .unwrap();
        let second = storage
            .collect_orphans_at(
                &options,
                now + chrono::Duration::seconds(1),
                tokio::time::Instant::now(),
            )
            .await
            .unwrap();
        assert!(second.delete_errors > 0, "the refusal is reported");
        assert!(second.deleted_objects > 0, "the objects it did not refuse are still collected");
        assert!(storage.store.head(&key).await.is_ok());

        let third = storage
            .collect_orphans_at(
                &options,
                now + chrono::Duration::seconds(2),
                tokio::time::Instant::now(),
            )
            .await
            .unwrap();
        assert!(third.delete_errors > 0, "the refused object is tried again");
    }

    /// The dry run is what an operator reads before letting the collector
    /// delete tens of thousands of objects, so it has to report the same
    /// number a real pass would delete, and delete none of it.
    #[tokio::test]
    async fn a_dry_run_reports_its_candidates_and_deletes_nothing() {
        let storage = ObjectStorage::in_memory();
        let (_, retired) = store_with_orphans(&storage, "gc-dry-run", 3).await;
        let now = chrono::Utc::now() + chrono::Duration::hours(1);
        let mut options = bounded_options(std::time::Duration::ZERO, usize::MAX, usize::MAX);
        options.dry_run = true;
        storage
            .collect_orphans_at(&options, now, tokio::time::Instant::now())
            .await
            .unwrap();

        let reported = storage
            .collect_orphans_at(
                &options,
                now + chrono::Duration::seconds(1),
                tokio::time::Instant::now(),
            )
            .await
            .unwrap();
        assert_eq!(reported.candidate_objects, retired.len() * PART_FILES.len());
        assert!(reported.candidate_bytes > 0);
        assert_eq!(reported.deleted_objects, 0);
        for part in &retired {
            assert!(storage.store.head(&storage.part_path(part, DATA_FILE)).await.is_ok());
        }

        options.dry_run = false;
        let deleted = storage
            .collect_orphans_at(
                &options,
                now + chrono::Duration::seconds(2),
                tokio::time::Instant::now(),
            )
            .await
            .unwrap();
        assert_eq!(
            deleted.deleted_objects, reported.candidate_objects,
            "the real pass deletes exactly what the dry run reported"
        );
    }

    /// The delete budget bounds one pass without hiding how much is waiting.
    #[tokio::test]
    async fn the_delete_budget_bounds_a_pass_and_still_reports_the_rest() {
        let storage = ObjectStorage::in_memory();
        let (_, retired) = store_with_orphans(&storage, "gc-delete-budget", 3).await;
        let total = retired.len() * PART_FILES.len();
        let now = chrono::Utc::now() + chrono::Duration::hours(1);
        let options = bounded_options(std::time::Duration::ZERO, usize::MAX, 2);
        storage
            .collect_orphans_at(&options, now, tokio::time::Instant::now())
            .await
            .unwrap();

        let outcome = storage
            .collect_orphans_at(
                &options,
                now + chrono::Duration::seconds(1),
                tokio::time::Instant::now(),
            )
            .await
            .unwrap();
        assert_eq!(outcome.deleted_objects, 2, "the budget is what one pass deletes");
        assert_eq!(outcome.candidate_objects, total, "and the rest is still reported");

        let byte_bounded = OrphanCollectionOptions {
            max_deleted_objects: usize::MAX,
            max_deleted_bytes: 1,
            ..bounded_options(std::time::Duration::ZERO, usize::MAX, usize::MAX)
        };
        let outcome = storage
            .collect_orphans_at(
                &byte_bounded,
                now + chrono::Duration::seconds(2),
                tokio::time::Instant::now(),
            )
            .await
            .unwrap();
        assert_eq!(
            outcome.deleted_objects, 1,
            "a byte budget smaller than one object still makes progress"
        );
    }

    /// An entry whose object something else removed has to leave the ledger,
    /// or the ledger grows without bound over the store's lifetime.
    #[tokio::test]
    async fn a_completed_cycle_forgets_entries_whose_objects_are_gone() {
        let storage = ObjectStorage::in_memory();
        let (_, retired) = store_with_orphans(&storage, "gc-ledger-prune", 2).await;
        let now = chrono::Utc::now() + chrono::Duration::hours(1);
        // A grace nothing reaches, so the ledger can only shrink by forgetting.
        let options = bounded_options(std::time::Duration::from_secs(86_400), usize::MAX, usize::MAX);
        let first = storage
            .collect_orphans_at(&options, now, tokio::time::Instant::now())
            .await
            .unwrap();
        assert_eq!(first.ledger_entries, retired.len() * PART_FILES.len());

        for part in &retired {
            for file in PART_FILES {
                storage.store.delete(&storage.part_path(part, file)).await.unwrap();
            }
        }
        let second = storage
            .collect_orphans_at(
                &options,
                now + chrono::Duration::seconds(1),
                tokio::time::Instant::now(),
            )
            .await
            .unwrap();
        assert_eq!(second.scan_cycles_completed, 1);
        assert_eq!(second.ledger_entries, 0, "a closed cycle drops what it did not see");
    }

    /// A ledger written before the scan became resumable still has to be read,
    /// or every object in it is handed a fresh grace by the upgrade.
    #[tokio::test]
    async fn a_first_sightings_ledger_from_an_older_version_still_ages() {
        let storage = ObjectStorage::in_memory();
        let (_, retired) = store_with_orphans(&storage, "gc-ledger-upgrade", 2).await;
        let retired_at = chrono::Utc::now() + chrono::Duration::hours(1);
        let sightings: BTreeMap<String, String> = retired
            .iter()
            .flat_map(|part| {
                PART_FILES
                    .iter()
                    .map(|file| (storage.part_path(part, file).to_string(), retired_at.to_rfc3339()))
                    .collect::<Vec<_>>()
            })
            .collect();
        storage
            .store
            .put(
                &storage.path(GC_ORPHANS_FILE),
                Bytes::from(serde_json::to_vec(&sightings).unwrap()).into(),
            )
            .await
            .unwrap();

        let options = bounded_options(std::time::Duration::from_secs(300), usize::MAX, usize::MAX);
        let outcome = storage
            .collect_orphans_at(
                &options,
                retired_at + chrono::Duration::seconds(301),
                tokio::time::Instant::now(),
            )
            .await
            .unwrap();
        assert_eq!(
            outcome.deleted_objects,
            retired.len() * PART_FILES.len(),
            "the sightings the older ledger recorded are what makes them deletable"
        );
    }

    /// Catalog history is pruned on its own terms. It used to be sequenced
    /// behind orphan collection inside one function, so a store whose part
    /// listing failed also kept every catalog commit it had ever written.
    #[tokio::test]
    async fn catalog_pruning_runs_even_when_orphan_collection_fails() {
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        let storage = ObjectStorage::from_store(Arc::new(ScriptedStore::wrapping(inner.clone())), "signy-test");
        commit_alternating_trace_generations(&storage, CATALOG_SNAPSHOT_INTERVAL * 3 + 5).await;
        let commits_before = count_objects(&storage, "catalog/commits").await;

        let blinded = Arc::new(ObjectStorage::from_store(
            Arc::new(ScriptedStore::wrapping(inner).refusing_listing_of("parts")),
            "signy-test",
        ));
        let cache = RemoteCache::new(blinded.clone(), temp_dir("gc-independent").join("parts"));
        let config = crate::config::Config {
            catalog_prune_min_age: Some(std::time::Duration::from_secs(0)),
            ..Default::default()
        };
        let metrics = crate::metrics::RuntimeMetrics::new();

        let (orphans, catalog) =
            crate::object_store_gc::object_store_gc_pass(&cache, &config, &metrics).await;

        assert!(orphans.is_err(), "the part listing this store refuses is what fails collection");
        assert!(catalog.is_ok(), "and pruning runs anyway");
        assert!(
            count_objects(&blinded, "catalog/commits").await < commits_before,
            "superseded commits are gone"
        );
    }

    async fn count_objects(storage: &ObjectStorage, relative_prefix: &str) -> usize {
        use futures_util::StreamExt;
        storage
            .store
            .list(Some(&storage.path(relative_prefix)))
            .filter(|item| futures_util::future::ready(item.is_ok()))
            .count()
            .await
    }
