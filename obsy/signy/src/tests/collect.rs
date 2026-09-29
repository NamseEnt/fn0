    use super::*;
    use crate::config::Config;
    use crate::journal::Journal;
    use crate::memtable::MemTable;
    use crate::part_registry::PartRegistry;
    use crate::tenant::test_tenant;
    use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
    use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use tower::ServiceExt;

    fn fixture() -> (Arc<MemTable>, Arc<AppState>) {
        fixture_with(|_| {})
    }

    fn fixture_with(edit: impl FnOnce(&mut Config)) -> (Arc<MemTable>, Arc<AppState>) {
        let mut config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-collect-{}", uuid::Uuid::new_v4())),
            ..Config::default()
        };
        edit(&mut config);
        let config = config;
        std::fs::create_dir_all(&config.data_dir).unwrap();
        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
        let parts = Arc::new(PartRegistry::new());
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            parts.operation_lock(),
        ));
        let state = crate::test_support::state(config, memtable.clone(), journal, parts, trace_parts, None);
        (memtable, state)
    }

    fn log_request(body: &str) -> ExportLogsServiceRequest {
        log_request_for(&test_tenant(), body)
    }

    /// An export naming `tenant` in its resource, which is the only thing that
    /// files it under one: nothing outside the payload says whose it is.
    fn log_request_for(
        tenant: &crate::tenant::TenantId,
        body: &str,
    ) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue {
                            key: crate::otlp_tenant::TENANT_ATTRIBUTE.to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue(
                                    tenant.as_str().to_string(),
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
                    log_records: vec![LogRecord {
                        time_unix_nano: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64,
                        body: Some(AnyValue {
                            value: Some(any_value::Value::StringValue(body.to_string())),
                        }),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }

    fn lines(memtable: &MemTable) -> Vec<String> {
        memtable
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
            .iter()
            .flat_map(|stream| stream.entries.iter())
            .map(|entry| entry.line.clone())
            .collect()
    }

    fn inflight(state: &Arc<AppState>) -> u64 {
        state.ingest_gate.inflight_body_bytes()
    }

    /// One byte is the tightest ceiling the knob accepts, and every legal
    /// record is larger than it. If the handler refused on arithmetic alone
    /// this server would answer 429 forever with nothing in flight, which is
    /// why `IngestGate::admit_body` always admits into an empty server — this
    /// is that rule reached through the router.
    #[tokio::test]
    async fn the_tightest_inflight_ceiling_still_serves_a_lone_record() {
        let (memtable, state) = fixture_with(|config| {
            config.max_inflight_push_bytes = Some(1);
        });

        let frames = zstd_frames(&logs(&["admitted into an empty server"]));
        let (status, _) = post_collected(&state, LOGS, "zstd", frames).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(lines(&memtable), vec!["admitted into an empty server"]);
        // And every charge was released rather than held by the answered batch.
        assert_eq!(inflight(&state), 0);
    }

    /// A record is charged while it is in the heap and gives the bytes back on
    /// the way out — down every path out, not just the one that stores it.
    ///
    /// The dropped record is what this is for: it leaves through
    /// `never_acceptable`, which is neither the success path nor a returned
    /// error, and a charge leaked there would shrink the ceiling for every
    /// batch after it until the process restarted.
    #[tokio::test]
    async fn a_dropped_record_releases_its_inflight_charge() {
        let (memtable, state) = fixture();
        let mut records = logs(&["stored"]);
        records.push(b"not a protobuf message at all".to_vec());
        records.extend(logs(&["also stored"]));

        let (status, _) = post_collected(&state, LOGS, "zstd", zstd_frames(&records)).await;

        assert_eq!(status, StatusCode::OK, "one bad record does not end a batch");
        assert_eq!(lines(&memtable), vec!["stored", "also stored"]);
        assert_eq!(
            state
                .metrics
                .collect_dropped_records
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert_eq!(inflight(&state), 0);
    }

    /// With per-tenant policy enabled, the pushed policies are the tenant
    /// registry — and a tenant nothing was pushed for is *dropped* rather than
    /// refused: the answer an ingest gives says whether the batch arrived, and
    /// nothing about whose it was. The counter is the only place the loss
    /// shows.
    #[tokio::test]
    async fn a_record_from_a_tenant_without_a_pushed_policy_is_dropped_silently() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-collect-allow-{}", uuid::Uuid::new_v4())),
            ..Config::default()
        };
        std::fs::create_dir_all(&config.data_dir).unwrap();
        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
        let parts = Arc::new(PartRegistry::new());
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            parts.operation_lock(),
        ));
        let tenant_policy = Arc::new(crate::tenant_policy::TenantPolicy::enabled_with_clock(
            crate::clock::Clock::system(),
        ));
        tenant_policy
            .push(&test_tenant(), 1, "30d", None)
            .await
            .expect("the test tenant is onboarded by pushing a policy");
        let state = crate::test_support::state_with_tenant_policy(
            config,
            memtable.clone(),
            journal,
            parts,
            trace_parts,
            None,
            tenant_policy,
        );

        let frames = zstd_frames(&logs(&["accepted"]));
        let (status, _) = post_collected(&state, LOGS, "zstd", frames).await;
        assert_eq!(status, StatusCode::OK, "a listed tenant is accepted");

        let stranger = crate::tenant::TenantId::parse("stranger").unwrap();
        let frames = zstd_frames(&[log_request_for(&stranger, "refused").encode_to_vec()]);
        let (status, _) = post_collected(&state, LOGS, "zstd", frames).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "the batch arrived and decoded, which is all the status answers"
        );
        assert!(
            memtable
                .query(
                    &stranger,
                    &[],
                    crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                    10,
                    true
                )
                .is_empty(),
            "an unserved tenant must not have been written"
        );
        assert_eq!(
            state
                .metrics
                .ingest_dropped_tenant_not_served
                .load(std::sync::atomic::Ordering::Relaxed),
            1,
            "the drop is counted, because nothing else reports it"
        );
    }

    /// A record nobody configured, and one configured with a value this engine
    /// cannot store. Both are dropped, and counted apart from each other so an
    /// operator knows which mistake was made.
    #[tokio::test]
    async fn a_record_naming_no_usable_tenant_is_dropped_under_its_own_reason() {
        let (memtable, state) = fixture();
        let mut anonymous = log_request("nobody said whose");
        anonymous.resource_logs[0]
            .resource
            .as_mut()
            .unwrap()
            .attributes
            .retain(|attribute| attribute.key != crate::otlp_tenant::TENANT_ATTRIBUTE);

        let mut malformed = log_request("a tenant id this is not");
        for attribute in &mut malformed.resource_logs[0].resource.as_mut().unwrap().attributes {
            if attribute.key == crate::otlp_tenant::TENANT_ATTRIBUTE {
                attribute.value = Some(AnyValue {
                    value: Some(any_value::Value::StringValue("not a tenant".to_string())),
                });
            }
        }

        let frames = zstd_frames(&[anonymous.encode_to_vec(), malformed.encode_to_vec()]);
        let (status, _) = post_collected(&state, LOGS, "zstd", frames).await;
        assert_eq!(status, StatusCode::OK);

        assert!(lines(&memtable).is_empty(), "neither record was stored");
        assert_eq!(
            state
                .metrics
                .ingest_dropped_no_tenant
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert_eq!(
            state
                .metrics
                .ingest_dropped_invalid_tenant
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }

    /// One record, two tenants, two journal records. The bytes that arrived no
    /// longer describe what is stored, so the passthrough gives way to a
    /// re-encode per group — and each tenant sees only its own line.
    #[tokio::test]
    async fn a_record_naming_two_tenants_is_split_between_them() {
        let (memtable, state) = fixture();
        let acme = crate::tenant::TenantId::parse("acme").unwrap();
        let beta = crate::tenant::TenantId::parse("beta").unwrap();
        let request = ExportLogsServiceRequest {
            resource_logs: vec![
                log_request_for(&acme, "acme line").resource_logs.remove(0),
                log_request_for(&beta, "beta line").resource_logs.remove(0),
            ],
        };

        let frames = zstd_frames(&[request.encode_to_vec()]);
        let (status, _) = post_collected(&state, LOGS, "zstd", frames).await;
        assert_eq!(status, StatusCode::OK);

        let read = |tenant: &crate::tenant::TenantId| -> Vec<String> {
            memtable
                .query(
                    tenant,
                    &[],
                    crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                    10,
                    true,
                )
                .iter()
                .flat_map(|stream| stream.entries.iter())
                .map(|entry| entry.line.clone())
                .collect()
        };
        assert_eq!(read(&acme), vec!["acme line".to_string()]);
        assert_eq!(read(&beta), vec!["beta line".to_string()]);
    }

    /// The routing key is not something the row is about. Left in, it would be
    /// a second copy of the isolation the tenant column already enforces, and
    /// one a query could select on.
    #[tokio::test]
    async fn the_tenant_attribute_is_not_stored_as_metadata() {
        let (memtable, state) = fixture();

        let frames = zstd_frames(&logs(&["filed under the test tenant"]));
        let (status, _) = post_collected(&state, LOGS, "zstd", frames).await;
        assert_eq!(status, StatusCode::OK);

        let streams = memtable.query(
            &test_tenant(),
            &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            10,
            true,
        );
        let keys: Vec<String> = streams
            .iter()
            .flat_map(|stream| stream.entries.iter())
            .flat_map(|entry| entry.structured_metadata.iter())
            .map(|(key, _)| key.clone())
            .collect();
        assert!(
            keys.iter().any(|key| key == "service_name"),
            "the other resource attributes are still stored: {keys:?}"
        );
        assert!(
            !keys.iter().any(|key| key.starts_with("tenant")),
            "the routing key is not stored: {keys:?}"
        );
    }

    fn span_request(mark: u8) -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![opentelemetry_proto::tonic::trace::v1::ResourceSpans {
                resource: Some(crate::otlp_tenant::test_tenant_resource()),
                scope_spans: vec![opentelemetry_proto::tonic::trace::v1::ScopeSpans {
                    scope: None,
                    spans: vec![opentelemetry_proto::tonic::trace::v1::Span {
                        trace_id: vec![mark; 16],
                        span_id: vec![mark; 8],
                        start_time_unix_nano: 10,
                        end_time_unix_nano: 20,
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }

    fn spans(state: &Arc<AppState>) -> usize {
        state
            .journal
            .trace_memtable()
            .snapshot_limited(&test_tenant(), 10)
            .unwrap()
            .len()
    }

    const LOGS: CollectSignal = CollectSignal::Logs;
    const TRACES: CollectSignal = CollectSignal::Traces;
    const METRICS: CollectSignal = CollectSignal::Metrics;

    /// One gauge datapoint per `instance`, all at the same timestamp, so a run
    /// of them is a run of distinct series.
    fn metric_request(instances: &[&str]) -> ExportMetricsServiceRequest {
        use opentelemetry_proto::tonic::metrics::v1::{
            Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric,
            number_data_point,
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(crate::otlp_tenant::test_tenant_resource()),
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        name: "queue_depth".to_string(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: instances
                                .iter()
                                .map(|instance| NumberDataPoint {
                                    attributes: vec![KeyValue {
                                        key: "instance".to_string(),
                                        value: Some(AnyValue {
                                            value: Some(any_value::Value::StringValue(
                                                (*instance).to_string(),
                                            )),
                                        }),
                                        ..Default::default()
                                    }],
                                    time_unix_nano: now,
                                    value: Some(number_data_point::Value::AsDouble(1.0)),
                                    ..Default::default()
                                })
                                .collect(),
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn series(state: &Arc<AppState>) -> usize {
        state
            .journal
            .series_memtable()
            .sorted_samples(&test_tenant())
            .map(|samples| samples.len())
            .unwrap_or(0)
    }

    #[tokio::test]
    async fn a_collected_batch_of_metrics_reaches_the_series_memtable() {
        let (_memtable, state) = fixture();
        let records = vec![
            metric_request(&["a"]).encode_to_vec(),
            metric_request(&["b"]).encode_to_vec(),
        ];

        let (status, body) = post_collected(&state, METRICS, "zstd", zstd_frames(&records)).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, r#"{"stored":0,"rejected":0}"#);
        assert_eq!(series(&state), 2);
    }

    /// Cardinality pressure is an export-level decision.  The first export is
    /// durable, while the following export is refused as a whole with 429;
    /// no partial-success filtering is performed.
    #[tokio::test]
    async fn a_metric_export_past_the_count_guard_is_refused_whole() {
        let (_memtable, state) = fixture_with(|config| {
            config.max_active_series = Some(1);
        });
        let records = vec![
            metric_request(&["a"]).encode_to_vec(),
            metric_request(&["a", "b", "c"]).encode_to_vec(),
        ];

        let (status, body) = post_collected(&state, METRICS, "zstd", zstd_frames(&records)).await;

        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert!(body.contains("SIGNY_MAX_ACTIVE_SERIES"), "{body}");
    }

    /// The emergency count guard is process-wide and is evaluated per export,
    /// independent of how a collecty packs its records.
    #[tokio::test]
    async fn the_series_cap_holds_across_a_batch_the_way_it_holds_across_two() {
        let records = vec![
            metric_request(&["a"]).encode_to_vec(),
            metric_request(&["a", "b", "c"]).encode_to_vec(),
        ];

        let (_memtable, one) = fixture_with(|config| config.max_active_series = Some(1));
        let (together_status, together) =
            post_collected(&one, METRICS, "zstd", zstd_frames(&records)).await;

        let (_memtable, apart) = fixture_with(|config| config.max_active_series = Some(1));
        let mut separately = Vec::new();
        for record in &records {
        let (status, body) =
                post_collected(&apart, METRICS, "zstd", zstd_frames(std::slice::from_ref(record)))
                    .await;
            separately.push((status, body));
        }

        assert_eq!(together_status, StatusCode::TOO_MANY_REQUESTS);
        assert!(together.contains("SIGNY_MAX_ACTIVE_SERIES"), "{together}");
        assert_eq!(separately[1].0, StatusCode::TOO_MANY_REQUESTS);
        assert!(separately[1].1.contains("SIGNY_MAX_ACTIVE_SERIES"));
        // The first record was handed to the journal before the second one
        // hit the guard.  Its pending append is owned by the writer and may
        // still be settling when the collect response is returned.
    }

    /// The batch collecty ships: each payload behind its own length, each
    /// record its own zstd frame, all of it concatenated. A real collecty
    /// sends one stream over the whole segment, which decompresses the same
    /// way — this shape is the harder one for the reader, so it is what the
    /// tests use. Nothing here names a signal: the request does, once.
    fn zstd_frames(records: &[Vec<u8>]) -> Vec<u8> {
        let mut frames = Vec::new();
        for payload in records {
            let mut plain = Vec::with_capacity(4 + payload.len());
            plain.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            plain.extend_from_slice(payload);
            frames.extend_from_slice(&zstd::bulk::compress(&plain, 3).unwrap());
        }
        frames
    }

    /// The shape a real collecty ships: one zstd stream over the whole
    /// segment, the records back to back inside it.
    fn zstd_stream(records: &[Vec<u8>]) -> Vec<u8> {
        let mut plain = Vec::new();
        for payload in records {
            plain.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            plain.extend_from_slice(payload);
        }
        zstd::bulk::compress(&plain, 3).unwrap()
    }

    /// A collecty names no tenant: it never decodes a payload, so it has
    /// nothing to name one with. Each record inside the batch says whose it is.
    async fn post_collected(
        state: &Arc<AppState>,
        signal: CollectSignal,
        encoding: &str,
        frames: Vec<u8>,
    ) -> (StatusCode, String) {
        post_request(
            axum::http::Request::builder()
                .method("POST")
                .uri("/signy/api/v1/collect")
                .header(header::CONTENT_TYPE, "application/x-protobuf")
                .header(header::CONTENT_ENCODING, encoding)
                .header(crate::collect::COLLECT_SIGNAL_HEADER, signal.as_str())
                .body(axum::body::Body::from(frames))
                .unwrap(),
            state,
        )
        .await
    }

    const SENDER: &str = "0123456789abcdef0123456789abcdef";

    /// The same post, named and numbered the way collecty makes one.
    async fn post_collected_from(
        state: &Arc<AppState>,
        sender: &str,
        signal: CollectSignal,
        segment: u64,
        frames: Vec<u8>,
    ) -> (StatusCode, String) {
        post_request(
            axum::http::Request::builder()
                .method("POST")
                .uri("/signy/api/v1/collect")
                .header(header::CONTENT_TYPE, "application/x-protobuf")
                .header(header::CONTENT_ENCODING, "zstd")
                .header(crate::collect::COLLECT_SENDER_HEADER, sender)
                .header(crate::collect::COLLECT_SIGNAL_HEADER, signal.as_str())
                .header(
                    crate::collect::COLLECT_SEGMENT_HEADER,
                    segment.to_string(),
                )
                .body(axum::body::Body::from(frames))
                .unwrap(),
            state,
        )
        .await
    }

    async fn post_request(
        request: axum::http::Request<axum::body::Body>,
        state: &Arc<AppState>,
    ) -> (StatusCode, String) {
        let response = crate::build_router(state.clone())
            .oneshot(request)
            .await
            .unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        (status, String::from_utf8_lossy(&bytes).to_string())
    }

    fn skipped(state: &Arc<AppState>) -> u64 {
        state
            .metrics
            .collect_skipped_records
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn logs(lines: &[&str]) -> Vec<Vec<u8>> {
        lines
            .iter()
            .map(|line| log_request(line).encode_to_vec())
            .collect()
    }

    /// A segment is one compression over every record it holds, so the reader
    /// cannot lean on a frame boundary to find where one ends.
    #[tokio::test]
    async fn a_segment_compressed_as_one_stream_is_read_a_record_at_a_time() {
        let (memtable, state) = fixture();
        let records = logs(&["first", "second", "third", "fourth"]);

        let (status, body) = post_collected_from(&state, SENDER, LOGS, 1, zstd_stream(&records)).await;

        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body, r#"{"stored":1,"rejected":0}"#);
        let mut stored = lines(&memtable);
        stored.sort();
        assert_eq!(stored, vec!["first", "fourth", "second", "third"]);
    }

    /// The whole point of the numbering: a collecty that died before it could
    /// write the answer down offers the same segment again, and this instance
    /// answers it without reading a byte of the body it already has.
    #[tokio::test]
    async fn a_segment_sent_again_is_answered_without_being_read() {
        let (memtable, state) = fixture();
        let records = logs(&["first", "second", "third"]);

        let (status, body) = post_collected_from(&state, SENDER, LOGS, 1, zstd_frames(&records)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body, r#"{"stored":1,"rejected":0}"#);

        let (status, body) = post_collected_from(&state, SENDER, LOGS, 1, zstd_frames(&records)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body, r#"{"stored":1,"rejected":0}"#, "the answer does not move");

        let mut lines = lines(&memtable);
        lines.sort();
        assert_eq!(lines, vec!["first", "second", "third"]);
        assert_eq!(
            skipped(&state),
            0,
            "nothing was counted off because nothing was read"
        );
    }

    /// The segment after it goes on top rather than replacing anything.
    #[tokio::test]
    async fn the_next_segment_carries_on_from_the_one_before() {
        let (memtable, state) = fixture();
        post_collected_from(&state, SENDER, LOGS, 1, zstd_frames(&logs(&["first"]))).await;

        let (status, body) =
            post_collected_from(&state, SENDER, LOGS, 2, zstd_frames(&logs(&["second"]))).await;

        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body, r#"{"stored":2,"rejected":0}"#);
        let mut lines = lines(&memtable);
        lines.sort();
        assert_eq!(lines, vec!["first", "second"]);
    }

    /// A segment whose delivery was cut halfway. What reached the WAL is
    /// durable and its position with it, so the resend counts those records
    /// off and stores only the rest.
    #[tokio::test]
    async fn a_segment_cut_halfway_is_taken_up_where_it_left_off() {
        let (memtable, state) = fixture();
        // What the writer would have left behind: two of segment one's
        // records stored, and nothing to say the segment finished.
        state
            .journal
            .collect_marks()
            .advance(crate::journal::CollectMark {
                sender: crate::journal::SenderId::parse(SENDER).unwrap(),
                signal: LOGS,
                at: crate::journal::Position {
                    segment: 1,
                    records: 2,
                },
            });

        let records = logs(&["first", "second", "third", "fourth"]);
        let (status, body) = post_collected_from(&state, SENDER, LOGS, 1, zstd_frames(&records)).await;

        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body, r#"{"stored":1,"rejected":0}"#);
        let mut lines = lines(&memtable);
        lines.sort();
        assert_eq!(lines, vec!["fourth", "third"], "only what was still owed");
        assert_eq!(skipped(&state), 2);
    }

    /// Two collectys number their own segments, so one's numbers say nothing
    /// about the other's.
    #[tokio::test]
    async fn one_sender_s_segments_do_not_skip_another_s() {
        let (memtable, state) = fixture();
        post_collected_from(&state, SENDER, LOGS, 1, zstd_frames(&logs(&["mine"]))).await;

        let other = "ffffffffffffffffffffffffffffffff";
        let (status, body) =
            post_collected_from(&state, other, LOGS, 1, zstd_frames(&logs(&["theirs"]))).await;

        assert_eq!(status, StatusCode::OK, "{body}");
        let mut lines = lines(&memtable);
        lines.sort();
        assert_eq!(lines, vec!["mine", "theirs"]);
        assert_eq!(skipped(&state), 0);
    }

    /// A record signy will never take leaves nothing in the WAL. The mark
    /// written when the body ends is what says the segment finished anyway,
    /// and without it the collecty would offer it forever.
    #[tokio::test]
    async fn a_segment_whose_last_record_is_dropped_still_finishes() {
        let (_memtable, state) = fixture();
        let records = vec![log_request("first").encode_to_vec(), vec![0xFFu8; 16]];

        let (status, body) = post_collected_from(&state, SENDER, LOGS, 1, zstd_frames(&records)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body, r#"{"stored":1,"rejected":0}"#);

        let (status, body) = post_collected_from(&state, SENDER, LOGS, 1, zstd_frames(&records)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(
            state
                .metrics
                .collect_dropped_records
                .load(std::sync::atomic::Ordering::Relaxed),
            1,
            "the resend is answered unread, so the drop is counted once"
        );
    }

    #[tokio::test]
    async fn a_sender_header_that_is_not_an_id_is_refused() {
        let (_memtable, state) = fixture();

        let (status, body) =
            post_collected_from(&state, "not-an-id", LOGS, 1, zstd_frames(&logs(&["never stored"]))).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    }

    #[tokio::test]
    async fn a_segment_number_of_zero_is_refused() {
        let (_memtable, state) = fixture();

        let (status, body) =
            post_collected_from(&state, SENDER, LOGS, 0, zstd_frames(&logs(&["never stored"]))).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    }

    #[tokio::test]
    async fn a_collected_batch_of_separate_exports_lands_as_every_line_it_carried() {
        let (memtable, state) = fixture();
        let records = logs(&["first", "second", "third"]);
        let frames = zstd_frames(&records);

        let (status, body) = post_collected(&state, LOGS, "zstd", frames).await;

        assert_eq!(status, StatusCode::OK, "{body}");
        let mut lines = lines(&memtable);
        lines.sort();
        assert_eq!(lines, vec!["first", "second", "third"]);
    }

    #[tokio::test]
    async fn a_collected_batch_of_spans_reaches_the_trace_memtable() {
        let (_memtable, state) = fixture();
        let records = vec![
            span_request(1).encode_to_vec(),
            span_request(2).encode_to_vec(),
        ];
        let frames = zstd_frames(&records);

        let (status, body) = post_collected(&state, TRACES, "zstd", frames).await;

        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(
            state
                .journal
                .trace_memtable()
                .snapshot_limited(&test_tenant(), 10)
                .unwrap()
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn a_collected_batch_that_is_not_zstd_is_refused_without_writing() {
        let (memtable, state) = fixture();
        let body = log_request("never stored").encode_to_vec();

        let (status, _) = post_collected(&state, LOGS, "gzip", body).await;

        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert!(lines(&memtable).is_empty());
    }

    /// The batch itself has no ceiling any more, so the one left is a record's
    /// own payload, and it is refused on what the header claims rather than
    /// after the bytes have been waited for.
    #[tokio::test]
    async fn a_record_claiming_more_than_one_export_is_refused_before_it_arrives() {
        let (memtable, state) = fixture();
        let mut plain = Vec::new();
        plain.extend_from_slice(
            &((crate::collect::MAX_COLLECT_RECORD_BYTES + 1) as u32).to_le_bytes(),
        );
        plain.extend_from_slice(b"and nothing like that much behind it");
        let frames = zstd::bulk::compress(&plain, 3).unwrap();

        let (status, body) = post_collected(&state, LOGS, "zstd", frames).await;

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
        assert!(lines(&memtable).is_empty());
    }

    /// A batch larger than anything the old ceiling would have taken, landing
    /// whole. Nothing here holds the batch, so its size is collecty's business.
    #[tokio::test]
    async fn a_batch_past_the_old_ceiling_lands_whole() {
        let (memtable, state) = fixture();
        let filler = "x".repeat(64 * 1024);
        let records: Vec<Vec<u8>> = (0..400)
            .map(|index| log_request(&format!("{index} {filler}")).encode_to_vec())
            .collect();
        let plain_bytes: usize = records.iter().map(|payload| 4 + payload.len()).sum();
        assert!(plain_bytes > crate::trace_ingest::MAX_OTLP_REQUEST_BYTES);

        let (status, body) = post_collected(&state, LOGS, "zstd", zstd_frames(&records)).await;

        assert_eq!(status, StatusCode::OK, "{body}");
        let stored: usize = memtable
            .query(
                &test_tenant(),
                &[],
                crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                1000,
                true,
            )
            .iter()
            .map(|stream| stream.entries.len())
            .sum();
        assert_eq!(stored, 400);
    }

    /// A batch is one signal's now, so a collecty that has both to send sends
    /// them as two, and each lands in the store its request names.
    #[tokio::test]
    async fn a_batch_per_signal_lands_in_the_store_its_request_names() {
        let (memtable, state) = fixture();

        let (status, body) =
            post_collected(&state, LOGS, "zstd", zstd_frames(&logs(&["first", "second"]))).await;
        assert_eq!(status, StatusCode::OK, "{body}");

        let (status, body) = post_collected(
            &state,
            TRACES,
            "zstd",
            zstd_frames(&[span_request(7).encode_to_vec()]),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");

        let mut lines = lines(&memtable);
        lines.sort();
        assert_eq!(lines, vec!["first", "second"]);
        assert_eq!(spans(&state), 1);
    }

    #[tokio::test]
    async fn a_record_that_will_never_decode_is_dropped_and_the_rest_of_the_batch_lands() {
        let (memtable, state) = fixture();
        let poison = vec![0xFFu8; 16];
        let records = vec![poison.clone(), log_request("after it").encode_to_vec()];
        let frames = zstd_frames(&records);

        let (status, body) = post_collected(&state, LOGS, "zstd", frames).await;

        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(lines(&memtable), vec!["after it"]);
        assert_eq!(
            state
                .metrics
                .collect_dropped_records
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert_eq!(
            state
                .metrics
                .collect_dropped_bytes
                .load(std::sync::atomic::Ordering::Relaxed),
            poison.len() as u64
        );
    }

    #[tokio::test]
    async fn a_batch_whose_framing_does_not_add_up_is_refused() {
        let (memtable, state) = fixture();
        let mut plain = Vec::new();
        plain.extend_from_slice(&64u32.to_le_bytes());
        plain.extend_from_slice(b"eight...");
        let frames = zstd::bulk::compress(&plain, 3).unwrap();

        let (status, body) = post_collected(&state, LOGS, "zstd", frames).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert!(body.contains("claims 64 bytes"), "{body}");
        assert!(lines(&memtable).is_empty());
    }

    /// A record no longer names its own signal, so a request that does not
    /// name one says nothing about what its body holds.
    #[tokio::test]
    async fn a_batch_that_does_not_name_its_signal_is_refused() {
        let (_memtable, state) = fixture();
        let frames = zstd_frames(&logs(&["never stored"]));

        for headers in [Vec::new(), vec![("x-collecty-signal", "profiles")]] {
            let mut request = axum::http::Request::builder()
                .method("POST")
                .uri("/signy/api/v1/collect")
                .header(header::CONTENT_TYPE, "application/x-protobuf")
                .header(header::CONTENT_ENCODING, "zstd")
                .header(crate::tenant::TENANT_HEADER, test_tenant().as_str());
            for (name, value) in headers {
                request = request.header(name, value);
            }
            let (status, body) = post_request(
                request.body(axum::body::Body::from(frames.clone())).unwrap(),
                &state,
            )
            .await;

            assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
            assert!(body.contains("must name a signal"), "{body}");
        }
    }

    /// A tenant nothing was pushed for is a policy mistake, and the record
    /// carrying it is dropped rather than held: the collector has one queue per
    /// signal, so holding it would stop every other application on the host
    /// behind one misconfigured process. The rest of the batch still lands.
    #[tokio::test]
    async fn a_tenant_this_instance_does_not_serve_is_dropped_rather_than_held() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-otlp-http-collect-{}", uuid::Uuid::new_v4())),
            ..Config::default()
        };
        std::fs::create_dir_all(&config.data_dir).unwrap();
        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
        let parts = Arc::new(PartRegistry::new());
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            parts.operation_lock(),
        ));
        let tenant_policy = Arc::new(crate::tenant_policy::TenantPolicy::enabled_with_clock(
            crate::clock::Clock::system(),
        ));
        tenant_policy
            .push(&test_tenant(), 1, "30d", None)
            .await
            .expect("the test tenant is onboarded by pushing a policy");
        let state = crate::test_support::state_with_tenant_policy(
            config,
            memtable.clone(),
            journal,
            parts,
            trace_parts,
            None,
            tenant_policy,
        );

        let stranger = crate::tenant::TenantId::parse("stranger").unwrap();
        let records = vec![
            log_request_for(&stranger, "dropped").encode_to_vec(),
            log_request("kept").encode_to_vec(),
        ];
        let frames = zstd_frames(&records);

        let (status, body) = post_collected(&state, LOGS, "zstd", frames).await;

        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(
            lines(&memtable),
            vec!["kept".to_string()],
            "one application's policy mistake does not take the batch with it"
        );
        assert!(
            memtable
                .query(
                    &stranger,
                    &[],
                    crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                    10,
                    true
                )
                .is_empty()
        );
        assert_eq!(
            state
                .metrics
                .ingest_dropped_tenant_not_served
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }
