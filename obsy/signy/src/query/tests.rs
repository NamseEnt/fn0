    use super::*;
    use crate::tenant::test_tenant;
    use crate::config::Config;
    use crate::journal::Journal;
    use crate::memtable::{LogEntry, MemTable};
    use crate::part::{self, Row};
    use crate::part_registry::PartRegistry;
    use crate::series::{METRIC_NAME_LABEL, MetricSample, MetricValue, SampleKind, SeriesLabels};
    use tower::ServiceExt;

    /// Engine-level tests build their queries through the shared flat grammar
    /// — the one parser the read path has.
    fn flat_query(raw: &str) -> logql::LogQuery {
        parse_filter_params(raw, 0, LOGS_PARAMS)
            .expect("test query parses")
            .query
    }

    /// The last day of `attr=app=api`, through the first-party endpoint.
    async fn lines_in_last_day(state: Arc<AppState>) -> Vec<String> {
        let (status, _, body) =
            first_party_logs(state, "start=-24h&attr=app=api&direction=forward").await;
        assert_eq!(status, StatusCode::OK, "{body}");
        ndjson_rows(&body)
            .iter()
            .map(|row| row["line"].as_str().unwrap().to_string())
            .collect()
    }

    async fn first_party_logs_status(state: Arc<AppState>) -> (StatusCode, String) {
        let (status, _, body) = first_party_logs(state, "start=-24h&attr=app=api").await;
        (status, body)
    }

    /// The stream era attached these pairs as labels; storage now keeps them
    /// as each row's own attributes, which is what selectors match against.
    fn rows_with_attributes(attributes: &Labels, mut rows: Vec<Row>) -> Vec<Row> {
        for row in &mut rows {
            row.structured_metadata
                .extend(attributes.iter().map(|(k, v)| (k.clone(), v.clone())));
            crate::memtable::canonicalize_structured_metadata(&mut row.structured_metadata);
        }
        rows
    }

    /// The same fold for buffered entries.
    fn with_attributes(attributes: Labels, mut entries: Vec<LogEntry>) -> Vec<LogEntry> {
        for entry in &mut entries {
            entry
                .structured_metadata
                .extend(attributes.iter().map(|(k, v)| (k.clone(), v.clone())));
            crate::memtable::canonicalize_structured_metadata(&mut entry.structured_metadata);
        }
        entries
    }

    fn temp_dir() -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "signy-query-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn test_state(
        data_dir: &std::path::Path,
        memtable: Arc<MemTable>,
        parts: Arc<PartRegistry>,
        remote_cache: Option<Arc<crate::object_storage::RemoteCache>>,
    ) -> Arc<AppState> {
        let config = Config {
            data_dir: data_dir.to_path_buf(),
            ..Config::default()
        };
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            parts.operation_lock(),
        ));
        crate::test_support::state(
            config.clone(),
            memtable.clone(),
            Arc::new(Journal::spawn(&config, memtable).unwrap()),
            parts,
            trace_parts,
            remote_cache,
        )
    }

    /// The window boundary at every level at once: a row on `start` and a row
    /// on `end` in the memtable, the same pair in a part, and one `query_range`
    /// over exactly that window.
    ///
    /// This is the test that was missing. The memtable scan, the part-level
    /// prune, the row-group prune and the row-level reject each spelled the
    /// comparison out for itself, so `end` could be — and was — inclusive on the
    /// log path while Loki's `query_range` is `[start, end)`; nothing caught it
    /// because no test put a row exactly on a boundary. Asserting the two
    /// sources together is what makes a single site drifting back a failure.
    #[tokio::test]
    async fn query_range_includes_start_and_excludes_end_in_the_memtable_and_in_parts() {
        const START_NS: i64 = 1_700_000_000_000_000_000;
        const END_NS: i64 = 1_700_000_001_000_000_000;

        let data_dir = temp_dir();
        let labels: Labels = [("app".to_string(), "api".to_string())]
            .into_iter()
            .collect();
        let memtable = Arc::new(MemTable::new());
        memtable.insert(test_tenant(), with_attributes(labels.clone(), vec![
                LogEntry {
                    timestamp_ns: START_NS,
                    line: "memtable on start".to_string(),
                    structured_metadata: Vec::new(),
                },
                LogEntry {
                    timestamp_ns: END_NS,
                    line: "memtable on end".to_string(),
                    structured_metadata: Vec::new(),
                },
            ]));
        let parts = Arc::new(PartRegistry::new());
        parts
            .register(
                part::flush_rows(
                    [(START_NS, "part on start"), (END_NS, "part on end")]
                        .into_iter()
                        .map(|(timestamp_ns, line)| Row {
                            tenant: test_tenant(),
                            timestamp_ns,
                            line: line.to_string(),
                            structured_metadata: vec![(
                                "app".to_string(),
                                "api".to_string(),
                            )],
                        })
                        .collect(),
                    &data_dir.join("parts"),
                    // One row per row group, so the row-group prune has to make
                    // the same call the row-level test does.
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        let state = test_state(&data_dir, memtable, parts, None);

        let (status, _, body) = first_party_logs(
            state.clone(),
            &format!("start={START_NS}&end={END_NS}&attr=app=api&direction=forward"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let mut lines: Vec<String> = ndjson_rows(&body)
            .iter()
            .map(|row| row["line"].as_str().unwrap().to_string())
            .collect();
        lines.sort();
        assert_eq!(lines, vec!["memtable on start", "part on start"]);

        // The same window one nanosecond wider does return the rows on `end`,
        // so what is being asserted above is the boundary and not a lost row.
        let (status, _, body) = first_party_logs(
            state.clone(),
            &format!(
                "start={START_NS}&end={}&attr=app=api&direction=forward",
                END_NS + 1
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(ndjson_rows(&body).len(), 4);

        // The unified scan decides nothing itself: it answers whatever range it
        // is handed, which is what lets one caller own the contract.
        let parsed = flat_query("attr=app=api");
        let rows = |range| {
            unified_query(&state, &test_tenant(), &parsed, range, 100, true)
                .unwrap()
                .iter()
                .map(|stream| stream.entries.len())
                .sum::<usize>()
        };
        assert_eq!(rows(part::QueryTimeRange::half_open(START_NS, END_NS)), 2);
        assert_eq!(rows(part::QueryTimeRange::closed(START_NS, END_NS)), 4);
        assert_eq!(rows(part::QueryTimeRange::half_open(END_NS, END_NS)), 0);
    }

    /// The output limit bounds the physical scan.
    ///
    /// This test used to assert the opposite, under the name
    /// `query_scan_stats_count_rows_before_applying_output_limit`: three rows
    /// were read to answer a `limit=1`, because the limit could only truncate
    /// the result after the scan had produced it. That is
    /// `docs/VISION.md` III — "the worst violation is the most common query" —
    /// and what the bounded sink removes. The scan budget is still a budget on
    /// rows *read*, so it is asserted with a limit loose enough not to bound
    /// them.
    #[tokio::test]
    async fn the_output_limit_bounds_the_physical_scan() {
        let data_dir = temp_dir();
        let labels: Labels = [("app".to_string(), "api".to_string())]
            .into_iter()
            .collect();
        let memtable = Arc::new(MemTable::new());
        memtable.insert(test_tenant(), with_attributes(labels.clone(), (0..3)
                .map(|timestamp_ns| LogEntry {
                    timestamp_ns,
                    line: format!("line-{timestamp_ns}"),
                    structured_metadata: Vec::new(),
                })
                .collect()));
        let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);
        let parsed = flat_query("");
        for (forward, wanted_line) in [(true, "line-0"), (false, "line-2")] {
            let execution = run_unified_query_with_stats(
                state.clone(),
                test_tenant(),
                parsed.clone(),
                crate::part::QueryTimeRange::closed(0, 2),
                1,
                forward,
                None,
                crate::metrics::QueryEndpoint::Query,
            )
            .await
            .unwrap();
            assert_eq!(execution.results[0].entries.len(), 1);
            assert_eq!(execution.results[0].entries[0].line, wanted_line);
            assert_eq!(
                execution.scanned_rows, 1,
                "one row of answer must cost one row of scan"
            );
        }

        let error = match unified_query_with_stats_cancellable(
            &state,
            &test_tenant(),
            &parsed,
            crate::part::QueryTimeRange::closed(0, 2),
            10,
            true,
            Some(2),
            None,
        ) {
            Ok(_) => panic!("scan budget should reject the third physical row"),
            Err(error) => error,
        };
        assert!(error.contains("2 scanned rows"));
    }

    /// The bed the early-termination tests share: rows in both the memtable and
    /// several parts, out of order, with timestamps spread far wider than any
    /// limit those tests ask for.
    fn early_termination_state(data_dir: &std::path::Path) -> Arc<AppState> {
        let attributes = vec![("app".to_string(), "api".to_string())];
        let line = |timestamp_ns: i64| {
            format!(r#"{{"status":"{}","seq":{timestamp_ns}}}"#, 500 - (timestamp_ns % 3))
        };
        let memtable = Arc::new(MemTable::new());
        // Newest, and out of order.
        memtable.insert(
            test_tenant(),
            [93i64, 90, 92, 91]
                .into_iter()
                .map(|timestamp_ns| LogEntry {
                    timestamp_ns,
                    line: line(timestamp_ns),
                    structured_metadata: attributes.clone(),
                })
                .collect(),
        );
        let parts = Arc::new(PartRegistry::new());
        // Three parts whose windows do not overlap, so the frontier can reject a
        // whole part from its metadata, in row groups of three so it can also
        // reject a row group inside the part it does open. Adjacent rather than
        // separated, so "the answer is contiguous" is a statement the boundary
        // test can make.
        for base in [0i64, 30, 60] {
            parts
                .register(
                    part::flush_rows(
                        (0..30)
                            .rev()
                            .map(|offset| Row {
                                tenant: test_tenant(),
                                timestamp_ns: base + offset,
                                line: line(base + offset),
                                structured_metadata: attributes.clone(),
                            })
                            .collect(),
                        &data_dir.join("parts"),
                        3,
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        test_state(data_dir, memtable, parts, None)
    }

    fn flatten(results: &[StreamResult]) -> Vec<(i64, String)> {
        let mut rows: Vec<(i64, String)> = results
            .iter()
            .flat_map(|stream| {
                stream
                    .entries
                    .iter()
                    .map(|entry| (entry.timestamp_ns, entry.line.clone()))
            })
            .collect();
        rows.sort_by_key(|(timestamp_ns, _)| *timestamp_ns);
        rows
    }

    /// A limit changes how many rows a query answers with, never which ones.
    ///
    /// Stated as an identity rather than as a fixture, because that is what
    /// makes early termination a performance change: `limit = n` has to return
    /// the first `n` rows of the answer the same query gives when nothing is
    /// truncated. Checked in both directions and through a `| json | field=`
    /// pipeline, which is the shape whose limit used to be `usize::MAX`.
    #[tokio::test]
    async fn a_limited_query_returns_a_prefix_of_the_unlimited_answer() {
        let data_dir = temp_dir();
        let state = early_termination_state(&data_dir);
        let range = crate::part::QueryTimeRange::closed(0, 1_000);
        for query in ["attr=app=api", "parse=json&attr=app=api&attr=status=500"] {
            let parsed = flat_query(query);
            for forward in [true, false] {
                let unlimited = flatten(
                    &unified_query(&state, &test_tenant(), &parsed, range, 1_000, forward).unwrap(),
                );
                assert!(
                    unlimited.len() > 20,
                    "{query} must match more than any limit below asks for"
                );
                for limit in [1usize, 2, 7, 20] {
                    let limited =
                        unified_query(&state, &test_tenant(), &parsed, range, limit, forward)
                            .unwrap();
                    let rows = flatten(&limited);
                    assert_eq!(
                        rows.len(),
                        limit,
                        "{query} at limit {limit} ({}) must return exactly the limit",
                        if forward { "forward" } else { "backward" }
                    );
                    // Forwards the answer starts at the oldest row, backwards at
                    // the newest, so the prefix is taken from the matching end.
                    let wanted: Vec<(i64, String)> = if forward {
                        unlimited[..limit].to_vec()
                    } else {
                        unlimited[unlimited.len() - limit..].to_vec()
                    };
                    assert_eq!(
                        rows, wanted,
                        "{query} at limit {limit} returned different rows, not fewer"
                    );
                }
            }
        }
    }

    /// The limit is honoured exactly on the boundary where the parts stop and the
    /// memtable starts, and on the boundary between two parts.
    ///
    /// Those are the two places a bounded scan decides to stop reading, so an
    /// off-by-one there returns 99 rows or 101 rather than an obviously wrong
    /// answer.
    #[tokio::test]
    async fn a_limit_is_honoured_exactly_at_a_source_boundary() {
        let data_dir = temp_dir();
        let state = early_termination_state(&data_dir);
        let range = crate::part::QueryTimeRange::closed(0, 1_000);
        let parsed = flat_query("attr=app=api");
        // 4 memtable rows and 30 rows in the newest part: 4 is the memtable
        // exactly, 5 is one row past it, 34 is the newest part exactly, and 35
        // is one row into the part before it.
        for limit in [3usize, 4, 5, 33, 34, 35] {
            let rows = flatten(
                &unified_query(&state, &test_tenant(), &parsed, range, limit, false).unwrap(),
            );
            assert_eq!(rows.len(), limit, "backward limit {limit}");
            let newest = rows.last().unwrap().0;
            assert_eq!(newest, 93, "backward always answers from the newest row");
            let oldest = rows.first().unwrap().0;
            assert_eq!(
                oldest,
                94 - limit as i64,
                "the rows are contiguous from the newest downwards at limit {limit}"
            );
        }
    }

    /// Early termination must not reach a row the scan budget already refused.
    /// The budget counts rows read, so a limit that stops the scan early makes a
    /// query that used to be refused succeed — and one whose pipeline discards
    /// most rows still has to be refused.
    #[tokio::test]
    async fn the_scan_budget_still_refuses_a_query_the_limit_cannot_bound() {
        let data_dir = temp_dir();
        let state = early_termination_state(&data_dir);
        let range = crate::part::QueryTimeRange::closed(0, 1_000);
        // Matches nothing, so no limit can ever be reached and the scan runs
        // until the budget stops it. A regex rather than an equality, because an
        // equality on an indexed field is answered by the exact-field bloom and
        // never reaches a row at all — which is the pruning working, not the
        // budget.
        let parsed = flat_query("parse=json&attr=app=api&attr=status=~418");
        let error = match unified_query_with_stats(
            &state,
            &test_tenant(),
            &parsed,
            range,
            10,
            false,
            Some(20),
        ) {
            Ok(execution) => panic!(
                "a query that survives nothing must exhaust the budget, read {} rows",
                execution.scanned_rows
            ),
            Err(error) => error,
        };
        assert!(error.contains("20 scanned rows"), "{error}");
    }

    #[test]
    fn parse_time_ns_unix_seconds() {
        assert_eq!(parse_time_ns("0").unwrap(), 0);
        assert_eq!(
            parse_time_ns("1700000000").unwrap(),
            1_700_000_000_000_000_000
        );
    }

    #[test]
    fn parse_time_ns_negative_unix_seconds() {
        assert_eq!(
            parse_time_ns("-1700000000").unwrap(),
            -1_700_000_000_000_000_000
        );
    }

    #[test]
    fn parse_time_ns_unix_nanos() {
        assert_eq!(
            parse_time_ns("1700000000000000000").unwrap(),
            1_700_000_000_000_000_000
        );
    }

    #[test]
    fn parse_time_ns_unix_millis_and_micros() {
        assert_eq!(
            parse_time_ns("1700000000000").unwrap(),
            1_700_000_000_000_000_000
        );
        assert_eq!(
            parse_time_ns("1700000000000000").unwrap(),
            1_700_000_000_000_000_000
        );
    }

    #[test]
    fn parse_time_ns_rfc3339() {
        let ns = parse_time_ns("2023-11-14T22:13:20Z").unwrap();
        assert_eq!(ns, 1_700_000_000_000_000_000);
    }

    #[test]
    fn parse_time_ns_rfc3339_with_nanos() {
        let ns = parse_time_ns("2023-11-14T22:13:20.123456789Z").unwrap();
        assert_eq!(ns, 1_700_000_000_123_456_789);
    }

    #[test]
    fn parse_time_ns_invalid() {
        assert!(parse_time_ns("not-a-time").is_err());
        assert!(parse_time_ns("").is_err());
    }

    #[test]
    fn parse_time_ns_accepts_exact_decimal_unix_seconds() {
        assert_eq!(
            parse_time_ns("1700000000.000000001").unwrap(),
            1_700_000_000_000_000_001
        );
        assert_eq!(parse_time_ns("1.5e0").unwrap(), 1_500_000_000);
        assert!(parse_time_ns("1.0000000001").is_err());
    }

    #[test]
    fn query_input_limits_and_direction_are_validated() {
        assert_eq!(parse_limit(None, MAX_LOG_LIMIT).unwrap(), 100);
        assert_eq!(
            parse_limit(Some(MAX_LOG_LIMIT), MAX_LOG_LIMIT).unwrap(),
            MAX_LOG_LIMIT
        );
        assert!(parse_limit(Some(MAX_LOG_LIMIT + 1), MAX_LOG_LIMIT).is_err());
        assert!(parse_direction(&Some("sideways".to_string())).is_err());
        assert!(parse_direction(&Some("FORWARD".to_string())).unwrap());
        assert!(!parse_direction(&Some("backward".to_string())).unwrap());
    }

    #[tokio::test]
    async fn readiness_reflects_background_worker_health() {
        let data_dir = temp_dir();
        let config = Config {
            data_dir,
            ..Config::default()
        };
        let memtable = Arc::new(MemTable::new());
        let state = crate::test_support::state(
            config.clone(),
            memtable.clone(),
            Arc::new(Journal::spawn(&config, memtable).unwrap()),
            Arc::new(PartRegistry::new()),
            Arc::new(crate::trace_registry::TraceRegistry::standalone()),
            None,
        );

        assert_eq!(ready(State(state.clone())).await.unwrap(), "ready");

        state.flush_healthy.store(false, Ordering::Release);
        state.merge_healthy.store(false, Ordering::Release);
        let error = ready(State(state)).await.unwrap_err();
        assert_eq!(error.0, StatusCode::SERVICE_UNAVAILABLE);
        assert!(error.1.contains("flush worker"));
        assert!(error.1.contains("merge worker"));
    }

    /// Readiness follows a sustained object-store outage, not a single failed
    /// request.
    ///
    /// One failure used to mark the store unhealthy, and `/ready` reads that.
    /// Measured under a 3% injected write-error rate — which the engine
    /// survives with no ingest errors and no lost data — the flag flipped
    /// 14-17 times a minute and read false 34-59% of the time. An orchestrator
    /// watching it pulls the instance out of service over an error rate that
    /// cost nothing.
    #[tokio::test]
    async fn readiness_ignores_an_isolated_object_store_failure() {
        let data_dir = temp_dir();
        let config = Config {
            data_dir: data_dir.clone(),
            ..Config::default()
        };
        let memtable = Arc::new(MemTable::new());
        let remote = Arc::new(crate::object_storage::RemoteCache::new(
            Arc::new(crate::object_storage::ObjectStorage::in_memory()),
            data_dir.join("parts"),
        ));
        remote.mark_cache_healthy();
        let state = crate::test_support::state(
            config.clone(),
            memtable.clone(),
            Arc::new(Journal::spawn(&config, memtable).unwrap()),
            Arc::new(PartRegistry::new()),
            Arc::new(crate::trace_registry::TraceRegistry::standalone()),
            Some(remote.clone()),
        );

        remote.record_remote_failure();
        assert!(
            remote.is_remote_healthy(),
            "one failed request is not an outage"
        );
        assert!(ready(State(state.clone())).await.is_ok());

        // A success between failures is what distinguishes a flaky request
        // from a store that is gone, so it resets the count.
        remote.record_remote_failure();
        remote.record_remote_success();
        remote.record_remote_failure();
        remote.record_remote_failure();
        assert!(remote.is_remote_healthy(), "{}", remote.consecutive_remote_failures());
        assert!(ready(State(state.clone())).await.is_ok());

        // Failing without a success in between does mean the store is gone.
        remote.record_remote_failure();
        assert!(!remote.is_remote_healthy());
        assert_eq!(
            ready(State(state.clone())).await.unwrap_err().0,
            StatusCode::SERVICE_UNAVAILABLE
        );

        // And one success is enough to come back, because the evidence that
        // the store works is the store working.
        remote.record_remote_success();
        assert!(ready(State(state)).await.is_ok());
    }

    #[tokio::test]
    async fn readiness_reflects_remote_storage_health() {
        let data_dir = temp_dir();
        let config = Config {
            data_dir: data_dir.clone(),
            ..Config::default()
        };
        let memtable = Arc::new(MemTable::new());
        let remote = Arc::new(crate::object_storage::RemoteCache::new(
            Arc::new(crate::object_storage::ObjectStorage::in_memory()),
            data_dir.join("parts"),
        ));
        remote.mark_unhealthy();
        remote.mark_cache_healthy();
        let state = crate::test_support::state(
            config.clone(),
            memtable.clone(),
            Arc::new(Journal::spawn(&config, memtable).unwrap()),
            Arc::new(PartRegistry::new()),
            Arc::new(crate::trace_registry::TraceRegistry::standalone()),
            Some(remote.clone()),
        );

        let error = ready(State(state)).await.unwrap_err();
        assert_eq!(error.0, StatusCode::SERVICE_UNAVAILABLE);
        assert!(error.1.contains("object store"));

        remote.record_remote_success();
        remote.mark_cache_unhealthy();
        let memtable = Arc::new(MemTable::new());
        let state = crate::test_support::state(
            config.clone(),
            memtable.clone(),
            Arc::new(Journal::spawn(&config, memtable).unwrap()),
            Arc::new(PartRegistry::new()),
            Arc::new(crate::trace_registry::TraceRegistry::standalone()),
            Some(remote),
        );
        let error = ready(State(state)).await.unwrap_err();
        assert!(error.1.contains("local cache"));
    }

    #[tokio::test]
    async fn query_restores_evicted_part_from_object_store() {
        let data_dir = temp_dir();
        let config = Config {
            data_dir: data_dir.clone(),
            ..Config::default()
        };
        let parts_root = data_dir.join("parts");
        let storage = Arc::new(crate::object_storage::ObjectStorage::in_memory());
        let labels: Labels = [("app".to_string(), "remote".to_string())]
            .into_iter()
            .collect();
        let local_parts = part::flush_rows(rows_with_attributes(&labels, vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000,
                line: "restored after eviction".to_string(),
                structured_metadata: Vec::new(),
            }]),
            &parts_root,
            config.row_group_size)
        .unwrap();
        let other_labels: Labels = [("app".to_string(), "other".to_string())]
            .into_iter()
            .collect();
        let other_parts = part::flush_rows(rows_with_attributes(&other_labels, vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_001,
                line: "must remain remote".to_string(),
                structured_metadata: Vec::new(),
            }]),
            &parts_root,
            config.row_group_size)
        .unwrap();
        let other_data_path = other_parts[0].data_path();
        let mut published_parts = local_parts.clone();
        published_parts.extend(other_parts);
        storage.publish(&published_parts, &[]).await.unwrap();
        let parts = Arc::new(PartRegistry::load_from_disk(&parts_root).unwrap());
        storage
            .evict_cache(&parts_root, 0, &parts.part_dirs())
            .unwrap();
        assert!(parts.has_missing_cache_files());

        let memtable = Arc::new(MemTable::new());
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            parts.operation_lock(),
        ));
        let state = crate::test_support::state(
            config.clone(),
            memtable.clone(),
            Arc::new(Journal::spawn(&config, memtable).unwrap()),
            parts,
            trace_parts,
            Some(Arc::new(crate::object_storage::RemoteCache::new(
                storage, parts_root,
            ))),
        );
        let parsed = flat_query("attr=app=remote");
        let result = run_unified_query(state, test_tenant(), parsed, crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].entries[0].line, "restored after eviction");
        assert!(
            !other_data_path.exists(),
            "query restored an unrelated part"
        );
    }

    #[tokio::test]
    async fn query_replans_restore_after_registry_changes_in_lock_gap() {
        let data_dir = temp_dir();
        let config = Config {
            data_dir: data_dir.clone(),
            ..Config::default()
        };
        let parts_root = data_dir.join("parts");
        let storage = Arc::new(crate::object_storage::ObjectStorage::in_memory());
        let labels: Labels = [("app".to_string(), "remote".to_string())]
            .into_iter()
            .collect();
        let old = part::flush_rows(rows_with_attributes(&labels, vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 1,
                line: "old generation".to_string(),
                structured_metadata: Vec::new(),
            }]),
            &parts_root,
            config.row_group_size)
        .unwrap();
        storage.publish(&old, &[]).await.unwrap();
        let parts = Arc::new(PartRegistry::load_from_disk(&parts_root).unwrap());

        let new = part::flush_rows(rows_with_attributes(&labels, vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 2,
                line: "new generation".to_string(),
                structured_metadata: Vec::new(),
            }]),
            &parts_root,
            config.row_group_size)
        .unwrap();
        let manifest = storage
            .publish(&new, &[old[0].meta.id.clone()])
            .await
            .unwrap();
        let eligible = vec![old[0].dir.clone(), new[0].dir.clone()];
        storage.evict_cache(&parts_root, 0, &eligible).unwrap();

        let memtable = Arc::new(MemTable::new());
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            parts.operation_lock(),
        ));
        let state = crate::test_support::state(
            config.clone(),
            memtable.clone(),
            Arc::new(Journal::spawn(&config, memtable).unwrap()),
            parts.clone(),
            trace_parts,
            Some(Arc::new(crate::object_storage::RemoteCache::new(
                storage,
                parts_root.clone(),
            ))),
        );
        let parsed = flat_query("attr=app=remote");
        let guard = pin_query_parts_with_gap_hook(&state, &test_tenant(), &parsed, crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), || {
            parts.reload_from_manifest(&parts_root, &manifest)
        })
        .await
        .unwrap();
        let result = unified_query(&state, &test_tenant(), &parsed, crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 10, true).unwrap();
        drop(guard);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].entries[0].line, "new generation");
        assert!(new[0].data_path().exists());
    }

    #[tokio::test]
    async fn structured_filters_match_identically_in_memtable_and_parts() {
        let data_dir = temp_dir();
        let labels: Labels = [("app".to_string(), "api".to_string())]
            .into_iter()
            .collect();
        let line = r#"{"code":503,"elapsed":"250ms","json":"ok","level":"error"}"#;
        // Canonical (key-sorted) order, because this fixture hands a `Row`
        // straight to the part writer, bypassing the memtable door that
        // canonicalizes real ingest.
        let metadata = vec![
            ("logfmt".to_string(), "value".to_string()),
            ("trace_id".to_string(), "abc".to_string()),
        ];
        let memtable = Arc::new(MemTable::new());
        memtable.insert(test_tenant(), with_attributes(labels.clone(), vec![LogEntry {
                timestamp_ns: 10,
                line: line.to_string(),
                structured_metadata: metadata.clone(),
            }]));
        let parts = Arc::new(PartRegistry::new());
        parts
            .register(
                part::flush_rows(rows_with_attributes(&labels, vec![Row {
                        tenant: test_tenant(),
                        timestamp_ns: 20,
                        line: line.to_string(),
                        structured_metadata: metadata,
                    }]),
                    &data_dir.join("parts"),
                    1)
                .unwrap(),
            )
            .unwrap();
        let state = test_state(&data_dir, memtable, parts, None);
        let mut parsed =
            flat_query("parse=json&attr=app=api&attr=trace_id=abc&attr=level=~err.*");
        parsed.stages.push(logql::PipelineStage::Field(logql::FieldFilter {
            name: "code".to_string(),
            op: logql::FieldOp::Gte,
            value: logql::FieldValue::Number(logql::Decimal::parse("500").unwrap()),
        }));
        parsed.stages.push(logql::PipelineStage::Field(logql::FieldFilter {
            name: "elapsed".to_string(),
            op: logql::FieldOp::Lt,
            value: logql::FieldValue::Duration(logql::parse_duration_ns("1s").unwrap()),
        }));

        let result = run_unified_query(state.clone(), test_tenant(), parsed, crate::part::QueryTimeRange::closed(0, 30), 10, true)
            .await
            .unwrap();
        let timestamps: Vec<_> = result[0]
            .entries
            .iter()
            .map(|entry| entry.timestamp_ns)
            .collect();
        assert_eq!(timestamps, vec![10, 20]);

        for query in ["parse=json&attr=json=ok", "attr=logfmt=value"] {
            let result =
                run_unified_query(state.clone(), test_tenant(), flat_query(query), crate::part::QueryTimeRange::closed(0, 30), 10, true)
                    .await
                    .unwrap();
            let timestamps: Vec<_> = result[0]
                .entries
                .iter()
                .map(|entry| entry.timestamp_ns)
                .collect();
            assert_eq!(timestamps, vec![10, 20]);
        }
    }

    /// `| trace_id="x"` with no parser stage — the shape the claim rests on —
    /// takes the two-pass path in a part: the narrow `_sm:trace_id` column
    /// selects the rows before anything wide is decoded. The memtable cannot
    /// do that, so memtable-against-part equality is the late-materialization
    /// on/off check, covering a hit, a miss, and a predicate the pass must
    /// refuse (a stream-label name, answered by the label instead).
    #[tokio::test]
    async fn a_metadata_filter_answers_identically_through_the_two_pass_scan() {
        let labels: Labels = [("app".to_string(), "api".to_string())]
            .into_iter()
            .collect();
        let entries: Vec<LogEntry> = (0..50i64)
            .map(|i| LogEntry {
                timestamp_ns: 1_000 + i,
                line: format!("line-{i}"),
                structured_metadata: vec![("trace_id".to_string(), format!("t{}", i % 10))],
            })
            .collect();
        let in_memtable = {
            let data_dir = temp_dir();
            let memtable = Arc::new(MemTable::new());
            memtable.insert(test_tenant(), with_attributes(labels.clone(), entries.clone()));
            test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None)
        };
        let in_parts = {
            let data_dir = temp_dir();
            let rows: Vec<Row> = entries
                .iter()
                .map(|entry| Row {
                    tenant: test_tenant(),
                    timestamp_ns: entry.timestamp_ns,
                    line: entry.line.clone(),
                    structured_metadata: entry.structured_metadata.clone(),
                })
                .collect();
            let parts = Arc::new(PartRegistry::new());
            parts
                .register(part::flush_rows(rows_with_attributes(&labels, rows), &data_dir.join("parts"), 16).unwrap())
                .unwrap();
            test_state(&data_dir, Arc::new(MemTable::new()), parts, None)
        };

        for expression in [
            "attr=app=api&attr=trace_id=t3",
            "attr=app=api&attr=trace_id=absent",
            "attr=app=api&attr=app=api&attr=trace_id=t3",
        ] {
            let parsed = flat_query(expression);
            let flat = |results: Vec<StreamResult>| -> Vec<(i64, String)> {
                let mut rows: Vec<(i64, String)> = results
                    .into_iter()
                    .flat_map(|stream| {
                        stream
                            .entries
                            .into_iter()
                            .map(|entry| (entry.timestamp_ns, entry.line))
                    })
                    .collect();
                rows.sort();
                rows
            };
            let range = crate::part::QueryTimeRange::closed(0, 10_000);
            let from_memtable = flat(
                run_unified_query(
                    in_memtable.clone(),
                    test_tenant(),
                    parsed.clone(),
                    range,
                    1000,
                    false,
                )
                .await
                .unwrap(),
            );
            let from_parts = flat(
                run_unified_query(in_parts.clone(), test_tenant(), parsed, range, 1000, false)
                    .await
                    .unwrap(),
            );
            assert_eq!(
                from_memtable, from_parts,
                "the two-pass scan must not change the answer to {expression}"
            );
            if expression.contains("t3") {
                assert_eq!(from_parts.len(), 5, "the hit case must actually hit");
            }
        }
    }

    /// The selector matches the row's own attributes: an attribute that says
    /// something else keeps the row out of the answer, and the value the row
    /// pushed is the one the response shows.
    #[tokio::test]
    async fn the_selector_matches_the_rows_own_attributes() {
        let memtable = Arc::new(MemTable::new());
        memtable.insert(
            test_tenant(),
            vec![LogEntry {
                timestamp_ns: 10,
                line: "line".to_string(),
                structured_metadata: vec![
                    ("app".to_string(), "smuggled".to_string()),
                    ("trace_id".to_string(), "t1".to_string()),
                ],
            }],
        );
        let data_dir = temp_dir();
        let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);

        let refused = run_unified_query(
            state.clone(),
            test_tenant(),
            flat_query("attr=app=api"),
            crate::part::QueryTimeRange::closed(0, 20),
            10,
            false,
        )
        .await
        .unwrap();
        assert!(refused.is_empty() || refused.iter().all(|s| s.entries.is_empty()));

        let results = run_unified_query(
            state,
            test_tenant(),
            flat_query("attr=app=smuggled"),
            crate::part::QueryTimeRange::closed(0, 20),
            10,
            false,
        )
        .await
        .unwrap();
        let body = log_rows_ndjson(results, false);
        let rows = ndjson_rows(&body);
        assert_eq!(rows[0]["attributes"]["app"], "smuggled");
        assert_eq!(rows[0]["attributes"]["trace_id"], "t1");
    }

    /// `| json | field="x"` — `json_field_rare`'s shape — now takes the
    /// two-pass path too: the `_pf:` column is what `| json` would extract,
    /// and metadata still wins where both exist. Memtable-against-parts is the
    /// on/off pair; the cases are a line-value hit, a miss, and the shadowing
    /// row whose metadata must silence its line.
    #[tokio::test]
    async fn a_json_extraction_filter_answers_identically_through_the_two_pass_scan() {
        let labels: Labels = [("app".to_string(), "api".to_string())]
            .into_iter()
            .collect();
        let mut entries: Vec<LogEntry> = (0..40i64)
            .map(|i| LogEntry {
                timestamp_ns: 1_000 + i,
                line: format!(r#"{{"trace_id":"t{}","status":200}}"#, i % 8),
                structured_metadata: vec![],
            })
            .collect();
        // The shadowing row: the line says t3, the pushed metadata says
        // other — Loki's rule discards the extraction, so `trace_id="t3"`
        // must not return this row and `trace_id="other"` must.
        entries.push(LogEntry {
            timestamp_ns: 2_000,
            line: r#"{"trace_id":"t3"}"#.to_string(),
            structured_metadata: vec![("trace_id".to_string(), "other".to_string())],
        });
        // A non-JSON row: extraction fails, the filter sees nothing.
        entries.push(LogEntry {
            timestamp_ns: 2_001,
            line: "trace_id=t3 not json".to_string(),
            structured_metadata: vec![],
        });

        let in_memtable = {
            let data_dir = temp_dir();
            let memtable = Arc::new(MemTable::new());
            memtable.insert(test_tenant(), with_attributes(labels.clone(), entries.clone()));
            test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None)
        };
        let in_parts = {
            let data_dir = temp_dir();
            let rows: Vec<Row> = entries
                .iter()
                .map(|entry| Row {
                    tenant: test_tenant(),
                    timestamp_ns: entry.timestamp_ns,
                    line: entry.line.clone(),
                    structured_metadata: entry.structured_metadata.clone(),
                })
                .collect();
            let parts = Arc::new(PartRegistry::new());
            parts
                .register(
                    part::flush_rows(
                        rows_with_attributes(&labels, rows),
                        &data_dir.join("parts"),
                        16,
                    )
                    .unwrap(),
                )
                .unwrap();
            test_state(&data_dir, Arc::new(MemTable::new()), parts, None)
        };

        for (expression, expected_rows) in [
            ("parse=json&attr=app=api&attr=trace_id=t3", 5usize),
            ("parse=json&attr=app=api&attr=trace_id=absent", 0),
            ("parse=json&attr=app=api&attr=trace_id=other", 1),
        ] {
            let parsed = flat_query(expression);
            let flat = |results: Vec<StreamResult>| -> Vec<(i64, String)> {
                let mut rows: Vec<(i64, String)> = results
                    .into_iter()
                    .flat_map(|stream| {
                        stream
                            .entries
                            .into_iter()
                            .map(|entry| (entry.timestamp_ns, entry.line))
                    })
                    .collect();
                rows.sort();
                rows
            };
            let range = crate::part::QueryTimeRange::closed(0, 10_000);
            let from_memtable = flat(
                run_unified_query(
                    in_memtable.clone(),
                    test_tenant(),
                    parsed.clone(),
                    range,
                    1000,
                    false,
                )
                .await
                .unwrap(),
            );
            let from_parts = flat(
                run_unified_query(in_parts.clone(), test_tenant(), parsed, range, 1000, false)
                    .await
                    .unwrap(),
            );
            assert_eq!(
                from_memtable, from_parts,
                "the two-pass json path must not change the answer to {expression}"
            );
            assert_eq!(from_parts.len(), expected_rows, "{expression}");
        }
    }

    fn state_with_config(
        config: Config,
        memtable: Arc<MemTable>,
        parts: Arc<PartRegistry>,
    ) -> Arc<AppState> {
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            parts.operation_lock(),
        ));
        crate::test_support::state(
            config.clone(),
            memtable.clone(),
            Arc::new(Journal::spawn(&config, memtable).unwrap()),
            parts,
            trace_parts,
            None,
        )
    }

    fn state_with(
        data_dir: &std::path::Path,
        memtable: Arc<MemTable>,
        parts: Arc<PartRegistry>,
    ) -> Arc<AppState> {
        state_with_config(
            Config {
                data_dir: data_dir.to_path_buf(),
                ..Config::default()
            },
            memtable,
            parts,
        )
    }

    fn tenant_policy_state(
        data_dir: &std::path::Path,
        memtable: Arc<MemTable>,
        parts: Arc<PartRegistry>,
        tenant_policy: Arc<crate::tenant_policy::TenantPolicy>,
    ) -> Arc<AppState> {
        let config = Config {
            data_dir: data_dir.to_path_buf(),
            ..Config::default()
        };
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            parts.operation_lock(),
        ));
        crate::test_support::state_with_tenant_policy(
            config.clone(),
            memtable.clone(),
            Arc::new(Journal::spawn(&config, memtable).unwrap()),
            parts,
            trace_parts,
            None,
            tenant_policy,
        )
    }

    /// A downgrade takes effect at query time within one poll, long before the
    /// bytes are reclaimed.
    #[tokio::test]
    async fn a_downgrade_hides_data_before_the_bytes_are_gone() {
        let data_dir = temp_dir();
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;
        let one_hour_ago = now_ns - 3_600_000_000_000;
        let memtable = Arc::new(MemTable::new());
        memtable.insert(
            test_tenant(),
            vec![LogEntry {
                timestamp_ns: one_hour_ago,
                line: "an hour old".to_string(),
                structured_metadata: vec![("app".to_string(), "api".to_string())],
            }],
        );
        let policy = Arc::new(crate::tenant_policy::TenantPolicy::enabled_for_test());
        let state = tenant_policy_state(
            &data_dir,
            memtable,
            Arc::new(PartRegistry::new()),
            policy.clone(),
        );

        // Nothing pushed yet: the pushed policies are the tenant registry, so
        // the tenant is not served until the control plane onboards it.
        let (status, body) = first_party_logs_status(state.clone()).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

        policy.install_for_test(
            [(
                test_tenant(),
                crate::tenant_policy::TenantRetention::Finite(std::time::Duration::from_secs(
                    7 * 24 * 60 * 60,
                )),
            )]
            .into_iter()
            .collect(),
        );
        assert_eq!(lines_in_last_day(state.clone()).await.len(), 1);

        policy.install_for_test(
            [(
                test_tenant(),
                crate::tenant_policy::TenantRetention::Finite(std::time::Duration::from_secs(60)),
            )]
            .into_iter()
            .collect(),
        );
        assert!(lines_in_last_day(state.clone()).await.is_empty());

        // An upgrade brings back everything still on disk.
        policy.install_for_test(
            [(
                test_tenant(),
                crate::tenant_policy::TenantRetention::Finite(std::time::Duration::from_secs(
                    7 * 24 * 60 * 60,
                )),
            )]
            .into_iter()
            .collect(),
        );
        assert_eq!(lines_in_last_day(state).await.len(), 1);
    }

    /// A cumulative latency sum cannot produce a quantile, and every target in
    /// the plan documents is a p95 or p99. The histogram has to be in the shape
    /// `histogram_quantile` reads: cumulative in `le`, with a `+Inf` bucket
    /// that equals `_count`.
    #[tokio::test]
    async fn the_latency_histogram_is_shaped_for_histogram_quantile() {
        let data_dir = temp_dir();
        let state = state_with(
            &data_dir,
            Arc::new(MemTable::new()),
            Arc::new(PartRegistry::new()),
        );
        for millis in [0, 3, 40, 900] {
            state.metrics.observe_query(
                crate::metrics::QueryEndpoint::Logs,
                std::time::Duration::from_millis(millis),
            );
        }
        // One observation on a second endpoint, so the assertions below prove
        // the split is real rather than one histogram wearing a label.
        state.metrics.observe_query(
            crate::metrics::QueryEndpoint::Histogram,
            std::time::Duration::from_millis(900),
        );

        let rendered = metrics(State(state)).await;
        let bucket = |bound: &str| -> u64 {
            let needle = format!(
                "signy_query_latency_ms_bucket{{endpoint=\"logs\",le=\"{bound}\"}} "
            );
            rendered
                .lines()
                .find_map(|line| line.strip_prefix(&needle))
                .unwrap_or_else(|| panic!("no bucket {bound} in:\n{rendered}"))
                .trim()
                .parse()
                .unwrap()
        };

        assert_eq!(bucket("1"), 1, "only the 0 ms observation is <= 1 ms");
        assert_eq!(bucket("5"), 2);
        assert!(
            rendered.contains("signy_query_latency_ms_count{endpoint=\"logs_histogram\"} 1"),
            "volume's own series must carry only volume's observation:\n{rendered}"
        );
        assert!(
            rendered.contains("signy_query_latency_ms_count{endpoint=\"logs\"} 4"),
            "and query_range's must carry only its own:\n{rendered}"
        );
        assert!(
            rendered.contains("signy_query_latency_ms_count{endpoint=\"tail\"} 0"),
            "an endpoint nothing reached is still exported, so a dashboard shows a gap rather than a missing series:\n{rendered}"
        );
        assert_eq!(bucket("50"), 3);
        assert_eq!(bucket("1000"), 4);
        assert_eq!(bucket("+Inf"), 4);

        // Monotonic in `le`, which is what makes the series a valid histogram.
        let mut previous = 0;
        for bound in ["1", "5", "10", "25", "50", "100", "250", "500", "1000", "+Inf"] {
            let current = bucket(bound);
            assert!(current >= previous, "bucket {bound} went backwards");
            previous = current;
        }
        // The unlabeled series is gone on purpose: an aggregate that is also a
        // labeled series double-counts under `sum`, and the whole-path number
        // is `sum by (le)` across the endpoints.
        assert!(!rendered.contains("signy_query_latency_ms_count 4"));
        assert!(rendered.contains("# TYPE signy_query_latency_ms histogram"));
        assert!(rendered.contains("signy_build_info{"));
        assert!(rendered.contains("signy_capacity_probe 0\n"));
        assert!(
            !rendered.contains("signy_series_states_len"),
            "structural series gauges are probe-only:\n{rendered}"
        );
    }

    #[tokio::test]
    async fn capacity_probe_metrics_render_series_container_shape() {
        let data_dir = temp_dir();
        let config = Config {
            data_dir: data_dir.clone(),
            capacity_probe: true,
            ..Config::default()
        };
        let state = state_with_config(config, Arc::new(MemTable::new()), Arc::new(PartRegistry::new()));
        let labels = SeriesLabels::from_pairs(vec![
            (METRIC_NAME_LABEL.to_string(), "queue_depth".to_string()),
            ("instance".to_string(), "probe".to_string()),
        ]);
        state.journal.series_memtable().insert(vec![MetricSample {
            tenant: test_tenant(),
            labels,
            ts_ns: 1,
            value: MetricValue::Scalar(1.0),
            kind: SampleKind::Gauge,
            datapoint_index: 0,
        }]);

        let rendered = metrics(State(state.clone())).await;
        assert!(rendered.contains("signy_series_states_len 1\n"));
        assert!(rendered.contains("signy_series_buffers_len 1\n"));
        assert!(rendered.contains("signy_series_buffers_inline 1\n"));
        assert!(rendered.contains("signy_series_buffers_stream 0\n"));
        assert!(rendered.contains("signy_series_flushing_series 0\n"));

        let snapshot = state.journal.series_memtable().begin_flush();
        let rendered = metrics(State(state.clone())).await;
        assert!(rendered.contains("signy_series_states_len 1\n"));
        assert!(rendered.contains("signy_series_buffers_len 0\n"));
        assert!(rendered.contains("signy_series_buffers_inline 0\n"));
        assert!(rendered.contains("signy_series_flushing_series 1\n"));
        state.journal.series_memtable().commit_flush();
        drop(snapshot);
    }

    /// The scrape renders whatever the retention worker last published. It
    /// deliberately does not recompute the number: that walk is
    /// `unknown_tenant_count`'s, and its own coverage lives with it.
    #[tokio::test]
    async fn the_unknown_tenant_gauge_renders_the_published_value() {
        let data_dir = temp_dir();
        let policy = Arc::new(crate::tenant_policy::TenantPolicy::enabled_for_test());
        let state = tenant_policy_state(
            &data_dir,
            Arc::new(MemTable::new()),
            Arc::new(PartRegistry::new()),
            policy,
        );
        state
            .metrics
            .unknown_tenants
            .store(3, std::sync::atomic::Ordering::Relaxed);

        let rendered = metrics(State(state)).await;
        assert!(
            rendered.contains("signy_tenant_policy_unknown_tenants 3\n"),
            "{rendered}"
        );
    }

    /// Grafana sends `start`/`end` on every label call. Answering from the
    /// whole history both returns labels that do not exist in the requested
    /// range and reads every part to find them.
    #[tokio::test]
    async fn metadata_endpoints_honour_the_requested_time_range() {
        let data_dir = temp_dir();
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;
        let hour_ns = 3_600_000_000_000i64;
        let memtable = Arc::new(MemTable::new());
        for (app, timestamp_ns) in [("recent", now_ns - hour_ns), ("ancient", now_ns - 48 * hour_ns)]
        {
            memtable.insert(test_tenant(), with_attributes([("app".to_string(), app.to_string())].into_iter().collect(), vec![LogEntry {
                    timestamp_ns,
                    line: format!("{app} line"),
                    structured_metadata: Vec::new(),
                }]));
        }
        let state = state_with(&data_dir, memtable, Arc::new(PartRegistry::new()));

        let values_in = async |start_ns: i64, end_ns: i64| {
            let (status, body) = first_party_get(
                state.clone(),
                &format!(
                    "/signy/api/v1/logs/attributes/app/values?start={start_ns}&end={end_ns}"
                ),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "{body}");
            ndjson_rows(&body)
                .iter()
                .map(|row| row["value"].as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        };

        assert_eq!(
            values_in(now_ns - 2 * hour_ns, now_ns).await,
            vec!["recent".to_string()],
            "a value outside the range must not be offered"
        );
        let mut both = values_in(now_ns - 72 * hour_ns, now_ns).await;
        both.sort();
        assert_eq!(both, vec!["ancient".to_string(), "recent".to_string()]);

        // A range entirely in the future is an empty answer, not an error:
        // a dashboard asks this whenever its window outruns the data.
        assert!(values_in(now_ns + hour_ns, now_ns + 2 * hour_ns).await.is_empty());
    }

    /// With per-tenant policy enabled, the pushed policies are the tenant
    /// registry: a tenant the control plane never mentioned is refused at
    /// query time rather than served its full history.
    #[tokio::test]
    async fn a_tenant_without_a_pushed_policy_is_refused_at_query_time() {
        let data_dir = temp_dir();
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;
        let memtable = Arc::new(MemTable::new());
        memtable.insert(
            test_tenant(),
            vec![LogEntry {
                timestamp_ns: now_ns - 3_600_000_000_000,
                line: "an hour old".to_string(),
                structured_metadata: Vec::new(),
            }],
        );
        let policy = Arc::new(crate::tenant_policy::TenantPolicy::enabled_for_test());
        policy.install_for_test(
            [(
                crate::tenant::TenantId::parse("someone-else").unwrap(),
                crate::tenant_policy::TenantRetention::Finite(std::time::Duration::from_secs(1)),
            )]
            .into_iter()
            .collect(),
        );
        let state =
            tenant_policy_state(&data_dir, memtable, Arc::new(PartRegistry::new()), policy);

        let refused = match logs(
            State(state),
            crate::tenant::test_tenant_headers(),
            RawQuery(Some(format!(
                "start={}&end={now_ns}&attr=app=api",
                now_ns - 86_400_000_000_000
            ))),
        )
        .await
        {
            Err(error) => error,
            Ok(_) => panic!("a tenant nothing was pushed for must be refused"),
        };
        assert_eq!(refused.0, StatusCode::FORBIDDEN);
    }

    /// The attribute endpoints have no range of their own, so they inherit
    /// the retention clamp directly. Memtable entries are filtered per entry;
    /// parts are pruned per part.
    #[tokio::test]
    async fn attribute_endpoints_inherit_the_retention_clamp() {
        let data_dir = temp_dir();
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;
        let memtable = Arc::new(MemTable::new());
        memtable.insert(
            test_tenant(),
            vec![LogEntry {
                timestamp_ns: now_ns - 3_600_000_000_000,
                line: "an hour old".to_string(),
                structured_metadata: vec![("app".to_string(), "stale".to_string())],
            }],
        );
        memtable.insert(
            test_tenant(),
            vec![LogEntry {
                timestamp_ns: now_ns,
                line: "just now".to_string(),
                structured_metadata: vec![("app".to_string(), "fresh".to_string())],
            }],
        );
        let policy = Arc::new(crate::tenant_policy::TenantPolicy::enabled_for_test());
        policy.install_for_test(
            [(
                test_tenant(),
                crate::tenant_policy::TenantRetention::Finite(std::time::Duration::from_secs(60)),
            )]
            .into_iter()
            .collect(),
        );
        let state = tenant_policy_state(
            &data_dir,
            memtable,
            Arc::new(PartRegistry::new()),
            policy.clone(),
        );

        let values = logs_attribute_values(
            State(state.clone()),
            crate::tenant::test_tenant_headers(),
            Path("app".to_string()),
            RawQuery(None),
        )
        .await
        .unwrap();
        let body = axum::body::to_bytes(values.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        let values: Vec<String> = ndjson_rows(&body)
            .iter()
            .map(|row| row["value"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(values, vec!["fresh".to_string()]);
    }

    fn tail_labels() -> Labels {
        [("app".to_string(), "tailed".to_string())]
            .into_iter()
            .collect()
    }

    fn tail_entry(timestamp_ns: i64, line: &str) -> LogEntry {
        LogEntry {
            timestamp_ns,
            line: line.to_string(),
            structured_metadata: vec![],
        }
    }

    fn tail_query() -> logql::LogQuery {
        flat_query("attr=app=tailed")
    }

    /// A tail must send each line exactly once. A cursor alone cannot do that:
    /// advancing past the newest timestamp drops every other entry sharing that
    /// nanosecond, and stopping on it resends them. Lines written as one batch
    /// routinely share a timestamp, so this is the ordinary case, not an edge.
    #[tokio::test]
    async fn a_tail_sends_each_line_once_even_when_timestamps_collide() {
        let dir = temp_dir();
        let memtable = Arc::new(MemTable::new());
        let state = test_state(&dir, memtable.clone(), Arc::new(PartRegistry::new()), None);

        let collision_ns = 1_700_000_000_000_000_000;
        memtable.insert(test_tenant(), with_attributes(tail_labels(), vec![
                tail_entry(collision_ns, "first"),
                tail_entry(collision_ns, "second"),
            ]));

        let mut cursor = TailCursor::new(collision_ns - 1);
        let query = tail_query();
        let first = tail_poll(&state, &test_tenant(), &query, &mut cursor, collision_ns, 100)
            .await
            .expect("the first poll delivers both lines");
        let lines = tail_lines(&first);
        assert_eq!(lines, vec!["first", "second"]);

        // Nothing new: the same window must not resend what it already sent.
        assert!(
            tail_poll(&state, &test_tenant(), &query, &mut cursor, collision_ns, 100)
                .await
                .is_none(),
            "a repeated poll over the same window has nothing to send"
        );

        // A third line at the very same nanosecond still has to arrive.
        memtable.insert(test_tenant(), with_attributes(tail_labels(), vec![tail_entry(collision_ns, "third")]));
        let third = tail_poll(&state, &test_tenant(), &query, &mut cursor, collision_ns, 100)
            .await
            .expect("a late arrival at the same timestamp is not lost");
        assert_eq!(tail_lines(&third), vec!["third"]);
    }

    /// A burst larger than one poll's limit is left for the next poll, not
    /// skipped. The tail falls behind rather than losing lines, which is what
    /// makes reporting an empty `dropped_entries` true rather than a stub.
    #[tokio::test]
    async fn a_burst_larger_than_the_limit_is_delivered_across_polls() {
        let dir = temp_dir();
        let memtable = Arc::new(MemTable::new());
        let state = test_state(&dir, memtable.clone(), Arc::new(PartRegistry::new()), None);

        let base_ns = 1_700_000_000_000_000_000;
        let entries: Vec<LogEntry> = (0..5)
            .map(|index| tail_entry(base_ns + index, &format!("line-{index}")))
            .collect();
        memtable.insert(test_tenant(), with_attributes(tail_labels(), entries));

        let mut cursor = TailCursor::new(base_ns - 1);
        let query = tail_query();
        let end_ns = base_ns + 10;

        let first = tail_poll(&state, &test_tenant(), &query, &mut cursor, end_ns, 2)
            .await
            .expect("the oldest two arrive first");
        assert_eq!(tail_lines(&first), vec!["line-0", "line-1"]);

        let second = tail_poll(&state, &test_tenant(), &query, &mut cursor, end_ns, 2)
            .await
            .expect("the next poll continues rather than skipping ahead");
        assert_eq!(tail_lines(&second), vec!["line-2", "line-3"]);

        let third = tail_poll(&state, &test_tenant(), &query, &mut cursor, end_ns, 2)
            .await
            .expect("and drains the rest");
        assert_eq!(tail_lines(&third), vec!["line-4"]);
    }

    fn tail_lines(fresh: &[StreamResult]) -> Vec<String> {
        let mut rows: Vec<(i64, String)> = fresh
            .iter()
            .flat_map(|stream| {
                stream
                    .entries
                    .iter()
                    .map(|entry| (entry.timestamp_ns, entry.line.clone()))
            })
            .collect();
        rows.sort_by_key(|(timestamp_ns, _)| *timestamp_ns);
        rows.into_iter().map(|(_, line)| line).collect()
    }

    async fn get_json(state: &Arc<AppState>, uri: &str) -> (StatusCode, serde_json::Value) {
        let request = axum::http::Request::builder()
            .uri(uri)
            .header(crate::tenant::TENANT_HEADER, test_tenant().as_str())
            .body(axum::body::Body::empty())
            .unwrap();
        let response = crate::build_router(state.clone())
            .oneshot(request)
            .await
            .unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        (
            status,
            serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null),
        )
    }

    /// The quota reaches the HTTP query path, and it answers 429 rather than
    /// 500 — the difference between "come back later" and "this is broken".
    #[tokio::test]
    async fn a_tenant_over_its_query_quota_is_refused_with_429() {
        let dir = temp_dir();
        let memtable = Arc::new(MemTable::new());
        let config = Config {
            data_dir: dir.clone(),
            max_concurrent_queries_per_tenant: 1,
            ..Config::default()
        };
        let parts = Arc::new(PartRegistry::new());
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::standalone());
        let state = crate::test_support::state(
            config.clone(),
            memtable.clone(),
            Arc::new(Journal::spawn(&config, memtable).unwrap()),
            parts,
            trace_parts,
            None,
        );

        // The tenant's one slot is taken by a query still in flight.
        let slot = state.tenant_quota.begin_query(&test_tenant()).unwrap();

        let uri = "/signy/api/v1/logs?attr=app%3Dquota";
        let (refused, body) = get_json(&state, uri).await;
        assert_eq!(
            refused,
            StatusCode::TOO_MANY_REQUESTS,
            "a tenant at its concurrency limit is told to come back: {body}"
        );
        assert_eq!(
            state
                .metrics
                .query_quota_rejected
                .load(std::sync::atomic::Ordering::Relaxed),
            1,
            "counted apart from queries this instance failed to answer"
        );
        assert_eq!(
            state
                .metrics
                .query_errors
                .load(std::sync::atomic::Ordering::Relaxed),
            0,
            "a refused query is not a failed one"
        );

        drop(slot);
        let (allowed, _) = get_json(&state, uri).await;
        assert_eq!(allowed, StatusCode::OK, "the freed slot admits the next query");
    }

    async fn send(
        state: &Arc<AppState>,
        method: &str,
        uri: &str,
    ) -> (StatusCode, serde_json::Value) {
        let request = axum::http::Request::builder()
            .method(method)
            .uri(uri)
            .header(crate::tenant::TENANT_HEADER, test_tenant().as_str())
            .body(axum::body::Body::empty())
            .unwrap();
        let response = crate::build_router(state.clone())
            .oneshot(request)
            .await
            .unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        (
            status,
            serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null),
        )
    }

    fn delete_state() -> (Arc<AppState>, i64) {
        let dir = temp_dir();
        let memtable = Arc::new(MemTable::new());
        let state = test_state(&dir, memtable.clone(), Arc::new(PartRegistry::new()), None);
        let base_ns = 1_700_000_000_000_000_000i64;
        for (app, line) in [("keep", "kept line"), ("drop", "secret line")] {
            memtable.insert(test_tenant(), with_attributes([("app".to_string(), app.to_string())]
                    .into_iter()
                    .collect::<Labels>(), vec![LogEntry {
                    timestamp_ns: base_ns,
                    line: line.to_string(),
                    structured_metadata: vec![],
                }]));
        }
        (state, base_ns)
    }

    async fn lines_for(state: &Arc<AppState>, filters: &str) -> Vec<String> {
        let (status, _, body) = first_party_logs(
            state.clone(),
            &format!("start=1699999999&end=1700000001&limit=100&{filters}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        ndjson_rows(&body)
            .iter()
            .map(|row| row["line"].as_str().unwrap().to_string())
            .collect()
    }

    /// The promise of the endpoint: the lines stop being readable when the
    /// request is accepted, not when a rewrite eventually runs. Only the
    /// selected stream goes.
    #[tokio::test]
    async fn an_accepted_deletion_hides_its_lines_immediately() {
        let (state, base_ns) = delete_state();
        assert_eq!(lines_for(&state, "attr=app%3D~.%2B").await.len(), 2);

        let (status, _) = send(
            &state,
            "POST",
            &format!(
                "/signy/api/v1/logs/delete?attr=app%3Ddrop&start={}&end={}",
                (base_ns - 1_000_000_000) / 1_000_000_000,
                (base_ns + 1_000_000_000) / 1_000_000_000
            ),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        assert_eq!(
            lines_for(&state, "attr=app%3D~.%2B").await,
            vec!["kept line".to_string()],
            "the deleted stream is gone and the other one is untouched"
        );
    }

    /// A request outside the window the lines fall in deletes nothing. Getting
    /// this wrong deletes more than was asked for, which is unrecoverable.
    #[tokio::test]
    async fn a_deletion_outside_the_window_removes_nothing() {
        let (state, base_ns) = delete_state();
        let (status, _) = send(
            &state,
            "POST",
            &format!(
                "/signy/api/v1/logs/delete?attr=app%3Ddrop&start={}&end={}",
                (base_ns / 1_000_000_000) + 10,
                (base_ns / 1_000_000_000) + 20
            ),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert_eq!(lines_for(&state, "attr=app%3D~.%2B").await.len(), 2);
    }

    /// Listing reports what was asked for, and cancelling puts the lines back —
    /// which is only honest while the request has not been applied.
    #[tokio::test]
    async fn a_cancelled_request_makes_its_lines_readable_again() {
        let (state, base_ns) = delete_state();
        let window = format!(
            "start={}&end={}",
            (base_ns - 1_000_000_000) / 1_000_000_000,
            (base_ns + 1_000_000_000) / 1_000_000_000
        );
        send(
            &state,
            "POST",
            &format!("/signy/api/v1/logs/delete?attr=app%3Ddrop&{window}"),
        )
        .await;

        let (status, body) =
            first_party_get(state.clone(), "/signy/api/v1/logs/delete").await;
        assert_eq!(status, StatusCode::OK);
        let listed = ndjson_rows(&body);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0]["status"], "received");
        // The listed query is the persisted canonical form — resubmittable
        // as-is.
        assert_eq!(listed[0]["query"], "attr=app%3Ddrop");
        let request_id = listed[0]["request_id"].as_str().unwrap().to_string();

        let (status, _) = send(
            &state,
            "DELETE",
            &format!("/signy/api/v1/logs/delete?request_id={request_id}"),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert_eq!(lines_for(&state, "attr=app%3D~.%2B").await.len(), 2);

        let (_, body) = first_party_get(state.clone(), "/signy/api/v1/logs/delete").await;
        assert!(ndjson_rows(&body).is_empty());

        let (status, _) = send(
            &state,
            "DELETE",
            "/signy/api/v1/logs/delete?request_id=never-existed",
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    /// A selector this cannot honour consistently is refused rather than
    /// accepted and half-applied. A pipeline stage names a value derived from
    /// the line, so what it deletes would change whenever the parser does.
    #[tokio::test]
    async fn a_deletion_query_must_be_a_selector() {
        let (state, base_ns) = delete_state();
        let window = format!(
            "start={}&end={}",
            base_ns / 1_000_000_000,
            base_ns / 1_000_000_000 + 1
        );
        for query_string in [
            // A parsed field would change meaning whenever the parser does;
            // `parse` is simply not in this endpoint's grammar.
            "parse=json&attr=app%3Ddrop&attr=status%3D500",
            // Line filters alone name no attribute; at least one attr is
            // required.
            "contains=secret",
            // An empty selector deletes everything, which is refused.
            "",
        ] {
            let (status, _) = send(
                &state,
                "POST",
                &format!("/signy/api/v1/logs/delete?{query_string}&{window}"),
            )
            .await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{query_string}");
        }

        // A deletion without an explicit start is refused rather than guessed
        // at.
        let (status, _) = send(
            &state,
            "POST",
            "/signy/api/v1/logs/delete?attr=app%3Ddrop",
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);

        // A window that ends before it starts deletes an empty set, which is
        // far more likely to be a mistake than an intent.
        let (status, _) = send(
            &state,
            "POST",
            "/signy/api/v1/logs/delete?attr=app%3Ddrop&start=1700000010&end=1700000000",
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    fn log_entry(timestamp_ns: i64, line: &str, metadata: &[(&str, &str)]) -> LogEntry {
        LogEntry {
            timestamp_ns,
            line: line.to_string(),
            structured_metadata: metadata
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
        }
    }

    /// The comparison corpus names its JSON free-text key `_msg` so that
    /// VictoriaLogs keeps the text as its message. On this side the name must
    /// stay an ordinary extracted field: reserved names are enforced for
    /// stream labels at ingest, not for what `| json` extracts, so `_msg`
    /// must extract, filter and promote like any other field.
    #[tokio::test]
    async fn a_json_field_named_msg_extracts_and_filters_like_any_other() {
        let labels: Labels = [("app".to_string(), "api".to_string())]
            .into_iter()
            .collect();
        let memtable = Arc::new(MemTable::new());
        memtable.insert(test_tenant(), with_attributes(labels.clone(), vec![
                log_entry(10, r#"{"level":"error","_msg":"boom"}"#, &[]),
                log_entry(11, r#"{"level":"info","_msg":"fine"}"#, &[]),
            ]));
        let data_dir = temp_dir();
        let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);
        let parsed = flat_query("parse=json&attr=app=api&attr=_msg=boom");
        let results = run_unified_query(
            state,
            test_tenant(),
            parsed,
            crate::part::QueryTimeRange::closed(0, 20),
            10,
            false,
        )
        .await
        .unwrap();

        let body = log_rows_ndjson(results, false);
        let rows = ndjson_rows(&body);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["attributes"]["_msg"], "boom");
    }

    /// An extraction whose name is a pushed metadata key is dropped rather than
    /// renamed, so `trace_id_extracted` — a name Loki's response never contained
    /// — must not appear. Loki 3.3.2 answers `| json | trace_id="<the JSON
    /// value>"` with nothing and renders `{{.trace_id}}` as the metadata value.
    #[tokio::test]
    async fn an_extraction_shadowed_by_pushed_metadata_never_reaches_the_response() {
        let labels: Labels = [("app".to_string(), "api".to_string())]
            .into_iter()
            .collect();
        let memtable = Arc::new(MemTable::new());
        memtable.insert(test_tenant(), with_attributes(labels.clone(), vec![log_entry(
                10,
                r#"{"trace_id":"from-the-line","level":"error"}"#,
                &[("trace_id", "from-the-push")],
            )]));
        let data_dir = temp_dir();
        let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);
        let results = run_unified_query(
            state,
            test_tenant(),
            flat_query("parse=json&attr=app=api"),
            crate::part::QueryTimeRange::closed(0, 20),
            10,
            false,
        )
        .await
        .unwrap();

        let body = log_rows_ndjson(results, false);
        let rows = ndjson_rows(&body);
        assert_eq!(rows[0]["attributes"]["trace_id"], "from-the-push");
        assert!(rows[0]["attributes"].get("trace_id_extracted").is_none());
    }

/// Every refusal this classifier can see, and the one that used to be a fault.
///
/// The scan path reports `String`, so this function is the only thing standing
/// between a working limit and a `500`. An exhausted query memory pool reached
/// clients as `INTERNAL_SERVER_ERROR` until 2026-08-13 — measured once in a
/// 45-minute run at the capacity ceiling — which pages an operator for a
/// refusal and tells a client library that backing off is pointless. The other
/// arms are pinned beside it because the distinctions are the point: a broad
/// query is permanently wrong (`400`), a busy instance is temporarily unable
/// (`429`), and only an unrecognized error is a fault.
#[test]
fn a_refusal_is_never_reported_as_a_server_fault() {
    use crate::memory_budget::{EXHAUSTED_PREFIX, OVER_BUDGET_PREFIX};

    assert_eq!(
        metric_error_status(&format!(
            "{EXHAUSTED_PREFIX} 322122547 bytes holds 322122547 in use and this request needs 16"
        )),
        StatusCode::TOO_MANY_REQUESTS,
        "an instance out of query memory is busy, not broken"
    );
    assert_eq!(
        metric_error_status(&format!(
            "{OVER_BUDGET_PREFIX} 999999999 bytes, more than the whole instance memory account"
        )),
        StatusCode::BAD_REQUEST,
        "a request larger than the whole account can never be served, so a retry is a lie"
    );
    assert_eq!(
        metric_error_status(&format!("{TENANT_QUOTA_PREFIX}scan rate exceeded")),
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        metric_error_status("query exceeds the maximum of 1000000 scanned rows"),
        StatusCode::BAD_REQUEST,
        "a query too broad to answer is permanently too broad"
    );
    assert_eq!(
        metric_error_status("query timed out"),
        StatusCode::GATEWAY_TIMEOUT
    );
    assert_eq!(
        metric_error_status("trace query exceeds the maximum of 100000 spans"),
        StatusCode::PAYLOAD_TOO_LARGE,
        "a trace too large to carry is about the data, not the request or the load"
    );
    assert_eq!(
        metric_error_status("trace query timed out"),
        StatusCode::GATEWAY_TIMEOUT
    );
    assert_eq!(
        metric_error_status("trace object store restore timed out"),
        StatusCode::GATEWAY_TIMEOUT
    );
    assert_eq!(
        metric_error_status("part reader returned garbage"),
        StatusCode::INTERNAL_SERVER_ERROR,
        "and something nobody recognizes is still a fault"
    );
}

/// The runbook decides what to alert on; `deploy/alerts.yml` is that decision
/// in the form a Prometheus can load. Two files holding one set of decisions
/// drift, and the drift is silent in the worst direction: the operator is
/// paged by neither because they believe the other has it. So the tie is a
/// test rather than a convention.
///
/// Three ways it can fail, and each is a real defect rather than tidiness. A
/// row added to the table with no rule is an alert someone decided to have and
/// nobody will receive. A rule naming a metric this engine does not export is a
/// rule that can never fire — a renamed metric leaves exactly this shape. And a
/// rule on a signal the table never listed is a page whose meaning is written
/// down nowhere.
#[test]
fn every_alert_signal_in_the_runbook_has_a_rule() {
    let runbook = include_str!("../../docs/RUNBOOK.md");
    let rules = include_str!("../../deploy/alerts.yml");
    let exported = include_str!("prometheus.rs");

    // The signals are the first column of the "What to alert on" table, one
    // backticked metric per row.
    let table = runbook
        .split_once("## What to alert on")
        .expect("the runbook must still have the table these rules come from")
        .1;
    let table = table.split("\n## ").next().unwrap();
    let signals: Vec<&str> = table
        .lines()
        .filter_map(|line| line.strip_prefix("| `signy_"))
        .filter_map(|rest| rest.split_once('`').map(|(name, _)| name))
        .collect();
    // A row's condition and meaning name metrics too — "increasing while
    // `flush_success_total` is flat" is half of an expression, and the memory
    // row sends the reader to `signy_query_latency_ms`. Those are
    // explained by the table even though they are not its signals, so a rule
    // may reach for them.
    let mentioned: Vec<&str> = table
        .match_indices('`')
        .filter_map(|(at, _)| table[at + 1..].split_once('`').map(|(name, _)| name))
        .map(|name| name.trim_start_matches("signy_"))
        .collect();
    assert!(
        signals.len() > 10,
        "the table parsed as {} rows, which means its shape changed, not that it shrank",
        signals.len()
    );

    let missing: Vec<&&str> = signals
        .iter()
        .filter(|name| !rules.contains(&format!("signy_{name}")))
        .collect();
    assert!(
        missing.is_empty(),
        "these signals are in the runbook's table but no rule in deploy/alerts.yml uses them: {missing:#?}"
    );

    // Every metric a rule names must be one this engine actually renders, and
    // must be a signal the table explains. Helper metrics a rule divides or
    // guards by are still the first: an expression cannot evaluate against a
    // series nobody exports.
    let mut rest = rules;
    let mut unexported: Vec<String> = Vec::new();
    let mut unexplained: Vec<String> = Vec::new();
    while let Some(start) = rest.find("signy_") {
        rest = &rest[start..];
        let end = rest
            .find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'))
            .unwrap_or(rest.len());
        let name = rest[..end].to_string();
        rest = &rest[end..];
        let bare = name.trim_start_matches("signy_").to_string();
        if !exported.contains(&name) && !unexported.contains(&name) {
            unexported.push(name.clone());
        }
        // A histogram is exported under its base name and queried by its
        // derived series, so the table naming the base is enough.
        let explained = signals
            .iter()
            .chain(mentioned.iter())
            .any(|known| bare == *known || bare.starts_with(known));
        // Guards and denominators are not signals; they are named here so the
        // list of what a rule may reach for stays visible.
        const HELPERS: [&str; 3] = [
            "signy_part_count",
            "signy_tenant_policy_known_tenants",
            "signy_drain_in_progress",
        ];
        if !explained && !HELPERS.contains(&name.as_str()) && !unexplained.contains(&name) {
            unexplained.push(name);
        }
    }
    assert!(
        unexported.is_empty(),
        "deploy/alerts.yml names metrics this engine does not export: {unexported:#?}"
    );
    assert!(
        unexplained.is_empty(),
        "deploy/alerts.yml alerts on signals the runbook's table does not explain: {unexplained:#?}"
    );
}

// --- The first-party API: parameter grammar (`params.rs`) ---

#[test]
fn attr_operator_scan_takes_the_longest_match_at_the_first_operator() {
    let params = parse_filter_params("attr=level=error", 0, LOGS_PARAMS).unwrap();
    let matcher = &params.query.matchers[0];
    assert_eq!(matcher.name, "level");
    assert_eq!(matcher.op, logql::MatcherOp::Eq);
    assert_eq!(matcher.value, "error");

    let params = parse_filter_params("attr=level!%3Ddebug", 0, LOGS_PARAMS).unwrap();
    assert_eq!(params.query.matchers[0].op, logql::MatcherOp::Neq);
    assert_eq!(params.query.matchers[0].value, "debug");

    let params = parse_filter_params("attr=path%3D~/api/.*", 0, LOGS_PARAMS).unwrap();
    assert_eq!(params.query.matchers[0].op, logql::MatcherOp::Re);

    let params = parse_filter_params("attr=host!~db-.*", 0, LOGS_PARAMS).unwrap();
    assert_eq!(params.query.matchers[0].op, logql::MatcherOp::NRe);

    // The value keeps any later operator characters verbatim.
    let params = parse_filter_params("attr=formula=a%3Db", 0, LOGS_PARAMS).unwrap();
    assert_eq!(params.query.matchers[0].name, "formula");
    assert_eq!(params.query.matchers[0].value, "a=b");
}

#[test]
fn attr_without_an_operator_teaches_the_form() {
    let error = parse_filter_params("attr=level", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("has no operator"), "{error}");
    assert!(error.contains("attr=key=value"), "{error}");

    let error = parse_filter_params("attr=%3Derror", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("empty key"), "{error}");
}

#[test]
fn comparison_operators_split_at_the_longest_match_and_logs_refuse_them() {
    // The refusal text carries the reassembled filter, which is the proof the
    // operator scan split key, operator and value correctly — `>=` before `>`.
    let error = parse_filter_params("attr=duration>=1.5s", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("'duration>=1.5s' uses a comparison"), "{error}");
    assert!(error.contains("trace endpoints"), "{error}");
    let error = parse_filter_params("attr=duration<=2s", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("'duration<=2s'"), "{error}");
    let error = parse_filter_params("attr=k>v", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("'k>v'"), "{error}");
    let error = parse_filter_params("attr=k<v", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("'k<v'"), "{error}");

    // A comparison character after a matcher operator stays in the value.
    let params = parse_filter_params("attr=formula=a>b", 0, LOGS_PARAMS).unwrap();
    assert_eq!(params.query.matchers[0].value, "a>b");

    let error = parse_filter_params("attr=>=v", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("empty key"), "{error}");

    // The refusal is the parser's, so `parse=` endpoints inherit it too.
    let error = parse_filter_params("parse=json&attr=duration>=1s", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("uses a comparison"), "{error}");
}

#[test]
fn unknown_parameters_name_themselves_and_the_accepted_set() {
    let error = parse_filter_params("atr=level=error", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("unknown parameter 'atr'"), "{error}");
    assert!(error.contains("attr"), "{error}");
    assert!(error.contains("docs/QUERY_API.md"), "{error}");
}

#[test]
fn relative_times_need_a_unit_and_negative_epochs_stay_epochs() {
    let now_ns = 1_700_000_000_000_000_000;
    assert_eq!(
        parse_time_or_relative_ns("-1h", now_ns).unwrap(),
        now_ns - 3_600_000_000_000
    );
    assert_eq!(
        parse_time_or_relative_ns("-90s", now_ns).unwrap(),
        now_ns - 90_000_000_000
    );
    assert_eq!(
        parse_time_or_relative_ns("-5", now_ns).unwrap(),
        -5_000_000_000
    );
    let error = parse_time_or_relative_ns("-1hh", now_ns).unwrap_err();
    assert!(error.contains("invalid relative time"), "{error}");
}

#[test]
fn parse_json_recompiles_attr_filters_into_field_stages() {
    let params =
        parse_filter_params("parse=json&attr=status=500&attr=path%3D~/api/.*", 0, LOGS_PARAMS)
            .unwrap();
    assert!(params.query.matchers.is_empty());
    assert!(matches!(
        params.query.stages[0],
        logql::PipelineStage::Json
    ));
    let logql::PipelineStage::Field(filter) = &params.query.stages[1] else {
        panic!("attr must become a field filter after parse=");
    };
    assert_eq!(filter.name, "status");
    assert_eq!(filter.op, logql::FieldOp::Eq);
    assert!(matches!(
        &params.query.stages[2],
        logql::PipelineStage::Field(filter) if filter.op == logql::FieldOp::Regex
    ));

    let error = parse_filter_params("parse=json&parse=json", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("more than once"), "{error}");
    let error = parse_filter_params("parse=regex", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("expected json or logfmt"), "{error}");
}

#[test]
fn scalar_parameters_refuse_duplicates() {
    let error = parse_filter_params("start=1&start=2", 0, LOGS_PARAMS).unwrap_err();
    assert!(error.contains("'start' was given more than once"), "{error}");
}

// --- The first-party API: `GET /signy/api/v1/logs` ---

async fn first_party_logs(
    state: Arc<AppState>,
    query_string: &str,
) -> (StatusCode, axum::http::HeaderMap, String) {
    let request = axum::http::Request::builder()
        .uri(format!("/signy/api/v1/logs?{query_string}"))
        .header(crate::tenant::TENANT_HEADER, test_tenant().as_str())
        .body(axum::body::Body::empty())
        .unwrap();
    let response = crate::build_router(state).oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    (status, headers, String::from_utf8(body.to_vec()).unwrap())
}

fn ndjson_rows(body: &str) -> Vec<serde_json::Value> {
    body.lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[tokio::test]
async fn first_party_logs_answers_ndjson_with_string_timestamps() {
    let data_dir = temp_dir();
    let labels: Labels = [("app".to_string(), "api".to_string())]
        .into_iter()
        .collect();
    let memtable = Arc::new(MemTable::new());
    memtable.insert(
        test_tenant(),
        with_attributes(
            labels,
            vec![
                LogEntry {
                    timestamp_ns: 5_000_000_000,
                    line: "hello timeout".to_string(),
                    structured_metadata: Vec::new(),
                },
                LogEntry {
                    timestamp_ns: 6_000_000_000,
                    line: "unrelated".to_string(),
                    structured_metadata: Vec::new(),
                },
            ],
        ),
    );
    let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);

    let (status, headers, body) =
        first_party_logs(state, "start=4&end=7&attr=app=api&contains=timeout").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers[axum::http::header::CONTENT_TYPE],
        "application/x-ndjson"
    );
    assert!(headers.contains_key(SCANNED_ROWS_HEADER));
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["line"], "hello timeout");
    assert_eq!(rows[0]["timestamp"], "5000000000");
    assert!(rows[0]["timestamp"].is_string());
    assert_eq!(rows[0]["attributes"]["app"], "api");
}

/// The log path prices a request from its own shape, not from a fixed cap.
///
/// This is what pricing each request buys over charging every query the
/// per-query ceiling: two requests differing only in `limit` are two different
/// prices, so a small query is not made to wait behind the worst case a large
/// one might reach. The parts here record what their rows materialize into, so
/// the per-row half of the multiplication is measured rather than assumed.
#[tokio::test]
async fn a_log_query_is_priced_from_its_own_limit_and_the_rows_its_parts_recorded() {
    let data_dir = temp_dir();
    let parts = Arc::new(PartRegistry::new());
    parts
        .register(
            part::flush_rows(
                (0..2_000)
                    .map(|at| Row {
                        tenant: test_tenant(),
                        timestamp_ns: 1_000_000 + at,
                        line: "x".repeat(256),
                        structured_metadata: vec![("app".to_string(), "api".to_string())],
                    })
                    .collect(),
                &data_dir.join("parts"),
                256,
            )
            .unwrap(),
        )
        .unwrap();
    let (rows, bytes_per_row) = parts
        .materialization_estimate(&test_tenant(), part::QueryTimeRange::half_open(0, i64::MAX))
        .expect("the part records what its rows materialize into");
    assert_eq!(rows, 2_000);
    assert!(
        bytes_per_row >= 256,
        "a 256-byte line cannot materialize into less than itself: {bytes_per_row}"
    );

    // Sized in rows of this very data, and above the floors a real
    // configuration has to clear (`flush_chunk_bytes` alone is 1 MiB).
    let account_bytes = bytes_per_row * 8_000;
    let config = Config {
        data_dir: data_dir.to_path_buf(),
        memory_account: Arc::new(crate::memory_budget::MemoryAccount::new(account_bytes)),
        max_query_memory_bytes: account_bytes,
        merge_max_memory_bytes: account_bytes,
        merge_max_input_bytes: account_bytes / 2,
        flush_chunk_bytes: 1024 * 1024,
        ..Config::default()
    };
    config
        .validate()
        .expect("the shape under test is one an operator could actually configure");
    let state = state_with_config(config, Arc::new(MemTable::new()), parts);

    // Both limits fit an empty account. Leave room for a hundred rows and not
    // a thousand, and the two requests part company on their own arithmetic:
    // `limit × bytes_per_row`, nothing else about them differs.
    let peer = state
        .memory_account
        .admit(account_bytes - bytes_per_row * 200)
        .expect("a peer leaves room for two hundred rows");

    let (status, _, body) = first_party_logs(state.clone(), "start=0&end=9999999&limit=100").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ndjson_rows(&body).len(), 100);

    let (status, _, body) = first_party_logs(state.clone(), "start=0&end=9999999&limit=1000").await;
    assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "ten times the rows is ten times the price, against the same free space: {body}"
    );
    assert!(body.contains(crate::memory_budget::EXHAUSTED_PREFIX), "{body}");

    drop(peer);
    let (status, _, body) = first_party_logs(state.clone(), "start=0&end=9999999&limit=1000").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ndjson_rows(&body).len(), 1_000);
    assert_eq!(
        state.memory_account.in_use_bytes(),
        0,
        "every charge left with its response"
    );
}

/// The same query, refused and then served, with only a peer's charge between
/// the two — which is the whole claim a `429` makes to a client.
#[tokio::test]
async fn a_log_query_with_no_room_beside_a_peer_is_a_429_and_recovers() {
    let data_dir = temp_dir();
    let labels: Labels = [("app".to_string(), "api".to_string())]
        .into_iter()
        .collect();
    let memtable = Arc::new(MemTable::new());
    memtable.insert(
        test_tenant(),
        with_attributes(
            labels,
            vec![LogEntry {
                timestamp_ns: 5_000_000_000,
                line: "hello".to_string(),
                structured_metadata: Vec::new(),
            }],
        ),
    );
    let account_bytes = 4 * 1024 * 1024;
    let state = state_with_config(
        Config {
            data_dir: data_dir.to_path_buf(),
            memory_account: Arc::new(crate::memory_budget::MemoryAccount::new(account_bytes)),
            max_query_memory_bytes: account_bytes,
            merge_max_memory_bytes: account_bytes,
            flush_chunk_bytes: account_bytes / 2,
            ..Config::default()
        },
        memtable,
        Arc::new(PartRegistry::new()),
    );

    let peer = state
        .memory_account
        .admit(account_bytes)
        .expect("a peer takes the whole account");
    let (status, _, body) = first_party_logs(state.clone(), "start=4&end=7&limit=10").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{body}");
    assert!(body.contains(crate::memory_budget::EXHAUSTED_PREFIX), "{body}");
    assert_eq!(state.memory_account.exhausted(), 1);

    drop(peer);
    let (status, _, body) = first_party_logs(state.clone(), "start=4&end=7&limit=10").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ndjson_rows(&body).len(), 1);
    assert_eq!(state.memory_account.in_use_bytes(), 0);
}

#[tokio::test]
async fn first_party_logs_default_direction_is_backward_and_limit_applies() {
    let data_dir = temp_dir();
    let memtable = Arc::new(MemTable::new());
    memtable.insert(
        test_tenant(),
        (1..=5)
            .map(|at| LogEntry {
                timestamp_ns: at * 1_000_000_000,
                line: format!("row {at}"),
                structured_metadata: vec![("app".to_string(), "api".to_string())],
            })
            .collect(),
    );
    let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);

    let (status, _, body) = first_party_logs(state.clone(), "start=0&end=10&limit=2").await;
    assert_eq!(status, StatusCode::OK);
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["line"], "row 5");
    assert_eq!(rows[1]["line"], "row 4");

    let (_, _, body) =
        first_party_logs(state, "start=0&end=10&limit=2&direction=forward").await;
    let rows = ndjson_rows(&body);
    assert_eq!(rows[0]["line"], "row 1");
    assert_eq!(rows[1]["line"], "row 2");
}

#[tokio::test]
async fn first_party_logs_selects_without_any_filter() {
    let data_dir = temp_dir();
    let memtable = Arc::new(MemTable::new());
    memtable.insert(
        test_tenant(),
        vec![LogEntry {
            timestamp_ns: 5_000_000_000,
            line: "bare row".to_string(),
            structured_metadata: Vec::new(),
        }],
    );
    let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);

    let (status, _, body) = first_party_logs(state, "start=4&end=6").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ndjson_rows(&body).len(), 1);
}

#[tokio::test]
async fn first_party_logs_parse_json_filters_on_extracted_fields() {
    let data_dir = temp_dir();
    let memtable = Arc::new(MemTable::new());
    memtable.insert(
        test_tenant(),
        vec![
            LogEntry {
                timestamp_ns: 5_000_000_000,
                line: r#"{"status":"500","path":"/api/x"}"#.to_string(),
                structured_metadata: vec![("app".to_string(), "api".to_string())],
            },
            LogEntry {
                timestamp_ns: 6_000_000_000,
                line: r#"{"status":"200","path":"/api/y"}"#.to_string(),
                structured_metadata: vec![("app".to_string(), "api".to_string())],
            },
        ],
    );
    let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);

    let (status, _, body) = first_party_logs(
        state,
        "start=4&end=7&parse=json&attr=app=api&attr=status=500",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["line"], r#"{"status":"500","path":"/api/x"}"#);
}

#[tokio::test]
async fn first_party_logs_refuses_unknown_parameters_with_a_teaching_error() {
    let data_dir = temp_dir();
    let state = test_state(
        &data_dir,
        Arc::new(MemTable::new()),
        Arc::new(PartRegistry::new()),
        None,
    );

    let (status, _, body) = first_party_logs(state, "atr=app=api").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error: serde_json::Value = serde_json::from_str(&body).unwrap();
    let message = error["error"].as_str().unwrap();
    assert!(message.contains("unknown parameter 'atr'"), "{message}");
    assert!(message.contains("docs/QUERY_API.md"), "{message}");
}

#[tokio::test]
async fn unmatched_routes_answer_with_the_first_party_surface() {
    let data_dir = temp_dir();
    let state = test_state(
        &data_dir,
        Arc::new(MemTable::new()),
        Arc::new(PartRegistry::new()),
        None,
    );
    let request = axum::http::Request::builder()
        .uri("/signy/api/v1/nope")
        .body(axum::body::Body::empty())
        .unwrap();
    let response = crate::build_router(state).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let message = error["error"].as_str().unwrap();
    assert!(message.contains("/signy/api/v1/logs"), "{message}");
}

#[tokio::test]
async fn a_comparison_operator_on_a_log_endpoint_is_refused_with_the_fix() {
    let data_dir = temp_dir();
    let state = test_state(
        &data_dir,
        Arc::new(MemTable::new()),
        Arc::new(PartRegistry::new()),
        None,
    );
    let (status, _, body) = first_party_logs(state.clone(), "attr=duration%3E%3D1s").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("trace endpoints"), "{body}");

    let (status, body) = first_party_histogram(state, "attr=duration%3E%3D1s&bucket=30s").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("trace endpoints"), "{body}");
}

// --- The first-party API: `GET /signy/api/v1/logs/histogram` ---

async fn first_party_histogram(
    state: Arc<AppState>,
    query_string: &str,
) -> (StatusCode, String) {
    let request = axum::http::Request::builder()
        .uri(format!("/signy/api/v1/logs/histogram?{query_string}"))
        .header(crate::tenant::TENANT_HEADER, test_tenant().as_str())
        .body(axum::body::Body::empty())
        .unwrap();
    let response = crate::build_router(state).oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

/// The boundary test the plan calls the likeliest silent bug: a row exactly on
/// a bucket boundary belongs to the bucket it starts, `[start, end)` at every
/// bucket, including the partial first and last.
#[tokio::test]
async fn histogram_buckets_are_half_open_and_epoch_aligned() {
    const BUCKET_NS: i64 = 10_000_000_000;
    let data_dir = temp_dir();
    let memtable = Arc::new(MemTable::new());
    memtable.insert(
        test_tenant(),
        [
            BUCKET_NS,                  // exactly on a bucket start
            BUCKET_NS + 1,              // inside the same bucket
            2 * BUCKET_NS - 1,          // last instant of the same bucket
            2 * BUCKET_NS,              // exactly on the next bucket start
        ]
        .into_iter()
        .map(|timestamp_ns| LogEntry {
            timestamp_ns,
            line: "row".to_string(),
            structured_metadata: Vec::new(),
        })
        .collect(),
    );
    let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);

    // The query range is not bucket-aligned; the buckets must be.
    let (status, body) = first_party_histogram(
        state,
        "start=9.999999997&end=20.000000003&bucket=10s",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["bucket_start"], (0).to_string());
    assert_eq!(rows[0]["bucket_end"], BUCKET_NS.to_string());
    assert_eq!(rows[0]["count"], 0);
    assert_eq!(rows[1]["bucket_start"], BUCKET_NS.to_string());
    assert_eq!(rows[1]["count"], 3);
    assert_eq!(rows[2]["bucket_start"], (2 * BUCKET_NS).to_string());
    assert_eq!(rows[2]["count"], 1);
}

/// The partial first and last buckets count only rows inside the query range.
#[tokio::test]
async fn histogram_clips_partial_buckets_to_the_range() {
    const BUCKET_NS: i64 = 10_000_000_000;
    let data_dir = temp_dir();
    let memtable = Arc::new(MemTable::new());
    memtable.insert(
        test_tenant(),
        [
            BUCKET_NS + 2, // in the first bucket, before start
            BUCKET_NS + 5, // in range
            BUCKET_NS + 7, // on end: excluded
            BUCKET_NS + 8, // after end, same bucket
        ]
        .into_iter()
        .map(|timestamp_ns| LogEntry {
            timestamp_ns,
            line: "row".to_string(),
            structured_metadata: Vec::new(),
        })
        .collect(),
    );
    let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);

    let (status, body) = first_party_histogram(
        state,
        "start=10.000000004&end=10.000000007&bucket=10s",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["count"], 1);
}

/// The histogram and the search endpoint must agree on what the filters
/// select: `sum(count)` equals the row count `/logs` returns.
#[tokio::test]
async fn histogram_counts_agree_with_the_search_endpoint() {
    let data_dir = temp_dir();
    let memtable = Arc::new(MemTable::new());
    memtable.insert(
        test_tenant(),
        (1..=20)
            .map(|at| LogEntry {
                timestamp_ns: at * 1_000_000_000,
                line: if at % 2 == 0 {
                    format!("timeout {at}")
                } else {
                    format!("fine {at}")
                },
                structured_metadata: vec![("app".to_string(), "api".to_string())],
            })
            .collect(),
    );
    let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);

    let filters = "start=0&end=30&attr=app=api&contains=timeout";
    let (_, _, search_body) =
        first_party_logs(state.clone(), &format!("{filters}&limit=1000")).await;
    let (status, histogram_body) =
        first_party_histogram(state, &format!("{filters}&bucket=5s")).await;
    assert_eq!(status, StatusCode::OK, "{histogram_body}");
    let total: u64 = ndjson_rows(&histogram_body)
        .iter()
        .map(|bucket| bucket["count"].as_u64().unwrap())
        .sum();
    assert_eq!(total, ndjson_rows(&search_body).len() as u64);
    assert_eq!(total, 10);
}

#[tokio::test]
async fn histogram_refuses_a_bucket_count_over_the_cap() {
    let data_dir = temp_dir();
    let state = test_state(
        &data_dir,
        Arc::new(MemTable::new()),
        Arc::new(PartRegistry::new()),
        None,
    );
    let (status, body) =
        first_party_histogram(state, "start=0&end=100000&bucket=1s").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let error: serde_json::Value = serde_json::from_str(&body).unwrap();
    let message = error["error"].as_str().unwrap();
    assert!(message.contains("buckets"), "{message}");
    assert!(message.contains("widen bucket="), "{message}");
}

#[tokio::test]
async fn histogram_rejects_search_only_parameters() {
    let data_dir = temp_dir();
    let state = test_state(
        &data_dir,
        Arc::new(MemTable::new()),
        Arc::new(PartRegistry::new()),
        None,
    );
    let (status, body) = first_party_histogram(state, "start=0&end=10&limit=5").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("unknown parameter 'limit'"), "{body}");
}

// --- The first-party API: attribute autocomplete ---

async fn first_party_get(state: Arc<AppState>, path_and_query: &str) -> (StatusCode, String) {
    let request = axum::http::Request::builder()
        .uri(path_and_query)
        .header(crate::tenant::TENANT_HEADER, test_tenant().as_str())
        .body(axum::body::Body::empty())
        .unwrap();
    let response = crate::build_router(state).oneshot(request).await.unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[tokio::test]
async fn attribute_keys_come_from_the_memtable_and_the_parts() {
    let data_dir = temp_dir();
    let memtable = Arc::new(MemTable::new());
    memtable.insert(
        test_tenant(),
        vec![LogEntry {
            timestamp_ns: 5_000_000_000,
            line: "row".to_string(),
            structured_metadata: vec![
                ("app".to_string(), "api".to_string()),
                ("level".to_string(), "error".to_string()),
            ],
        }],
    );
    let parts = Arc::new(PartRegistry::new());
    parts
        .register(
            part::flush_rows(
                vec![Row {
                    tenant: test_tenant(),
                    timestamp_ns: 6_000_000_000,
                    line: "part row".to_string(),
                    structured_metadata: vec![("host".to_string(), "db-1".to_string())],
                }],
                &data_dir.join("parts"),
                8,
            )
            .unwrap(),
        )
        .unwrap();
    let state = test_state(&data_dir, memtable, parts, None);

    let (status, body) =
        first_party_get(state, "/signy/api/v1/logs/attributes?start=4&end=7").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let keys: Vec<String> = ndjson_rows(&body)
        .iter()
        .map(|row| row["key"].as_str().unwrap().to_string())
        .collect();
    assert!(keys.contains(&"app".to_string()), "{keys:?}");
    assert!(keys.contains(&"level".to_string()), "{keys:?}");
    assert!(keys.contains(&"host".to_string()), "{keys:?}");
}

#[tokio::test]
async fn attribute_values_narrow_by_attr_filters() {
    let data_dir = temp_dir();
    let memtable = Arc::new(MemTable::new());
    memtable.insert(
        test_tenant(),
        vec![
            LogEntry {
                timestamp_ns: 5_000_000_000,
                line: "row".to_string(),
                structured_metadata: vec![
                    ("app".to_string(), "api".to_string()),
                    ("level".to_string(), "error".to_string()),
                ],
            },
            LogEntry {
                timestamp_ns: 6_000_000_000,
                line: "row".to_string(),
                structured_metadata: vec![
                    ("app".to_string(), "worker".to_string()),
                    ("level".to_string(), "info".to_string()),
                ],
            },
        ],
    );
    let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);

    let (status, body) = first_party_get(
        state.clone(),
        "/signy/api/v1/logs/attributes/level/values?start=4&end=7",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let values: Vec<String> = ndjson_rows(&body)
        .iter()
        .map(|row| row["value"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(values, vec!["error".to_string(), "info".to_string()]);

    let (status, body) = first_party_get(
        state,
        "/signy/api/v1/logs/attributes/level/values?start=4&end=7&attr=app=api",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let values: Vec<String> = ndjson_rows(&body)
        .iter()
        .map(|row| row["value"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(values, vec!["error".to_string()]);
}

#[tokio::test]
async fn attribute_values_refuse_line_filters_by_the_unknown_parameter_rule() {
    let data_dir = temp_dir();
    let state = test_state(
        &data_dir,
        Arc::new(MemTable::new()),
        Arc::new(PartRegistry::new()),
        None,
    );
    let (status, body) = first_party_get(
        state,
        "/signy/api/v1/logs/attributes/level/values?contains=x",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("unknown parameter 'contains'"), "{body}");
}

// --- The first-party API: `GET /signy/api/v1/logs/tail` ---

#[tokio::test]
async fn first_party_tail_streams_the_backlog_as_ndjson_rows() {
    use futures_util::StreamExt;

    let data_dir = temp_dir();
    let memtable = Arc::new(MemTable::new());
    let now_ns = 1_700_000_000_000_000_000;
    memtable.insert(
        test_tenant(),
        vec![
            LogEntry {
                timestamp_ns: now_ns,
                line: "tailed line".to_string(),
                structured_metadata: vec![("app".to_string(), "api".to_string())],
            },
            LogEntry {
                timestamp_ns: now_ns + 1,
                line: "second line".to_string(),
                structured_metadata: vec![("app".to_string(), "api".to_string())],
            },
        ],
    );
    let state = test_state(&data_dir, memtable, Arc::new(PartRegistry::new()), None);

    let request = axum::http::Request::builder()
        .uri(format!(
            "/signy/api/v1/logs/tail?start={now_ns}&attr=app=api"
        ))
        .header(crate::tenant::TENANT_HEADER, test_tenant().as_str())
        .body(axum::body::Body::empty())
        .unwrap();
    let response = crate::build_router(state).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[axum::http::header::CONTENT_TYPE],
        "application/x-ndjson"
    );
    let mut body = response.into_body().into_data_stream();
    let chunk = tokio::time::timeout(Duration::from_secs(5), body.next())
        .await
        .expect("the first poll is immediate")
        .expect("the stream is open")
        .expect("the chunk is data");
    let text = String::from_utf8(chunk.to_vec()).unwrap();
    let rows = ndjson_rows(&text);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["line"], "tailed line");
    assert_eq!(rows[0]["attributes"]["app"], "api");
    assert_eq!(rows[1]["line"], "second line");
}

#[tokio::test]
async fn first_party_tail_ends_cleanly_on_drain() {
    let data_dir = temp_dir();
    let state = test_state(
        &data_dir,
        Arc::new(MemTable::new()),
        Arc::new(PartRegistry::new()),
        None,
    );
    state.shutdown.begin_drain();

    let request = axum::http::Request::builder()
        .uri("/signy/api/v1/logs/tail")
        .header(crate::tenant::TENANT_HEADER, test_tenant().as_str())
        .body(axum::body::Body::empty())
        .unwrap();
    let response = crate::build_router(state).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = tokio::time::timeout(
        Duration::from_secs(5),
        axum::body::to_bytes(response.into_body(), 1024 * 1024),
    )
    .await
    .expect("a draining tail closes its stream")
    .unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn first_party_tail_refuses_what_a_tail_cannot_honour() {
    let data_dir = temp_dir();
    let state = test_state(
        &data_dir,
        Arc::new(MemTable::new()),
        Arc::new(PartRegistry::new()),
        None,
    );

    let (status, body) =
        first_party_get(state.clone(), "/signy/api/v1/logs/tail?direction=forward").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("unknown parameter 'direction'"), "{body}");

    let held: Vec<_> = (0..state.config.max_concurrent_tails)
        .map(|_| {
            Arc::clone(&state.tail_semaphore)
                .try_acquire_owned()
                .expect("the limit is not reached yet")
        })
        .collect();
    let (status, body) = first_party_get(state, "/signy/api/v1/logs/tail").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(body.contains("live tail connections"), "{body}");
    drop(held);
}

/// The persisted delete selector is the canonical flat form, and the shared
/// parser reads it back to the same query — the round trip the startup load
/// depends on.
#[test]
fn the_canonical_filter_form_round_trips_through_the_shared_parser() {
    let original = parse_filter_params(
        "attr=app=drop&attr=host!~db-.*&contains=secret%20word&not_regex=a%7Cb",
        0,
        DELETE_PARAMS,
    )
    .unwrap()
    .query;
    let canonical = canonical_filter_query(&original);
    let reparsed = parse_filter_params(&canonical, 0, DELETE_FILTER_PARAMS)
        .unwrap()
        .query;
    assert_eq!(canonical_filter_query(&reparsed), canonical);
    assert_eq!(reparsed.matchers.len(), 2);
    assert_eq!(reparsed.matchers[0].value, "drop");
    assert_eq!(reparsed.matchers[1].op, logql::MatcherOp::NRe);
    assert_eq!(reparsed.line_filters.len(), 2);
}

/// An API reference that silently falls behind the code is worse than none —
/// especially here, where an agent's only knowledge of the surface may be
/// this document. Adding a route or parameter without documenting it breaks
/// this test rather than shipping quietly.
#[test]
fn every_query_api_route_and_param_is_documented() {
    let reference = include_str!("../../docs/QUERY_API.md");
    let mut missing: Vec<String> = Vec::new();
    for route in ROUTES {
        if !reference.contains(route) {
            missing.push((*route).to_string());
        }
    }
    for params in [
        LOGS_PARAMS,
        HISTOGRAM_PARAMS,
        TAIL_PARAMS,
        ATTRIBUTE_KEYS_PARAMS,
        ATTRIBUTE_VALUES_PARAMS,
        DELETE_PARAMS,
        DELETE_FILTER_PARAMS,
        TRACE_SEARCH_PARAMS,
        TRACE_ATTRIBUTE_KEYS_PARAMS,
        TRACE_ATTRIBUTE_VALUES_PARAMS,
    ] {
        for name in params {
            let documented = format!("`{name}`");
            if !reference.contains(&documented) && !missing.contains(&documented) {
                missing.push(documented);
            }
        }
    }
    assert!(
        missing.is_empty(),
        "undocumented in docs/QUERY_API.md: {missing:?}"
    );
}

// --- The first-party API: `GET /signy/api/v1/traces/{trace_id}` ---

fn trace_state(
    data_dir: &std::path::Path,
    mutate: impl FnOnce(&mut Config),
) -> Arc<AppState> {
    let mut config = Config {
        data_dir: data_dir.to_path_buf(),
        ..Config::default()
    };
    mutate(&mut config);
    let memtable = Arc::new(MemTable::new());
    let parts = Arc::new(PartRegistry::new());
    let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
        parts.operation_lock(),
    ));
    crate::test_support::state(
        config.clone(),
        memtable.clone(),
        Arc::new(Journal::spawn(&config, memtable).unwrap()),
        parts,
        trace_parts,
        None,
    )
}

fn otlp_kv(key: &str, value: &str) -> opentelemetry_proto::tonic::common::v1::KeyValue {
    opentelemetry_proto::tonic::common::v1::KeyValue {
        key: key.to_string(),
        value: Some(opentelemetry_proto::tonic::common::v1::AnyValue {
            value: Some(
                opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(
                    value.to_string(),
                ),
            ),
        }),
        ..Default::default()
    }
}

fn trace_span_at(
    trace_id: &str,
    span_id: &str,
    start_time_ns: i64,
    attribute: (&str, &str),
) -> crate::trace::TraceSpan {
    crate::trace::TraceSpan {
        tenant: test_tenant(),
        trace_id: trace_id.to_string(),
        span_id: span_id.to_string(),
        start_time_ns,
        end_time_ns: start_time_ns + 1_000,
        span: opentelemetry_proto::tonic::trace::v1::Span {
            name: format!("span-{span_id}"),
            attributes: vec![otlp_kv(attribute.0, attribute.1)],
            ..Default::default()
        },
        resource: None,
        resource_schema_url: String::new(),
        scope: None,
        scope_schema_url: String::new(),
    }
}

#[tokio::test]
async fn trace_by_id_returns_flat_span_rows_sorted_by_start() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    let trace_id = "ab".repeat(16);
    let mut child = trace_span_at(&trace_id, "cc".repeat(8).as_str(), 2_000, ("env", "span-wins"));
    child.span.parent_span_id = vec![0xbb; 8];
    child.span.kind = opentelemetry_proto::tonic::trace::v1::span::SpanKind::Client as i32;
    let mut root = trace_span_at(&trace_id, "bb".repeat(8).as_str(), 1_000, ("env", "span-wins"));
    root.span.kind = opentelemetry_proto::tonic::trace::v1::span::SpanKind::Server as i32;
    root.span.status = Some(opentelemetry_proto::tonic::trace::v1::Status {
        code: opentelemetry_proto::tonic::trace::v1::status::StatusCode::Ok as i32,
        ..Default::default()
    });
    root.span.events = vec![opentelemetry_proto::tonic::trace::v1::span::Event {
        time_unix_nano: 1_500,
        name: "exception".to_string(),
        attributes: vec![otlp_kv("exception.type", "IOError")],
        ..Default::default()
    }];
    root.resource = Some(opentelemetry_proto::tonic::resource::v1::Resource {
        attributes: vec![
            otlp_kv("service.name", "api"),
            otlp_kv("env", "resource-loses"),
        ],
        ..Default::default()
    });
    // Inserted child-first: the response order must come from the sort.
    state.journal.trace_memtable().insert(vec![child, root]);

    let (status, body) =
        first_party_get(state, &format!("/signy/api/v1/traces/{trace_id}")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 2);

    let root_row = &rows[0];
    assert_eq!(root_row["trace_id"], trace_id.as_str());
    assert_eq!(root_row["span_id"], "bb".repeat(8).as_str());
    assert_eq!(root_row["parent_span_id"], "");
    assert_eq!(root_row["name"], "span-bbbbbbbbbbbbbbbb");
    assert_eq!(root_row["kind"], "server");
    assert_eq!(root_row["service"], "api");
    assert_eq!(root_row["status"], "ok");
    assert_eq!(root_row["start"], "1000");
    assert_eq!(root_row["end"], "2000");
    assert_eq!(root_row["duration"], "1000");
    // One merged map: the span attribute shadows the resource's same-named key.
    assert_eq!(root_row["attributes"]["env"], "span-wins");
    assert_eq!(root_row["attributes"]["service.name"], "api");
    assert_eq!(root_row["events"][0]["name"], "exception");
    assert_eq!(root_row["events"][0]["timestamp"], "1500");
    assert_eq!(root_row["events"][0]["attributes"]["exception.type"], "IOError");

    let child_row = &rows[1];
    assert_eq!(child_row["parent_span_id"], "bb".repeat(8).as_str());
    assert_eq!(child_row["kind"], "client");
    assert_eq!(child_row["status"], "unset");
    assert_eq!(child_row["service"], "");
}

#[tokio::test]
async fn trace_by_id_rejects_a_malformed_id_and_any_parameter() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});

    let (status, body) = first_party_get(state.clone(), "/signy/api/v1/traces/nope").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("32 hexadecimal characters"), "{body}");

    let uri = format!("/signy/api/v1/traces/{}?start=-1h", "ab".repeat(16));
    let (status, body) = first_party_get(state, &uri).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("takes no parameters"), "{body}");
}

#[tokio::test]
async fn an_unknown_trace_id_is_a_404_that_names_retention() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    let (status, body) = first_party_get(
        state,
        &format!("/signy/api/v1/traces/{}", "ab".repeat(16)),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("retention"), "{body}");
}

/// A trace lookup carries no window, so the floor is applied span by span: a
/// wholly expired trace is a 404 and a straddling one keeps only the spans
/// that are still retained.
#[tokio::test]
async fn trace_by_id_drops_spans_below_the_retention_floor() {
    const HOUR_NS: i64 = 60 * 60 * 1_000_000_000;
    let now_ns = crate::tenant_policy::now_ns();
    let data_dir = temp_dir();
    let config = Config {
        data_dir: data_dir.to_path_buf(),
        ..Config::default()
    };
    let memtable = Arc::new(MemTable::new());
    let parts = Arc::new(PartRegistry::new());
    let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
        parts.operation_lock(),
    ));
    let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
    journal.trace_memtable().insert(vec![
        trace_span_at(&"aa".repeat(16), "old-only", now_ns - 2 * HOUR_NS, ("k", "v")),
        trace_span_at(&"bb".repeat(16), "straddle-old", now_ns - 2 * HOUR_NS, ("k", "v")),
        trace_span_at(&"bb".repeat(16), "straddle-new", now_ns - 60_000_000_000, ("k", "v")),
    ]);
    let policy = Arc::new(crate::tenant_policy::TenantPolicy::enabled_for_test());
    policy.install_for_test(
        [(
            test_tenant(),
            crate::tenant_policy::TenantRetention::Finite(std::time::Duration::from_secs(
                60 * 60,
            )),
        )]
        .into_iter()
        .collect(),
    );
    let state = crate::test_support::state_with_tenant_policy(
        config, memtable, journal, parts, trace_parts, None, policy,
    );

    let (status, _) = first_party_get(
        state.clone(),
        &format!("/signy/api/v1/traces/{}", "aa".repeat(16)),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, body) = first_party_get(
        state,
        &format!("/signy/api/v1/traces/{}", "bb".repeat(16)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["name"], "span-straddle-new");
}

#[tokio::test]
async fn a_trace_with_more_spans_than_the_budget_is_a_413() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |config| config.max_trace_spans = 2);
    let trace_id = "ab".repeat(16);
    state.journal.trace_memtable().insert(vec![
        trace_span_at(&trace_id, "s1", 1_000, ("k", "v")),
        trace_span_at(&trace_id, "s2", 2_000, ("k", "v")),
        trace_span_at(&trace_id, "s3", 3_000, ("k", "v")),
    ]);
    let (status, body) =
        first_party_get(state, &format!("/signy/api/v1/traces/{trace_id}")).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    assert!(body.contains("trace query exceeds"), "{body}");
}

#[tokio::test]
async fn the_api_fallback_lists_the_trace_routes() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    let (status, body) = first_party_get(state, "/signy/api/v1/nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("/signy/api/v1/traces/{trace_id}"), "{body}");
}

// --- The first-party API: `GET /signy/api/v1/traces` ---

/// A span with an explicit end, for duration and overlap cases.
fn trace_span_between(
    trace_id: &str,
    span_id: &str,
    start_time_ns: i64,
    end_time_ns: i64,
) -> crate::trace::TraceSpan {
    let mut span = trace_span_at(trace_id, span_id, start_time_ns, ("k", "v"));
    span.end_time_ns = end_time_ns;
    span
}

#[tokio::test]
async fn trace_search_returns_newest_first_summaries() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    let older = "aa".repeat(16);
    let newer = "bb".repeat(16);
    let mut root = trace_span_at(&older, "a-root", 1_000, ("env", "prod"));
    root.resource = Some(opentelemetry_proto::tonic::resource::v1::Resource {
        attributes: vec![otlp_kv("service.name", "api")],
        ..Default::default()
    });
    state.journal.trace_memtable().insert(vec![
        root,
        trace_span_at(&newer, "b-root", 5_000, ("env", "prod")),
    ]);

    let (status, body) = first_party_get(state, "/signy/api/v1/traces?start=0&end=10").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["trace_id"], newer.as_str());
    assert_eq!(rows[1]["trace_id"], older.as_str());
    assert_eq!(rows[1]["root_service"], "api");
    assert_eq!(rows[1]["root_name"], "span-a-root");
    assert_eq!(rows[1]["start"], "1000");
    assert_eq!(rows[1]["end"], "2000");
    assert_eq!(rows[1]["duration"], "1000");
    assert_eq!(rows[1]["span_count"], 1);
}

#[tokio::test]
async fn trace_search_matches_on_a_child_span_but_summarizes_the_full_window() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    let trace_id = "ab".repeat(16);
    let mut child = trace_span_at(&trace_id, "child", 2_000, ("http.route", "/items"));
    child.span.parent_span_id = vec![0xbb; 8];
    state.journal.trace_memtable().insert(vec![
        trace_span_at(&trace_id, "root", 1_000, ("env", "prod")),
        child,
    ]);

    let (status, body) = first_party_get(
        state,
        "/signy/api/v1/traces?start=0&end=10&attr=http.route=/items",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["root_name"], "span-root");
    assert_eq!(rows[0]["start"], "1000");
    assert_eq!(rows[0]["span_count"], 2);
}

#[tokio::test]
async fn a_span_that_began_before_the_window_and_ran_into_it_still_matches() {
    const SECOND_NS: i64 = 1_000_000_000;
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    state.journal.trace_memtable().insert(vec![
        // Began at 3s, still running at 8s: overlaps the [5s, 10s] window.
        trace_span_between(&"aa".repeat(16), "running", 3 * SECOND_NS, 8 * SECOND_NS),
        // Over before the window opened.
        trace_span_between(&"bb".repeat(16), "finished", 3 * SECOND_NS, 4 * SECOND_NS),
    ]);

    let (status, body) = first_party_get(state, "/signy/api/v1/traces?start=5&end=10").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["trace_id"], "aa".repeat(16).as_str());
}

/// `bb` straddles the floor: one span expired, one retained. It belongs in
/// the results because the tenant still holds activity inside the clamped
/// window, and its summary must be built from the retained span alone — a
/// trace that survives the floor must not carry expired timestamps out.
#[tokio::test]
async fn trace_search_clamps_its_window_to_the_retention_floor() {
    const HOUR_NS: i64 = 60 * 60 * 1_000_000_000;
    let now_ns = crate::tenant_policy::now_ns();
    let data_dir = temp_dir();
    let config = Config {
        data_dir: data_dir.to_path_buf(),
        ..Config::default()
    };
    let memtable = Arc::new(MemTable::new());
    let parts = Arc::new(PartRegistry::new());
    let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
        parts.operation_lock(),
    ));
    let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
    let fresh_ns = now_ns - 60_000_000_000;
    journal.trace_memtable().insert(vec![
        trace_span_at(&"aa".repeat(16), "old-only", now_ns - 2 * HOUR_NS, ("k", "v")),
        trace_span_at(&"bb".repeat(16), "straddle-old", now_ns - 2 * HOUR_NS, ("k", "v")),
        trace_span_at(&"bb".repeat(16), "straddle-new", fresh_ns, ("k", "v")),
    ]);
    let policy = Arc::new(crate::tenant_policy::TenantPolicy::enabled_for_test());
    policy.install_for_test(
        [(
            test_tenant(),
            crate::tenant_policy::TenantRetention::Finite(std::time::Duration::from_secs(
                60 * 60,
            )),
        )]
        .into_iter()
        .collect(),
    );
    let state = crate::test_support::state_with_tenant_policy(
        config, memtable, journal, parts, trace_parts, None, policy,
    );

    let (status, body) =
        first_party_get(state.clone(), "/signy/api/v1/traces?start=-3h").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["trace_id"], "bb".repeat(16).as_str());
    assert_eq!(rows[0]["span_count"], 1);
    assert_eq!(rows[0]["start"], fresh_ns.to_string().as_str());

    // A window that ends below the floor is empty, not an error.
    let (status, body) =
        first_party_get(state, "/signy/api/v1/traces?start=-3h&end=-2h").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.is_empty(), "{body}");
}

#[tokio::test]
async fn duration_comparisons_select_traces_by_their_spans_own_durations() {
    const MS_NS: i64 = 1_000_000;
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    state.journal.trace_memtable().insert(vec![
        trace_span_between(&"aa".repeat(16), "fast", 1_000, 1_000 + 100 * MS_NS),
        trace_span_between(&"bb".repeat(16), "slow", 1_000, 1_000 + 1_000 * MS_NS),
        trace_span_between(&"cc".repeat(16), "slower", 1_000, 1_000 + 5_000 * MS_NS),
    ]);

    let (status, body) = first_party_get(
        state.clone(),
        "/signy/api/v1/traces?start=0&end=10&attr=duration%3E%3D1s",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ndjson_rows(&body).len(), 2);

    let (status, body) = first_party_get(
        state.clone(),
        "/signy/api/v1/traces?start=0&end=10&attr=duration%3E%3D1s&attr=duration%3C%3D2s",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["trace_id"], "bb".repeat(16).as_str());

    // Equality parses the unit too: nanoseconds, not the string "100ms".
    let (status, body) = first_party_get(
        state,
        "/signy/api/v1/traces?start=0&end=10&attr=duration=100ms",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["trace_id"], "aa".repeat(16).as_str());
}

#[tokio::test]
async fn a_comparison_on_a_non_duration_key_is_refused_with_the_fix() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    let (status, body) = first_party_get(
        state,
        "/signy/api/v1/traces?attr=latency%3E%3D1s",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("duration intrinsic only"), "{body}");
}

#[tokio::test]
async fn a_bare_number_duration_threshold_is_refused_with_the_unit_hint() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    let (status, body) = first_party_get(
        state,
        "/signy/api/v1/traces?attr=duration%3E%3D1000",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("write a unit"), "{body}");
}

#[tokio::test]
async fn trace_search_respects_its_limit_and_its_cap() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |config| config.max_trace_search_limit = 2);
    state.journal.trace_memtable().insert(vec![
        trace_span_at(&"aa".repeat(16), "s1", 1_000, ("k", "v")),
        trace_span_at(&"bb".repeat(16), "s2", 2_000, ("k", "v")),
        trace_span_at(&"cc".repeat(16), "s3", 3_000, ("k", "v")),
    ]);

    let (status, body) = first_party_get(
        state.clone(),
        "/signy/api/v1/traces?start=0&end=10&limit=1",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["trace_id"], "cc".repeat(16).as_str(), "newest first");

    let (status, body) = first_party_get(
        state,
        "/signy/api/v1/traces?start=0&end=10&limit=3",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("limit exceeds the maximum of 2"), "{body}");
}

/// Both halves of the scan answer the same overlap rule: a part-side span
/// that began before the window and ran into it, and a memtable span inside
/// it, join in one summary.
#[tokio::test]
async fn trace_search_reads_the_memtable_and_the_parts_through_one_window_rule() {
    const SECOND_NS: i64 = 1_000_000_000;
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    let trace_id = "ab".repeat(16);
    let flushed = crate::trace_part::flush_trace_spans(
        &[trace_span_between(&trace_id, "flushed", 3 * SECOND_NS, 6 * SECOND_NS)],
        &data_dir.join("traces"),
        8192,
    )
    .unwrap();
    state.trace_parts.register(flushed).unwrap();
    state.journal.trace_memtable().insert(vec![trace_span_between(
        &trace_id,
        "buffered",
        7 * SECOND_NS,
        8 * SECOND_NS,
    )]);

    let (status, body) = first_party_get(state, "/signy/api/v1/traces?start=5&end=10").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["span_count"], 2);
    assert_eq!(rows[0]["start"], (3 * SECOND_NS).to_string().as_str());
}

// --- The first-party API: `/traces/attributes` and `/traces/attributes/{key}/values` ---

#[tokio::test]
async fn trace_attribute_keys_come_from_the_window_and_include_the_intrinsics() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    let mut span = trace_span_at(&"aa".repeat(16), "s1", 1_000, ("http.route", "/items"));
    span.resource = Some(opentelemetry_proto::tonic::resource::v1::Resource {
        attributes: vec![otlp_kv("service.name", "api"), otlp_kv("env", "prod")],
        ..Default::default()
    });
    state.journal.trace_memtable().insert(vec![
        span,
        // Outside the window: its attribute key must not appear.
        trace_span_at(&"bb".repeat(16), "s2", 20_000_000_000_000, ("outside", "x")),
    ]);

    let (status, body) = first_party_get(
        state,
        "/signy/api/v1/traces/attributes?start=0&end=10",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let keys: Vec<String> = ndjson_rows(&body)
        .iter()
        .map(|row| row["key"].as_str().unwrap().to_string())
        .collect();
    for expected in ["duration", "name", "status", "service.name", "http.route", "env"] {
        assert!(keys.contains(&expected.to_string()), "{keys:?}");
    }
    assert!(!keys.contains(&"outside".to_string()), "{keys:?}");
}

#[tokio::test]
async fn trace_attribute_values_respect_the_window_and_are_narrowed_by_filters() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    let mut api_span = trace_span_at(&"aa".repeat(16), "s1", 1_000, ("http.route", "/items"));
    api_span.resource = Some(opentelemetry_proto::tonic::resource::v1::Resource {
        attributes: vec![otlp_kv("service.name", "api")],
        ..Default::default()
    });
    let mut worker_span = trace_span_at(&"bb".repeat(16), "s2", 2_000, ("http.route", "/jobs"));
    worker_span.resource = Some(opentelemetry_proto::tonic::resource::v1::Resource {
        attributes: vec![otlp_kv("service.name", "worker")],
        ..Default::default()
    });
    state.journal.trace_memtable().insert(vec![
        api_span,
        worker_span,
        trace_span_at(&"cc".repeat(16), "s3", 20_000_000_000_000, ("http.route", "/outside")),
    ]);

    let (status, body) = first_party_get(
        state.clone(),
        "/signy/api/v1/traces/attributes/http.route/values?start=0&end=10",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let values: Vec<String> = ndjson_rows(&body)
        .iter()
        .map(|row| row["value"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(values, ["/items", "/jobs"]);

    // A placed filter narrows the offer to traces it matches.
    let (status, body) = first_party_get(
        state,
        "/signy/api/v1/traces/attributes/http.route/values?start=0&end=10&attr=service.name=api",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let values: Vec<String> = ndjson_rows(&body)
        .iter()
        .map(|row| row["value"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(values, ["/items"]);
}

// --- Trace budgets and refusals, end to end ---

/// A busy instance refuses on arrival, and the refusal lifts when the memory
/// comes back.
///
/// The `429` used to be reachable only from inside a scan — a query that had
/// already read for seconds died partway through — so the test that stood here
/// filled the pool with the query's own rows. Admission is priced up front
/// now, which means a single request against an idle instance can never be
/// throttled: it either fits the whole account or is permanently too large.
/// So what makes this a `429` is a *peer* holding the account, which is the
/// only thing that ever really justified telling a client to retry.
#[tokio::test]
async fn a_trace_scan_that_finds_no_room_in_the_account_is_a_429_and_counts_exhaustion() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |_| {});
    let trace_id = "ab".repeat(16);
    state
        .journal
        .trace_memtable()
        .insert(vec![trace_span_at(&trace_id, "s0", 1_000, ("k", "v"))]);

    let peer = state
        .memory_account
        .admit(state.memory_account.budget_bytes())
        .expect("a peer takes the whole account");

    let (status, body) =
        first_party_get(state.clone(), "/signy/api/v1/traces?start=0&end=10").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{body}");
    assert!(body.contains(crate::memory_budget::EXHAUSTED_PREFIX), "{body}");
    assert_eq!(
        state.memory_account.exhausted(),
        1,
        "the refusal stays visible after the client was told the right thing"
    );

    // The peer finishes. The identical request now succeeds, which is what
    // makes `429` — rather than `400` — the honest answer above.
    drop(peer);
    let (status, body) =
        first_party_get(state.clone(), "/signy/api/v1/traces?start=0&end=10").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(state.memory_account.exhausted(), 1);
}

#[tokio::test]
async fn a_search_past_the_span_budget_is_a_413_too() {
    let data_dir = temp_dir();
    let state = trace_state(&data_dir, |config| config.max_trace_spans = 2);
    state.journal.trace_memtable().insert(vec![
        trace_span_at(&"aa".repeat(16), "s1", 1_000, ("k", "v")),
        trace_span_at(&"bb".repeat(16), "s2", 2_000, ("k", "v")),
        trace_span_at(&"cc".repeat(16), "s3", 3_000, ("k", "v")),
    ]);
    let (status, body) =
        first_party_get(state, "/signy/api/v1/traces?start=0&end=10").await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    assert!(body.contains("trace query exceeds"), "{body}");
}

#[tokio::test]
async fn a_tenant_without_a_pushed_policy_is_refused_on_every_trace_path() {
    let data_dir = temp_dir();
    let config = Config {
        data_dir: data_dir.to_path_buf(),
        ..Config::default()
    };
    let memtable = Arc::new(MemTable::new());
    let parts = Arc::new(PartRegistry::new());
    let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
        parts.operation_lock(),
    ));
    let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
    let state = crate::test_support::state_with_tenant_policy(
        config,
        memtable,
        journal,
        parts,
        trace_parts,
        None,
        Arc::new(crate::tenant_policy::TenantPolicy::enabled_for_test()),
    );

    for path in [
        "/signy/api/v1/traces?start=0&end=10".to_string(),
        format!("/signy/api/v1/traces/{}", "ab".repeat(16)),
        "/signy/api/v1/traces/attributes?start=0&end=10".to_string(),
        "/signy/api/v1/traces/attributes/k/values?start=0&end=10".to_string(),
    ] {
        let (status, body) = first_party_get(state.clone(), &path).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{path}: {body}");
    }
}

// --- The first-party metric API (M14, issue #8) ---

/// A metric state whose journal owns the series memtable the scan reads.
fn metric_state(
    data_dir: &std::path::Path,
    mutate: impl FnOnce(&mut Config),
) -> Arc<AppState> {
    let mut config = Config {
        data_dir: data_dir.to_path_buf(),
        ..Config::default()
    };
    mutate(&mut config);
    let memtable = Arc::new(MemTable::new());
    let parts = Arc::new(PartRegistry::new());
    let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
        parts.operation_lock(),
    ));
    crate::test_support::state(
        config.clone(),
        memtable.clone(),
        Arc::new(Journal::spawn(&config, memtable).unwrap()),
        parts,
        trace_parts,
        None,
    )
}

/// The scrape grid every metric test reads from: whole seconds so the
/// documented `start + k*step` alignment is checkable by eye.
const METRIC_ANCHOR_NS: i64 = 1_772_000_000_000_000_000;
const METRIC_SECOND_NS: i64 = 1_000_000_000;

fn metric_labels(name: &str, pairs: &[(&str, &str)]) -> crate::series::SeriesLabels {
    let mut all = vec![(
        crate::series::METRIC_NAME_LABEL.to_string(),
        name.to_string(),
    )];
    all.extend(
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string())),
    );
    crate::series::SeriesLabels::from_pairs(all)
}

/// Samples land through the memtable the journal owns, which is the half of
/// the read path a unit test can reach without a flush.
fn insert_metric_samples(
    state: &AppState,
    labels: &crate::series::SeriesLabels,
    samples: &[(i64, f64)],
) {
    state.journal.series_memtable().insert(
        samples
            .iter()
            .map(|(ts_ns, value)| crate::series::MetricSample {
                tenant: test_tenant(),
                labels: labels.clone(),
                ts_ns: *ts_ns,
                value: MetricValue::Scalar(*value),
                kind: crate::series::SampleKind::Gauge,
                datapoint_index: 0,
            })
            .collect(),
    );
}

/// One NDJSON series row's samples as `(nanoseconds, value)`, asserting the
/// pinned wire shape on the way: a `[string, number]` pair per sample, which
/// is what the comparison bed's parser requires of every answer.
fn metric_row_samples(row: &serde_json::Value) -> Vec<(i64, f64)> {
    row["samples"]
        .as_array()
        .unwrap_or_else(|| panic!("a series row carries a samples array: {row}"))
        .iter()
        .map(|pair| {
            let pair = pair.as_array().expect("a sample is a two-element array");
            assert_eq!(pair.len(), 2, "a sample is [timestamp, value]");
            let ts: i64 = pair[0]
                .as_str()
                .expect("a sample timestamp is a nanosecond string")
                .parse()
                .expect("a sample timestamp parses");
            (ts, pair[1].as_f64().expect("a sample value is a number"))
        })
        .collect()
}

/// Storage holds one identity per instrument; a caller still asks in
/// Prometheus terms and has to get Prometheus answers.
fn insert_histogram(
    state: &AppState,
    labels: &crate::series::SeriesLabels,
    points: &[(i64, &[u64], u64)],
) {
    state.journal.series_memtable().insert(
        points
            .iter()
            .map(|(ts_ns, cumulative, count)| crate::series::MetricSample {
                tenant: test_tenant(),
                labels: labels.clone(),
                ts_ns: *ts_ns,
                value: MetricValue::Histogram(crate::series::HistogramPoint {
                    bounds: vec![0.005, 0.01].into(),
                    cumulative: cumulative.to_vec(),
                    sum: Some(*count as f64 * 0.001),
                    count: *count,
                }),
                kind: crate::series::SampleKind::Cumulative,
                datapoint_index: 0,
            })
            .collect(),
    );
}

#[tokio::test]
async fn a_native_histogram_answers_the_bucket_series_a_selector_asks_for() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    let labels = metric_labels("http_request_duration_seconds", &[("instance", "a")]);
    insert_histogram(
        &state,
        &labels,
        &[
            (METRIC_ANCHOR_NS, &[1, 2], 3),
            (METRIC_ANCHOR_NS + METRIC_SECOND_NS, &[4, 9], 11),
        ],
    );

    let (status, body) = first_party_get(
        state.clone(),
        &format!(
            "/signy/api/v1/metrics/query?metric=http_request_duration_seconds_bucket\
&attr=le%3D0.01&start={METRIC_ANCHOR_NS}&end={}&step=1s",
            METRIC_ANCHOR_NS + METRIC_SECOND_NS
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows: Vec<serde_json::Value> = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("a row is JSON"))
        .collect();
    let series: Vec<&serde_json::Value> = rows
        .iter()
        .filter(|row| row.get("labels").is_some())
        .collect();
    assert_eq!(
        series.len(),
        1,
        "one bucket was asked for and one comes back: {body}"
    );
    assert_eq!(
        series[0]["labels"]["le"], "0.01",
        "the synthetic identity carries the boundary it answers for"
    );
    assert_eq!(series[0]["labels"]["instance"], "a");
    let samples = metric_row_samples(series[0]);
    assert_eq!(
        samples.iter().map(|(_, value)| *value).collect::<Vec<_>>(),
        vec![2.0, 9.0],
        "cumulative counts at le=0.01, read out of a stored histogram"
    );
    std::fs::remove_dir_all(&data_dir).ok();
}

#[tokio::test]
async fn a_native_histogram_answers_its_count_and_its_quantile() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    let labels = metric_labels("http_request_duration_seconds", &[("instance", "a")]);
    insert_histogram(
        &state,
        &labels,
        &[
            (METRIC_ANCHOR_NS, &[0, 0], 0),
            (METRIC_ANCHOR_NS + METRIC_SECOND_NS, &[10, 20], 20),
        ],
    );

    let (status, body) = first_party_get(
        state.clone(),
        &format!(
            "/signy/api/v1/metrics/query?metric=http_request_duration_seconds_count\
&start={METRIC_ANCHOR_NS}&end={}&step=1s",
            METRIC_ANCHOR_NS + METRIC_SECOND_NS
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows: Vec<serde_json::Value> = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("a row is JSON"))
        .collect();
    let series: Vec<&serde_json::Value> = rows
        .iter()
        .filter(|row| row.get("labels").is_some())
        .collect();
    assert_eq!(series.len(), 1, "the _count series is answerable too: {body}");
    assert_eq!(
        metric_row_samples(series[0])
            .iter()
            .map(|(_, value)| *value)
            .collect::<Vec<_>>(),
        vec![0.0, 20.0]
    );

    // And the quantile route, which folds the bucket family it never stored.
    let (status, body) = first_party_get(
        state,
        &format!(
            "/signy/api/v1/metrics/quantile?metric=http_request_duration_seconds\
&q=0.5&start={}&end={}&step=1s&range=1s",
            METRIC_ANCHOR_NS + METRIC_SECOND_NS,
            METRIC_ANCHOR_NS + METRIC_SECOND_NS
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains("samples"),
        "a quantile comes back from a stored histogram: {body}"
    );
    std::fs::remove_dir_all(&data_dir).ok();
}

#[tokio::test]
async fn a_metric_route_refuses_an_unknown_parameter_with_its_own_list() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    let (status, body) = first_party_get(
        state,
        "/signy/api/v1/metrics/query?metric=up&start=0&step=30s&direction=backward",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(body.contains("unknown parameter 'direction'"), "{body}");
    assert!(body.contains("lookback"), "{body}");
    assert!(body.contains("docs/QUERY_API.md"), "{body}");
}

#[tokio::test]
async fn a_comparison_operator_on_a_metric_label_is_refused_with_the_fix() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    let (status, body) = first_party_get(
        state,
        "/signy/api/v1/metrics/query?metric=up&start=0&step=30s&attr=shard%3E=3",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(body.contains("uses a comparison"), "{body}");
    assert!(body.contains("=, !=, =~, !~"), "{body}");
}

#[tokio::test]
async fn the_cross_field_metric_rules_each_teach_their_pair() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    for (query, expected) in [
        (
            "metric=c&start=0&step=30s&func=rate",
            "func was given without range",
        ),
        (
            "metric=c&start=0&step=30s&range=60s",
            "range was given without func",
        ),
        (
            "metric=c&start=0&step=30s&by=service",
            "by was given without agg",
        ),
        ("metric=c&step=30s", "start is required"),
        ("metric=c&start=0", "step is required"),
        ("start=0&step=30s", "metric is required"),
    ] {
        let (status, body) =
            first_party_get(state.clone(), &format!("/signy/api/v1/metrics/query?{query}"))
                .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}: {body}");
        assert!(body.contains(expected), "{query}: {body}");
    }
}

#[tokio::test]
async fn a_gauge_range_query_answers_the_documented_ndjson_on_the_step_grid() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    let labels = metric_labels("queue_depth", &[("instance", "a"), ("service", "api")]);
    insert_metric_samples(
        &state,
        &labels,
        &[
            (METRIC_ANCHOR_NS, 1.0),
            (METRIC_ANCHOR_NS + 30 * METRIC_SECOND_NS, 2.0),
            (METRIC_ANCHOR_NS + 60 * METRIC_SECOND_NS, 3.0),
        ],
    );
    let (status, body) = first_party_get(
        state,
        &format!(
            "/signy/api/v1/metrics/query?metric=queue_depth&start={}&end={}&step=30s",
            METRIC_ANCHOR_NS,
            METRIC_ANCHOR_NS + 60 * METRIC_SECOND_NS
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1, "one line per output series");
    assert_eq!(rows[0]["labels"]["instance"], "a");
    assert_eq!(rows[0]["labels"]["service"], "api");
    assert!(
        rows[0]["labels"].get(crate::series::METRIC_NAME_LABEL).is_none(),
        "the query named the metric, so the identity does not repeat it: {body}"
    );
    assert_eq!(
        metric_row_samples(&rows[0]),
        vec![
            (METRIC_ANCHOR_NS, 1.0),
            (METRIC_ANCHOR_NS + 30 * METRIC_SECOND_NS, 2.0),
            (METRIC_ANCHOR_NS + 60 * METRIC_SECOND_NS, 3.0),
        ]
    );
}

#[tokio::test]
async fn lookback_carries_a_value_forward_and_then_stops() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    let labels = metric_labels("queue_depth", &[("instance", "a")]);
    // One sample at the anchor and nothing after: a 45s lookback answers the
    // step 30s later and leaves the step at 60s without a point.
    insert_metric_samples(&state, &labels, &[(METRIC_ANCHOR_NS, 7.0)]);
    let (status, body) = first_party_get(
        state,
        &format!(
            "/signy/api/v1/metrics/query?metric=queue_depth&start={}&end={}&step=30s&lookback=45s",
            METRIC_ANCHOR_NS,
            METRIC_ANCHOR_NS + 60 * METRIC_SECOND_NS
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(
        metric_row_samples(&rows[0]),
        vec![
            (METRIC_ANCHOR_NS, 7.0),
            (METRIC_ANCHOR_NS + 30 * METRIC_SECOND_NS, 7.0),
        ],
        "a step past the lookback horizon is omitted, not zero: {body}"
    );
}

#[tokio::test]
async fn a_counter_reset_is_absorbed_by_the_positive_delta_sum() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    let labels = metric_labels("http_requests_total", &[("instance", "a")]);
    // 5 → 8 → 2 → 4: the restart contributes its post-reset value, so the
    // window's increase is 3 + 2 + 2 = 7 — the VictoriaMetrics definition the
    // M14 decision record chose, with no extrapolation.
    insert_metric_samples(
        &state,
        &labels,
        &[
            (METRIC_ANCHOR_NS, 5.0),
            (METRIC_ANCHOR_NS + 10 * METRIC_SECOND_NS, 8.0),
            (METRIC_ANCHOR_NS + 20 * METRIC_SECOND_NS, 2.0),
            (METRIC_ANCHOR_NS + 30 * METRIC_SECOND_NS, 4.0),
        ],
    );
    let at = METRIC_ANCHOR_NS + 30 * METRIC_SECOND_NS;
    let (status, body) = first_party_get(
        state.clone(),
        &format!(
            "/signy/api/v1/metrics/query?metric=http_requests_total&start={at}&end={at}\
&step=30s&func=increase&range=60s"
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(metric_row_samples(&rows[0]), vec![(at, 7.0)]);

    // The same window as a rate is the increase over its seconds.
    let (status, body) = first_party_get(
        state,
        &format!(
            "/signy/api/v1/metrics/query?metric=http_requests_total&start={at}&end={at}\
&step=30s&func=rate&range=60s"
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    let samples = metric_row_samples(&rows[0]);
    assert!(
        (samples[0].1 - 7.0 / 60.0).abs() < 1e-12,
        "rate is increase over the window's seconds: {body}"
    );
}

#[tokio::test]
async fn an_aggregation_groups_by_the_named_keys_and_count_counts_series() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    for (service, instance, value) in [
        ("api", "a", 1.0),
        ("api", "b", 2.0),
        ("worker", "a", 10.0),
    ] {
        insert_metric_samples(
            &state,
            &metric_labels("queue_depth", &[("service", service), ("instance", instance)]),
            &[(METRIC_ANCHOR_NS, value)],
        );
    }
    let window = format!(
        "start={}&end={}&step=30s",
        METRIC_ANCHOR_NS, METRIC_ANCHOR_NS
    );
    let (status, body) = first_party_get(
        state.clone(),
        &format!("/signy/api/v1/metrics/query?metric=queue_depth&{window}&agg=sum&by=service"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 2, "one group per service: {body}");
    let by_service: std::collections::BTreeMap<String, f64> = rows
        .iter()
        .map(|row| {
            (
                row["labels"]["service"].as_str().unwrap().to_string(),
                metric_row_samples(row)[0].1,
            )
        })
        .collect();
    assert_eq!(by_service["api"], 3.0);
    assert_eq!(by_service["worker"], 10.0);

    // `agg` without `by` folds everything into the one empty-labeled group.
    let (status, body) = first_party_get(
        state,
        &format!("/signy/api/v1/metrics/query?metric=queue_depth&{window}&agg=count"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["labels"].as_object().unwrap().len(), 0);
    assert_eq!(metric_row_samples(&rows[0])[0].1, 3.0, "three series: {body}");
}

#[tokio::test]
async fn an_instant_query_answers_one_row_per_series_at_its_instant() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    let labels = metric_labels("queue_depth", &[("instance", "a")]);
    insert_metric_samples(
        &state,
        &labels,
        &[
            (METRIC_ANCHOR_NS, 1.0),
            (METRIC_ANCHOR_NS + 30 * METRIC_SECOND_NS, 9.0),
        ],
    );
    let at = METRIC_ANCHOR_NS + 30 * METRIC_SECOND_NS;
    let (status, body) = first_party_get(
        state,
        &format!("/signy/api/v1/metrics/instant?metric=queue_depth&at={at}&agg=max"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0]["timestamp"].as_str().unwrap(),
        at.to_string(),
        "the alert shape carries its instant as a nanosecond string: {body}"
    );
    assert_eq!(rows[0]["value"].as_f64().unwrap(), 9.0);
}

#[tokio::test]
async fn a_quantile_interpolates_within_the_bracketing_bucket() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    // Cumulative bucket counts over the window: 0 → (2, 6, 10). The p50 of
    // ten observations ranks at 5, inside the (0.1, 0.5] bucket whose four
    // observations span it — 0.1 + (5-2)/4 * 0.4 = 0.4.
    for (le, counts) in [
        ("0.1", [0.0, 2.0]),
        ("0.5", [0.0, 6.0]),
        ("+Inf", [0.0, 10.0]),
    ] {
        insert_metric_samples(
            &state,
            &metric_labels(
                "http_request_duration_seconds_bucket",
                &[("instance", "a"), ("le", le)],
            ),
            &[
                (METRIC_ANCHOR_NS, counts[0]),
                (METRIC_ANCHOR_NS + 30 * METRIC_SECOND_NS, counts[1]),
            ],
        );
    }
    let at = METRIC_ANCHOR_NS + 30 * METRIC_SECOND_NS;
    let (status, body) = first_party_get(
        state,
        &format!(
            "/signy/api/v1/metrics/quantile?metric=http_request_duration_seconds\
&q=0.5&start={at}&end={at}&step=30s&range=60s"
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1, "{body}");
    assert!(
        rows[0]["labels"].get("le").is_none(),
        "the bucket label is consumed by the interpolation: {body}"
    );
    let value = metric_row_samples(&rows[0])[0].1;
    assert!((value - 0.4).abs() < 1e-9, "p50 interpolates to 0.4: {value}");
}

#[tokio::test]
async fn a_summary_backed_name_refuses_the_quantile_route_with_the_alternative() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    insert_metric_samples(
        &state,
        &metric_labels("gc_pause_seconds", &[("quantile", "0.99")]),
        &[(METRIC_ANCHOR_NS, 0.25)],
    );
    let (status, body) = first_party_get(
        state,
        &format!(
            "/signy/api/v1/metrics/quantile?metric=gc_pause_seconds&q=0.99\
&start={METRIC_ANCHOR_NS}&end={METRIC_ANCHOR_NS}&step=30s&range=60s"
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(body.contains("summary-backed"), "{body}");
    assert!(body.contains("/metrics/query"), "{body}");
}

#[tokio::test]
async fn a_selection_past_the_series_cap_is_a_413_before_any_decode() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |config| config.max_metric_series_per_query = 2);
    for instance in ["a", "b", "c"] {
        insert_metric_samples(
            &state,
            &metric_labels("queue_depth", &[("instance", instance)]),
            &[(METRIC_ANCHOR_NS, 1.0)],
        );
    }
    let (status, body) = first_party_get(
        state,
        &format!(
            "/signy/api/v1/metrics/query?metric=queue_depth&start={METRIC_ANCHOR_NS}\
&end={METRIC_ANCHOR_NS}&step=30s"
        ),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    assert!(body.contains("metric selection exceeds"), "{body}");
    assert!(body.contains("matched 3"), "{body}");
    assert!(
        body.contains("SIGNY_MAX_METRIC_SERIES_PER_QUERY"),
        "{body}"
    );
}

#[tokio::test]
async fn a_selection_past_the_point_cap_names_the_steps_that_made_it() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |config| config.max_metric_points_per_query = 2);
    insert_metric_samples(
        &state,
        &metric_labels("queue_depth", &[("instance", "a")]),
        &[(METRIC_ANCHOR_NS, 1.0)],
    );
    let (status, body) = first_party_get(
        state,
        &format!(
            "/signy/api/v1/metrics/query?metric=queue_depth&start={METRIC_ANCHOR_NS}\
&end={}&step=30s",
            METRIC_ANCHOR_NS + 90 * METRIC_SECOND_NS
        ),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    assert!(body.contains("output points"), "{body}");
    assert!(body.contains("coarsen step"), "{body}");
}

/// The metric surface prices itself after selection rather than before the
/// scan permit, because its cost is a function of what the selector matched.
/// The refusal is the same one, from the same account, and it still reaches
/// the client before a single chunk is decoded.
#[tokio::test]
async fn a_metric_scan_that_finds_no_room_in_the_account_is_a_429_and_counts_exhaustion() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |config| {
        config.memory_account =
            Arc::new(crate::memory_budget::MemoryAccount::new(32 * 1024 * 1024));
    });
    insert_metric_samples(
        &state,
        &metric_labels("queue_depth", &[("instance", "a")]),
        &[(METRIC_ANCHOR_NS, 1.0)],
    );
    // 1 000 001 steps at 16 estimated bytes each is 16 MB, which fits a 32 MiB
    // account alone and does not fit beside a peer holding 24 MiB of it.
    let peer = state
        .memory_account
        .admit(24 * 1024 * 1024)
        .expect("a peer takes most of the account");

    let url = format!(
        "/signy/api/v1/metrics/query?metric=queue_depth&start={METRIC_ANCHOR_NS}\
&end={}&step=1s",
        METRIC_ANCHOR_NS + 1_000_000 * METRIC_SECOND_NS
    );
    let (status, body) = first_party_get(state.clone(), &url).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{body}");
    assert!(body.contains(crate::memory_budget::EXHAUSTED_PREFIX), "{body}");
    assert_eq!(
        state.memory_account.exhausted(),
        1,
        "the refusal stays visible after the client was told the right thing"
    );

    drop(peer);
    let (status, body) = first_party_get(state.clone(), &url).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        state.memory_account.in_use_bytes(),
        0,
        "the charge left with the response"
    );
}

/// The other half of the refusal, and the opposite instruction to the client.
///
/// A request larger than the whole account is not a busy instance — nothing is
/// holding anything, and waiting changes nothing — so a `429` would loop a
/// well-behaved client forever on something that can never succeed. It is a
/// `400` naming the budget, and it does not move the exhaustion counter, which
/// exists to say the instance ran out of room for work it was *willing* to do.
///
/// The metric surface is where this is reachable. A log or trace query is
/// priced no higher than `max_query_memory_bytes`, which startup keeps at or
/// under the account, so one of those always fits an idle instance. A metric
/// query is priced from `series × steps`, which no per-query byte cap bounds —
/// only the point cap, which is deliberately far larger.
#[tokio::test]
async fn a_metric_query_larger_than_the_whole_account_is_a_400_and_counts_nothing() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |config| {
        config.memory_account = Arc::new(crate::memory_budget::MemoryAccount::new(4 * 1024 * 1024));
        config.max_query_memory_bytes = 2 * 1024 * 1024;
        config.merge_max_memory_bytes = 2 * 1024 * 1024;
        config.merge_max_input_bytes = 1024 * 1024;
        config.flush_chunk_bytes = 1024 * 1024;
        config
            .validate()
            .expect("the shape under test is one an operator could actually configure");
    });
    insert_metric_samples(
        &state,
        &metric_labels("queue_depth", &[("instance", "a")]),
        &[(METRIC_ANCHOR_NS, 1.0)],
    );

    // 1 000 001 steps at 16 bytes each is 16 MB against a 4 MiB account, and
    // stays under the point cap so the refusal is the account's.
    let (status, body) = first_party_get(
        state.clone(),
        &format!(
            "/signy/api/v1/metrics/query?metric=queue_depth&start={METRIC_ANCHOR_NS}\
&end={}&step=1s",
            METRIC_ANCHOR_NS + 1_000_000 * METRIC_SECOND_NS
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(
        body.contains(crate::memory_budget::OVER_BUDGET_PREFIX),
        "{body}"
    );
    assert_eq!(
        state.memory_account.exhausted(),
        0,
        "a request too large for any instance is not evidence this one is full"
    );
    assert_eq!(
        state.memory_account.in_use_bytes(),
        0,
        "a refusal charges nothing"
    );
}

#[tokio::test]
async fn metric_discovery_answers_exactly_and_within_its_window() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    insert_metric_samples(
        &state,
        &metric_labels("queue_depth", &[("service", "api")]),
        &[(METRIC_ANCHOR_NS, 1.0)],
    );
    insert_metric_samples(
        &state,
        &metric_labels("http_requests_total", &[("service", "worker")]),
        &[(METRIC_ANCHOR_NS + 3_600 * METRIC_SECOND_NS, 5.0)],
    );
    let early = format!(
        "start={}&end={}",
        METRIC_ANCHOR_NS,
        METRIC_ANCHOR_NS + 60 * METRIC_SECOND_NS
    );

    let (status, body) =
        first_party_get(state.clone(), &format!("/signy/api/v1/metrics/names?{early}")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let names: Vec<String> = ndjson_rows(&body)
        .iter()
        .map(|row| row["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        names,
        vec!["queue_depth".to_string()],
        "the later series is outside the window: {body}"
    );

    let (status, body) =
        first_party_get(state.clone(), &format!("/signy/api/v1/metrics/labels?{early}")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let keys: Vec<String> = ndjson_rows(&body)
        .iter()
        .map(|row| row["key"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(keys, vec!["service".to_string()], "{body}");

    let (status, body) = first_party_get(
        state.clone(),
        &format!("/signy/api/v1/metrics/labels/service/values?{early}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let values: Vec<String> = ndjson_rows(&body)
        .iter()
        .map(|row| row["value"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(values, vec!["api".to_string()], "{body}");

    let (status, body) = first_party_get(
        state,
        &format!("/signy/api/v1/metrics/series?metric=queue_depth&{early}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = ndjson_rows(&body);
    assert_eq!(rows.len(), 1, "{body}");
    assert_eq!(rows[0]["labels"]["service"], "api");
    assert_eq!(
        rows[0]["labels"][crate::series::METRIC_NAME_LABEL], "queue_depth",
        "the series route enumerates identities, so it keeps the name: {body}"
    );
}

#[tokio::test]
async fn a_tenant_without_a_pushed_policy_is_refused_on_every_metric_path() {
    let data_dir = temp_dir();
    let config = Config {
        data_dir: data_dir.to_path_buf(),
        ..Config::default()
    };
    let memtable = Arc::new(MemTable::new());
    let parts = Arc::new(PartRegistry::new());
    let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
        parts.operation_lock(),
    ));
    let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
    let state = crate::test_support::state_with_tenant_policy(
        config,
        memtable,
        journal,
        parts,
        trace_parts,
        None,
        Arc::new(crate::tenant_policy::TenantPolicy::enabled_for_test()),
    );

    for path in [
        "/signy/api/v1/metrics/query?metric=c&start=0&end=1&step=30s",
        "/signy/api/v1/metrics/instant?metric=c&at=0",
        "/signy/api/v1/metrics/quantile?metric=c&q=0.9&start=0&end=1&step=30s&range=60s",
        "/signy/api/v1/metrics/names?start=0&end=1",
        "/signy/api/v1/metrics/labels?start=0&end=1",
        "/signy/api/v1/metrics/labels/k/values?start=0&end=1",
        "/signy/api/v1/metrics/series?start=0&end=1",
    ] {
        let (status, body) = first_party_get(state.clone(), path).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{path}: {body}");
    }
}

#[tokio::test]
async fn the_api_fallback_lists_the_metric_routes() {
    let data_dir = temp_dir();
    let state = metric_state(&data_dir, |_| {});
    let (status, body) = first_party_get(state, "/signy/api/v1/nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("/signy/api/v1/metrics/query"), "{body}");
    assert!(
        body.contains("/signy/api/v1/metrics/labels/{key}/values"),
        "{body}"
    );
}

    /// A metric part's body is evictable — `SeriesPartReader::open` says so,
    /// and `evict_metric_cache` acts on it — so any read that reaches the body
    /// has to pin what it is about to read, the way the metric scan path does.
    ///
    /// Discovery reaches the body: a stored histogram is enumerated under the
    /// names it answers as, which means decoding its chunk. Without a pin the
    /// route opens a path the cache no longer holds and the caller is handed a
    /// bare `No such file or directory` as a 500 — observed once in a
    /// three-hour production-shaped run, on `/metrics/series`.
    #[tokio::test]
    async fn metric_discovery_restores_a_histogram_body_the_cache_evicted() {
        let data_dir = temp_dir();
        let config = Config {
            data_dir: data_dir.clone(),
            ..Config::default()
        };
        let parts_root = data_dir.join("parts");
        let storage = Arc::new(crate::object_storage::ObjectStorage::in_memory());
        let remote = Arc::new(crate::object_storage::RemoteCache::new(
            storage.clone(),
            parts_root.clone(),
        ));
        let metrics_root = remote.metric_parts_root();

        // One part, one series, and that series a histogram: discovery cannot
        // answer about it without decoding the chunk.
        let source = crate::series::SeriesMemTable::new();
        let labels = metric_labels("request_duration_seconds", &[("service", "api")]);
        source.insert(vec![MetricSample {
            tenant: test_tenant(),
            labels: labels.clone(),
            ts_ns: METRIC_ANCHOR_NS,
            value: MetricValue::Histogram(crate::series::HistogramPoint {
                bounds: vec![1.0, 5.0].into(),
                cumulative: vec![1, 2],
                sum: Some(3.0),
                count: 3,
            }),
            kind: SampleKind::Cumulative,
            datapoint_index: 0,
        }]);
        let snapshot = source.begin_flush();
        let parts = crate::series_part::flush_series_snapshot(&snapshot, &metrics_root).unwrap();
        source.commit_flush();
        assert_eq!(parts.len(), 1);
        storage.publish_metric_parts(&parts, &[]).await.unwrap();

        let memtable = Arc::new(MemTable::new());
        let registry = Arc::new(PartRegistry::new());
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            registry.operation_lock(),
        ));
        let state = crate::test_support::state(
            config.clone(),
            memtable.clone(),
            Arc::new(Journal::spawn(&config, memtable).unwrap()),
            registry,
            trace_parts,
            Some(remote),
        );
        state.series_parts.register_opened(
            crate::series_registry::SeriesRegistry::open_parts(parts.clone()).unwrap(),
        );

        // Eviction happens under a live reader, which is the production order:
        // the registry opened the part while its body was there, and the
        // disk-cache budget took the body afterwards.
        let eligible: Vec<_> = parts.iter().map(|part| part.dir.clone()).collect();
        storage
            .evict_metric_cache(&metrics_root, 0, &eligible)
            .unwrap();
        assert!(
            !parts[0].data_path().exists(),
            "the fixture has to reach the query with the body evicted"
        );

        let window = format!(
            "start={}&end={}",
            METRIC_ANCHOR_NS,
            METRIC_ANCHOR_NS + 60 * METRIC_SECOND_NS
        );
        let (status, body) = first_party_get(
            state,
            // A stored histogram is enumerated under the names it answers as,
            // so this asks for one of those rather than the stored name.
            &format!("/signy/api/v1/metrics/series?metric=request_duration_seconds_count&{window}"),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "discovery must restore the body it needs rather than fail on it: {body}"
        );
        assert!(
            !ndjson_rows(&body).is_empty(),
            "the restored histogram has to be enumerated: {body}"
        );
    }
