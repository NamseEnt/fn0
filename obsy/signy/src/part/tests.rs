    use super::*;
    use crate::tenant::test_tenant;

    fn make_rows() -> Vec<Row> {
        vec![
            Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000,
                line: "error connecting to database".to_string(),
                structured_metadata: vec![
                    ("app".to_string(), "test".to_string()),
                    ("host".to_string(), "h1".to_string()),
                ],
            },
            Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_001_000_000_000,
                line: "all good now".to_string(),
                structured_metadata: vec![
                    ("app".to_string(), "test".to_string()),
                    ("host".to_string(), "h1".to_string()),
                    ("trace_id".to_string(), "abc".to_string()),
                ],
            },
            Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_002_000_000_000,
                line: "other app log line".to_string(),
                structured_metadata: vec![("app".to_string(), "other".to_string())],
            },
        ]
    }

    fn row_at(ts: i64, line: &str) -> Row {
        Row {
            tenant: test_tenant(),
            timestamp_ns: ts,
            line: line.to_string(),
            structured_metadata: vec![],
        }
    }

    fn reader_for(rows: Vec<Row>, tmp: &std::path::Path, row_group_size: usize) -> Arc<PartReader> {
        let part = flush_rows(rows, tmp, row_group_size)
            .expect("flush")
            .remove(0);
        Arc::new(PartReader::open(part).expect("open"))
    }

    fn drain(merged: &mut MergedRows) -> Vec<Row> {
        let mut out = Vec::new();
        while let Some(row) = merged.next_row().expect("stream") {
            out.push(row);
        }
        out
    }

    #[test]
    fn a_paged_stream_yields_every_row_of_a_part_in_order() {
        let tmp = tempfile_dir();
        let base = 1_700_000_000_000_000_000i64;
        let rows: Vec<Row> = (0..50).map(|i| row_at(base + i, &format!("line {i}"))).collect();
        // Row groups of two, and a page budget of one byte, so the stream is
        // forced to page rather than reading the part in one go: what is being
        // tested is that paging does not lose or reorder a row.
        let reader = reader_for(rows.clone(), &tmp, 2);
        assert!(reader.row_group_count() > 1, "the part must actually be paged");

        let mut merged = MergedRows::new(&[reader], 1);
        let read = drain(&mut merged);
        assert_eq!(read.len(), rows.len());
        for (got, want) in read.iter().zip(&rows) {
            assert_eq!(got.timestamp_ns, want.timestamp_ns);
            assert_eq!(got.line, want.line);
        }
    }

    #[test]
    fn merging_parts_yields_one_sorted_run() {
        let tmp = tempfile_dir();
        let base = 1_700_000_000_000_000_000i64;
        // Interleaved on purpose: each part is sorted, the two together are not,
        // and the merge has to produce the order a sort would have.
        let evens: Vec<Row> = (0..20).map(|i| row_at(base + i * 2, &format!("even {i}"))).collect();
        let odds: Vec<Row> = (0..20).map(|i| row_at(base + i * 2 + 1, &format!("odd {i}"))).collect();
        let a = reader_for(evens, &tmp, 3);
        let b = reader_for(odds, &tmp, 3);

        let read = drain(&mut MergedRows::new(&[a, b], 1));
        assert_eq!(read.len(), 40);
        let timestamps: Vec<i64> = read.iter().map(|row| row.timestamp_ns).collect();
        let mut sorted = timestamps.clone();
        sorted.sort_unstable();
        assert_eq!(timestamps, sorted, "a merge must arrive sorted, not merely complete");
    }

    #[test]
    fn a_row_in_two_parts_is_yielded_once() {
        // At-least-once recovery is what puts the same record in two parts, and
        // the first merge that sees both is where the pair is supposed to
        // collapse. `sort_rows` did this; a stream that skipped it would
        // resurrect every duplicate a crash introduced.
        let tmp = tempfile_dir();
        let base = 1_700_000_000_000_000_000i64;
        let shared: Vec<Row> = (0..5).map(|i| row_at(base + i, &format!("dup {i}"))).collect();
        let a = reader_for(shared.clone(), &tmp, 2);
        let b = reader_for(shared.clone(), &tmp, 2);

        let read = drain(&mut MergedRows::new(&[a, b], 1));
        assert_eq!(read.len(), shared.len(), "duplicates survived the merge");

        // A row differing only in its line is not a duplicate: the key is the
        // whole row, not its timestamp.
        let tmp2 = tempfile_dir();
        let c = reader_for(shared.clone(), &tmp2, 2);
        let different: Vec<Row> = (0..5).map(|i| row_at(base + i, &format!("other {i}"))).collect();
        let d = reader_for(different, &tmp2, 2);
        assert_eq!(drain(&mut MergedRows::new(&[c, d], 1)).len(), 10);
    }

    #[test]
    fn the_streaming_writer_produces_the_same_part_as_the_batch_one() {
        // The whole argument for streaming a merge is that the format never
        // needed the rows held. This is what says so: identical inputs through
        // both writers must leave identical sidecars, or the two have drifted
        // and a merged part is not what a flushed one would have been.
        let tmp = tempfile_dir();
        let base = 1_700_000_000_000_000_000i64;
        let mut rows: Vec<Row> = Vec::new();
        for tenant in ["tenant-a", "tenant-b"] {
            for i in 0..40i64 {
                rows.push(Row {
                    tenant: TenantId::parse(tenant).expect("valid"),
                    timestamp_ns: base + i,
                    line: format!("{{\"status\":\"{}\",\"msg\":\"row {i}\"}}", 200 + i % 5),
                    structured_metadata: vec![
                        ("app".to_string(), format!("app-{}", i % 3)),
                        ("host".to_string(), tenant.to_string()),
                        ("trace_id".to_string(), format!("t{i}")),
                    ],
                });
            }
        }

        let batch = flush_rows(rows.clone(), &tmp, 7).expect("flush").remove(0);

        // `load_part` checks both the directory name against the id and its
        // parent against the partition, so the streamed copy mirrors the layout
        // rather than sitting in a directory of its own choosing.
        let streamed_dir = tmp
            .join("streamed")
            .join(&batch.meta.partition)
            .join(&batch.meta.id);
        std::fs::create_dir_all(&streamed_dir).expect("mkdir");
        let metadata_keys: Vec<String> = select_metadata_columns(metadata_column_counts(&rows))
            .into_iter()
            .map(|(key, _)| key)
            .collect();
        let parsed_keys: Vec<String> =
            select_metadata_columns(parsed_column_counts(&parse_rows(&rows)))
                .into_iter()
                .map(|(key, _)| key)
                .collect();
        let mut writer =
            StreamingPartWriter::create(&streamed_dir, metadata_keys, parsed_keys, 7)
                .expect("create");
        // The batch path sorts; the streaming path is promised sorted input,
        // which is MergedRows' job. Sorting here is standing in for it.
        let mut sorted = rows.clone();
        sort_rows(&mut sorted);
        for row in sorted {
            writer.push(row).expect("push");
        }
        writer
            .finish(&streamed_dir, &batch.meta.id, &batch.meta.partition)
            .expect("finish");

        let batch_index = std::fs::read(batch.dir.join(INDEX_FILE)).expect("batch index");
        let streamed_index = std::fs::read(streamed_dir.join(INDEX_FILE)).expect("streamed index");
        assert_eq!(
            batch_index, streamed_index,
            "the blooms must be byte-identical"
        );

        let batch_meta = load_part(&batch.dir).expect("batch meta").meta;
        let streamed_part = load_part(&streamed_dir).expect("streamed meta");
        let streamed_meta = streamed_part.meta.clone();
        let segments = |meta: &PartMeta| -> Vec<(String, u32, u32, u64, i64, i64)> {
            meta.tenants
                .iter()
                .map(|segment| {
                    (
                        segment.tenant.to_string(),
                        segment.row_group_start,
                        segment.row_group_end,
                        segment.row_count,
                        segment.min_ts_ns,
                        segment.max_ts_ns,
                    )
                })
                .collect()
        };
        assert_eq!(segments(&batch_meta), segments(&streamed_meta));
        assert_eq!(batch_meta.row_count, streamed_meta.row_count);
        assert_eq!(batch_meta.row_group_count, streamed_meta.row_group_count);
        assert_eq!(batch_meta.min_ts_ns, streamed_meta.min_ts_ns);
        assert_eq!(batch_meta.max_ts_ns, streamed_meta.max_ts_ns);
        assert_eq!(batch_meta.row_group_min_ts, streamed_meta.row_group_min_ts);
        assert_eq!(batch_meta.row_group_max_ts, streamed_meta.row_group_max_ts);
        assert_eq!(
            batch_meta.materialized_bytes,
            streamed_meta.materialized_bytes
        );

        // And it must be readable, not merely equal on paper.
        let reader = PartReader::open(streamed_part).expect("open streamed part");
        let read = reader
            .read_all_rows(None)
            .expect("read back")
            .len();
        assert_eq!(read, 80);
    }

    #[test]
    fn flush_then_query_roundtrip() {
        let tmp = tempfile_dir();
        let rows = make_rows();
        let parts = flush_rows(rows.clone(), &tmp, 2).expect("flush");
        assert_eq!(parts.len(), 1);
        let reader = PartReader::open(parts.into_iter().next().unwrap()).expect("open");

        // all
        let r = reader
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .expect("q");
        let total: usize = r.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 3);

        // exact-field predicate app="test"
        let predicate = [ExactFieldPredicate::new("app", "test")];
        let r = reader
            .query_with_exact_field_pruning(
                &test_tenant(),
                ExactFieldPruning::new(&[], &predicate),
                crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                100,
                true,
            )
            .expect("q");
        let total: usize = r.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 2);

        // line filter "error"
        let f = LineFilter::Contains("error".to_string());
        let r = reader
            .query(&test_tenant(), &[f], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .expect("q");
        let total: usize = r.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 1);

        // time range
        let r = reader
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(1_700_000_001_000_000_000, 1_700_000_003_000_000_000),
                100,
                true,
            )
            .expect("q");
        let total: usize = r.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 2);
    }

    #[test]
    fn bloom_prunes_nonexistent_substring() {
        let tmp = tempfile_dir();
        let rows = make_rows();
        let parts = flush_rows(rows, &tmp, 100).expect("flush");
        let reader = PartReader::open(parts.into_iter().next().unwrap()).expect("open");
        let f = LineFilter::Contains("zzzzzz-not-present".to_string());
        assert!(
            reader
                .select_row_groups(&test_tenant(),
                    std::slice::from_ref(&f),
                    QueryTimeRange::closed(i64::MIN, i64::MAX),
                ).unwrap()
                .is_empty(),
            "bloom miss must avoid selecting the parquet row group"
        );
        let r = reader
            .query(&test_tenant(), &[f], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .expect("q");
        let total: usize = r.iter().map(|s| s.entries.len()).sum();
        assert_eq!(total, 0);
    }

    #[test]
    fn exact_field_bloom_prunes_structured_metadata_by_row_group() {
        let tmp = tempfile_dir();
        let mut rows = make_rows();
        rows[0].structured_metadata.push(("trace_id".to_string(), "first".to_string()));
        if let Some(pair) = rows[1]
            .structured_metadata
            .iter_mut()
            .find(|(key, _)| key == "trace_id")
        {
            pair.1 = "second".to_string();
        }
        let part = flush_rows(rows, &tmp, 1).unwrap().remove(0);
        let index_bytes = fs::read(part.index_path()).unwrap();
        assert_eq!(&index_bytes[..INDEX_MAGIC.len()], INDEX_MAGIC);
        assert_eq!(&split_index(&index_bytes).unwrap()[..4], BLOOM_MAGIC);
        let reader = PartReader::open(part).unwrap();

        let selected = reader.select_row_groups_with_exact_fields(&test_tenant(),
            &[],
            &[ExactFieldPredicate::new("trace_id", "second")],
            QueryTimeRange::closed(i64::MIN, i64::MAX),
        ).unwrap();
        // The bloom selects exactly the one group holding the value.
        assert_eq!(selected, vec![1]);

        assert_eq!(
            reader.select_row_groups_with_exact_fields(&test_tenant(),
                &[],
                &[ExactFieldPredicate::new("app", "test")],
                QueryTimeRange::closed(i64::MIN, i64::MAX),
            ).unwrap(),
            // The two groups holding `app="test"`.
            vec![0, 1]
        );

        assert!(!reader.may_match_exact_fields(&test_tenant(),
            &[],
            &[ExactFieldPredicate::new("trace_id", "not-present")], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
        ));
        assert!(reader.may_match_exact_fields(&test_tenant(),
            &[],
            &[ExactFieldPredicate::new("missing", "")], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
        ));
    }

    /// A row group with nothing to index spends no exact-field filter.
    ///
    /// `optimal_bits` has a 1024-bit floor, so the empty filter V3 stored cost
    /// 140 bytes whether or not there was a token to put in it, and
    /// tenant-aligned row groups charge that per tenant per part. Absence has
    /// to keep pruning exactly as the empty filter did, which is the second
    /// half of this test.
    #[test]
    fn a_row_group_with_no_exact_field_token_stores_no_filter() {
        let tmp = tempfile_dir();
        let mut plain = make_rows();
        for row in &mut plain {
            row.line = "nothing here parses as a field".to_string();
            row.structured_metadata = vec![];
        }
        let plain_part = flush_rows(plain.clone(), &tmp, 1).unwrap().remove(0);
        let plain_bytes = fs::metadata(plain_part.index_path()).unwrap().len();

        let mut indexed = plain;
        for row in &mut indexed {
            row.structured_metadata = vec![("trace_id".to_string(), "abc".to_string())];
        }
        let indexed_part = flush_rows(indexed, &tempfile_dir(), 1).unwrap().remove(0);
        let indexed_bytes = fs::metadata(indexed_part.index_path()).unwrap().len();

        let row_groups = plain_part.meta.row_group_count as u64;
        assert!(row_groups >= 2, "need more than one row group to see a floor");
        assert!(
            indexed_bytes >= plain_bytes + 140 * row_groups,
            "an unused filter used to cost the 1024-bit floor per row group: \
{indexed_bytes} vs {plain_bytes} over {row_groups} row groups"
        );

        // Absence prunes. A predicate on a field the row group never indexed
        // cannot match it, which is the same answer the all-zero filter gave.
        let reader = PartReader::open(plain_part).unwrap();
        assert!(
            reader
                .select_row_groups_with_exact_fields(
                    &test_tenant(),
                    &[],
                    &[ExactFieldPredicate::new("trace_id", "abc")],
                    QueryTimeRange::closed(i64::MIN, i64::MAX),
                ).unwrap()
                .is_empty(),
            "no token indexed means no exact-field predicate can match"
        );
        // The conservative case still is: an empty value stands for field
        // absence.
        assert!(reader.may_match_exact_fields(
            &test_tenant(),
            &[],
            &[ExactFieldPredicate::new("trace_id", "")], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
        ));
    }

    #[test]
    fn exact_field_bloom_indexes_parser_scalars_without_raw_substring_assumptions() {
        let tmp = tempfile_dir();
        let mut rows = make_rows();
        rows[0].line = r#"{"user":"\u0061lice","namespace:key":"value"}"#.to_string();
        rows[1].line = r#"user=bob elapsed=250ms"#.to_string();
        let part = flush_rows(rows, &tmp, 1).unwrap().remove(0);
        let reader = PartReader::open(part).unwrap();
        let time_range = QueryTimeRange::closed(i64::MIN, i64::MAX);

        assert_eq!(
            reader.select_row_groups_with_exact_fields(&test_tenant(),
                &[],
                &[ExactFieldPredicate::new_with_extraction(
                    "user", "alice", true,
                )],
                time_range,
            ).unwrap(),
            // The property under test — one group per value, selected
            // exactly.
            vec![0]
        );
        assert_eq!(
            reader.select_row_groups_with_exact_fields(&test_tenant(),
                &[],
                &[ExactFieldPredicate::new_with_extraction(
                    "user", "bob", true,
                )],
                time_range,
            ).unwrap(),
            vec![1]
        );
        assert_eq!(
            reader.select_row_groups_with_exact_fields(&test_tenant(),
                &[],
                &[ExactFieldPredicate::new_with_extraction(
                    "namespace_key",
                    "value",
                    true,
                )],
                time_range,
            ).unwrap(),
            // The first row's own group.
            vec![0]
        );
    }

    #[test]
    fn exact_field_bloom_indexes_canonical_numeric_and_duration_values() {
        let tmp = tempfile_dir();
        let rows = vec![
            Row {
                tenant: test_tenant(),
                timestamp_ns: 1,
                line: r#"{"value":9007199254740992,"elapsed":"1s"}"#.to_string(),
                structured_metadata: vec![],
            },
            Row {
                tenant: test_tenant(),
                timestamp_ns: 2,
                line: r#"{"value":9007199254740993,"elapsed":"1000ms"}"#.to_string(),
                structured_metadata: vec![],
            },
        ];
        let part = flush_rows(rows, &tmp, 1).unwrap().remove(0);
        let reader = PartReader::open(part).unwrap();
        let range = QueryTimeRange::closed(i64::MIN, i64::MAX);

        let field_query = |name: &str, value: crate::logql::FieldValue| {
            let mut query = crate::query::parse_filter_params("parse=json", 0, crate::query::LOGS_PARAMS)
                .unwrap()
                .query;
            query
                .stages
                .push(crate::logql::PipelineStage::Field(crate::logql::FieldFilter {
                    name: name.to_string(),
                    op: crate::logql::FieldOp::Eq,
                    value,
                }));
            query
        };
        let numeric = field_query(
            "value",
            crate::logql::FieldValue::Number(
                crate::logql::Decimal::parse("9007199254740993").unwrap(),
            ),
        );
        assert_eq!(
            reader.select_row_groups_with_exact_fields(&test_tenant(),
                &[],
                &numeric.exact_field_predicates(),
                range,
            ).unwrap(),
            vec![1]
        );

        let duration = field_query(
            "elapsed",
            crate::logql::FieldValue::Duration(crate::logql::parse_duration_ns("1s").unwrap()),
        );
        assert_eq!(
            reader.select_row_groups_with_exact_fields(&test_tenant(),
                &[],
                &duration.exact_field_predicates(),
                range,
            ).unwrap(),
            vec![0, 1]
        );
    }

    #[test]
    fn the_bloom_container_rejects_invalid_framing() {
        // One 2000-row group: a line bloom, then two exact-field windows —
        // one real filter, one zero-length (token-less) slot.
        let rows: &[u32] = &[2000];
        let mut valid = Vec::new();
        valid.extend_from_slice(BLOOM_MAGIC);
        valid.extend_from_slice(&1u32.to_le_bytes());
        let encoded = BloomFilter::with_capacity(1, 0.01).encode();
        valid.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        valid.extend_from_slice(&encoded);
        valid.extend_from_slice(&2u32.to_le_bytes());
        valid.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        valid.extend_from_slice(&encoded);
        valid.extend_from_slice(&0u32.to_le_bytes());

        let (line, windows) = locate_blooms(&valid, 0, rows).unwrap();
        assert_eq!(line.len(), 1);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].len(), 2);
        assert!(matches!(windows[0][0], WindowBloom::Filter(_)));
        assert!(matches!(windows[0][1], WindowBloom::Absent));

        // A token-less group writes a window count of zero, whatever its
        // row count.
        let mut empty = Vec::new();
        empty.extend_from_slice(BLOOM_MAGIC);
        empty.extend_from_slice(&1u32.to_le_bytes());
        empty.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        empty.extend_from_slice(&encoded);
        empty.extend_from_slice(&0u32.to_le_bytes());
        let (_, windows) = locate_blooms(&empty, 0, rows).unwrap();
        assert!(windows[0].is_empty());

        assert!(
            locate_blooms(&valid, 0, &[2000, 2000])
                .err()
                .unwrap()
                .contains("row group count mismatch")
        );

        // A non-zero window count must match what the row count implies.
        assert!(
            locate_blooms(&valid, 0, &[5000])
                .err()
                .unwrap()
                .contains("bloom window count mismatch")
        );

        let mut trailing = valid.clone();
        trailing.push(0);
        assert!(
            locate_blooms(&trailing, 0, rows)
                .err()
                .unwrap()
                .contains("trailing bytes")
        );

        let mut unknown = valid.clone();
        unknown[..4].copy_from_slice(b"XXXX");
        assert!(
            locate_blooms(&unknown, 0, rows)
                .err()
                .unwrap()
                .contains("magic mismatch")
        );

        let truncated = &valid[..valid.len() - 2];
        assert!(locate_blooms(truncated, 0, rows).is_err());

        // The previous on-disk generation fails loudly rather than decoding
        // wrong: a stale data directory is deleted and re-ingested, per the
        // no-versioning policy.
        let mut old = valid.clone();
        old[..4].copy_from_slice(b"BTF4");
        assert!(
            locate_blooms(&old, 0, rows)
                .err()
                .unwrap()
                .contains("magic mismatch")
        );

        // The saturation sentinel decodes as admit-everything: distinct from
        // the zero length, which prunes.
        let mut saturated = Vec::new();
        saturated.extend_from_slice(BLOOM_MAGIC);
        saturated.extend_from_slice(&1u32.to_le_bytes());
        saturated.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        saturated.extend_from_slice(&encoded);
        saturated.extend_from_slice(&2u32.to_le_bytes());
        saturated.extend_from_slice(&crate::part::SATURATED_WINDOW_SENTINEL.to_le_bytes());
        saturated.extend_from_slice(&0u32.to_le_bytes());
        let (_, windows) = locate_blooms(&saturated, 0, rows).unwrap();
        assert!(matches!(windows[0][0], WindowBloom::Saturated));
        assert!(matches!(windows[0][1], WindowBloom::Absent));
    }

    /// A wide-JSON window past the token cap is stored saturated: the index
    /// stops growing with the attack, and the window admits every predicate
    /// instead of false-negativing any.
    #[test]
    fn a_token_flooded_window_saturates_instead_of_outgrowing_the_data() {
        let tmp = tempfile_dir();
        let base = 1_700_000_000_000_000_000i64;
        // ~130 metadata tokens per row × 1024 rows ≈ 133k tokens — past the
        // 65,536 cap in the very first window.
        let rows: Vec<Row> = (0..1100i64)
            .map(|i| {
                let metadata = (0..130)
                    .map(|k| (format!("k{k:03}"), format!("v{i}-{k}")))
                    .collect();
                window_row(base + i, metadata)
            })
            .collect();
        let part = flush_rows(rows, &tmp, 4096).unwrap().remove(0);
        let index_bytes = fs::metadata(part.index_path()).unwrap().len();
        let data_bytes = fs::metadata(part.data_path()).unwrap().len();
        assert!(
            index_bytes < data_bytes,
            "index ({index_bytes}) must not outgrow data ({data_bytes})"
        );
        let reader = PartReader::open(part).unwrap();

        // Present and absent values both admit — saturation prunes nothing —
        // and the present one still answers correctly end to end.
        for value in ["v5-7", "not-there-at-all"] {
            let predicate = [ExactFieldPredicate::new("k007", value)];
            assert!(reader.may_match_exact_fields(
                &test_tenant(),
                &[],
                &predicate,
                QueryTimeRange::closed(i64::MIN, i64::MAX),
            ));
        }
        let predicate = [ExactFieldPredicate::new("k007", "v5-7")];
        let results = reader
            .query_with_exact_field_pruning_and_scan_limit(
                &test_tenant(),
                ExactFieldPruning::new(&[], &predicate),
                QueryTimeRange::closed(i64::MIN, i64::MAX),
                100,
                true,
                None,
                None,
            )
            .unwrap();
        assert_eq!(
            results.results.iter().map(|s| s.entries.len()).sum::<usize>(),
            1
        );
    }

    #[test]
    fn forward_limit_stops_physical_part_scan() {
        let tmp = tempfile_dir();
        let rows: Vec<Row> = (0..20)
            .map(|timestamp_ns| Row {
                tenant: test_tenant(),
                timestamp_ns,
                line: format!("line-{timestamp_ns}"),
                structured_metadata: vec![],
            })
            .collect();
        let part = flush_rows(rows, &tmp, 20).unwrap().remove(0);
        let reader = PartReader::open(part).unwrap();
        let result = reader
            .query_with_exact_field_pruning_and_scan_limit(&test_tenant(),
                ExactFieldPruning::new(&[], &[]), crate::part::QueryTimeRange::closed(0, 19),
                1,
                true,
                Some(100),
                None,
            )
            .unwrap();
        assert_eq!(result.scanned_rows, 1);
        assert_eq!(result.results[0].entries[0].timestamp_ns, 0);
    }

    /// The ordering the bounded scan's early termination rests on.
    ///
    /// `PartReader::scan_into` treats the first row on the far side of the
    /// sink's frontier as the end of the part, which is only sound because a
    /// tenant's rows come back in timestamp order — `Row::sort_key` starts
    /// `(tenant, timestamp_ns, …)` and every writer sorts. Written deliberately
    /// out of order, in two tenants and across several row groups, and asserted
    /// in both directions, because an unsorted part would make a limited query
    /// silently drop rows rather than fail.
    #[test]
    fn a_parts_rows_come_back_in_timestamp_order_within_a_tenant() {
        let tmp = tempfile_dir();
        let other = crate::tenant::TenantId::parse("other").unwrap();
        let rows: Vec<Row> = [7i64, 2, 9, 0, 5, 3, 8, 1, 6, 4]
            .into_iter()
            .flat_map(|timestamp_ns| {
                [test_tenant(), other.clone()]
                    .into_iter()
                    .map(move |tenant| Row {
                        tenant,
                        timestamp_ns,
                        line: format!("line-{timestamp_ns}"),
                        structured_metadata: vec![],
                    })
            })
            .collect();
        let part = flush_rows(rows, &tmp, 3).unwrap().remove(0);
        let reader = PartReader::open(part).unwrap();
        for tenant in [test_tenant(), other] {
            for forward in [true, false] {
                let mut collector = RowCollector::new(&tenant);
                reader
                    .scan_into(
                        &tenant,
                        &[],
                        &[],
                        QueryTimeRange::unbounded(),
                        forward,
                        None,
                        None,
                        None,
                        None,
                        &ColumnSet::all(),
                        &mut collector,
                    )
                    .unwrap();
                let seen: Vec<i64> = collector
                    .into_rows()
                    .iter()
                    .map(|row| row.timestamp_ns)
                    .collect();
                let mut wanted: Vec<i64> = (0..10).collect();
                if !forward {
                    wanted.reverse();
                }
                assert_eq!(
                    seen, wanted,
                    "tenant {tenant} scanned {} order",
                    if forward { "forward" } else { "backward" }
                );
            }
        }
    }

    /// A bounded query must answer exactly what an unbounded scan truncated
    /// to the limit would. The comparison bed caught a violation of this
    /// live: a backward `limit=100` over one app returned rows from the
    /// middle of the window while Loki returned the newest hundred.
    #[test]
    fn a_limited_scan_over_interleaved_rows_returns_what_truncation_would() {
        let tmp = tempfile_dir();
        // App `a` owns the even timestamps, app `b` the odd ones, and one
        // row group holds both.
        let rows: Vec<Row> = (0..40)
            .map(|timestamp_ns: i64| Row {
                tenant: test_tenant(),
                timestamp_ns,
                line: format!("line-{timestamp_ns}"),
                structured_metadata: vec![(
                    "app".to_string(),
                    if timestamp_ns % 2 == 0 { "a" } else { "b" }.to_string(),
                )],
            })
            .collect();
        let part = flush_rows(rows, &tmp, 64).unwrap().remove(0);
        assert_eq!(part.meta.row_group_count, 1, "the test needs a shared group");
        let reader = PartReader::open(part).unwrap();
        for forward in [true, false] {
            let results = reader
                .query(&test_tenant(), &[],
                    QueryTimeRange::unbounded(),
                    5,
                    forward,
                )
                .unwrap();
            let mut seen: Vec<i64> = results
                .iter()
                .flat_map(|stream| stream.entries.iter().map(|entry| entry.timestamp_ns))
                .collect();
            seen.sort_unstable();
            let wanted: Vec<i64> = if forward {
                (0..5).collect()
            } else {
                (35..40).collect()
            };
            assert_eq!(
                seen,
                wanted,
                "a limited {} scan must return the rows truncation would",
                if forward { "forward" } else { "backward" }
            );
        }
    }

    /// More distinct keys than `MAX_METADATA_COLUMNS`: the frequent ones get
    /// columns, the rest ride the residual blob, and a read cannot tell —
    /// every row's pairs come back exactly, in canonical order, dotted OTLP
    /// names included. The split is by row count so key churn cannot evict a
    /// key every row carries.
    #[test]
    fn metadata_past_the_column_cap_survives_in_the_residual() {
        let tmp = tempfile_dir();
        let rows: Vec<Row> = (0..(MAX_METADATA_COLUMNS as i64 + 40))
            .map(|i| Row {
                tenant: test_tenant(),
                timestamp_ns: i,
                line: format!("line-{i}"),
                // Canonical order: "k.rare..." sorts before "service.name"
                // and "trace_id". `trace_id` and `service.name` are on every
                // row; each `k.rare-N` is on one row only, and there are more
                // of them than the cap leaves room for.
                structured_metadata: vec![
                    (format!("k.rare-{i:04}"), format!("v{i}")),
                    ("service.name".to_string(), "worker".to_string()),
                    ("trace_id".to_string(), format!("t{i}")),
                ],
            })
            .collect();
        let part = flush_rows(rows.clone(), &tmp, 8192).unwrap().remove(0);
        assert_eq!(part.meta.metadata_columns.len(), MAX_METADATA_COLUMNS);
        assert!(
            part.meta
                .metadata_columns
                .iter()
                .any(|(key, count)| key == "trace_id" && *count == rows.len() as u64),
            "an every-row key must hold a column against any amount of churn"
        );
        let reader = PartReader::open(part).unwrap();
        let tenant = test_tenant();
        let mut collector = RowCollector::new(&tenant);
        reader
            .scan_into(
                &tenant,
                &[],
                &[],
                QueryTimeRange::unbounded(),
                true,
                None,
                None,
                None,
                None,
                &ColumnSet::all(),
                &mut collector,
            )
            .unwrap();
        let read = collector.into_rows();
        assert_eq!(read.len(), rows.len());
        for (read, wrote) in read.iter().zip(&rows) {
            assert_eq!(
                read.structured_metadata, wrote.structured_metadata,
                "every pair must round-trip whichever side of the cap it landed on"
            );
        }
    }

    /// Page-index time pruning may skip a page, never a row: every sub-window
    /// answer must equal the brute-force filter of the same rows, and windows
    /// at every alignment sweep the page boundaries.
    #[test]
    fn page_index_time_pruning_never_drops_a_row_any_window_asks_for() {
        let tmp = tempfile_dir();
        let rows: Vec<Row> = (0..2_000)
            .map(|timestamp_ns: i64| Row {
                tenant: test_tenant(),
                timestamp_ns,
                line: format!("line-{timestamp_ns}"),
                structured_metadata: vec![],
            })
            .collect();
        let part = flush_rows(rows.clone(), &tmp, 4096).unwrap().remove(0);
        let reader = PartReader::open(part).unwrap();
        for (start, end) in [(0, 2_000), (137, 411), (999, 1_001), (1_990, 2_500), (0, 1)] {
            for forward in [true, false] {
                let results = reader
                    .query(&test_tenant(), &[],
                        QueryTimeRange::half_open(start, end),
                        usize::MAX,
                        forward,
                    )
                    .unwrap();
                let mut seen: Vec<i64> = results
                    .iter()
                    .flat_map(|stream| stream.entries.iter().map(|entry| entry.timestamp_ns))
                    .collect();
                seen.sort_unstable();
                let wanted: Vec<i64> = (start.max(0)..end.min(2_000)).collect();
                assert_eq!(seen, wanted, "window [{start}, {end}) {forward}");
            }
        }
    }

    #[test]
    fn scan_limit_stops_before_collecting_the_rest_of_a_part() {
        let tmp = tempfile_dir();
        let rows: Vec<Row> = (0..20)
            .map(|timestamp_ns| Row {
                tenant: test_tenant(),
                timestamp_ns,
                line: format!("line-{timestamp_ns}"),
                structured_metadata: vec![],
            })
            .collect();
        let part = flush_rows(rows, &tmp, 20).unwrap().remove(0);
        let reader = PartReader::open(part).unwrap();
        let result = reader
            .query_with_exact_field_pruning_and_scan_limit(&test_tenant(),
                ExactFieldPruning::new(&[], &[]), crate::part::QueryTimeRange::closed(0, 19),
                usize::MAX,
                true,
                Some(3),
                None,
            )
            .unwrap();
        assert_eq!(result.scanned_rows, 3);
        assert_eq!(result.results[0].entries.len(), 3);
    }

    /// One row per row group, so the row-group bound and the row bound are both
    /// exercised on the same boundary. A pruning bound tighter than the row
    /// bound drops rows silently, so the two are asserted against each other
    /// rather than each on its own.
    #[test]
    fn a_row_on_end_belongs_to_a_closed_window_and_not_to_a_half_open_one() {
        let tmp = tempfile_dir();
        let rows: Vec<Row> = [100, 200, 300]
            .into_iter()
            .map(|timestamp_ns| Row {
                tenant: test_tenant(),
                timestamp_ns,
                line: format!("line-{timestamp_ns}"),
                structured_metadata: vec![],
            })
            .collect();
        let reader = PartReader::open(flush_rows(rows, &tmp, 1).unwrap().remove(0)).unwrap();

        let timestamps = |range| {
            let mut seen: Vec<i64> = reader
                .query(&test_tenant(), &[], range, 100, true)
                .expect("query")
                .iter()
                .flat_map(|stream| stream.entries.iter().map(|entry| entry.timestamp_ns))
                .collect();
            seen.sort_unstable();
            seen
        };
        let selected = |range| reader.select_row_groups(&test_tenant(), &[], range).unwrap();

        assert_eq!(timestamps(QueryTimeRange::closed(100, 200)), vec![100, 200]);
        assert_eq!(timestamps(QueryTimeRange::half_open(100, 200)), vec![100]);
        assert_eq!(timestamps(QueryTimeRange::half_open(100, 201)), vec![
            100, 200
        ]);
        assert_eq!(
            timestamps(QueryTimeRange::half_open(200, 200)),
            Vec::<i64>::new(),
            "an empty window returns nothing, not the row on its boundary"
        );

        // Row groups here are one row each, so the group a boundary row lives in
        // must be selected exactly when that row is returned.
        assert_eq!(selected(QueryTimeRange::closed(100, 200)).len(), 2);
        assert_eq!(selected(QueryTimeRange::half_open(100, 200)).len(), 1);
        for range in [
            QueryTimeRange::closed(100, 200),
            QueryTimeRange::half_open(100, 200),
            QueryTimeRange::half_open(100, 201),
            QueryTimeRange::half_open(200, 200),
        ] {
            assert_eq!(
                selected(range).len(),
                timestamps(range).len(),
                "row-group pruning and the row-level test must agree"
            );
        }

        assert!(
            reader.may_match_exact_fields(
                &test_tenant(),
                &[],
                &[],
                QueryTimeRange::closed(200, 200)
            )
        );
        assert!(
            !reader.may_match_exact_fields(
                &test_tenant(),
                &[],
                &[],
                QueryTimeRange::half_open(200, 200)
            ),
            "an empty window cannot need a part restored for it"
        );
    }

    #[test]
    fn discover_parts_after_flush() {
        let tmp = tempfile_dir();
        let rows = make_rows();
        let _ = flush_rows(rows, &tmp, 100).expect("flush");
        let parts = discover_parts(&tmp).unwrap();
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn backward_limit_returns_most_recent() {
        let tmp = tempfile_dir();
        let rows: Vec<Row> = (0..3_000)
            .map(|i| Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000 + i * 1_000_000_000,
                line: format!("line-{i:04}"),
                structured_metadata: vec![],
            })
            .collect();
        let parts = flush_rows(rows, &tmp, 3_000).expect("flush");
        let reader = PartReader::open(parts.into_iter().next().unwrap()).expect("open");

        let r = reader
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 3, false)
            .expect("q");
        let lines: Vec<&str> = r
            .iter()
            .flat_map(|s| s.entries.iter().map(|e| e.line.as_str()))
            .collect();
        assert_eq!(lines, vec!["line-2999", "line-2998", "line-2997"]);

        let r = reader
            .query(&test_tenant(), &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 3, true)
            .expect("q");
        let lines: Vec<&str> = r
            .iter()
            .flat_map(|s| s.entries.iter().map(|e| e.line.as_str()))
            .collect();
        assert_eq!(lines, vec!["line-0000", "line-0001", "line-0002"]);
    }

    #[test]
    fn concurrent_queries_on_same_part_no_race() {
        let tmp = tempfile_dir();
        let rows: Vec<Row> = (0..50)
            .map(|i| Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000 + i * 1_000_000_000,
                line: format!("concurrent-line-{:02}", i),
                structured_metadata: vec![("app".to_string(), "concurrent".to_string())],
            })
            .collect();
        let parts = flush_rows(rows, &tmp, 8).expect("flush");
        let reader = Arc::new(PartReader::open(parts.into_iter().next().unwrap()).expect("open"));

        let predicate = ExactFieldPredicate::new("app", "concurrent");

        let num_threads = 16;
        let queries_per_thread = 50;
        let mut handles = Vec::with_capacity(num_threads);
        for thread_index in 0..num_threads {
            let reader = reader.clone();
            let predicate = predicate.clone();
            handles.push(std::thread::spawn(move || {
                let mut errors = 0u32;
                let mut wrong = 0u32;
                for q in 0..queries_per_thread {
                    let forward = (thread_index + q) % 2 == 0;
                    let limit = 3 + (q % 5);
                    let result = reader
                        .query_with_exact_field_pruning(&test_tenant(),
                            ExactFieldPruning::new(&[], std::slice::from_ref(&predicate)),
                            crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                            limit,
                            forward,
                        )
                        .expect("query must not error");
                    let total: usize = result.iter().map(|s| s.entries.len()).sum();
                    if total == 0 {
                        errors += 1;
                    } else if total > limit {
                        wrong += 1;
                    }
                    if forward {
                        let first = result
                            .iter()
                            .flat_map(|s| s.entries.iter())
                            .map(|e| e.timestamp_ns)
                            .next()
                            .unwrap_or(0);
                        let expected_first = 1_700_000_000_000_000_000;
                        if first != expected_first {
                            wrong += 1;
                        }
                    } else {
                        let first = result
                            .iter()
                            .flat_map(|s| s.entries.iter())
                            .map(|e| e.timestamp_ns)
                            .next()
                            .unwrap_or(0);
                        let expected_first = 1_700_000_000_000_000_000 + 49 * 1_000_000_000;
                        if first != expected_first {
                            wrong += 1;
                        }
                    }
                }
                (errors, wrong)
            }));
        }
        let mut total_errors = 0u32;
        let mut total_wrong = 0u32;
        for h in handles {
            let (e, w) = h.join().expect("thread");
            total_errors += e;
            total_wrong += w;
        }
        assert_eq!(
            total_errors, 0,
            "some queries returned empty (race-induced decode failure)"
        );
        assert_eq!(
            total_wrong, 0,
            "some queries returned wrong ordering or count (race-induced corruption)"
        );
    }

    fn tempfile_dir() -> PathBuf {
        crate::test_support::temp_dir("part")
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_tmp_rejects_symlinked_root_and_tmp_directory() {
        use std::os::unix::fs::symlink;

        let outside = tempfile_dir();
        let outside_tmp = outside.join(".tmp");
        fs::create_dir_all(&outside_tmp).unwrap();
        let sentinel = outside_tmp.join("sentinel");
        fs::write(&sentinel, b"must survive").unwrap();

        let link_parent = tempfile_dir();
        let linked_root = link_parent.join("parts");
        symlink(&outside, &linked_root).unwrap();
        let error = cleanup_tmp(&linked_root).unwrap_err();
        assert!(error.contains("unsafe parts root"));
        assert!(sentinel.exists());

        let normal_root = tempfile_dir();
        let linked_tmp = normal_root.join(".tmp");
        symlink(&outside_tmp, &linked_tmp).unwrap();
        let error = cleanup_tmp(&normal_root).unwrap_err();
        assert!(error.contains("unsafe tmp directory"));
        assert!(sentinel.exists());
    }

    #[test]
    fn load_rejects_metadata_checksum_mismatch() {
        let tmp = tempfile_dir();
        let part = flush_rows(make_rows(), &tmp, 100).expect("flush").remove(0);
        let meta_path = part.meta_path();
        let mut meta: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();
        meta["row_count"] = serde_json::Value::from(999u64);
        fs::write(&meta_path, serde_json::to_vec(&meta).unwrap()).unwrap();

        assert!(load_part(&part.dir).is_err());
    }

    /// What tenant breadth costs a part.
    ///
    /// Row groups are cut at tenant boundaries, so the number of tenants in a
    /// flush is a *lower bound* on the number of row groups — regardless of
    /// `row_group_size`. Parquet carries per-column metadata for every row
    /// group and this engine carries a bloom filter for every row group, so
    /// that bound is a real cost, and the target workload is many small
    /// tenants. This measures it rather than assuming it.
    ///
    /// The unit is a (tenant, part) pair, not a row: the fixed cost below is
    /// spent per tenant per part it appears in, so it is largest in ratio for
    /// the tenants with the least data. `index_resident_bytes` is called out
    /// separately because the local cache budget does not cover it — that part
    /// stays in memory for as long as the part is open.
    #[test]
    fn tenant_breadth_sets_the_row_group_floor_and_what_that_costs() {
        fn build(tenants: usize, rows_total: usize) -> (u32, u64, u64, u64, u64, u64) {
            let tmp = tempfile_dir();
            let rows: Vec<Row> = (0..rows_total)
                .map(|index| Row {
                    tenant: TenantId::parse(&format!("t{:04}", index % tenants)).unwrap(),
                    timestamp_ns: 1_700_000_000_000_000_000 + index as i64,
                    line: format!("row {index} with enough text to compress like a log line"),
                    structured_metadata: vec![],
                })
                .collect();
            let part = flush_rows(rows, &tmp, 8192).expect("flush").remove(0);
            let on_disk = fs::metadata(part.data_path()).unwrap().len();
            let bloom = fs::metadata(part.index_path()).unwrap().len();
            let reader = PartReader::open(part).expect("open");
            let resident = reader.decoded_blooms().expect("blooms").index_bytes();
            (
                reader.meta().row_group_count,
                on_disk,
                bloom,
                reader.meta().meta_bytes,
                resident,
                reader.meta().materialized_bytes,
            )
        }

        let rows_total = 5_000;
        let (one_groups, one_bytes, one_bloom, one_meta, one_resident, one_materialized) =
            build(1, rows_total);
        let (many_groups, many_bytes, many_bloom, many_meta, many_resident, many_materialized) =
            build(500, rows_total);

        // Same rows, same `row_group_size`; only the tenant breadth differs.
        assert_eq!(one_groups, 1, "one tenant fits one row group");
        assert_eq!(
            many_groups, 500,
            "500 tenants force 500 row groups even though 8192 rows would fit in one"
        );
        assert_eq!(
            one_materialized, many_materialized,
            "the rows themselves are identical in size"
        );

        // The cost is entirely structural. Printed rather than asserted at a
        // threshold: the ratio depends on the parquet writer and the data, and
        // a hard bound here would be a test of zstd, not of this engine.
        println!(
            "tenant fragmentation: 1 tenant = {one_bytes} B / {one_groups} row groups, \
500 tenants = {many_bytes} B / {many_groups} row groups, \
ratio {:.2}x",
            many_bytes as f64 / one_bytes as f64
        );
        println!(
            "  per (tenant, part) pair: parquet {:.0} B, bloom {:.0} B, meta.json {:.0} B, \
resident {:.0} B",
            (many_bytes - one_bytes) as f64 / many_groups as f64,
            (many_bloom - one_bloom) as f64 / many_groups as f64,
            (many_meta - one_meta) as f64 / many_groups as f64,
            (many_resident - one_resident) as f64 / many_groups as f64,
        );
        assert!(
            many_bytes > one_bytes,
            "fragmentation cannot make the file smaller"
        );
        assert!(
            many_resident > one_resident,
            "every extra row group carries its own filters into memory"
        );
    }

    #[test]
    fn bloom_handles_large_trigram_volume_in_single_row_group() {
        // An 8192-row group with dozens of trigrams per line produces several to dozens of times
        // as many unique items as rows. The old implementation (capacity = row count) reached
        // a fill ratio near 99% and produced false positives, while the new implementation
        // (capacity = unique trigram count) accurately prunes absent substrings.
        let tmp = tempfile_dir();
        let rows: Vec<Row> = (0..8192usize)
            .map(|i| Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000 + (i as i64) * 1_000_000,
                line: format!(
                    "log line index {} some random words here for trigrams unique fragment {}",
                    i, i
                ),
                structured_metadata: vec![],
            })
            .collect();
        let parts = flush_rows(rows, &tmp, 8192).expect("flush");
        let reader = PartReader::open(parts.into_iter().next().unwrap()).expect("open");
        let f = LineFilter::Contains("zzzzzz-not-present-substr".to_string());
        let r = reader
            .query(&test_tenant(), &[f], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
            .expect("q");
        let total: usize = r.iter().map(|s| s.entries.len()).sum();
        assert_eq!(
            total, 0,
            "bloom should prune nonexistent substring even with 8192-row group"
        );
    }

    #[test]
    fn merge_flush_renames_part_with_tombstone_already_present() {
        let tmp = tempfile_dir();
        let parts_root = tmp.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let old = flush_rows(
            vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000,
                line: "old".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            100,
        )
        .unwrap()
        .remove(0);

        let merged = flush_rows_with_merge_tombstone(
            vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000,
                line: "merged".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            100,
            std::slice::from_ref(&old.dir),
        )
        .unwrap();
        let new_dir = &merged[0].dir;
        assert!(new_dir.join(MERGE_TOMBSTONE_FILE).exists());
        assert_eq!(
            read_merge_tombstone(new_dir).unwrap(),
            vec![old.dir.strip_prefix(&parts_root).unwrap().to_path_buf()]
        );
    }

    #[test]
    fn discover_keeps_old_parts_when_tombstoned_replacement_is_corrupt() {
        let tmp = tempfile_dir();
        let parts_root = tmp.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let old = flush_rows(
            vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000,
                line: "old".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            100,
        )
        .unwrap()
        .remove(0);
        let merged = flush_rows_with_merge_tombstone(
            vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000,
                line: "merged".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            100,
            std::slice::from_ref(&old.dir),
        )
        .unwrap();
        let new_dir = &merged[0].dir;
        std::fs::write(new_dir.join(INDEX_FILE), b"corrupt").unwrap();

        let discovered = discover_parts(&parts_root).unwrap();
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].dir, old.dir);
        assert!(old.dir.exists());
        assert!(new_dir.join(MERGE_TOMBSTONE_FILE).exists());
    }

    #[test]
    fn merge_tombstone_cleanup_during_discover() {
        // If a crash occurs after renaming the new part and recording the tombstone in merge,
        // but before deleting old_dirs, verify that on restart discover_parts finds the tombstone,
        // cleans up old_dirs, and loads only the one new part.
        let tmp = tempfile_dir();
        let parts_root = tmp.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();

        let parts1 = flush_rows(
            vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000,
                line: "old1 line".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            100,
        )
        .expect("flush1");
        let parts2 = flush_rows(
            vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_002_000_000_000,
                line: "old2 line".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            100,
        )
        .expect("flush2");

        let old_dirs: Vec<PathBuf> = parts1
            .iter()
            .chain(parts2.iter())
            .map(|p| p.dir.clone())
            .collect();

        // Simulated merge: collect rows from two parts and create a new part.
        let merged_rows = vec![
            Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000,
                line: "old1 line".to_string(),
                structured_metadata: vec![],
            },
            Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_002_000_000_000,
                line: "old2 line".to_string(),
                structured_metadata: vec![],
            },
        ];
        let merged_parts = flush_rows(merged_rows, &parts_root, 100).expect("flush merged");

        let new_dir = merged_parts[0].dir.clone();
        write_merge_tombstone(&new_dir, &parts_root, &old_dirs).expect("tombstone write");

        // Simulate a crash: old_dirs remain, and the tombstone is in the new part directory.
        for old_dir in &old_dirs {
            assert!(
                old_dir.exists(),
                "old_dirs should still exist before discover"
            );
        }
        assert!(new_dir.join(MERGE_TOMBSTONE_FILE).exists());

        let discovered = discover_parts(&parts_root).unwrap();
        assert_eq!(
            discovered.len(),
            1,
            "tombstone cleanup should leave only the new part"
        );

        for old_dir in &old_dirs {
            assert!(
                !old_dir.exists(),
                "old part dir {} should be removed by tombstone cleanup",
                old_dir.display()
            );
        }
        assert!(
            !new_dir.join(MERGE_TOMBSTONE_FILE).exists(),
            "tombstone file should be removed during discover"
        );
    }

    #[test]
    fn discover_cleans_transitive_merge_tombstone_chain() {
        let tmp = tempfile_dir();
        let parts_root = tmp.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let row = |line: &str| Row {
            tenant: test_tenant(),
            timestamp_ns: 1_700_000_000_000_000_000,
            line: line.to_string(),
            structured_metadata: vec![],
        };

        let oldest = flush_rows(vec![row("oldest")], &parts_root, 100)
            .unwrap()
            .remove(0);
        let middle = flush_rows_with_merge_tombstone(
            vec![row("middle")],
            &parts_root,
            100,
            std::slice::from_ref(&oldest.dir),
        )
        .unwrap()
        .remove(0);
        let newest = flush_rows_with_merge_tombstone(
            vec![row("newest")],
            &parts_root,
            100,
            std::slice::from_ref(&middle.dir),
        )
        .unwrap()
        .remove(0);

        let discovered = discover_parts(&parts_root).unwrap();

        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].meta.id, newest.meta.id);
        assert!(!oldest.dir.exists());
        assert!(!middle.dir.exists());
        assert!(newest.dir.exists());
        assert!(!newest.dir.join(MERGE_TOMBSTONE_FILE).exists());
    }

    #[test]
    fn discover_rejects_tombstone_paths_outside_parts_root() {
        let tmp = tempfile_dir();
        let parts_root = tmp.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let outside = tmp.join("outside");
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("keep"), b"data").unwrap();
        let replacement = flush_rows(
            vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000,
                line: "replacement".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            100,
        )
        .unwrap()
        .remove(0);
        std::fs::write(
            replacement.dir.join(MERGE_TOMBSTONE_FILE),
            r#"{"old_dirs":["../../outside"]}"#,
        )
        .unwrap();

        let discovered = discover_parts(&parts_root).unwrap();

        assert!(discovered.is_empty());
        assert!(outside.join("keep").exists());
        assert!(replacement.dir.join(MERGE_TOMBSTONE_FILE).exists());
    }

    #[test]
    fn tombstone_path_validation_rejects_absolute_and_parent_paths() {
        assert!(validate_tombstone_part_path(Path::new("/tmp/part")).is_err());
        assert!(validate_tombstone_part_path(Path::new("partition/../part")).is_err());
        assert!(validate_tombstone_part_path(Path::new("partition/part")).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn tombstone_resolution_rejects_symlink_escape() {
        let tmp = tempfile_dir();
        let parts_root = tmp.join("parts");
        let marker_dir = tmp.join("marker");
        let outside_partition = tmp.join("outside-partition");
        std::fs::create_dir_all(&parts_root).unwrap();
        std::fs::create_dir_all(&marker_dir).unwrap();
        std::fs::create_dir_all(outside_partition.join("part")).unwrap();
        std::os::unix::fs::symlink(&outside_partition, parts_root.join("escape")).unwrap();
        std::fs::write(
            marker_dir.join(MERGE_TOMBSTONE_FILE),
            r#"{"old_dirs":["escape/part"]}"#,
        )
        .unwrap();

        let result = read_merge_tombstone_dirs(&marker_dir, &parts_root);

        assert!(result.is_err());
        assert!(outside_partition.join("part").exists());
    }

    #[test]
    fn discover_retains_tombstone_when_old_part_deletion_fails() {
        let tmp = tempfile_dir();
        let parts_root = tmp.join("parts");
        std::fs::create_dir_all(&parts_root).unwrap();
        let replacement = flush_rows(
            vec![Row {
                tenant: test_tenant(),
                timestamp_ns: 1_700_000_000_000_000_000,
                line: "replacement".to_string(),
                structured_metadata: vec![],
            }],
            &parts_root,
            100,
        )
        .unwrap()
        .remove(0);
        let partition = replacement.dir.parent().unwrap();
        let undeletable_as_directory = partition.join("old-part");
        std::fs::write(&undeletable_as_directory, b"not a directory").unwrap();
        let relative = undeletable_as_directory
            .strip_prefix(&parts_root)
            .unwrap()
            .to_string_lossy();
        std::fs::write(
            replacement.dir.join(MERGE_TOMBSTONE_FILE),
            format!(r#"{{"old_dirs":["{relative}"]}}"#),
        )
        .unwrap();

        let discovered = discover_parts(&parts_root).unwrap();

        assert_eq!(discovered.len(), 1);
        assert!(undeletable_as_directory.exists());
        assert!(replacement.dir.join(MERGE_TOMBSTONE_FILE).exists());
    }

    fn tenant_row(tenant: &str, line: &str, timestamp_ns: i64) -> Row {
        Row {
            tenant: TenantId::parse(tenant).unwrap(),
            timestamp_ns,
            line: line.to_string(),
            structured_metadata: vec![("app".to_string(), format!("{tenant}-app"))],
        }
    }

    /// A scan parses the Parquet footer once, not once per row group.
    ///
    /// Opening the body sat inside the row-group loop, so the cost scaled with
    /// the number of groups a query touched — and a part holds at least one row
    /// group per tenant in it, at about 1.3 KB of metadata each (LOAD_RESULTS.md
    /// §2). Nothing about it was per group: both handles are a file descriptor
    /// and an `Arc` over metadata that describes the whole file.
    #[test]
    fn a_scan_opens_the_body_once_however_many_row_groups_it_reads() {
        let tmp = tempfile_dir();
        let rows: Vec<Row> = (0..512)
            .map(|index| tenant_row("solo", &format!("line {index}"), 1_000 + index as i64))
            .collect();
        // One row per group, so a full scan touches every one of them.
        let part = flush_rows(rows, &tmp, 1).unwrap().remove(0);
        assert_eq!(part.meta.row_group_count, 512);
        let reader = PartReader::open(part).unwrap();
        let tenant = TenantId::parse("solo").unwrap();

        let before = PART_DATA_OPENS.with(|opens| opens.get());
        let found = reader
            .query(&tenant, &[], QueryTimeRange::closed(i64::MIN, i64::MAX), usize::MAX, true)
            .unwrap();
        let opens = PART_DATA_OPENS.with(|opens| opens.get()) - before;

        assert_eq!(
            found.iter().map(|stream| stream.entries.len()).sum::<usize>(),
            512,
            "the scan must actually have read every row group"
        );
        assert_eq!(
            opens, 1,
            "a scan over 512 row groups opened the body {opens} times"
        );
    }

    /// A tenant's rows decode from that tenant's bytes and nothing else.
    ///
    /// This is the mechanism range reads rest on, and the reason the decoder is
    /// the push one rather than the async object-store reader: the caller
    /// supplies the bytes, so the scan stays synchronous and the bytes can come
    /// from a cached slice, a range GET, or a mix.
    ///
    /// The assertion that matters is not that the rows come out — it is that
    /// every range the decoder asks for falls inside the segment. That is what
    /// makes fetching one tenant's slice *sufficient*, and it is the same
    /// fail-closed property a per-tenant file would have had.
    #[test]
    fn a_tenant_decodes_from_its_own_byte_range_and_asks_for_nothing_else() {
        use parquet::DecodeResult;
        use parquet::arrow::push_decoder::ParquetPushDecoderBuilder;

        let tmp = tempfile_dir();
        let rows: Vec<Row> = (0..600)
            .map(|index| {
                tenant_row(
                    &format!("tenant{}", index % 3),
                    &format!("line {index} with enough text to compress like a log line"),
                    1_000 + index as i64,
                )
            })
            .collect();
        let part = flush_rows(rows, &tmp, 64).unwrap().remove(0);

        let wanted = TenantId::parse("tenant1").unwrap();
        let segment = part.meta.tenant_segment(&wanted).unwrap().clone();
        let body = fs::read(part.data_path()).unwrap();

        // What a restore would have fetched: the footer, and this tenant's
        // range. Nothing else of the body is available below.
        let metadata = {
            let file = fs::File::open(part.data_path()).unwrap();
            ArrowReaderMetadata::load(&file, Default::default()).unwrap()
        };
        let slice = &body[segment.bytes.start as usize..segment.bytes.end as usize];
        assert_eq!(crc32fast::hash(slice), segment.crc32);

        let mut decoder = ParquetPushDecoderBuilder::new_with_metadata(metadata)
            .with_row_groups(
                (segment.row_group_start..segment.row_group_end)
                    .map(|group| group as usize)
                    .collect(),
            )
            .build()
            .unwrap();

        let mut lines = Vec::new();
        loop {
            match decoder.try_decode().unwrap() {
                DecodeResult::NeedsData(ranges) => {
                    let data = ranges
                        .iter()
                        .map(|range| {
                            assert!(
                                range.start >= segment.bytes.start
                                    && range.end <= segment.bytes.end,
                                "decoder asked for {}..{} outside {}'s range {}..{}",
                                range.start,
                                range.end,
                                wanted,
                                segment.bytes.start,
                                segment.bytes.end
                            );
                            let from = (range.start - segment.bytes.start) as usize;
                            let to = (range.end - segment.bytes.start) as usize;
                            bytes::Bytes::copy_from_slice(&slice[from..to])
                        })
                        .collect();
                    decoder.push_ranges(ranges, data).unwrap();
                }
                DecodeResult::Data(batch) => {
                    let column = batch.column(2).as_string::<i32>();
                    for index in 0..batch.num_rows() {
                        lines.push(column.value(index).to_string());
                    }
                }
                DecodeResult::Finished => break,
            }
        }

        // The same rows an ordinary whole-file read of this tenant returns.
        let reader = PartReader::open(part).unwrap();
        let expected: Vec<String> = reader
            .query(&wanted, &[], QueryTimeRange::closed(i64::MIN, i64::MAX), usize::MAX, true)
            .unwrap()
            .into_iter()
            .flat_map(|stream| stream.entries)
            .map(|entry| entry.line)
            .collect();
        assert_eq!(lines.len(), segment.row_count as usize);
        assert_eq!(lines, expected);
    }

    /// A tenant's recorded byte range really is that tenant's row groups.
    ///
    /// This is the fact a range read rests on, and nothing else in the part can
    /// check it: `data_crc32` covers the whole body, so it says nothing about a
    /// slice. The range is therefore verified three ways — against the Parquet
    /// footer that actually decides the layout, against its own recorded
    /// checksum, and against the other tenants' ranges, because a range that
    /// overlapped a neighbour would fetch rows the tenant may not address.
    #[test]
    fn each_tenant_segment_records_the_byte_range_of_its_own_row_groups() {
        let tmp = tempfile_dir();
        let rows: Vec<Row> = (0..600)
            .map(|index| {
                tenant_row(
                    &format!("tenant{}", index % 3),
                    &format!("line {index} with enough text to compress like a log line"),
                    1_000 + index as i64,
                )
            })
            .collect();
        let part = flush_rows(rows, &tmp, 64).unwrap().remove(0);

        let body = fs::read(part.data_path()).unwrap();
        let reader_metadata = {
            let file = fs::File::open(part.data_path()).unwrap();
            ArrowReaderMetadata::load(&file, Default::default()).unwrap()
        };
        let row_groups = reader_metadata.metadata().row_groups();

        let mut previous_end = None;
        for segment in &part.meta.tenants {
            let range = segment.bytes;
            assert!(!range.is_empty(), "{} recorded an empty range", segment.tenant);

            // Ground truth: every column chunk of every row group the segment
            // claims must fall inside the range, and no other row group may.
            for (ordinal, row_group) in row_groups.iter().enumerate() {
                let ordinal = ordinal as u32;
                let claimed =
                    ordinal >= segment.row_group_start && ordinal < segment.row_group_end;
                for column in row_group.columns() {
                    let (start, length) = column.byte_range();
                    let inside = start >= range.start && start + length <= range.end;
                    assert_eq!(
                        inside, claimed,
                        "row group {ordinal} column chunk at {start}..{} vs {}'s range {}..{}",
                        start + length,
                        segment.tenant,
                        range.start,
                        range.end
                    );
                }
            }

            assert_eq!(
                segment.crc32,
                crc32fast::hash(&body[range.start as usize..range.end as usize]),
                "{} recorded a checksum its own bytes do not produce",
                segment.tenant
            );

            // Segments are written in file order, so a later one starting
            // before the previous ended would mean two tenants share bytes.
            if let Some(end) = previous_end {
                assert!(
                    range.start >= end,
                    "{} starts at {} inside the previous segment ending at {end}",
                    segment.tenant,
                    range.start
                );
            }
            previous_end = Some(range.end);
        }
        assert_eq!(part.meta.tenants.len(), 3);
    }

    #[test]
    fn a_shared_part_confines_every_read_to_the_querying_tenant() {
        let tmp = tempfile_dir();
        // Interleaved in time and supplied out of tenant order, so the part
        // writer has to do the `(tenant, timestamp)` sort itself.
        let rows = vec![
            tenant_row("globex", "globex second", 2_000),
            tenant_row("acme", "acme first", 1_000),
            tenant_row("globex", "globex first", 500),
            tenant_row("acme", "acme second", 3_000),
        ];
        let part = flush_rows(rows, &tmp, 1).unwrap().remove(0);

        let acme = TenantId::parse("acme").unwrap();
        let globex = TenantId::parse("globex").unwrap();
        let outsider = TenantId::parse("initech").unwrap();

        let acme_segment = part.meta.tenant_segment(&acme).unwrap();
        let globex_segment = part.meta.tenant_segment(&globex).unwrap();
        assert_eq!(acme_segment.row_count, 2);
        assert_eq!(globex_segment.row_count, 2);
        assert_eq!(acme_segment.row_group_end, globex_segment.row_group_start);
        assert!(part.meta.tenant_segment(&outsider).is_none());
        // Per-tenant time bounds, not the part-wide range.
        assert_eq!((acme_segment.min_ts_ns, acme_segment.max_ts_ns), (1_000, 3_000));
        assert_eq!(
            (globex_segment.min_ts_ns, globex_segment.max_ts_ns),
            (500, 2_000)
        );

        let reader = PartReader::open(part).unwrap();
        let lines = |tenant: &TenantId| -> Vec<String> {
            reader
                .query(tenant, &[], crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX), 100, true)
                .unwrap()
                .into_iter()
                .flat_map(|stream| stream.entries)
                .map(|entry| entry.line)
                .collect()
        };
        assert_eq!(lines(&acme), vec!["acme first", "acme second"]);
        assert_eq!(lines(&globex), vec!["globex first", "globex second"]);
        assert!(lines(&outsider).is_empty());

        // A predicate that only another tenant satisfies must return nothing
        // rather than reaching across the segment boundary.
        let globex_predicate = [ExactFieldPredicate::new("app", "globex-app")];
        assert!(
            reader
                .query_with_exact_field_pruning(
                    &acme,
                    ExactFieldPruning::new(&[], &globex_predicate),
                    crate::part::QueryTimeRange::closed(i64::MIN, i64::MAX),
                    100,
                    true,
                )
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn row_groups_never_straddle_a_tenant_boundary() {
        let rows = vec![
            tenant_row("acme", "one", 1),
            tenant_row("acme", "two", 2),
            tenant_row("acme", "three", 3),
            tenant_row("globex", "four", 4),
        ];
        // A row group size of 2 would put acme's third row and globex's row in
        // one group if the boundary were not tenant-aligned.
        let bounds = row_group_bounds(&rows, 2);
        assert_eq!(bounds, vec![(0, 2), (2, 3), (3, 4)]);
        for (start, end) in bounds {
            let tenants: BTreeSet<_> = rows[start..end].iter().map(|row| &row.tenant).collect();
            assert_eq!(tenants.len(), 1);
        }
    }

    /// At-least-once delivery makes a second, byte-identical copy of an entry a
    /// normal event rather than a defect: a retried push whose first response
    /// was lost, or a WAL suffix replayed after a crash. Two copies of one entry
    /// must not become two log lines.
    #[test]
    fn identical_entries_are_written_once() {
        let root = tempfile_dir();
        let original = make_rows().remove(0);
        let replayed = original.clone();
        let parts = flush_rows(vec![original, replayed], &root, 100).unwrap();
        let rows: u64 = parts.iter().map(|part| part.meta.row_count).sum();
        assert_eq!(rows, 1);
    }

    /// Only a copy in every field is a copy. A shared timestamp is not enough,
    /// and neither is a shared line — dropping either of these would be losing
    /// data, not deduplicating it.
    #[test]
    fn entries_differing_in_any_field_are_all_kept() {
        let root = tempfile_dir();
        let base = make_rows().remove(0);

        let mut different_line = base.clone();
        different_line.line = "second".to_string();

        let mut different_timestamp = base.clone();
        different_timestamp.timestamp_ns += 1;

        let mut different_metadata = base.clone();
        different_metadata.structured_metadata = vec![("trace".to_string(), "abc".to_string())];

        let mut different_tenant = base.clone();
        different_tenant.tenant = crate::tenant::TenantId::parse("other-tenant").unwrap();

        let parts = flush_rows(
            vec![
                base,
                different_line,
                different_timestamp,
                different_metadata,
                different_tenant,
            ],
            &root,
            100,
        )
        .unwrap();
        let rows: u64 = parts.iter().map(|part| part.meta.row_count).sum();
        assert_eq!(rows, 5);
    }

    /// The container is the first thing read, so a file from a build that laid
    /// it out differently has to be refused rather than parsed as sections.
    #[test]
    fn an_index_file_without_the_container_header_is_refused() {
        assert!(split_index(b"BTF4nonsense").is_err());
        assert!(split_index(b"").is_err());
        // Header present, first length prefix truncated.
        let mut truncated = INDEX_MAGIC.to_vec();
        truncated.extend_from_slice(&[0u8; 2]);
        assert!(split_index(&truncated).is_err());
        // Length prefix claims more than follows it.
        let mut lying = INDEX_MAGIC.to_vec();
        lying.extend_from_slice(&99u32.to_le_bytes());
        lying.extend_from_slice(b"short");
        assert!(split_index(&lying).is_err());
    }

    /// The section comes back exactly as written, and a part opens from it.
    /// Getting the framing wrong would parse the blooms as garbage silently.
    #[test]
    fn the_index_container_round_trips_its_bloom_section() {
        let tmp = tempfile_dir();
        let part = flush_rows(make_rows(), &tmp, 1).unwrap().remove(0);
        let bytes = fs::read(part.index_path()).unwrap();
        let bloom = split_index(&bytes).unwrap();
        assert_eq!(&bloom[..4], BLOOM_MAGIC);
        assert_eq!(
            INDEX_MAGIC.len() + 4 + bloom.len(),
            bytes.len(),
            "the container holds the bloom section and nothing else"
        );
        assert!(PartReader::open(part).is_ok());
    }

    /// A snapshot for the chunked-flush tests: several tenants, several apps,
    /// entries deliberately in arrival order rather than time order, JSON
    /// lines so the `_pf:` columns are exercised.
    fn chunked_test_snapshot() -> MemTableSnapshot {
        let base = 1_700_000_000_000_000_000i64;
        let mut snapshot: MemTableSnapshot = HashMap::new();
        for tenant_name in ["tenant-a", "tenant-b"] {
            let tenant = TenantId::parse(tenant_name).expect("valid");
            let mut entries: Vec<LogEntry> = Vec::new();
            for app in ["api", "worker", "web"] {
                // Reversed timestamps: arrival order is not time order.
                entries.extend((0..37i64).rev().map(|i| LogEntry {
                    timestamp_ns: base + i * 1_000,
                    line: format!("{{\"status\":\"{}\",\"msg\":\"{app} row {i}\"}}", 200 + i % 5),
                    structured_metadata: vec![
                        ("app".to_string(), app.to_string()),
                        ("host".to_string(), tenant_name.to_string()),
                        ("trace_id".to_string(), format!("t{i}")),
                    ],
                }));
            }
            snapshot.insert(tenant, entries);
        }
        snapshot
    }

    fn read_sorted_rows(parts: &[Part]) -> Vec<Row> {
        let readers: Vec<Arc<PartReader>> = parts
            .iter()
            .map(|part| Arc::new(PartReader::open(part.clone()).expect("open")))
            .collect();
        drain(&mut MergedRows::new(&readers, u64::MAX))
    }

    /// The chunked flush is the batch flush with a bounded transient: forced
    /// through cuts small enough to land mid-tenant, it must yield the same
    /// rows in the same order, and every part it writes must be internally
    /// sorted — otherwise the bound was bought with a different on-disk truth.
    #[test]
    fn chunked_flush_equals_batch_flush() {
        let snapshot = chunked_test_snapshot();

        let batch_dir = tempfile_dir();
        let batch_parts =
            flush_rows(rows_from_snapshot(&snapshot), &batch_dir, 8).expect("batch flush");

        let chunked_dir = tempfile_dir();
        // A one-byte budget cuts after every row: the most hostile chunking.
        let chunked_parts =
            flush_snapshot_chunked(&snapshot, &chunked_dir, 8, 1).expect("chunked flush");
        assert!(
            chunked_parts.len() > batch_parts.len(),
            "a one-byte budget must actually produce more parts"
        );

        let batch_rows = read_sorted_rows(&batch_parts);
        let chunked_rows = read_sorted_rows(&chunked_parts);
        assert_eq!(batch_rows.len(), chunked_rows.len());
        for (index, (batch, chunked)) in batch_rows.iter().zip(&chunked_rows).enumerate() {
            assert_eq!(batch.tenant, chunked.tenant, "row {index}");
            assert_eq!(
                batch.timestamp_ns, chunked.timestamp_ns,
                "row {index}: batch line {:?}, chunked line {:?}",
                batch.line, chunked.line
            );
            assert_eq!(batch.line, chunked.line, "row {index}");
            assert_eq!(batch.structured_metadata, chunked.structured_metadata, "row {index}");
        }

        // A generous budget takes the single-chunk path and must agree too.
        let single_dir = tempfile_dir();
        let single_parts =
            flush_snapshot_chunked(&snapshot, &single_dir, 8, u64::MAX).expect("single chunk");
        let single_rows = read_sorted_rows(&single_parts);
        assert_eq!(single_rows.len(), batch_rows.len());
    }

    /// A run of entries that crosses midnight lands in two partition
    /// directories no matter where the chunk cuts fall, and a scan over all
    /// parts returns every row exactly once.
    #[test]
    fn chunked_flush_splits_entries_across_partitions() {
        // 2023-11-14T22:13:20Z; the next day starts 6400 seconds later.
        let base = 1_700_000_000_000_000_000i64;
        let day = partition_of(base);
        let entries: Vec<LogEntry> = (0..20i64)
            .map(|i| LogEntry {
                timestamp_ns: base + i * 700 * 1_000_000_000,
                line: format!("line {i}"),
                structured_metadata: vec![],
            })
            .collect();
        assert_ne!(
            partition_of(entries.last().unwrap().timestamp_ns),
            day,
            "the entries must actually cross midnight"
        );
        let mut snapshot: MemTableSnapshot = HashMap::new();
        snapshot.insert(test_tenant(), entries);

        let dir = tempfile_dir();
        let parts = flush_snapshot_chunked(&snapshot, &dir, 4, 1).expect("chunked flush");
        let partitions: BTreeSet<&str> =
            parts.iter().map(|part| part.meta.partition.as_str()).collect();
        assert_eq!(partitions.len(), 2, "two days, two partition directories");
        for part in &parts {
            assert_eq!(
                part.dir.parent().and_then(|p| p.file_name()),
                Some(std::ffi::OsStr::new(part.meta.partition.as_str())),
                "a part must live under its own partition directory"
            );
        }
        assert_eq!(read_sorted_rows(&parts).len(), 20);
    }

    /// The batch path deduplicated identical `(tenant, ts, line, metadata)`
    /// rows in one global pass. The chunked path deduplicates per tenant at
    /// emission, so a duplicate pair must collapse even when the chunk cut
    /// falls exactly between the two copies.
    #[test]
    fn chunked_flush_dedups_across_a_chunk_cut() {
        let base = 1_700_000_000_000_000_000i64;
        let entry = |ts: i64, line: &str| LogEntry {
            timestamp_ns: base + ts,
            line: line.to_string(),
            structured_metadata: vec![],
        };
        // Shuffled arrival order; the duplicate pair sorts adjacent, and the
        // one-byte budget guarantees a cut lands between them.
        let entries = vec![
            entry(3, "c"),
            entry(1, "twin"),
            entry(2, "b"),
            entry(1, "twin"),
            entry(1, "a"),
        ];
        let mut snapshot: MemTableSnapshot = HashMap::new();
        snapshot.insert(test_tenant(), entries);

        let dir = tempfile_dir();
        let parts = flush_snapshot_chunked(&snapshot, &dir, 4, 1).expect("chunked flush");
        let rows = read_sorted_rows(&parts);
        assert_eq!(rows.len(), 4, "one twin must survive, not both");
        assert_eq!(
            rows_from_snapshot(&snapshot).len(),
            4,
            "the batch path agrees on what a duplicate is"
        );
    }

    /// A later chunk that fails must take every part the earlier chunks
    /// committed with it: the caller aborts the whole flush and retries the
    /// whole snapshot, so a survivor would come back as a duplicate part.
    #[test]
    fn a_failed_chunk_rolls_back_every_committed_part() {
        let base = 1_700_000_000_000_000_000i64;
        let entries: Vec<LogEntry> = (0..4i64)
            .map(|i| LogEntry {
                // Two rows today, two tomorrow.
                timestamp_ns: base + i * 50_000 * 1_000_000_000,
                line: format!("line {i}"),
                structured_metadata: vec![],
            })
            .collect();
        let second_day = partition_of(entries[2].timestamp_ns);
        assert_ne!(partition_of(base), second_day);
        let mut snapshot: MemTableSnapshot = HashMap::new();
        snapshot.insert(test_tenant(), entries);

        let dir = tempfile_dir();
        // A regular file where the second day's partition directory must go:
        // the first chunk (first day) commits, the later one fails.
        fs::write(dir.join(&second_day), b"not a directory").unwrap();

        let result = flush_snapshot_chunked(&snapshot, &dir, 8, 1);
        assert!(result.is_err(), "the flush must report the failure");
        // `.tmp` staging leftovers are invisible to discovery and startup
        // sweeps them; what must not survive is a *visible* part.
        let leftover_parts: Vec<PathBuf> = walk_meta_dirs(&dir)
            .into_iter()
            .filter(|path| !path.starts_with(dir.join(".tmp")))
            .collect();
        assert!(
            leftover_parts.is_empty(),
            "committed chunk parts survived the rollback: {leftover_parts:?}"
        );
    }

    /// A rewrite read must return the part in layout (`Row::sort_key`) order,
    /// because `MergedRows` k-way-merges its pages on that promise and writes
    /// what comes out.
    #[test]
    fn a_rewrite_read_returns_layout_order_not_query_order() {
        let tmp = tempfile_dir();
        let base = 1_700_000_000_000_000_000i64;
        let mut rows: Vec<Row> = Vec::new();
        for app in ["early", "late"] {
            for i in 0..21i64 {
                rows.push(Row {
                    tenant: test_tenant(),
                    timestamp_ns: base + i * 1_000,
                    line: format!("{app} row {i}"),
                    structured_metadata: vec![("app".to_string(), app.to_string())],
                });
            }
        }
        // Groups of 8 over 42 rows, two rows per timestamp: the tie-breaks
        // past the timestamp are what layout order settles.
        let part = flush_rows(rows.clone(), &tmp, 8).expect("flush").remove(0);
        let reader = Arc::new(PartReader::open(part).expect("open"));
        assert!(reader.row_group_count() > 2);

        sort_rows(&mut rows);
        let read = reader.read_all_rows(None).expect("read");
        assert_eq!(read.len(), rows.len());
        for (got, want) in read.iter().zip(&rows) {
            assert_eq!(
                (got.timestamp_ns, got.line.as_str(), &got.structured_metadata),
                (want.timestamp_ns, want.line.as_str(), &want.structured_metadata),
                "a rewrite read left layout order"
            );
        }
    }

    fn window_row(ts: i64, metadata: Vec<(String, String)>) -> Row {
        Row {
            tenant: test_tenant(),
            timestamp_ns: ts,
line: format!("plain text row at {ts}"),
            structured_metadata: metadata,
        }
    }

    /// Window-level exact-field pruning must refine the decode, never the
    /// answer: for a needle planted at every window boundary of a multi-window
    /// group, the pruned query returns exactly what a brute-force filter does,
    /// in both directions, with and without a time window in play.
    #[test]
    fn exact_field_window_pruning_never_drops_a_row() {
        let tmp = tempfile_dir();
        let base = 1_700_000_000_000_000_000i64;
        let planted = [0i64, 1023, 1024, 2047, 2048, 2500];
        let rows: Vec<Row> = (0..3000i64)
            .map(|i| {
                let metadata = if planted.contains(&i) {
                    vec![("needle".to_string(), format!("v{i}"))]
                } else {
                    vec![]
                };
                window_row(base + i, metadata)
            })
            .collect();
        // One 3000-row group: three windows, the last one short.
        let part = flush_rows(rows, &tmp, 4096).unwrap().remove(0);
        let reader = PartReader::open(part).unwrap();

        for target in planted {
            for forward in [true, false] {
                let predicate = [ExactFieldPredicate::new("needle", format!("v{target}"))];
                let results = reader
                    .query_with_exact_field_pruning_and_scan_limit(
                        &test_tenant(),
                        ExactFieldPruning::new(&[], &predicate),
                        QueryTimeRange::closed(i64::MIN, i64::MAX),
                        100,
                        forward,
                        None,
                        None,
                    )
                    .unwrap();
                let found: Vec<i64> = results
                    .results
                    .iter()
                    .flat_map(|stream| stream.entries.iter().map(|entry| entry.timestamp_ns))
                    .collect();
                assert_eq!(found, vec![base + target], "target {target} forward {forward}");
            }
        }

        // The time∩window intersection path: a range holding the window-1
        // needle finds it, and the same range excludes the window-2 one.
        let range = QueryTimeRange::closed(base + 1000, base + 1500);
        for (target, expect) in [(1024i64, 1usize), (2048, 0)] {
            let predicate = [ExactFieldPredicate::new("needle", format!("v{target}"))];
            let results = reader
                .query_with_exact_field_pruning_and_scan_limit(
                    &test_tenant(),
                    ExactFieldPruning::new(&[], &predicate),
                    range,
                    100,
                    true,
                    None,
                    None,
                )
                .unwrap();
            let found: usize = results.results.iter().map(|s| s.entries.len()).sum();
            assert_eq!(found, expect, "target {target} in a bounded window");
        }
    }

    /// The same value on the last row of one window and the first row of the
    /// next: both windows index it, both rows come back.
    #[test]
    fn a_value_straddling_a_window_boundary_returns_both_rows() {
        let tmp = tempfile_dir();
        let base = 1_700_000_000_000_000_000i64;
        let rows: Vec<Row> = (0..2048i64)
            .map(|i| {
                let metadata = if i == 1023 || i == 1024 {
                    vec![("twin".to_string(), "both".to_string())]
                } else {
                    vec![]
                };
                window_row(base + i, metadata)
            })
            .collect();
        let part = flush_rows(rows, &tmp, 4096).unwrap().remove(0);
        let reader = PartReader::open(part).unwrap();
        let predicate = [ExactFieldPredicate::new("twin", "both")];
        let results = reader
            .query_with_exact_field_pruning_and_scan_limit(
                &test_tenant(),
                ExactFieldPruning::new(&[], &predicate),
                QueryTimeRange::closed(i64::MIN, i64::MAX),
                100,
                true,
                None,
                None,
            )
            .unwrap();
        let found: Vec<i64> = results
            .results
            .iter()
            .flat_map(|stream| stream.entries.iter().map(|entry| entry.timestamp_ns))
            .collect();
        assert_eq!(found, vec![base + 1023, base + 1024]);
    }

    /// The window selection is what the narrow pass decodes: a token confined
    /// to one window of an 8192-row group must cost at most that window's
    /// rows, not the group's.
    #[test]
    fn a_windowed_narrow_pass_examines_one_window_not_the_group() {
        let tmp = tempfile_dir();
        let base = 1_700_000_000_000_000_000i64;
        let rows: Vec<Row> = (0..8192i64)
            .map(|i| {
                let metadata = if i == 2050 {
                    vec![("rare".to_string(), "here".to_string())]
                } else {
                    vec![]
                };
                window_row(base + i, metadata)
            })
            .collect();
        let part = flush_rows(rows, &tmp, 8192).unwrap().remove(0);
        let reader = PartReader::open(part).unwrap();
        let predicate = [ExactFieldPredicate::new("rare", "here")];
        let results = reader
            .query_with_exact_field_pruning_and_scan_limit(
                &test_tenant(),
                ExactFieldPruning::new(&[], &predicate),
                QueryTimeRange::closed(i64::MIN, i64::MAX),
                100,
                true,
                None,
                None,
            )
            .unwrap();
        assert_eq!(
            results.results.iter().map(|s| s.entries.len()).sum::<usize>(),
            1
        );
        assert!(
            results.scanned_rows <= crate::part::BLOOM_WINDOW_ROWS,
            "the narrow pass examined {} rows; the window is {}",
            results.scanned_rows,
            crate::part::BLOOM_WINDOW_ROWS
        );
    }

    /// Masks AND across predicates: two tokens that never share a window
    /// prune the whole group, and one row carrying both admits it again.
    #[test]
    fn cross_predicate_window_masks_intersect() {
        let tmp = tempfile_dir();
        let base = 1_700_000_000_000_000_000i64;
        let build = |joint: bool| -> Vec<Row> {
            (0..2048i64)
                .map(|i| {
                    let metadata = if i == 5 {
                        vec![("a".to_string(), "x".to_string())]
                    } else if i == 1035 {
                        if joint {
                            vec![
                                ("a".to_string(), "x".to_string()),
                                ("b".to_string(), "y".to_string()),
                            ]
                        } else {
                            vec![("b".to_string(), "y".to_string())]
                        }
                    } else {
                        vec![]
                    };
                    window_row(base + i, metadata)
                })
                .collect()
        };
        let predicates = [
            ExactFieldPredicate::new("a", "x"),
            ExactFieldPredicate::new("b", "y"),
        ];

        let disjoint = flush_rows(build(false), &tmp, 4096).unwrap().remove(0);
        let reader = PartReader::open(disjoint).unwrap();
        assert_eq!(
            reader.select_row_groups_with_exact_fields(
                &test_tenant(),
                &[],
                &predicates,
                QueryTimeRange::closed(i64::MIN, i64::MAX),
            ).unwrap(),
            Vec::<u32>::new(),
            "tokens in disjoint windows cannot share a row"
        );

        let tmp2 = tempfile_dir();
        let joint = flush_rows(build(true), &tmp2, 4096).unwrap().remove(0);
        let reader = PartReader::open(joint).unwrap();
        let results = reader
            .query_with_exact_field_pruning_and_scan_limit(
                &test_tenant(),
                ExactFieldPruning::new(&[], &predicates),
                QueryTimeRange::closed(i64::MIN, i64::MAX),
                100,
                true,
                None,
                None,
            )
            .unwrap();
        let found: Vec<i64> = results
            .results
            .iter()
            .flat_map(|stream| stream.entries.iter().map(|entry| entry.timestamp_ns))
            .collect();
        assert_eq!(found, vec![base + 1035]);
    }

    /// The byte-identity guarantee extended to multi-window groups: both
    /// writers must window their exact-field filters identically, short last
    /// window included.
    #[test]
    fn the_streaming_writer_windows_its_blooms_identically() {
        let tmp = tempfile_dir();
        let base = 1_700_000_000_000_000_000i64;
        let rows: Vec<Row> = (0..2500i64)
            .map(|i| {
                window_row(
                    base + i,
                    vec![("trace_id".to_string(), format!("t{}", i % 700))],
                )
            })
            .collect();

        let batch = flush_rows(rows.clone(), &tmp, 4096).expect("flush").remove(0);
        let streamed_dir = tmp
            .join("streamed")
            .join(&batch.meta.partition)
            .join(&batch.meta.id);
        std::fs::create_dir_all(&streamed_dir).expect("mkdir");
        let metadata_keys: Vec<String> = select_metadata_columns(metadata_column_counts(&rows))
            .into_iter()
            .map(|(key, _)| key)
            .collect();
        let parsed_keys: Vec<String> =
            select_metadata_columns(parsed_column_counts(&parse_rows(&rows)))
                .into_iter()
                .map(|(key, _)| key)
                .collect();
        let mut writer = StreamingPartWriter::create(
            &streamed_dir,
            metadata_keys,
            parsed_keys,
            4096,
        )
        .expect("create");
        let mut sorted = rows;
        sort_rows(&mut sorted);
        for row in sorted {
            writer.push(row).expect("push");
        }
        writer
            .finish(&streamed_dir, &batch.meta.id, &batch.meta.partition)
            .expect("finish");

        let batch_index = std::fs::read(batch.dir.join(INDEX_FILE)).expect("batch index");
        let streamed_index = std::fs::read(streamed_dir.join(INDEX_FILE)).expect("streamed index");
        assert_eq!(
            batch_index, streamed_index,
            "multi-window blooms must be byte-identical across writers"
        );
    }

    fn cache_enabled(part: Part, budget: u64) -> PartReader {
        let mut reader = PartReader::open(part).unwrap();
        reader.group_cache = GroupCache::new(
            Arc::new(std::sync::atomic::AtomicU64::new(0)),
            Some(budget),
        );
        reader
    }

    fn cache_fixture(tmp: &Path) -> (Part, Vec<Row>) {
        let base = 1_700_000_000_000_000_000i64;
        let mut rows = Vec::new();
        for (index, app) in ["aa", "bb", "cc"].iter().enumerate() {
            for i in 0..40i64 {
                rows.push(Row {
                    tenant: test_tenant(),
                    timestamp_ns: base + index as i64 * 1000 + i,
                    line: format!("{app} row {i}"),
                    structured_metadata: vec![
                        ("app".to_string(), app.to_string()),
                        ("trace_id".to_string(), format!("{app}-{i}")),
                    ],
                });
            }
        }
        let part = flush_rows(rows.clone(), tmp, 16).unwrap().remove(0);
        (part, rows)
    }

    /// A rewrite's read leaves the cache alone.
    ///
    /// `read_all_rows` is merge's way in, and it passes a row-group window —
    /// which is what tells the scan its decodes serve nobody: the parts are
    /// about to be replaced and their readers dropped. Filling from there
    /// spent the query cache's budget on batches no query would ask for and,
    /// since memprof attributes at allocation and the cache only retains,
    /// charged those bytes to the merge arena.
    #[test]
    fn a_merge_read_does_not_fill_the_group_cache() {
        let tmp = tempfile_dir();
        let (part, rows) = cache_fixture(&tmp);
        let cached = cache_enabled(part, 1 << 30);

        let read = cached.read_all_rows(None).expect("read every row");
        assert_eq!(read.len(), rows.len(), "the rewrite must still see them all");
        assert_eq!(
            cached.group_cache.resident_bytes_for_test(),
            0,
            "a rewrite's read must not spend the query cache's budget"
        );

        // The same reader still fills for a query: the gate is on the caller,
        // not on the reader.
        let full = QueryTimeRange::closed(i64::MIN, i64::MAX);
        cached
            .query(&test_tenant(), &[], full, 1000, false)
            .expect("query");
        assert!(
            cached.group_cache.resident_bytes_for_test() > 0,
            "a query's read still fills"
        );
    }

    /// Shrinking hands back Arrow's overshoot and changes nothing else.
    #[test]
    fn shrinking_a_batch_keeps_its_rows_and_drops_its_slack() {
        let mut builder = arrow::array::StringBuilder::with_capacity(8, 1 << 20);
        for i in 0..8 {
            builder.append_value(format!("row {i}"));
        }
        let schema = Arc::new(Schema::new(vec![Field::new("line", DataType::Utf8, false)]));
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(builder.finish()) as ArrayRef]).unwrap();
        let before = batch.get_array_memory_size();
        let expected: Vec<Option<&str>> = batch.column(0).as_string::<i32>().iter().collect();
        let expected: Vec<Option<String>> = expected
            .into_iter()
            .map(|value| value.map(str::to_string))
            .collect();
        let rows = batch.num_rows();

        // Moved in, not cloned: Arrow declines to shrink a shared buffer, so a
        // surviving handle here would make the assertion below vacuous.
        let shrunk = shrink_batches(vec![batch]).remove(0);
        assert_eq!(shrunk.num_rows(), rows);
        let after: Vec<Option<String>> = shrunk
            .column(0)
            .as_string::<i32>()
            .iter()
            .map(|value| value.map(str::to_string))
            .collect();
        assert_eq!(after, expected);
        assert!(
            shrunk.get_array_memory_size() < before,
            "shrinking must return the slack: {before} -> {}",
            shrunk.get_array_memory_size()
        );
    }

    /// A cache hit is invisible in the answer: the same queries return the
    /// same rows in the same order before and after the group is cached, in
    /// both directions, matchers and exact-field predicates included.
    #[test]
    fn a_cached_group_answers_identically_to_a_decoded_one() {
        let tmp = tempfile_dir();
        let (part, _) = cache_fixture(&tmp);
        let plain = PartReader::open(part.clone()).unwrap();
        let cached = cache_enabled(part, 1 << 30);

        let full = QueryTimeRange::closed(i64::MIN, i64::MAX);
        let broad = |reader: &PartReader, forward: bool| {
            reader
                .query(&test_tenant(), &[], full, 1000, forward)
                .unwrap()
                .iter()
                .flat_map(|s| s.entries.iter().map(|e| (e.timestamp_ns, e.line.clone())))
                .collect::<Vec<_>>()
        };
        // First scan fills (backward decodes groups whole), later scans hit.
        let fill = broad(&cached, false);
        assert!(
            cached.group_cache.resident_bytes_for_test() > 0,
            "the fill scan must actually populate the cache, \
or every equality below is vacuous"
        );
        assert!(
            crate::part::row_group_cache_bytes() == 0,
            "the test cache must not touch the global counter"
        );
        for forward in [true, false] {
            assert_eq!(broad(&cached, forward), broad(&plain, forward));
        }

        // Exact-field predicate served through the cached narrow pass.
        let predicate = [ExactFieldPredicate::new("trace_id", "bb-7")];
        for forward in [true, false] {
            let answer = |reader: &PartReader| {
                reader
                    .query_with_exact_field_pruning_and_scan_limit(
                        &test_tenant(),
                        ExactFieldPruning::new(&[], &predicate),
                        full,
                        100,
                        forward,
                        None,
                        None,
                    )
                    .unwrap()
                    .results
                    .iter()
                    .flat_map(|s| s.entries.iter().map(|e| e.timestamp_ns))
                    .collect::<Vec<_>>()
            };
            assert_eq!(answer(&cached), answer(&plain), "forward {forward}");
        }

        // A selective predicate evaluates identically through the hit path.
        let app_cc = [ExactFieldPredicate::new("app", "cc")];
        let matched = cached
            .query_with_exact_field_pruning(
                &test_tenant(),
                ExactFieldPruning::new(&[], &app_cc),
                full,
                1000,
                true,
            )
            .unwrap()
            .iter()
            .map(|s| s.entries.len())
            .sum::<usize>();
        assert_eq!(matched, 40);
        assert_eq!(fill.len(), 120);
    }

    /// The comparison bed's shape: a sub-window query whose time page
    /// selection keeps only part of each group. The decode is cached under
    /// that selection and an identical repeat replays it — this is the warm
    /// pass — with the same answer as an uncached reader.
    #[test]
    fn a_repeated_sub_window_query_replays_its_cached_decode() {
        let tmp = tempfile_dir();
        let (part, _) = cache_fixture(&tmp);
        let plain = PartReader::open(part.clone()).unwrap();
        let cached = cache_enabled(part, 1 << 30);

        let base = 1_700_000_000_000_000_000i64;
        let window = QueryTimeRange::closed(base + 10, base + 2010);
        let answer = |reader: &PartReader, forward: bool| {
            reader
                .query(&test_tenant(), &[], window, 1000, forward)
                .unwrap()
                .iter()
                .flat_map(|s| s.entries.iter().map(|e| (e.timestamp_ns, e.line.clone())))
                .collect::<Vec<_>>()
        };
        let first = answer(&cached, false);
        assert!(
            cached.group_cache.resident_bytes_for_test() > 0,
            "a sub-window decode must fill the cache — the bed's queries \
are all sub-windows, and a whole-group-only fill never fires there"
        );
        for forward in [true, false] {
            assert_eq!(answer(&cached, forward), answer(&plain, forward));
        }
        assert_eq!(first, answer(&plain, false));
    }

    /// A rare-shape query's narrow pass produces a row-exact selection; the
    /// wide decode of those rows is cached under it and an identical repeat
    /// replays it without touching Parquet.
    #[test]
    fn a_repeated_exact_field_query_replays_its_cached_decode() {
        let tmp = tempfile_dir();
        let (part, _) = cache_fixture(&tmp);
        let plain = PartReader::open(part.clone()).unwrap();
        let cached = cache_enabled(part, 1 << 30);

        let full = QueryTimeRange::closed(i64::MIN, i64::MAX);
        let predicate = [ExactFieldPredicate::new("trace_id", "bb-7")];
        let answer = |reader: &PartReader, forward: bool| {
            reader
                .query_with_exact_field_pruning_and_scan_limit(
                    &test_tenant(),
                    ExactFieldPruning::new(&[], &predicate),
                    full,
                    100,
                    forward,
                    None,
                    None,
                )
                .unwrap()
                .results
                .iter()
                .flat_map(|s| s.entries.iter().map(|e| (e.timestamp_ns, e.line.clone())))
                .collect::<Vec<_>>()
        };
        let first = answer(&cached, false);
        assert_eq!(first.len(), 1, "the fixture holds exactly one bb-7 row");
        assert!(
            cached.group_cache.resident_bytes_for_test() > 0,
            "a selection-narrowed decode must fill the cache under its \
selection, or warm rare shapes never hit"
        );
        for forward in [true, false] {
            assert_eq!(answer(&cached, forward), answer(&plain, forward));
        }

        // The narrow pass is remembered too: a repeat examines no rows —
        // both the selection and the rejections replay from the cache.
        let scanned = |reader: &PartReader| {
            reader
                .query_with_exact_field_pruning_and_scan_limit(
                    &test_tenant(),
                    ExactFieldPruning::new(&[], &predicate),
                    full,
                    100,
                    false,
                    None,
                    None,
                )
                .unwrap()
                .scanned_rows
        };
        let repeat = scanned(&cached);
        let uncached = scanned(&plain);
        assert!(
            repeat < uncached,
            "a repeated rare query must replay the narrow pass \
(repeat scanned {repeat}, uncached scans {uncached})"
        );
    }

    /// A narrowed selection is a subset of the base a broad query cached, so
    /// the wide pass is served by slicing that entry — no builder — and the
    /// answer equals the decode path's exactly.
    #[test]
    fn a_narrowed_wide_pass_is_served_by_slicing_the_cached_base_entry() {
        let tmp = tempfile_dir();
        // The subset serve needs what the bed's json_field has: a base
        // selection that trims pages (a sub-window over a group larger than
        // one page), a predicate whose value sits in every window (dense
        // mask, so its base equals the broad query's), and a narrow result
        // that keeps a proper subset. 4000 rows in one 4096-row group give
        // four 1024-row pages; `env` alternates so half of every page
        // matches.
        let base = 1_700_000_000_000_000_000i64;
        let rows: Vec<Row> = (0..4000i64)
            .map(|i| Row {
                tenant: test_tenant(),
                timestamp_ns: base + i,
                line: format!("row {i}"),
                structured_metadata: vec![(
                    "env".to_string(),
                    if i % 2 == 0 { "prod" } else { "dev" }.to_string(),
                )],
            })
            .collect();
        let part = flush_rows(rows, &tmp, 4096).unwrap().remove(0);
        let plain = PartReader::open(part.clone()).unwrap();
        let cached = cache_enabled(part, 1 << 30);

        let window = QueryTimeRange::closed(base + 100, base + 1900);
        // Broad fill under the window's base key.
        cached
            .query(&test_tenant(), &[], window, 5000, false)
            .unwrap();
        assert!(cached.group_cache.resident_bytes_for_test() > 0);

        let predicate = [ExactFieldPredicate::new("env", "prod")];
        let answer = |reader: &PartReader, forward: bool| {
            reader
                .query_with_exact_field_pruning_and_scan_limit(
                    &test_tenant(),
                    ExactFieldPruning::new(&[], &predicate),
                    window,
                    5000,
                    forward,
                    None,
                    None,
                )
                .unwrap()
                .results
                .iter()
                .flat_map(|s| s.entries.iter().map(|e| (e.timestamp_ns, e.line.clone())))
                .collect::<Vec<_>>()
        };
        for forward in [true, false] {
            assert_eq!(answer(&cached, forward), answer(&plain, forward));
        }
        assert!(
            cached
                .group_cache
                .subset_serves
                .load(std::sync::atomic::Ordering::Acquire)
                > 0,
            "the narrowed wide pass must have been served by slicing, \
not by the decode fallback"
        );
    }

    /// A metric scan — named columns, no line — reads the cache through a
    /// re-addressed view of the full-projection batches, and answers exactly
    /// what its own narrow decode answers. A named decode must not fill: its
    /// batches cannot serve later full-projection callers.
    #[test]
    fn a_named_scan_is_served_from_the_cache_through_a_view() {
        let tmp = tempfile_dir();
        let (part, _) = cache_fixture(&tmp);
        let plain = PartReader::open(part.clone()).unwrap();
        let cached = cache_enabled(part, 1 << 30);

        let base = 1_700_000_000_000_000_000i64;
        let window = QueryTimeRange::closed(base + 10, base + 2010);
        let metric_columns = ColumnSet {
            line: false,
            metadata: crate::part::MetadataProjection::Named(Default::default()),
            parsed_fields: false,
        };
        let tenant = test_tenant();
        let named_rows = |reader: &PartReader| {
            let mut collector = RowCollector::new(&tenant);
            reader
                .scan_into(
                    &tenant,
                    &[],
                    &[],
                    window,
                    true,
                    None,
                    None,
                    None,
                    None,
                    &metric_columns,
                    &mut collector,
                )
                .unwrap();
            collector
                .into_rows()
                .iter()
                .map(|row| (row.timestamp_ns, row.structured_metadata.clone()))
                .collect::<Vec<_>>()
        };
        // A named decode alone fills nothing.
        let uncached = named_rows(&cached);
        assert_eq!(
            cached.group_cache.resident_bytes_for_test(),
            0,
            "a named decode must not fill the cache"
        );
        // A broad query fills; the named scan then reads the same batches
        // through the view.
        cached
            .query(&test_tenant(), &[], window, 1000, false)
            .unwrap();
        assert!(cached.group_cache.resident_bytes_for_test() > 0);
        let served = named_rows(&cached);
        assert_eq!(served, uncached);
        assert_eq!(served, named_rows(&plain));
    }

    /// Two readers share one byte budget; a reader whose own cache is empty
    /// cannot evict the other's entries, so when the budget is already
    /// spent it retracts its own insert instead of holding the total over.
    #[test]
    fn a_reader_retracts_its_insert_rather_than_exceed_the_shared_budget() {
        let tmp = tempfile_dir();
        let (part, _) = cache_fixture(&tmp);

        let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));
        // First reader fills whatever the broad query decodes.
        let mut first = PartReader::open(part.clone()).unwrap();
        first.group_cache = GroupCache::new(counter.clone(), Some(1 << 30));
        first
            .query(&test_tenant(), &[],
                QueryTimeRange::closed(i64::MIN, i64::MAX),
                1000,
                false,
            )
            .unwrap();
        let held = counter.load(std::sync::atomic::Ordering::Acquire);
        assert!(held > 0);

        // Second reader under the same counter, with a budget the first
        // reader's residency already exhausts.
        let mut second = PartReader::open(part).unwrap();
        second.group_cache = GroupCache::new(counter.clone(), Some(held));
        second
            .query(&test_tenant(), &[],
                QueryTimeRange::closed(i64::MIN, i64::MAX),
                1000,
                false,
            )
            .unwrap();
        assert_eq!(
            counter.load(std::sync::atomic::Ordering::Acquire),
            held,
            "the shared total must not exceed the budget the first reader filled"
        );
        assert_eq!(second.group_cache.resident_bytes_for_test(), 0);
    }

    /// A scan that stops early leaves nothing behind: only a completed
    /// whole-group decode is cacheable.
    #[test]
    fn a_stopped_scan_does_not_cache_a_partial_group() {
        let tmp = tempfile_dir();
        let (part, _) = cache_fixture(&tmp);
        let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let mut reader = PartReader::open(part).unwrap();
        reader.group_cache = GroupCache::new(counter.clone(), Some(1 << 30));
        // A query limit alone does not stop the scan mid-group — a full sink
        // raises the frontier and the remaining rows are *skipped*, so the
        // group still decodes whole (and is then legitimately cacheable). The
        // scanned-rows quota is the lever that stops inside a group.
        reader
            .query_with_exact_field_pruning_and_scan_limit(
                &test_tenant(),
                ExactFieldPruning::new(&[], &[]),
                QueryTimeRange::closed(i64::MIN, i64::MAX),
                1000,
                true,
                Some(4),
                None,
            )
            .unwrap();
        assert_eq!(
            counter.load(std::sync::atomic::Ordering::Acquire),
            0,
            "a partial decode must not be cached"
        );
    }

    /// The budget evicts least-recently-used groups and the shared counter
    /// follows every insert, eviction and drop.
    #[test]
    fn the_group_cache_budget_evicts_and_the_counter_balances() {
        let tmp = tempfile_dir();
        let (part, _) = cache_fixture(&tmp);
        let plain = PartReader::open(part.clone()).unwrap();
        let group_count = plain.row_group_count();
        assert!(group_count >= 3, "fixture must span several groups");

        let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));
        // Budget sized to roughly one group so inserts evict predecessors.
        let one_group = {
            let mut reader = PartReader::open(part.clone()).unwrap();
            reader.group_cache =
                GroupCache::new(Arc::new(std::sync::atomic::AtomicU64::new(0)), Some(1 << 30));
            reader
                .query(&test_tenant(), &[],
                    QueryTimeRange::closed(i64::MIN, i64::MAX),
                    1000,
                    false,
                )
                .unwrap();
            reader.group_cache.resident_bytes_for_test() / group_count as u64
        };
        let mut reader = PartReader::open(part).unwrap();
        reader.group_cache = GroupCache::new(counter.clone(), Some(one_group * 2));
        reader
            .query(&test_tenant(), &[],
                QueryTimeRange::closed(i64::MIN, i64::MAX),
                1000,
                false,
            )
            .unwrap();
        let held = counter.load(std::sync::atomic::Ordering::Acquire);
        assert!(held > 0, "something must stay cached (one_group={one_group}, groups={group_count})");
        assert!(
            held <= one_group * 2,
            "held {held} exceeds the {} budget",
            one_group * 2
        );
        drop(reader);
        assert_eq!(
            counter.load(std::sync::atomic::Ordering::Acquire),
            0,
            "dropping the reader must return every byte"
        );
    }

    fn walk_meta_dirs(root: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(read_dir) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if path.join(META_FILE).exists() {
                        found.push(path);
                    } else {
                        stack.push(path);
                    }
                }
            }
        }
        found
    }


    /// A struct so a panicking assertion still restores the unbounded
    /// default: a leaked budget of one byte would evict every other test's
    /// blooms — correct but slow, and confusing in an unrelated failure.
    struct BloomBudgetGuard;
    impl Drop for BloomBudgetGuard {
        fn drop(&mut self) {
            configure_bloom_cache(None);
        }
    }

    #[test]
    fn evicted_blooms_reload_and_the_answer_does_not_change() {
        let tmp_a = tempfile_dir();
        let tmp_b = tempfile_dir();
        let part_a = flush_rows(make_rows(), &tmp_a, 100).unwrap().remove(0);
        let part_b = flush_rows(make_rows(), &tmp_b, 100).unwrap().remove(0);

        let filter = LineFilter::Contains("error connecting".to_string());
        let ask = |reader: &PartReader| -> Vec<String> {
            reader
                .query(
                    &test_tenant(),
                    std::slice::from_ref(&filter),
                    QueryTimeRange::closed(i64::MIN, i64::MAX),
                    100,
                    true,
                )
                .expect("query")
                .iter()
                .flat_map(|stream| stream.entries.iter().map(|entry| entry.line.clone()))
                .collect()
        };

        // One byte: any install pushes the total over budget, so every open
        // evicts every other slot. Answers must not change under that.
        let _guard = BloomBudgetGuard;
        configure_bloom_cache(Some(1));

        let reader_a = PartReader::open(part_a).expect("open a");
        let resident = ask(&reader_a);
        assert_eq!(
            resident,
            vec!["error connecting to database".to_string()],
            "the filter must match exactly the one seeded line"
        );

        let _reader_b = PartReader::open(part_b).expect("open b");
        assert!(
            reader_a.blooms.get().is_none(),
            "the second install must evict the first reader's blooms"
        );

        let evicted = ask(&reader_a);
        assert_eq!(
            evicted, resident,
            "a query over evicted blooms must re-read index.bin and answer identically"
        );
    }

    #[test]
    fn bloom_bytes_are_returned_on_eviction_and_on_drop() {
        // Deltas against the global total race with parallel tests, so the
        // probe bytes are absurdly large: nothing else in the suite reaches
        // a terabyte of accounted blooms.
        const PROBE: u64 = 1 << 40;
        let tmp = tempfile_dir();
        let part = flush_rows(make_rows(), &tmp, 100).unwrap().remove(0);
        let reader = PartReader::open(part).unwrap();
        let decoded = reader.decoded_blooms().unwrap();

        let slot = BloomSlot::new();
        slot.install(decoded.clone(), PROBE);
        assert!(
            bloom_cache_bytes() >= PROBE,
            "an installed slot's bytes must be in the global total"
        );
        // A reinstall replaces the old accounting rather than adding to it.
        slot.install(decoded.clone(), PROBE);
        assert!(
            bloom_cache_bytes() < 2 * PROBE,
            "a reinstall must not double-count the slot"
        );
        slot.remove();
        assert!(
            bloom_cache_bytes() < PROBE,
            "removal must give the bytes back"
        );

        // Drop without an explicit remove is the reader's own path.
        let slot = BloomSlot::new();
        slot.install(decoded, PROBE);
        assert!(bloom_cache_bytes() >= PROBE);
        slot.remove();
        assert!(bloom_cache_bytes() < PROBE);
    }

    /// The build phase's sub-timers observe the flush path and not the merge.
    ///
    /// They live in a `static` shared by both callers, so this discriminator is
    /// the only thing keeping a merge rewrite out of the distribution that
    /// exists to explain the flush ceiling. If it ever stops meaning "this is a
    /// merge", the numbers become a blend of two workloads and nothing else
    /// would say so.
    ///
    /// **What this cannot check**, said rather than implied: it pins the
    /// meaning of the discriminator, not that `flush_rows_internal` still calls
    /// it. Asserting on `FLUSH_BUILD`'s counters would check that — and would
    /// be a false failure the moment another test in this binary flushes a part
    /// on another thread, which is most of them.
    #[test]
    fn only_the_flush_path_measures_the_build_sub_phases() {
        assert!(measures_build(None), "a flush is measured");
        assert!(!measures_build(Some(&[])), "a merge rewrite is not");
        assert!(!measures_build(Some(&[PathBuf::from("/old/part")])));
    }
