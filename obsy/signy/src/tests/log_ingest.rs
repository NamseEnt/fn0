    use super::*;
    use crate::backpressure::IngestGate;
    use crate::config::Config;
    use crate::journal;
    use crate::memtable::MemTable;
    use crate::shutdown::ShutdownState;
    use crate::tenant::test_tenant;
    use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
    use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use std::sync::Arc;

    /// The collect route's own sequence over one record, so these tests refuse
    /// what the route refuses and store what it stores.
    struct Ingest {
        journal: Arc<crate::journal::Journal>,
        shutdown: Arc<ShutdownState>,
        config: Arc<Config>,
        ingest_gate: Arc<IngestGate>,
        tenant_quota: Arc<crate::tenant_quota::TenantQuota>,
        tenant_policy: Arc<crate::tenant_policy::TenantPolicy>,
        metrics: Arc<crate::metrics::RuntimeMetrics>,
        clock: Arc<crate::clock::Clock>,
    }

    impl Ingest {
        async fn accept(&self, request: ExportLogsServiceRequest) -> Result<(), IngestError> {
            let ingest = OtlpLogIngest {
                journal: &self.journal,
                config: &self.config,
                tenant_quota: &self.tenant_quota,
                tenant_policy: &self.tenant_policy,
                metrics: &self.metrics,
                clock: &self.clock,
            };
            crate::backpressure::admit_batch(&self.shutdown, &self.ingest_gate)?;
            let payload = request.encode_to_vec();
            ingest.admit_size(payload.len())?;
            for pending in ingest.enqueue_request(request, Some(payload), None).await? {
                pending.settle().await.map_err(journal_write_failed)?;
            }
            Ok(())
        }
    }

    fn fixture(config: Config) -> (Arc<MemTable>, Ingest) {
        std::fs::create_dir_all(&config.data_dir).unwrap();
        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(journal::Journal::spawn(&config, memtable.clone()).unwrap());
        let ingest = ingest_over(journal, &config);
        (memtable, ingest)
    }

    fn ingest_over(journal: Arc<crate::journal::Journal>, config: &Config) -> Ingest {
        let ingest_gate = IngestGate::for_test(&journal, config);
        Ingest {
            journal,
            shutdown: Arc::new(ShutdownState::new()),
            config: Arc::new(config.clone()),
            ingest_gate,
            tenant_quota: crate::tenant_quota::TenantQuota::for_test(config),
            tenant_policy: Arc::new(crate::tenant_policy::TenantPolicy::disabled()),
            metrics: Arc::new(crate::metrics::RuntimeMetrics::new()),
            clock: crate::clock::Clock::system(),
        }
    }

    fn config(label: &str) -> Config {
        Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-otlp-logs-{label}-{}", uuid::Uuid::new_v4())),
            ..Config::default()
        }
    }

    fn now_ns() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    /// The tenant rides in the payload, so what makes a request the test
    /// tenant's is the attribute stamped onto the resource here.
    fn request(records: Vec<LogRecord>) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue {
                            key: crate::otlp_tenant::TENANT_ATTRIBUTE.to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue(
                                    test_tenant().as_str().to_string(),
                                )),
                            }),
                            ..Default::default()
                        },
                        KeyValue {
                            key: "service.name".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue("checkout".to_string())),
                            }),
                            ..Default::default()
                        },
                    ],
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: records,
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }

    fn record(body: &str) -> LogRecord {
        LogRecord {
            time_unix_nano: now_ns(),
            body: Some(AnyValue {
                value: Some(any_value::Value::StringValue(body.to_string())),
            }),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn an_exported_record_is_queryable() {
        let (memtable, ingest) = fixture(config("accepted"));

        ingest
            .accept(request(vec![record("order placed")]))
            .await
            .expect("the export is accepted");

        let results = memtable.query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true);
        let entries: Vec<&str> = results
            .iter()
            .flat_map(|stream| stream.entries.iter())
            .map(|entry| entry.line.as_str())
            .collect();
        assert_eq!(entries, vec!["order placed"]);
        assert!(
            results[0]
                .entries[0]
                .structured_metadata
                .contains(&("service_name".to_string(), "checkout".to_string())),
            "the resource attribute rides in the entry's own metadata"
        );
    }

    /// The input bounds are the engine's, not a wire's. They exist to keep a
    /// line from being unbounded and a timestamp from landing in a partition
    /// retention has already swept, and neither depends on how the export got
    /// here.
    #[tokio::test]
    async fn the_input_limits_apply_to_a_collected_export() {
        let (memtable, ingest) = fixture(Config {
            max_line_bytes: 32,
            ..config("limits")
        });

        let error = ingest
            .accept(request(vec![record(&"x".repeat(64))]))
            .await
            .expect_err("an oversized line is refused");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);

        let (memtable_window, ingest_window) = fixture(Config {
            max_timestamp_age: Some(std::time::Duration::from_secs(60)),
            ..config("window")
        });
        let mut ancient = record("from last year");
        ancient.time_unix_nano = 1;
        let error = ingest_window
            .accept(request(vec![ancient]))
            .await
            .expect_err("a timestamp outside the window is refused");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);

        for table in [&memtable, &memtable_window] {
            assert!(
                table
                    .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
                    .is_empty(),
                "a refused export must not have been written"
            );
        }
    }

    /// A drained server refuses before the request is normalized.
    #[tokio::test]
    async fn a_drained_export_is_refused_before_it_is_normalized() {
        let (memtable, ingest) = fixture(config("drain"));
        ingest.shutdown.begin_drain();

        let error = ingest
            .accept(request(vec![record("while draining")]))
            .await
            .unwrap_err();

        assert_eq!(error.status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(
            memtable
                .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
                .is_empty(),
            "a refused export must not have been written"
        );
    }

    /// Backpressure is a state, not a latch. A full memtable refuses with the
    /// code a collecty holds its queue on, and a drained one accepts again.
    #[tokio::test]
    async fn an_export_is_refused_while_the_memtable_is_over_its_limit_and_recovers() {
        let config = Config {
            flush_max_bytes: 1,
            max_memtable_bytes: Some(1),
            ..config("memtable_backpressure")
        };
        let (memtable, ingest) = fixture(config.clone());
        let journal = ingest.journal.clone();

        ingest
            .accept(request(vec![record("first line")]))
            .await
            .expect("the first export is under the limit");
        let refused = ingest
            .accept(request(vec![record("second line")]))
            .await
            .expect_err("a full memtable must be refused");
        assert_eq!(refused.status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            refused.retry_after,
            Some(config.backpressure_retry_after),
            "backpressure must tell a collecty when to come back, or it has \
nothing to pace its queue against"
        );

        let checkpoint = journal.checkpoint().await.unwrap();
        memtable.commit_flush();
        journal.set_checkpoint(checkpoint.offset).unwrap();
        ingest
            .accept(request(vec![record("third line")]))
            .await
            .expect("a drained memtable accepts writes again");
    }

    /// The WAL-backlog half of the same gate.
    #[tokio::test]
    async fn an_export_is_refused_while_the_wal_backlog_is_over_its_limit_and_recovers() {
        let (memtable, ingest) = fixture(Config {
            max_memtable_bytes: None,
            max_wal_backlog_bytes: Some(1),
            ..config("wal_backpressure")
        });
        let journal = ingest.journal.clone();

        ingest
            .accept(request(vec![record("first line")]))
            .await
            .expect("an empty WAL accepts the first export");
        let refused = ingest
            .accept(request(vec![record("second line")]))
            .await
            .expect_err("an unretired WAL backlog must be refused");
        assert_eq!(refused.status, StatusCode::TOO_MANY_REQUESTS);

        let checkpoint = journal.checkpoint().await.unwrap();
        memtable.commit_flush();
        journal.set_checkpoint(checkpoint.offset).unwrap();
        ingest
            .accept(request(vec![record("third line")]))
            .await
            .expect("a retired backlog accepts writes again");
    }

    /// `time_unix_nano` is a u64 and the storage timestamp is an i64; the
    /// value in between must be a refusal, not a wraparound.
    #[tokio::test]
    async fn a_timestamp_past_i64_nanoseconds_is_refused() {
        let (memtable, ingest) = fixture(config("timestamp_overflow"));
        let mut too_far = record("must be rejected");
        too_far.time_unix_nano = u64::MAX;

        let error = ingest
            .accept(request(vec![too_far]))
            .await
            .expect_err("an out-of-range timestamp must fail");

        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert!(memtable.is_empty());
    }

    /// With the window disabled, a historical import is accepted however old.
    #[tokio::test]
    async fn a_disabled_timestamp_window_accepts_a_backfill() {
        let (memtable, ingest) = fixture(Config {
            max_timestamp_age: None,
            max_timestamp_skew: None,
            ..config("window_off")
        });
        let mut old = record("historical import");
        old.time_unix_nano = 1_000_000;

        ingest
            .accept(request(vec![old]))
            .await
            .expect("a disabled window must accept any in-range timestamp");

        assert!(!memtable.is_empty());
    }

    /// The journal keeps one encoding for a log record. If ingest wrote
    /// something replay could not decode, the WAL would be unreadable after a
    /// crash — and it is the WAL that makes an acknowledged write durable.
    #[tokio::test]
    async fn an_exported_record_survives_a_replay_of_the_journal() {
        let config = config("replay");
        let (_memtable, ingest) = fixture(config.clone());
        ingest
            .accept(request(vec![record("survives a crash")]))
            .await
            .unwrap();
        drop(ingest);

        let replayed = Arc::new(MemTable::new());
        journal::replay_with_traces(
            &config.data_dir.join("journal.wal"),
            &config.data_dir.join("journal.ckpt"),
            &replayed,
            &crate::trace::TraceMemTable::new())
        .expect("the WAL replays");
        let results = replayed.query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true);
        let lines: Vec<&str> = results
            .iter()
            .flat_map(|stream| stream.entries.iter())
            .map(|entry| entry.line.as_str())
            .collect();
        assert_eq!(lines, vec!["survives a crash"]);
        assert!(
            results[0]
                .entries[0]
                .structured_metadata
                .contains(&("service_name".to_string(), "checkout".to_string())),
            "replay reconstructs the attributes, not just the line"
        );
    }

    /// `Retry-After` carries the refusal's own delay, rounded to what the
    /// header can express.
    ///
    /// Rounded **up**, and never zero: `Retry-After: 0` reads as "retry
    /// immediately", the opposite of what an overloaded server is asking for,
    /// and truncating 1.7 s to 1 s sends the collecty back before the server's
    /// own arithmetic says it may.
    #[test]
    fn a_throttled_push_names_its_delay_in_whole_seconds() {
        use axum::http::header;
        use axum::response::IntoResponse;

        for (retry_after, expected) in [
            (std::time::Duration::from_secs(1), 1),
            (std::time::Duration::from_millis(300), 1),
            (std::time::Duration::from_millis(1700), 2),
        ] {
            let response = IngestError {
                status: StatusCode::TOO_MANY_REQUESTS,
                message: "flush is not keeping up".to_string(),
                retry_after: Some(retry_after),
            }
            .into_response();
            let seconds: u64 = response
                .headers()
                .get(header::RETRY_AFTER)
                .expect("a throttled push carries Retry-After")
                .to_str()
                .unwrap()
                .parse()
                .unwrap();

            assert_eq!(seconds, expected, "{retry_after:?}");
        }
    }
