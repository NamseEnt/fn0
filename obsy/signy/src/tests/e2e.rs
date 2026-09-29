    use super::*;
    use crate::tenant::test_tenant;
    use crate::journal::Journal;
    use crate::memtable::MemTable;
    use crate::part;
    use crate::part_registry::PartRegistry;
    use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
    use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
    use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use prost014::Message;

    const CRASH_MODE_ENV: &str = "SIGNY_CRASH_TEST_MODE";
    const CRASH_DIR_ENV: &str = "SIGNY_CRASH_TEST_DIR";

    fn tmp_data_dir(label: &str) -> std::path::PathBuf {
        crate::test_support::temp_dir(&format!("e2e-{label}"))
    }

    fn string_attribute(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.to_string())),
            }),
            ..Default::default()
        }
    }

    fn otlp_body(app: &str, lines: &[(&str, i64)]) -> Vec<u8> {
        otlp_body_for(crate::tenant::test_tenant().as_str(), app, lines)
    }

    /// An export naming `tenant` in its resource, which is the only thing that
    /// files it under one.
    fn otlp_body_for(tenant: &str, app: &str, lines: &[(&str, i64)]) -> Vec<u8> {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![
                        string_attribute(crate::otlp_tenant::TENANT_ATTRIBUTE, tenant),
                        string_attribute("service.name", app),
                    ],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    log_records: lines
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
            }],
        }
        .encode_to_vec()
    }

    fn build_push_req() -> Vec<u8> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        otlp_body(
            "test-app",
            &[
                ("hello world from signy", now - 60),
                ("error connecting to database", now - 30),
                ("third line all good", now),
            ],
        )
    }

    async fn ingest_once(journal: &Journal, raw: &[u8]) {
        let request = ExportLogsServiceRequest::decode(raw).unwrap();
        let streams = crate::otlp_log::normalize_request(request).unwrap();
        journal
            .append_otlp_logs(test_tenant(), raw.to_vec(), streams)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn e2e_recovery_persists_flushed_data() {
        let dir = tmp_data_dir("recovery");
        let config = Config {
            data_dir: dir.clone(),
            ..Config::default()
        };

        // Phase 1: ingest + flush
        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
        let raw = build_push_req();
        ingest_once(&journal, &raw).await;

        let ckpt = journal.checkpoint().await.unwrap();
        assert_eq!(ckpt.snapshot.len(), 1);
        let rows = part::rows_from_snapshot(&ckpt.snapshot);
        assert_eq!(rows.len(), 3);
        let parts_root = config.data_dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let new_parts = part::flush_rows(rows, &parts_root, config.row_group_size).unwrap();
        assert_eq!(new_parts.len(), 1);
        journal.set_checkpoint(ckpt.offset).unwrap();

        // Simulate shutdown: drop journal & memtable
        drop(journal);
        drop(memtable);

        // Phase 2: restart — empty memtable, replay journal, load parts
        let memtable2 = MemTable::new();
        let wal = config.data_dir.join("journal.wal");
        let ckpt_p = config.data_dir.join("journal.ckpt");
        let (start, end) = journal::replay(&wal, &ckpt_p, &memtable2).unwrap();
        assert_eq!(start, ckpt.offset);
        assert_eq!(end, ckpt.offset);
        assert!(memtable2.is_empty(), "no in-flight data after full flush");

        let registry = PartRegistry::load_from_disk(&parts_root).unwrap();
        assert_eq!(registry.part_count(), 1);

        let results = registry
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .expect("part query");
        let total: usize = results.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 3);

        // attribute predicate lookup: the resource attribute rode into each
        // entry's metadata, so an exact-field predicate selects on it.
        let hit = crate::part::ExactFieldPredicate::new("service_name", "test-app");
        let results = registry
            .query_with_exact_field_pruning(
                &test_tenant(),
                crate::part::ExactFieldPruning::new(&[], std::slice::from_ref(&hit)),
                crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                100,
                true,
            )
            .expect("part query");
        let total: usize = results.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 3);

        let miss = crate::part::ExactFieldPredicate::new("service_name", "missing");
        let results = registry
            .query_with_exact_field_pruning(
                &test_tenant(),
                crate::part::ExactFieldPruning::new(&[], std::slice::from_ref(&miss)),
                crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                100,
                true,
            )
            .expect("part query");
        let total: usize = results.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn e2e_inflight_data_restored_from_journal() {
        let dir = tmp_data_dir("inflight");
        let config = Config {
            data_dir: dir.clone(),
            ..Config::default()
        };

        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
        let raw = build_push_req();
        ingest_once(&journal, &raw).await;
        // do NOT checkpoint/flush — crash before flush
        drop(journal);
        drop(memtable);

        let memtable2 = MemTable::new();
        let wal = config.data_dir.join("journal.wal");
        let ckpt_p = config.data_dir.join("journal.ckpt");
        let (start, end) = journal::replay(&wal, &ckpt_p, &memtable2).unwrap();
        assert_eq!(start, 0);
        assert!(end > 0);

        let registry = PartRegistry::load_from_disk(&config.data_dir.join("parts")).unwrap();
        assert_eq!(registry.part_count(), 0);

        let results = memtable2.query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true);
        let total: usize = results.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 3);
    }

    /// The production `recover` restores an in-flight OTLP export — including
    /// one whose record has no resource at all, the barest body the protocol
    /// allows — alongside a full-shaped one.
    #[tokio::test]
    async fn e2e_inflight_otlp_logs_restored_through_recover() {
        let dir = tmp_data_dir("inflight_otlp");
        let config = Config {
            data_dir: dir.clone(),
            ..Config::default()
        };

        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
        ingest_once(&journal, &build_push_req()).await;

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        time_unix_nano: 4_000_000_000,
                        body: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("via otlp".to_string())),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let encoded = prost014::Message::encode_to_vec(&request);
        let streams = crate::otlp_log::normalize_request(request).unwrap();
        journal
            .append_otlp_logs(test_tenant(), encoded, streams)
            .await
            .unwrap();
        // No checkpoint, no flush — crash with both kinds in flight.
        drop(journal);
        drop(memtable);

        let recovered = MemTable::new();
        recover(&config, &recovered).unwrap();
        let results = recovered.query(&test_tenant(), &[],
            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
            100,
            true,
        );
        let mut lines: Vec<&str> = results
            .iter()
            .flat_map(|stream| stream.entries.iter().map(|entry| entry.line.as_str()))
            .collect();
        lines.sort_unstable();
        assert!(lines.contains(&"via otlp"), "OTLP record must replay");
        assert_eq!(lines.len(), 4, "three push lines plus the OTLP one");
    }

    #[tokio::test]
    async fn e2e_recovery_truncates_crc_corrupt_wal_tail() {
        let dir = tmp_data_dir("crc_tail");
        let config = Config {
            data_dir: dir.clone(),
            ..Config::default()
        };
        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable).unwrap());
        let raw = build_push_req();
        ingest_once(&journal, &raw).await;
        drop(journal);

        let wal_path = config.data_dir.join("journal.wal");
        let mut wal = std::fs::read(&wal_path).unwrap();
        let valid_len = wal.len();
        wal.extend_from_slice(&(raw.len() as u32).to_le_bytes());
        wal.extend_from_slice(&(crc32fast::hash(&raw) ^ 1).to_le_bytes());
        wal.extend_from_slice(&raw);
        std::fs::write(&wal_path, wal).unwrap();

        let recovered = MemTable::new();
        recover(&config, &recovered).unwrap();

        assert_eq!(
            std::fs::metadata(&wal_path).unwrap().len(),
            valid_len as u64
        );
        let results = recovered.query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true);
        assert_eq!(
            results
                .iter()
                .map(|stream| stream.entries.len())
                .sum::<usize>(),
            3
        );
    }

    #[tokio::test]
    async fn e2e_partial_flush_then_restart() {
        // Some data is flushed and some is in flight; recover it from both locations on restart.
        let dir = tmp_data_dir("partial");
        let config = Config {
            data_dir: dir.clone(),
            ..Config::default()
        };

        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());

        // first batch — flush
        let raw1 = build_push_req();
        ingest_once(&journal, &raw1).await;
        let ckpt1 = journal.checkpoint().await.unwrap();
        let rows1 = part::rows_from_snapshot(&ckpt1.snapshot);
        let parts_root = config.data_dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        part::flush_rows(rows1, &parts_root, config.row_group_size).unwrap();
        journal.set_checkpoint(ckpt1.offset).unwrap();

        // second batch — in-flight (no flush)
        let raw2 = build_push_req();
        ingest_once(&journal, &raw2).await;

        drop(journal);
        drop(memtable);

        let memtable2 = MemTable::new();
        let wal = config.data_dir.join("journal.wal");
        let ckpt_p = config.data_dir.join("journal.ckpt");
        journal::replay(&wal, &ckpt_p, &memtable2).unwrap();

        let registry = PartRegistry::load_from_disk(&parts_root).unwrap();
        assert_eq!(registry.part_count(), 1);

        // disk: 3, memtable: 3
        let disk_results = registry
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .expect("part query");
        let disk_total: usize = disk_results.iter().map(|s| s.entries.len()).sum();
        assert_eq!(disk_total, 3);

        let mem_results = memtable2.query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true);
        let mem_total: usize = mem_results.iter().map(|s| s.entries.len()).sum();
        assert_eq!(mem_total, 3);
    }

    #[tokio::test]
    async fn e2e_bloom_pruning_on_disk_part() {
        let dir = tmp_data_dir("bloom_prune");
        let config = Config {
            data_dir: dir.clone(),
            ..Config::default()
        };

        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
        let raw = build_push_req();
        ingest_once(&journal, &raw).await;

        let ckpt = journal.checkpoint().await.unwrap();
        let rows = part::rows_from_snapshot(&ckpt.snapshot);
        let parts_root = config.data_dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        part::flush_rows(rows, &parts_root, config.row_group_size).unwrap();
        journal.set_checkpoint(ckpt.offset).unwrap();
        drop(journal);
        drop(memtable);

        let registry = PartRegistry::load_from_disk(&parts_root).unwrap();
        assert_eq!(registry.part_count(), 1);

        // An existing substring.
        let f = crate::logql::LineFilter::Contains("database".to_string());
        let r = registry
            .query(&test_tenant(), &[f], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .expect("part query");
        let total: usize = r.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 1);

        // A nonexistent substring — bloom pruning.
        let f = crate::logql::LineFilter::Contains("zzzzzz-no-such-substr".to_string());
        let r = registry
            .query(&test_tenant(), &[f], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .expect("part query");
        let total: usize = r.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn e2e_double_restart_without_flush_no_loss() {
        // #1 regression: in-flight data must survive two "restart -> restart" cycles without flushing.
        let dir = tmp_data_dir("double_restart");
        let config = Config {
            data_dir: dir.clone(),
            ..Config::default()
        };

        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
        let raw = build_push_req();
        ingest_once(&journal, &raw).await;
        // Shut down without flushing.
        drop(journal);
        drop(memtable);

        // First restart.
        let memtable1 = MemTable::new();
        recover(&config, &memtable1).expect("recover 1");
        let r1 = memtable1.query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true);
        let t1: usize = r1.iter().map(|s| s.entries.len()).sum();
        assert_eq!(t1, 3, "first restart should restore in-flight data");
        drop(memtable1);

        // Second restart — the same data must be recovered because checkpoint did not advance.
        let memtable2 = MemTable::new();
        recover(&config, &memtable2).expect("recover 2");
        let r2 = memtable2.query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true);
        let t2: usize = r2.iter().map(|s| s.entries.len()).sum();
        assert_eq!(t2, 3, "second restart must NOT lose in-flight data");
    }

    fn run_crash_helper(mode: &str) -> std::path::PathBuf {
        let dir = tmp_data_dir(mode);
        let status = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", "tests::crash_recovery_helper", "--nocapture"])
            .env(CRASH_MODE_ENV, mode)
            .env(CRASH_DIR_ENV, &dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("spawn crash helper");
        assert!(!status.success(), "crash helper must terminate abruptly");
        dir
    }

    #[tokio::test]
    async fn crash_recovery_helper() {
        let Ok(mode) = std::env::var(CRASH_MODE_ENV) else {
            return;
        };
        let dir = std::path::PathBuf::from(
            std::env::var_os(CRASH_DIR_ENV).expect("crash test data directory"),
        );
        let config = Config {
            data_dir: dir.clone(),
            ..Config::default()
        };

        match mode.as_str() {
            "wal_after_ack" => {
                let memtable = Arc::new(MemTable::new());
                let journal = Journal::spawn(&config, memtable).unwrap();
                let raw = build_push_req();
                ingest_once(&journal, &raw).await;
            }
            "flush_before_checkpoint" => {
                let memtable = Arc::new(MemTable::new());
                let journal = Journal::spawn(&config, memtable).unwrap();
                let raw = build_push_req();
                ingest_once(&journal, &raw).await;
                let checkpoint = journal.checkpoint().await.unwrap();
                let parts_root = dir.join("parts");
                std::fs::create_dir_all(&parts_root).unwrap();
                part::flush_rows(
                    part::rows_from_snapshot(&checkpoint.snapshot),
                    &parts_root,
                    config.row_group_size,
                )
                .unwrap();
                // Deliberately do not advance the checkpoint. Recovery must
                // preserve both copies according to the documented
                // at-least-once flush-boundary semantics.
            }
            "flush_then_wal_compaction" => {
                // The local-compaction path end to end: flush, compact the
                // checkpoint (truncating the dead prefix), ingest more, then
                // crash. Recovery must replay only the retained suffix — the
                // flushed rows live in parts and nowhere else now.
                let memtable = Arc::new(MemTable::new());
                let journal = Journal::spawn(&config, memtable).unwrap();
                let raw = build_push_req();
                ingest_once(&journal, &raw).await;
                let checkpoint = journal.checkpoint().await.unwrap();
                let parts_root = dir.join("parts");
                std::fs::create_dir_all(&parts_root).unwrap();
                part::flush_snapshot_chunked(
                    &checkpoint.snapshot,
                    &parts_root,
                    config.row_group_size,
                    u64::MAX,
                )
                .unwrap();
                journal.compact_checkpoint(checkpoint.offset).await.unwrap();
                ingest_once(&journal, &raw).await;
            }
            "chunked_flush_before_checkpoint" => {
                // The same window as flush_before_checkpoint, but through the
                // chunked path with a one-byte budget so *several* parts are
                // committed before the checkpoint that never comes. The replay
                // semantics must not depend on how many parts a flush left.
                let memtable = Arc::new(MemTable::new());
                let journal = Journal::spawn(&config, memtable).unwrap();
                let raw = build_push_req();
                ingest_once(&journal, &raw).await;
                let checkpoint = journal.checkpoint().await.unwrap();
                let parts_root = dir.join("parts");
                std::fs::create_dir_all(&parts_root).unwrap();
                let parts = part::flush_snapshot_chunked(
                    &checkpoint.snapshot,
                    &parts_root,
                    config.row_group_size,
                    1,
                )
                .unwrap();
                assert!(parts.len() > 1, "the crash must leave several parts");
            }
            "merge_before_cleanup" => {
                let parts_root = dir.join("parts");
                std::fs::create_dir_all(&parts_root).unwrap();
                let _labels: crate::memtable::Labels =
                    [("app".to_string(), "crash-merge".to_string())]
                        .into_iter()
                        .collect();
                let rows = [
                    part::Row {
                        tenant: test_tenant(),
                        timestamp_ns: 1_700_000_000_000_000_000,
                        line: "old-one".to_string(),
                        structured_metadata: Vec::new(),
                    },
                    part::Row {
                        tenant: test_tenant(),
                        timestamp_ns: 1_700_000_001_000_000_000,
                        line: "old-two".to_string(),
                        structured_metadata: Vec::new(),
                    },
                ];
                let mut old_parts = Vec::new();
                for row in &rows {
                    old_parts.extend(
                        part::flush_rows(vec![row.clone()], &parts_root, config.row_group_size)
                            .unwrap(),
                    );
                }
                let old_dirs: Vec<_> = old_parts.iter().map(|p| p.dir.clone()).collect();
                part::flush_rows_with_merge_tombstone(
                    rows.to_vec(),
                    &parts_root,
                    config.row_group_size,
                    &old_dirs,
                )
                .unwrap();
                // Old parts are intentionally still present. Startup discovery
                // must finish the tombstone transaction.
            }
            other => panic!("unknown crash test mode: {other}"),
        }

        // End the child without running destructors, matching a process crash
        // rather than an orderly test shutdown.
        std::process::abort();
    }

    #[test]
    fn process_crash_after_wal_ack_recovers_unflushed_data() {
        let dir = run_crash_helper("wal_after_ack");
        let config = Config {
            data_dir: dir,
            ..Config::default()
        };
        let recovered = MemTable::new();

        recover(&config, &recovered).expect("recover acknowledged WAL");

        let results = recovered.query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true);
        assert_eq!(
            results
                .iter()
                .map(|stream| stream.entries.len())
                .sum::<usize>(),
            3
        );
    }

    #[test]
    fn process_crash_between_part_commit_and_checkpoint_is_at_least_once() {
        let dir = run_crash_helper("flush_before_checkpoint");
        let config = Config {
            data_dir: dir,
            ..Config::default()
        };
        let recovered = MemTable::new();

        recover(&config, &recovered).expect("recover pre-checkpoint crash");
        let registry = PartRegistry::load_from_disk(&config.data_dir.join("parts")).unwrap();

        let memory_rows: usize = recovered
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .iter()
            .map(|stream| stream.entries.len())
            .sum();
        let part_rows: usize = registry
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .unwrap()
            .iter()
            .map(|stream| stream.entries.len())
            .sum();
        assert_eq!((memory_rows, part_rows), (3, 3));
    }

    #[test]
    fn process_crash_after_local_wal_compaction_loses_nothing() {
        let dir = run_crash_helper("flush_then_wal_compaction");
        let config = Config {
            data_dir: dir,
            ..Config::default()
        };
        let recovered = MemTable::new();

        recover(&config, &recovered).expect("recover after compaction crash");
        let registry = PartRegistry::load_from_disk(&config.data_dir.join("parts")).unwrap();

        let memory_rows: usize = recovered
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .iter()
            .map(|stream| stream.entries.len())
            .sum();
        let part_rows: usize = registry
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .unwrap()
            .iter()
            .map(|stream| stream.entries.len())
            .sum();
        // The flushed rows come back from parts only; the post-compaction
        // ingest comes back from the retained WAL suffix.
        assert_eq!((memory_rows, part_rows), (3, 3));
    }

    #[test]
    fn process_crash_between_chunked_parts_and_checkpoint_is_at_least_once() {
        let dir = run_crash_helper("chunked_flush_before_checkpoint");
        let config = Config {
            data_dir: dir,
            ..Config::default()
        };
        let recovered = MemTable::new();

        recover(&config, &recovered).expect("recover pre-checkpoint crash");
        let registry = PartRegistry::load_from_disk(&config.data_dir.join("parts")).unwrap();
        assert!(registry.part_count() > 1, "the crash left several parts");

        let memory_rows: usize = recovered
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .iter()
            .map(|stream| stream.entries.len())
            .sum();
        let part_rows: usize = registry
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .unwrap()
            .iter()
            .map(|stream| stream.entries.len())
            .sum();
        // Both copies exist — at-least-once at the flush boundary — and each
        // side holds every row exactly once regardless of the part count.
        assert_eq!((memory_rows, part_rows), (3, 3));
    }

    #[test]
    fn process_crash_after_merge_commit_finishes_tombstone_recovery() {
        let dir = run_crash_helper("merge_before_cleanup");
        let registry = PartRegistry::load_from_disk(&dir.join("parts")).unwrap();

        assert_eq!(registry.part_count(), 1);
        let rows: usize = registry
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .unwrap()
            .iter()
            .map(|stream| stream.entries.len())
            .sum();
        assert_eq!(rows, 2);
    }

    /// One record inside a batch, framed the way collecty ships one: the
    /// payload behind its length, zstd over the whole thing.
    fn collected_body(tenant: &str, line: &str) -> Vec<u8> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let payload = otlp_body_for(tenant, &format!("{tenant}-app"), &[(line, now)]);
        let mut plain = Vec::with_capacity(4 + payload.len());
        plain.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        plain.extend_from_slice(&payload);
        zstd::bulk::compress(&plain, 3).unwrap()
    }


    /// Two tenants pushing through the real router must not be able to read,
    /// enumerate, or count each other's data — before or after a flush turns
    /// the memtable into one shared part.
    #[tokio::test]
    async fn two_tenants_never_see_each_others_logs_over_http() {
        use tower::ServiceExt;

        let data_dir = tmp_data_dir("tenant_isolation");
        let config = crate::config::Config {
            data_dir: data_dir.clone(),
            ..crate::config::Config::default()
        };
        let memtable = Arc::new(MemTable::new());
        let parts = Arc::new(PartRegistry::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            parts.operation_lock(),
        ));
        let state = crate::test_support::state(
            config.clone(),
            memtable.clone(),
            journal.clone(),
            parts.clone(),
            trace_parts.clone(),
            None,
        );

        let collect = |tenant: &'static str, line: &'static str| {
            let state = state.clone();
            async move {
                let request = axum::http::Request::builder()
                    .method("POST")
                    .uri("/signy/api/v1/collect")
                    .header("content-type", "application/x-protobuf")
                    .header("content-encoding", "zstd")
                    .header("x-collecty-signal", "logs")
                    .body(axum::body::Body::from(collected_body(tenant, line)))
                    .unwrap();
                let response = crate::build_router(state).oneshot(request).await.unwrap();
                assert_eq!(response.status(), axum::http::StatusCode::OK);
            }
        };
        collect("acme", "acme secret").await;
        collect("globex", "globex secret").await;

        let get = |tenant: &'static str, uri: &'static str| {
            let state = state.clone();
            async move {
                let request = axum::http::Request::builder()
                    .uri(uri)
                    .header(crate::tenant::TENANT_HEADER, tenant)
                    .body(axum::body::Body::empty())
                    .unwrap();
                let response = crate::build_router(state).oneshot(request).await.unwrap();
                assert_eq!(response.status(), axum::http::StatusCode::OK, "{uri}");
                let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
                    .await
                    .unwrap();
                String::from_utf8(body.to_vec()).unwrap()
            }
        };

        let query_uri = "/signy/api/v1/logs?start=0&limit=100&direction=forward";
        for stage in ["memtable", "part"] {
            let acme_lines = get("acme", query_uri).await;
            let globex_lines = get("globex", query_uri).await;
            assert!(acme_lines.contains("acme secret"), "{stage}: {acme_lines}");
            assert!(
                !acme_lines.contains("globex secret"),
                "{stage}: acme read globex's line: {acme_lines}"
            );
            assert!(
                !globex_lines.contains("acme secret"),
                "{stage}: globex read acme's line: {globex_lines}"
            );

            let values = get(
                "acme",
                "/signy/api/v1/logs/attributes/service_name/values",
            )
            .await;
            assert_eq!(
                values.trim(),
                serde_json::json!({"value": "acme-app"}).to_string(),
                "{stage}: attribute values leaked another tenant"
            );

            if stage == "memtable" {
                // Flush both tenants into one shared part and repeat every
                // assertion against the on-disk path.
                let mut pending_checkpoint = None;
                crate::flush::force_flush_pass(crate::flush::ForceFlush {
                    memtable: &memtable,
                    trace_memtable: &journal.trace_memtable(),
                    journal: &journal,
                    registry: &parts,
                    trace_registry: &trace_parts,
                    series_registry: &crate::series_registry::SeriesRegistry::standalone(),
                    remote_cache: None,
                    config: &config,
                    pending_checkpoint: &mut pending_checkpoint,
                })
                .await
                .unwrap();
                assert_eq!(parts.part_count(), 1, "both tenants share one part");
                assert!(memtable.is_empty());
            }
        }

        // A tenant that has never written sees nothing at all.
        let stranger = get("initech", query_uri).await;
        assert_eq!(stranger, "");
    }

    /// Delivery is at-least-once, so a restart between flush and checkpoint
    /// writes records that are already durable a second time. The trade is
    /// deliberate; what was missing is any way to know it happened. An operator
    /// could not tell a restart that duplicated nothing from one that
    /// duplicated a minute of logs.
    #[tokio::test]
    async fn a_replay_reports_what_it_put_back() {
        let dir = tmp_data_dir("replay-report");
        let config = Config {
            data_dir: dir.clone(),
            ..Config::default()
        };
        let memtable = Arc::new(MemTable::new());
        let journal = Arc::new(crate::journal::Journal::spawn(&config, memtable.clone()).unwrap());
        ingest_once(&journal, &build_push_req()).await;
        ingest_once(&journal, &build_push_req()).await;
        drop(journal);

        // Nothing checkpointed, so recovery replays both records.
        let recovered = Arc::new(MemTable::new());
        let report = crate::startup::recover(&config, &recovered).expect("recover");
        assert_eq!(report.records, 2, "both records came back");
        assert!(report.entries >= 2, "and the entries in them: {report:?}");
        assert_eq!(report.checkpoint, 0);

        // A second recovery over the same WAL reports the same, because the
        // checkpoint still has not moved. That is the case the number exists
        // for: recovery is not idempotent with respect to duplication.
        let again = Arc::new(MemTable::new());
        let second = crate::startup::recover(&config, &again).expect("recover again");
        assert_eq!(second.records, report.records);
    }

    /// A clean start replays nothing, so the number distinguishes the two
    /// rather than always being non-zero.
    #[tokio::test]
    async fn a_clean_start_reports_no_replay() {
        let dir = tmp_data_dir("clean-start");
        let config = Config {
            data_dir: dir.clone(),
            ..Config::default()
        };
        let memtable = Arc::new(MemTable::new());
        let report = crate::startup::recover(&config, &memtable).expect("recover");
        assert_eq!(report.records, 0);
        assert_eq!(report.entries, 0);
    }

    /// The whole write path under its production flush loop: OTLP exports in,
    /// parts out, an empty memtable after, and a restart that replays nothing
    /// while the parts answer. Ported from the Loki push tests when that
    /// ingest was removed — the pipeline it exercises never was the wire.
    #[tokio::test]
    async fn e2e_otlp_flush_loop_persists_through_restart() {
        let dir = tmp_data_dir("full_pipeline");
        let config = Config {
            data_dir: dir.clone(),
            flush_max_interval: std::time::Duration::from_millis(50),
            flush_check_interval: std::time::Duration::from_millis(20),
            ..Config::default()
        };

        let memtable = Arc::new(MemTable::new());
        let parts_root = config.data_dir.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let parts = Arc::new(PartRegistry::new());
        let journal = Arc::new(Journal::spawn(&config, memtable.clone()).unwrap());
        let trace_parts = Arc::new(crate::trace_registry::TraceRegistry::new(
            parts.operation_lock(),
        ));

        let flush_handle = {
            let memtable = memtable.clone();
            let journal = journal.clone();
            let parts = parts.clone();
            let trace_memtable = journal.trace_memtable();
            let trace_parts = trace_parts.clone();
            let config = std::sync::Arc::new(config.clone());
            let healthy = Arc::new(std::sync::atomic::AtomicBool::new(true));
            tokio::spawn(async move {
                crate::flush::flush_loop(
                    memtable,
                    trace_memtable,
                    journal,
                    parts,
                    trace_parts,
                    Arc::new(crate::series_registry::SeriesRegistry::standalone()),
                    None,
                    config,
                    healthy,
                    Arc::new(crate::metrics::RuntimeMetrics::new()),
                    tokio::sync::watch::channel(false).1,
                )
                .await;
            })
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        for i in 0..3i64 {
            ingest_once(
                &journal,
                &otlp_body("pipeline-app", &[(&format!("line-{i}"), now + i)]),
            )
            .await;
        }

        let predicate = crate::part::ExactFieldPredicate::new("service_name", "pipeline-app");
        let mut flushed_total = 0;
        for _ in 0..300 {
            let results = parts
                .query_with_exact_field_pruning(
                    &test_tenant(),
                    crate::part::ExactFieldPruning::new(&[], std::slice::from_ref(&predicate)),
                    crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                    100,
                    true,
                )
                .expect("part query");
            flushed_total = results.iter().map(|s| s.entries.len()).sum::<usize>();
            if flushed_total == 3 && memtable.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert_eq!(flushed_total, 3, "flush did not persist all 3 entries");
        assert!(memtable.is_empty(), "memtable must be empty after flush");

        flush_handle.abort();
        drop(parts);
        drop(journal);
        drop(memtable);

        let memtable2 = MemTable::new();
        let wal = dir.join("journal.wal");
        let ckpt = dir.join("journal.ckpt");
        journal::replay(&wal, &ckpt, &memtable2).expect("replay");
        assert!(
            memtable2.is_empty(),
            "after a full flush, replay yields no in-flight data"
        );

        let registry = PartRegistry::load_from_disk(&parts_root).unwrap();
        assert!(registry.part_count() >= 1);
        let results = registry
            .query_with_exact_field_pruning(
                &test_tenant(),
                crate::part::ExactFieldPruning::new(&[], std::slice::from_ref(&predicate)),
                crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                100,
                true,
            )
            .expect("part query after restart");
        let total: usize = results.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 3);
    }
