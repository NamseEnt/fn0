use crate::series::MetricValue;
    use super::*;
    use crate::memtable::MemTable;
    use crate::tenant::test_tenant;
    use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
    use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use std::sync::Arc;

    fn tmp_dir(name: &str) -> PathBuf {
        crate::test_support::temp_dir(&format!("journal-{name}"))
    }

    /// An encoded `ExportLogsServiceRequest`: one `ResourceLogs` per `(app,
    /// entries)` pair, the app riding as `service.name` so it lands as the
    /// promoted `service_name` label. Timestamps are seconds, as the Loki-push
    /// helper this replaces took them.
    fn make_otlp_req(streams: &[(&str, Vec<(&str, i64)>)]) -> Vec<u8> {
        let request = ExportLogsServiceRequest {
            resource_logs: streams
                .iter()
                .map(|(app, entries)| ResourceLogs {
                    resource: Some(Resource {
                        attributes: vec![KeyValue {
                            key: "service.name".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue(app.to_string())),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                    scope_logs: vec![ScopeLogs {
                        log_records: entries
                            .iter()
                            .map(|(line, ts_secs)| LogRecord {
                                time_unix_nano: (*ts_secs as u64) * 1_000_000_000,
                                body: Some(AnyValue {
                                    value: Some(any_value::Value::StringValue(line.to_string())),
                                }),
                                ..Default::default()
                            })
                            .collect(),
                        ..Default::default()
                    }],
                    ..Default::default()
                })
                .collect(),
        };
        Prost014Message::encode_to_vec(&request)
    }

    struct Harness {
        journal: Journal,
        memtable: Arc<MemTable>,
        dir: PathBuf,
    }

    async fn harness(name: &str) -> Harness {
        let dir = tmp_dir(name);
        let config = Config {
            data_dir: dir.clone(),
            ..Config::default()
        };
        let memtable = Arc::new(MemTable::new());
        let journal = Journal::spawn(&config, memtable.clone()).unwrap();
        Harness {
            journal,
            memtable,
            dir,
        }
    }

    fn sender(byte: u8) -> SenderId {
        SenderId::parse(&format!("{byte:02x}").repeat(16)).expect("a sender id")
    }

    async fn push_marked(h: &Harness, raw: Vec<u8>, mark: CollectMark) {
        let request = ExportLogsServiceRequest::decode(raw.as_slice()).unwrap();
        let streams = crate::otlp_log::normalize_request(request).unwrap();
        h.journal
            .enqueue_otlp_logs(test_tenant(), raw, streams, Some(mark))
            .await
            .unwrap()
            .settle()
            .await
            .unwrap();
    }

    async fn push(h: &Harness, raw: Vec<u8>) {
        let request = ExportLogsServiceRequest::decode(raw.as_slice()).unwrap();
        let streams = crate::otlp_log::normalize_request(request).unwrap();
        h.journal
            .append_otlp_logs(test_tenant(), raw, streams)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn a_failed_metric_batch_enqueue_rolls_back_every_tenant_portion() {
        let dir = tmp_dir("metric_batch_send_failure");
        std::fs::create_dir_all(&dir).unwrap();
        let (tx, rx) = mpsc::channel(1);
        let series_memtable = Arc::new(crate::series::SeriesMemTable::new());
        let journal = Journal {
            tx,
            collect_marks: Arc::new(CollectMarks::load(&dir)),
            wal_path: dir.join(WAL_FILE),
            ckpt_path: dir.join(CKPT_FILE),
            healthy: Arc::new(AtomicBool::new(true)),
            metrics: Arc::new(JournalMetrics::default()),
            memtable: Arc::new(MemTable::new()),
            trace_memtable: Arc::new(TraceMemTable::new()),
            series_memtable: series_memtable.clone(),
            backlog: Arc::new(WalBacklog::default()),
            metric_reserved_bytes: Arc::new(AtomicU64::new(0)),
        };
        let first_tenant = test_tenant();
        let second_tenant = TenantId::parse("other").unwrap();
        let first_labels = crate::series::SeriesLabels::from_pairs(vec![
            (crate::series::METRIC_NAME_LABEL.to_string(), "one".to_string()),
        ]);
        let second_labels = crate::series::SeriesLabels::from_pairs(vec![
            (crate::series::METRIC_NAME_LABEL.to_string(), "two".to_string()),
        ]);
        let first_sample = crate::series::MetricSample {
            tenant: first_tenant.clone(),
            labels: first_labels,
            ts_ns: 1,
            value: MetricValue::Scalar(1.0),
            kind: crate::series::SampleKind::Gauge,
            datapoint_index: 0,
        };
        let second_sample = crate::series::MetricSample {
            tenant: second_tenant.clone(),
            labels: second_labels,
            ts_ns: 1,
            value: MetricValue::Scalar(2.0),
            kind: crate::series::SampleKind::Gauge,
            datapoint_index: 0,
        };
        let groups = vec![
            (&first_tenant, std::slice::from_ref(&first_sample)),
            (&second_tenant, std::slice::from_ref(&second_sample)),
        ];
        let mut admissions = series_memtable
            .admit_request(&groups, None, i64::MIN)
            .unwrap()
            .into_iter();
        let first_admission = admissions.next().unwrap();
        let second_admission = admissions.next().unwrap();
        let first_permit = journal.try_reserve_metric_bytes(1, Some(u64::MAX)).unwrap();
        let second_permit = journal.try_reserve_metric_bytes(1, Some(u64::MAX)).unwrap();
        drop(rx);

        let error = match journal
            .enqueue_metrics_reserved_batch(vec![
                ReservedMetricAppend {
                    tenant: first_tenant.clone(),
                    data: Vec::new(),
                    samples: vec![first_sample],
                    mark: None,
                    metric_memory_permit: first_permit,
                    metric_series_admission: first_admission,
                },
                ReservedMetricAppend {
                    tenant: second_tenant.clone(),
                    data: Vec::new(),
                    samples: vec![second_sample],
                    mark: None,
                    metric_memory_permit: second_permit,
                    metric_series_admission: second_admission,
                },
            ])
            .await
        {
            Ok(_) => panic!("a closed journal rejects the complete batch"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        assert_eq!(series_memtable.active_series(&first_tenant), 0);
        assert_eq!(series_memtable.active_series(&second_tenant), 0);
        assert_eq!(journal.metric_reserved_bytes(), 0);
    }

    /// The push path's four phases are measured, and by the writer task rather
    /// than by the caller.
    ///
    /// This is the instrument the push-tail argument was missing: a p50 of
    /// 12 ms against a p95 that moved between 40 and 106 ms with nothing but
    /// the client's connection count (`todo.md`, 2026-08-12) is a queue in
    /// front of one writer, and no number in the process could say which phase
    /// it was. A test that only asserted the histograms exist would not catch
    /// the way this breaks — a phase wired to the wrong instant reads zero
    /// forever — so every phase an append passes through must have observed it.
    #[tokio::test]
    async fn an_append_is_measured_in_each_phase_the_writer_puts_it_through() {
        let h = harness("phase_metrics").await;
        let metrics = h.journal.metrics().clone();
        for phase in [
            &metrics.append_queue_wait,
            &metrics.batch_write,
            &metrics.batch_fsync,
            &metrics.batch_insert,
            &metrics.checkpoint,
        ] {
            assert_eq!(phase.count(), 0, "nothing is measured before the push");
        }

        push(&h, make_otlp_req(&[("a", vec![("hi", 100)])])).await;
        assert_eq!(metrics.append_queue_wait.count(), 1);
        assert_eq!(metrics.batch_write.count(), 1);
        assert_eq!(metrics.batch_fsync.count(), 1);
        assert_eq!(metrics.batch_insert.count(), 1);
        assert_eq!(
            metrics.checkpoint.count(),
            0,
            "a checkpoint is the flush's ask, not the push's"
        );
        assert_eq!(metrics.batches.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.batched_records.load(Ordering::Relaxed), 1);

        h.journal.checkpoint().await.unwrap();
        assert_eq!(
            metrics.checkpoint.count(),
            1,
            "a checkpoint runs in the same task, so its cost is time no push \
can be written in"
        );
    }

    #[tokio::test]
    async fn append_and_checkpoint() {
        let h = harness("append_checkpoint").await;
        push(&h, make_otlp_req(&[("a", vec![("hi", 100)])])).await;
        push(&h, make_otlp_req(&[("b", vec![("yo", 200)])])).await;

        let ckpt = h.journal.checkpoint().await.unwrap();
        assert!(ckpt.offset > 0);
        assert_eq!(ckpt.snapshot.len(), 1, "one tenant produced both streams");
        assert_eq!(ckpt.snapshot[&test_tenant()].len(), 2);
        h.journal.set_checkpoint(ckpt.offset).unwrap();

        let (start, end) = replay(
            h.journal.wal_path(),
            h.journal.ckpt_path(),
            &MemTable::new(),
        )
        .unwrap();
        assert_eq!(start, ckpt.offset);
        assert_eq!(end, ckpt.offset);
    }

    /// A WAL written before journal compression carries raw protobuf where a
    /// zstd frame now belongs. This engine versions nothing, so the failure
    /// must be loud and say what to do — not decode garbage.
    #[tokio::test]
    async fn an_uncompressed_record_refuses_replay_with_instructions() {
        let dir = tmp_dir("uncompressed-wal");
        let wal_path = dir.join("journal.wal");
        let ckpt_path = dir.join("journal.ckpt");
        let payload = frame_tenant_record(
            &test_tenant(),
            TENANT_RECORD_KIND_OTLP_LOGS,
            &make_otlp_req(&[("a", vec![("x", 1)])]),
        );
        let mut wal = Vec::new();
        wal.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        wal.extend_from_slice(&crc32fast::hash(&payload).to_le_bytes());
        wal.extend_from_slice(&payload);
        std::fs::write(&wal_path, &wal).unwrap();

        let error = replay(&wal_path, &ckpt_path, &MemTable::new())
            .expect_err("a raw-protobuf record must refuse replay");
        assert!(error.contains("delete the data directory"), "{error}");
    }

    /// The WAL holds the OTLP export as it arrived, so replay must produce
    /// exactly what ingest's own normalization produced before the crash —
    /// promoted labels, structured metadata and all.
    #[tokio::test]
    async fn an_otlp_log_record_replays_by_its_own_kind() {
        use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
        use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
        use opentelemetry_proto::tonic::resource::v1::Resource;

        let attr = |key: &str, value: &str| KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.to_string())),
            }),
            ..Default::default()
        };
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![attr("service.name", "api")],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        time_unix_nano: 100,
                        body: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("hello".to_string())),
                        }),
                        attributes: vec![attr("trace_id", "abc123")],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let encoded = Prost014Message::encode_to_vec(&request);
        let streams = crate::otlp_log::normalize_request(request).unwrap();

        let h = harness("otlp_replay").await;
        h.journal
            .append_otlp_logs(test_tenant(), encoded, streams)
            .await
            .unwrap();

        let restored = MemTable::new();
        replay(
            h.journal.wal_path(),
            h.journal.ckpt_path(),
            &restored,
        )
        .unwrap();
        let results = restored.query(&test_tenant(), &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            10,
            true,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entries[0].line, "hello");
        assert_eq!(
            results[0].entries[0].structured_metadata,
            vec![
                ("service_name".to_string(), "api".to_string()),
                ("trace_id".to_string(), "abc123".to_string()),
            ]
        );
    }

    #[tokio::test]
    async fn compact_checkpoint_retains_appends_after_snapshot() {
        let h = harness("compact_retains_suffix").await;
        push(
            &h,
            make_otlp_req(&[("flushed", vec![("old", 100)])]),
        )
        .await;
        let checkpoint = h.journal.checkpoint().await.unwrap();
        h.memtable.commit_flush();

        push(
            &h,
            make_otlp_req(&[("inflight", vec![("new", 200)])]),
        )
        .await;
        let before = std::fs::metadata(h.journal.wal_path()).unwrap().len();
        h.journal
            .compact_checkpoint(checkpoint.offset)
            .await
            .unwrap();

        assert_eq!(read_checkpoint(h.journal.ckpt_path()).unwrap(), 0);
        let after = std::fs::metadata(h.journal.wal_path()).unwrap().len();
        assert!(after < before);
        let restored = MemTable::new();
        replay(
            h.journal.wal_path(),
            h.journal.ckpt_path(),
            &restored,
        ).unwrap();
        let results = restored.query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true);
        let lines: Vec<_> = results
            .iter()
            .flat_map(|stream| stream.entries.iter().map(|entry| entry.line.as_str()))
            .collect();
        assert_eq!(lines, vec!["new"]);
    }

    #[tokio::test]
    async fn compaction_failure_does_not_fence_journal_writer() {
        let h = harness("compact_retry").await;
        push(&h, make_otlp_req(&[("old", vec![("old", 100)])])).await;
        let checkpoint = h.journal.checkpoint().await.unwrap();
        h.memtable.commit_flush();

        let compact_tmp = h.journal.wal_path().with_extension("wal.compact.tmp");
        std::fs::create_dir_all(&compact_tmp).unwrap();
        assert!(
            h.journal
                .compact_checkpoint(checkpoint.offset)
                .await
                .is_err()
        );
        std::fs::remove_dir(&compact_tmp).unwrap();

        push(&h, make_otlp_req(&[("new", vec![("new", 200)])])).await;
        h.journal
            .compact_checkpoint(checkpoint.offset)
            .await
            .unwrap();
        let restored = MemTable::new();
        replay(
            h.journal.wal_path(),
            h.journal.ckpt_path(),
            &restored,
        ).unwrap();
        let lines: Vec<_> = restored
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
            .into_iter()
            .flat_map(|stream| stream.entries.into_iter().map(|entry| entry.line))
            .collect();
        assert_eq!(lines, vec!["new"]);
    }

    #[tokio::test]
    async fn compaction_retry_after_rename_failure_keeps_acknowledged_suffix() {
        let h = harness("compact_rename_retry").await;
        push(&h, make_otlp_req(&[("old", vec![("old", 100)])])).await;
        let checkpoint = h.journal.checkpoint().await.unwrap();
        h.memtable.commit_flush();

        inject_compaction_fault(h.journal.wal_path(), CompactionFault::AfterRename);
        assert!(
            h.journal
                .compact_checkpoint(checkpoint.offset)
                .await
                .is_err()
        );

        // The writer was reopened after the injected post-rename failure;
        // this append must remain in the replacement WAL before retry.
        push(&h, make_otlp_req(&[("new", vec![("new", 200)])])).await;
        h.journal
            .compact_checkpoint(checkpoint.offset)
            .await
            .unwrap();

        let restored = MemTable::new();
        replay(
            h.journal.wal_path(),
            h.journal.ckpt_path(),
            &restored,
        ).unwrap();
        let lines: Vec<_> = restored
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
            .into_iter()
            .flat_map(|stream| stream.entries.into_iter().map(|entry| entry.line))
            .collect();
        assert_eq!(lines, vec!["new"]);
    }

    #[tokio::test]
    async fn replay_rolls_back_uncommitted_compaction_before_rename() {
        let h = harness("compact_replay_rollback").await;
        push(&h, make_otlp_req(&[("old", vec![("old", 100)])])).await;
        let checkpoint = h.journal.checkpoint().await.unwrap();
        h.memtable.commit_flush();

        let source_len = std::fs::metadata(h.journal.wal_path()).unwrap().len();
        let state_path = h.journal.wal_path().with_file_name(COMPACTION_STATE_FILE);
        let tmp_path = h.journal.wal_path().with_extension("wal.compact.tmp");
        let state = CompactionState {
            offset: checkpoint.offset,
            source_len,
            retained_len: 0,
        };
        write_compaction_state(&state_path, &state).unwrap();
        write_checkpoint(h.journal.ckpt_path(), 0).unwrap();
        std::fs::write(&tmp_path, []).unwrap();

        let restored = MemTable::new();
        replay(
            h.journal.wal_path(),
            h.journal.ckpt_path(),
            &restored,
        ).unwrap();
        assert!(
            restored
                .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
                .is_empty()
        );
        assert_eq!(
            read_checkpoint(h.journal.ckpt_path()).unwrap(),
            checkpoint.offset
        );
        assert!(!state_path.exists());
        assert!(!tmp_path.exists());
    }

    /// Records the caller just flushed, so the next compaction offset is a
    /// fresh one in whatever coordinate system the WAL is currently in.
    async fn flush_round(harness: &Harness, label: &str, line: &str) -> u64 {
        push(
            harness,
            make_otlp_req(&[(label, vec![(line, 100)])]),
        )
        .await;
        let checkpoint = harness.journal.checkpoint().await.unwrap();
        harness.memtable.commit_flush();
        checkpoint.offset
    }

    /// Compaction resets the checkpoint to zero, so every offset after the
    /// first one lives in a new coordinate system. Comparing them across that
    /// reset used to wedge the flush loop forever whenever a batch was smaller
    /// than its predecessor, which is most batches.
    #[tokio::test]
    async fn consecutive_compactions_truncate_whatever_the_batch_sizes_are() {
        let h = harness("consecutive_compactions").await;
        // Shrinking, then equal, then growing: the three ways the next offset
        // can compare against the previous one.
        for (label, line) in [
            ("aaaaaaaaaaaaaaaa", "first"),
            ("b", "second"),
            ("b", "second"),
            ("cccccccccccccccccccccccc", "third"),
        ] {
            let offset = flush_round(&h, label, line).await;
            h.journal.compact_checkpoint(offset).await.unwrap();
            assert_eq!(
                std::fs::metadata(h.journal.wal_path()).unwrap().len(),
                0,
                "compaction left the WAL untruncated for {label}"
            );
            assert_eq!(read_checkpoint(h.journal.ckpt_path()).unwrap(), 0);
        }
        let state_path = h.journal.wal_path().with_file_name(COMPACTION_STATE_FILE);
        assert!(!state_path.exists(), "the intent record outlived compaction");
    }

    /// The removal is the last durable step. A crash there leaves a phase-1
    /// record whose rename already committed. `flush.rs` retries the same
    /// offset, which that record must absorb as "already done" — applying a
    /// pre-reset offset to the replacement WAL would truncate live records.
    /// Only after it is retired can the next offset compact normally.
    #[tokio::test]
    async fn a_failed_state_removal_is_settled_by_the_callers_retry() {
        let h = harness("compact_state_removal_retry").await;
        let first = flush_round(&h, "old", "old").await;
        inject_compaction_fault(h.journal.wal_path(), CompactionFault::BeforeStateRemoval);
        assert!(h.journal.compact_checkpoint(first).await.is_err());

        let state_path = h.journal.wal_path().with_file_name(COMPACTION_STATE_FILE);
        assert!(state_path.exists(), "the injected failure kept the record");

        // The retry `flush.rs` issues: the same offset, now stale.
        h.journal.compact_checkpoint(first).await.unwrap();
        assert!(!state_path.exists());

        // A fresh offset in the post-reset coordinate system still compacts.
        let second = flush_round(&h, "new", "new").await;
        h.journal.compact_checkpoint(second).await.unwrap();
        assert_eq!(std::fs::metadata(h.journal.wal_path()).unwrap().len(), 0);

        let restored = MemTable::new();
        replay(
            h.journal.wal_path(),
            h.journal.ckpt_path(),
            &restored,
        )
        .unwrap();
        assert!(
            restored
                .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
                .is_empty(),
            "both batches were flushed, so replay must yield nothing"
        );
    }

    /// A single connection used to be capped at `1000/max_batch_ms` pushes per
    /// second because the batch loop waited out the full linger even with an
    /// empty channel. Sequential appends must now cost what the writes cost,
    /// not what the timer costs.
    #[tokio::test]
    async fn sequential_appends_do_not_wait_out_a_batch_timer() {
        let h = harness("no_batch_linger").await;
        assert_eq!(
            Config::default().max_batch_ms,
            0,
            "the default must not linger"
        );

        let started = std::time::Instant::now();
        for index in 0..20 {
            push(
                &h,
                make_otlp_req(&[("nolinger", vec![("line", 100 + index)])]),
            )
            .await;
        }
        let elapsed = started.elapsed();

        // 20 sequential round trips. Under the old default (200 ms linger) this
        // was ~4 s; the bound here is loose enough to survive a slow disk and
        // still fail decisively if the timer comes back.
        assert!(
            elapsed < Duration::from_millis(1500),
            "20 sequential appends took {elapsed:?}, which means something is waiting"
        );
    }

    /// The linger still works when asked for: it is an opt-in trade of latency
    /// for fewer fsyncs, not the default.
    #[tokio::test]
    async fn a_configured_linger_still_batches() {
        let dir = tmp_dir("batch_linger");
        let config = Config {
            data_dir: dir,
            max_batch_ms: 80,
            ..Config::default()
        };
        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());

        // Two appends issued concurrently: the second must join the first's
        // batch rather than waiting for its own.
        let started = std::time::Instant::now();
        let one = {
            let journal = journal.clone();
            tokio::spawn(async move {
                journal
                    .append_otlp_logs(
                        test_tenant(),
                        make_otlp_req(&[("a", vec![("first", 100)])]),
                        Vec::new(),
                    )
                    .await
            })
        };
        let two = {
            let journal = journal.clone();
            tokio::spawn(async move {
                journal
                    .append_otlp_logs(
                        test_tenant(),
                        make_otlp_req(&[("b", vec![("second", 200)])]),
                        Vec::new(),
                    )
                    .await
            })
        };
        one.await.unwrap().unwrap();
        two.await.unwrap().unwrap();

        assert!(
            started.elapsed() < Duration::from_millis(400),
            "both appends should have shared one linger window"
        );
    }

    #[tokio::test]
    async fn health_turns_false_when_writer_stops() {
        let h = harness("writer_health").await;
        let health = h.journal.healthy.clone();

        drop(h.journal);
        tokio::time::timeout(Duration::from_secs(1), async {
            while health.load(Ordering::Acquire) {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .expect("journal health did not reflect writer shutdown");
    }

    #[tokio::test]
    async fn replay_restores_unflushed_data() {
        let h = harness("replay_unflushed").await;
        push(
            &h,
            make_otlp_req(&[("a", vec![("line1", 100), ("line2", 200)])]),
        )
        .await;
        let mt = MemTable::new();
        let (start, end) = replay(h.journal.wal_path(), h.journal.ckpt_path(), &mt).unwrap();
        assert_eq!(start, 0);
        assert!(end > 0);
        let results = mt.query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true);
        let total: usize = results.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 2);
    }

    #[test]
    fn replay_truncates_crc_corruption_at_tail() {
        let dir = tmp_dir("replay_crc_corruption");
        let wal_path = dir.join(WAL_FILE);
        let ckpt_path = dir.join(CKPT_FILE);
        let data = make_otlp_req(&[("a", vec![("line", 100)])]);
        let mut record = Vec::new();
        record.extend_from_slice(&(data.len() as u32).to_le_bytes());
        record.extend_from_slice(&(crc32fast::hash(&data) ^ 1).to_le_bytes());
        record.extend_from_slice(&data);
        std::fs::write(&wal_path, record).unwrap();

        let (start, end) = replay(&wal_path, &ckpt_path, &MemTable::new()).unwrap();

        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn replay_does_not_allocate_declared_length_for_a_partial_tail() {
        let dir = tmp_dir("replay_oversized_partial_tail");
        let wal_path = dir.join(WAL_FILE);
        let ckpt_path = dir.join(CKPT_FILE);
        let mut header = Vec::new();
        header.extend_from_slice(&u32::MAX.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());
        std::fs::write(&wal_path, header).unwrap();

        let (start, end) = replay(&wal_path, &ckpt_path, &MemTable::new()).unwrap();

        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn replay_rejects_crc_corruption_before_valid_records() {
        let dir = tmp_dir("replay_interior_crc_corruption");
        let wal_path = dir.join(WAL_FILE);
        let ckpt_path = dir.join(CKPT_FILE);
        let first = make_otlp_req(&[("a", vec![("bad", 100)])]);
        let second = make_otlp_req(&[("b", vec![("good", 200)])]);
        let mut wal = Vec::new();
        wal.extend_from_slice(&(first.len() as u32).to_le_bytes());
        wal.extend_from_slice(&(crc32fast::hash(&first) ^ 1).to_le_bytes());
        wal.extend_from_slice(&first);
        wal.extend_from_slice(&(second.len() as u32).to_le_bytes());
        wal.extend_from_slice(&crc32fast::hash(&second).to_le_bytes());
        wal.extend_from_slice(&second);
        std::fs::write(&wal_path, wal).unwrap();

        let result = replay(&wal_path, &ckpt_path, &MemTable::new());

        assert!(result.is_err());
    }

    #[test]
    fn replay_rejects_checkpoint_without_wal() {
        let dir = tmp_dir("checkpoint_without_wal");
        let wal_path = dir.join(WAL_FILE);
        let ckpt_path = dir.join(CKPT_FILE);
        write_checkpoint(&ckpt_path, 128).unwrap();

        let result = replay(&wal_path, &ckpt_path, &MemTable::new());

        assert!(result.is_err());
    }

    #[test]
    fn replay_rejects_checkpoint_beyond_wal() {
        let dir = tmp_dir("checkpoint_beyond_wal");
        let wal_path = dir.join(WAL_FILE);
        let ckpt_path = dir.join(CKPT_FILE);
        std::fs::write(&wal_path, [0u8; 16]).unwrap();
        write_checkpoint(&ckpt_path, 32).unwrap();

        let result = replay(&wal_path, &ckpt_path, &MemTable::new());

        assert!(result.is_err());
    }

    #[test]
    fn replay_rejects_malformed_checkpoint() {
        let dir = tmp_dir("malformed_checkpoint");
        let wal_path = dir.join(WAL_FILE);
        let ckpt_path = dir.join(CKPT_FILE);
        std::fs::write(&wal_path, []).unwrap();

        for bytes in [&[1u8, 2, 3][..], &[0u8; 9][..]] {
            std::fs::write(&ckpt_path, bytes).unwrap();
            let error = replay(&wal_path, &ckpt_path, &MemTable::new())
                .expect_err("malformed checkpoint must stop recovery");
            assert!(error.contains("exactly 8 bytes"));
        }
    }

    #[test]
    fn spawn_reports_wal_open_failure_synchronously() {
        let dir = tmp_dir("spawn_open_failure");
        std::fs::create_dir_all(dir.join(WAL_FILE)).unwrap();
        let config = Config {
            data_dir: dir,
            ..Config::default()
        };

        let result = Journal::spawn(&config, std::sync::Arc::new(MemTable::new()));

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn checkpoint_clears_memtable_and_persists_offset() {
        let h = harness("ckpt_clear").await;
        push(&h, make_otlp_req(&[("a", vec![("x", 1)])])).await;
        let ckpt = h.journal.checkpoint().await.unwrap();
        // checkpoint clears inner and moves the data to the flushing buffer; unified_query still sees it.
        // Call commit_flush to simulate completed flushing.
        h.memtable.commit_flush();
        h.journal.set_checkpoint(ckpt.offset).unwrap();
        assert_eq!(h.memtable.approximate_size(), 0);

        push(&h, make_otlp_req(&[("b", vec![("y", 2)])])).await;

        let mt = MemTable::new();
        replay(h.journal.wal_path(), h.journal.ckpt_path(), &mt).unwrap();
        let results = mt.query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true);
        let total: usize = results.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn replay_restores_each_record_under_its_own_tenant() {
        let harness = harness("replay_tenant").await;
        let acme = TenantId::parse("acme").unwrap();
        let globex = TenantId::parse("globex").unwrap();
        harness
            .journal
            .append_otlp_logs(
                acme.clone(),
                make_otlp_req(&[("a", vec![("acme line", 100)])]),
                vec![],
            )
            .await
            .unwrap();
        harness
            .journal
            .append_otlp_logs(
                globex.clone(),
                make_otlp_req(&[("b", vec![("globex line", 200)])]),
                vec![],
            )
            .await
            .unwrap();

        let restored = MemTable::new();
        replay(
            harness.journal.wal_path(),
            harness.journal.ckpt_path(),
            &restored,
        )
        .unwrap();

        let lines = |tenant: &TenantId| -> Vec<String> {
            restored
                .query(tenant, &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
                .into_iter()
                .flat_map(|stream| stream.entries)
                .map(|entry| entry.line)
                .collect()
        };
        assert_eq!(lines(&acme), vec!["acme line"]);
        assert_eq!(lines(&globex), vec!["globex line"]);
        assert!(lines(&test_tenant()).is_empty());
    }

    /// Records this engine no longer writes fail replay loudly with the
    /// instruction the no-versioning policy implies: delete and re-ingest.
    /// Silently skipping either would drop acknowledged data on the floor.
    #[test]
    fn replay_refuses_records_from_before_the_otlp_only_change() {
        // A tenant-framed kind-0 record, the Loki push encoding.
        let dir = tmp_dir("pre_otlp_record");
        let wal_path = dir.join(WAL_FILE);
        let ckpt_path = dir.join(CKPT_FILE);
        let payload = frame_tenant_record(&test_tenant(), TENANT_RECORD_KIND_LOGS, b"anything");
        let mut wal = Vec::new();
        wal.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        wal.extend_from_slice(&crc32fast::hash(&payload).to_le_bytes());
        wal.extend_from_slice(&payload);
        // A valid record after it, so the refusal cannot be mistaken for the
        // tolerated corrupt-tail case.
        let good = frame_tenant_record(
            &test_tenant(),
            TENANT_RECORD_KIND_OTLP_LOGS,
            &make_otlp_req(&[("a", vec![("x", 1)])]),
        );
        wal.extend_from_slice(&(good.len() as u32).to_le_bytes());
        wal.extend_from_slice(&crc32fast::hash(&good).to_le_bytes());
        wal.extend_from_slice(&good);
        std::fs::write(&wal_path, &wal).unwrap();

        let error = replay(&wal_path, &ckpt_path, &MemTable::new())
            .expect_err("a pre-OTLP record must refuse replay");
        assert!(error.contains("delete the data directory"), "{error}");

        // An unframed record, the pre-tenancy WAL form, same policy.
        let unframed_dir = tmp_dir("unframed_record");
        let unframed_wal = unframed_dir.join(WAL_FILE);
        let payload = make_otlp_req(&[("legacy", vec![("bare", 100)])]);
        let mut wal = Vec::new();
        for _ in 0..2 {
            wal.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            wal.extend_from_slice(&crc32fast::hash(&payload).to_le_bytes());
            wal.extend_from_slice(&payload);
        }
        std::fs::write(&unframed_wal, &wal).unwrap();

        let error = replay(
            &unframed_wal,
            &unframed_dir.join(CKPT_FILE),
            &MemTable::new(),
        )
        .expect_err("an unframed record must refuse replay");
        assert!(error.contains("delete the data directory"), "{error}");
    }

    /// A mark rides in the same batch as the records it covers, so replay
    /// recovers it from the WAL alone. Anything less and a restart would take
    /// a collecty's resend of records the WAL still held as new.
    #[tokio::test]
    async fn replay_recovers_where_each_sender_got_to() {
        let h = harness("marks_replay").await;
        let id = sender(0x11);
        for records in 1..=3 {
            push_marked(
                &h,
                make_otlp_req(&[("api", vec![("line", 1)])]),
                CollectMark {
                    sender: id,
                    signal: CollectSignal::Logs,
                    at: Position {
                        segment: 4,
                        records,
                    },
                },
            )
            .await;
        }
        let reached = Position {
            segment: 4,
            records: 3,
        };
        assert_eq!(
            h.journal.collect_marks().position(&id, CollectSignal::Logs),
            reached
        );

        let recovered = CollectMarks::default();
        let memtable = MemTable::new();
        replay_reporting(
            h.journal.wal_path(),
            h.journal.ckpt_path(),
            &memtable,
            &TraceMemTable::new(),
            &SeriesMemTable::new(),
            &recovered,
        )
        .unwrap();

        assert_eq!(recovered.position(&id, CollectSignal::Logs), reached);
        assert_eq!(
            recovered.position(&id, CollectSignal::Traces),
            Position::START,
            "a signal is a stream of its own"
        );
        assert_eq!(
            recovered.position(&sender(0x22), CollectSignal::Logs),
            Position::START
        );
    }

    /// The WAL suffix is not enough on its own: a checkpoint retires the prefix
    /// the marks were written in, and a sender that has been quiet since would
    /// come back unknown. The file the checkpoint writes is what carries them
    /// across.
    #[tokio::test]
    async fn a_checkpoint_writes_the_marks_the_wal_prefix_would_lose() {
        let h = harness("marks_checkpoint").await;
        let id = sender(0x33);
        let reached = Position {
            segment: 9,
            records: 0,
        };
        push_marked(
            &h,
            make_otlp_req(&[("api", vec![("line", 1)])]),
            CollectMark {
                sender: id,
                signal: CollectSignal::Logs,
                at: reached,
            },
        )
        .await;
        let checkpoint = h.journal.checkpoint().await.unwrap();
        assert!(checkpoint.offset > 0);

        let carried = CollectMarks::load(&h.dir);
        assert_eq!(carried.position(&id, CollectSignal::Logs), reached);
        assert_eq!(
            carried.position(&id, CollectSignal::Logs).whole_segments(),
            8
        );
    }

    /// Never backwards, and a whole segment outranks any part of it. A mark
    /// that reaches the writer out of order, or a resend of one already
    /// covered, must not walk the position back over records whose twins would
    /// then be stored again.
    #[tokio::test]
    async fn a_position_only_moves_forward() {
        let marks = CollectMarks::default();
        let id = sender(0x44);
        let at = |signal, segment, records| CollectMark {
            sender: id,
            signal,
            at: Position { segment, records },
        };
        let logs = CollectSignal::Logs;

        marks.advance(at(logs, 3, 40));
        marks.advance(at(logs, 3, 12));
        assert_eq!(
            marks.position(&id, logs),
            Position {
                segment: 3,
                records: 40
            }
        );

        marks.advance(at(logs, 4, 0));
        marks.advance(at(logs, 3, 90));
        assert_eq!(
            marks.position(&id, logs),
            Position {
                segment: 4,
                records: 0
            }
        );
        assert_eq!(marks.position(&id, logs).whole_segments(), 3);

        // One signal's position says nothing about another's: they are
        // numbered apart and arrive interleaved.
        marks.advance(at(CollectSignal::Traces, 2, 5));
        assert_eq!(
            marks.position(&id, CollectSignal::Traces),
            Position {
                segment: 2,
                records: 5
            }
        );
        assert_eq!(
            marks.position(&id, logs),
            Position {
                segment: 4,
                records: 0
            },
            "and one does not move the other"
        );
    }
