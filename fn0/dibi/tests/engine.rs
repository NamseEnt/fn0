use std::{sync::Arc, thread};

use bytes::Bytes;
use dibi::{
    AdminScanRequest, ApplicationWrite, CommitMutation, ConditionalWrite, ConditionalWriteOutcome,
    DibiEngine, encode_document_key, encode_document_value,
};
use tempfile::TempDir;

fn open_temporary() -> (TempDir, DibiEngine) {
    let directory = tempfile::tempdir().unwrap();
    let engine = DibiEngine::open(directory.path()).unwrap();
    (directory, engine)
}

#[test]
fn basic_versions_and_reopen_persistence() {
    let directory = tempfile::tempdir().unwrap();
    let engine = DibiEngine::open(directory.path()).unwrap();
    let database_uuid = engine.db_uuid().unwrap();

    assert_eq!(engine.get("tenant-a", "pk", "sk").unwrap(), None);
    assert_eq!(
        engine
            .put("tenant-a", "pk", "sk", Bytes::from_static(b"\0binary\xff"))
            .unwrap(),
        1
    );
    assert_eq!(
        engine
            .get("tenant-a", "pk", "sk")
            .unwrap()
            .unwrap()
            .as_ref(),
        b"\0binary\xff"
    );
    assert_eq!(
        engine
            .get_with_version("tenant-a", "pk", "sk")
            .unwrap()
            .unwrap()
            .version,
        0
    );
    engine
        .put("tenant-a", "pk", "sk", Bytes::from_static(b"second"))
        .unwrap();
    assert_eq!(
        engine
            .get_with_version("tenant-a", "pk", "sk")
            .unwrap()
            .unwrap()
            .version,
        1
    );
    engine
        .put("tenant-a", "pk", "sk", Bytes::from_static(b"third"))
        .unwrap();
    assert_eq!(
        engine
            .get_with_version("tenant-a", "pk", "sk")
            .unwrap()
            .unwrap()
            .version,
        2
    );
    engine.delete("tenant-a", "pk", "sk").unwrap();
    assert_eq!(engine.get("tenant-a", "pk", "sk").unwrap(), None);
    engine.delete("tenant-a", "missing", "key").unwrap();
    engine
        .put(
            "tenant-a",
            "pk",
            "sk",
            Bytes::from_static(b"new incarnation"),
        )
        .unwrap();
    assert_eq!(
        engine
            .get_with_version("tenant-a", "pk", "sk")
            .unwrap()
            .unwrap()
            .version,
        0
    );
    assert_eq!(engine.last_commit_id().unwrap(), 6);
    let outbox_before = engine.read_outbox(None, 100).unwrap();
    drop(engine);

    let reopened = DibiEngine::open(directory.path()).unwrap();
    assert_eq!(reopened.db_uuid().unwrap(), database_uuid);
    assert_eq!(reopened.last_commit_id().unwrap(), 6);
    assert_eq!(
        reopened
            .get_with_version("tenant-a", "pk", "sk")
            .unwrap()
            .unwrap()
            .data,
        Bytes::from_static(b"new incarnation")
    );
    assert_eq!(reopened.read_outbox(None, 100).unwrap(), outbox_before);
}

#[test]
fn query_and_scan_preserve_order_and_exclusive_cursors() {
    let values = ["", "a", "a\0b", "a/b", "a&b", "한글", "日本語", "😀"];

    let (_query_directory, query_engine) = open_temporary();
    for value in values {
        query_engine
            .put(
                "tenant-a",
                "partition",
                value,
                Bytes::copy_from_slice(value.as_bytes()),
            )
            .unwrap();
    }
    query_engine
        .put("tenant-a", "other", "z", Bytes::from_static(b"ignored"))
        .unwrap();
    let mut expected = values.to_vec();
    expected.sort();
    let mut found = Vec::new();
    let mut cursor = None;
    loop {
        let page = query_engine
            .query("tenant-a", "partition", cursor.as_deref(), 3)
            .unwrap();
        if page.is_empty() {
            break;
        }
        cursor = page.last().map(|document| document.sk.clone());
        found.extend(page.into_iter().map(|document| document.sk));
    }
    assert_eq!(found, expected);

    let (_scan_directory, scan_engine) = open_temporary();
    for value in values {
        scan_engine
            .put(
                "tenant-a",
                value,
                value,
                Bytes::copy_from_slice(value.as_bytes()),
            )
            .unwrap();
    }
    let mut expected_pairs = values
        .into_iter()
        .map(|value| (value.to_owned(), value.to_owned()))
        .collect::<Vec<_>>();
    expected_pairs.sort();
    let mut found_pairs = Vec::new();
    let mut scan_cursor: Option<(String, String)> = None;
    loop {
        let page = scan_engine
            .scan(
                "tenant-a",
                scan_cursor
                    .as_ref()
                    .map(|(pk, sk)| (pk.as_str(), sk.as_str())),
                3,
            )
            .unwrap();
        if page.is_empty() {
            break;
        }
        scan_cursor = page
            .last()
            .map(|document| (document.pk.clone(), document.sk.clone()));
        found_pairs.extend(page.into_iter().map(|document| (document.pk, document.sk)));
    }
    assert_eq!(found_pairs, expected_pairs);
}

#[test]
fn tenant_keyspaces_are_independent_for_reads_scans_and_occ() {
    let (_directory, engine) = open_temporary();
    engine
        .put("tenant-a", "User", "1", Bytes::from_static(b"A"))
        .unwrap();
    engine
        .put("tenant-b", "User", "1", Bytes::from_static(b"B"))
        .unwrap();

    assert_eq!(
        engine.get("tenant-a", "User", "1").unwrap(),
        Some(Bytes::from_static(b"A"))
    );
    assert_eq!(
        engine.get("tenant-b", "User", "1").unwrap(),
        Some(Bytes::from_static(b"B"))
    );
    assert_eq!(
        engine
            .query("tenant-a", "User", None, 10)
            .unwrap()
            .into_iter()
            .map(|document| document.data)
            .collect::<Vec<_>>(),
        vec![Bytes::from_static(b"A")]
    );
    assert_eq!(
        engine
            .scan("tenant-a", None, 10)
            .unwrap()
            .into_iter()
            .map(|document| document.data)
            .collect::<Vec<_>>(),
        vec![Bytes::from_static(b"A")]
    );
    assert_eq!(
        engine
            .admin_scan(
                "tenant-a",
                AdminScanRequest {
                    after: None,
                    limit: 10,
                    pk_prefix: None,
                },
            )
            .unwrap()
            .documents
            .into_iter()
            .map(|document| document.data)
            .collect::<Vec<_>>(),
        vec![Bytes::from_static(b"A")]
    );

    assert!(matches!(
        engine.conditional_write_batch(
            "tenant-a",
            &[ConditionalWrite::Put {
                pk: "User".to_owned(),
                sk: "1".to_owned(),
                expected_version: 0,
                data: Bytes::from_static(b"A2"),
            }],
        ),
        Ok(ConditionalWriteOutcome::Applied(_))
    ));
    assert!(matches!(
        engine.conditional_write_batch(
            "tenant-b",
            &[ConditionalWrite::Put {
                pk: "User".to_owned(),
                sk: "1".to_owned(),
                expected_version: 0,
                data: Bytes::from_static(b"B2"),
            }],
        ),
        Ok(ConditionalWriteOutcome::Applied(_))
    ));
}

#[test]
fn application_batch_uses_sequential_version_semantics() {
    let (_directory, engine) = open_temporary();
    for version in 0..=3 {
        engine
            .put(
                "tenant-a",
                "existing",
                "key",
                Bytes::from(format!("value-{version}")),
            )
            .unwrap();
    }
    engine
        .put(
            "tenant-a",
            "incarnation",
            "key",
            Bytes::from_static(b"old incarnation"),
        )
        .unwrap();
    engine
        .put(
            "tenant-a",
            "removed",
            "key",
            Bytes::from_static(b"old value"),
        )
        .unwrap();
    let result = engine
        .application_write_batch(
            "tenant-a",
            &[
                ApplicationWrite::Put {
                    pk: "existing".to_owned(),
                    sk: "key".to_owned(),
                    data: Bytes::from_static(b"A"),
                },
                ApplicationWrite::Put {
                    pk: "existing".to_owned(),
                    sk: "key".to_owned(),
                    data: Bytes::from_static(b"B"),
                },
                ApplicationWrite::Delete {
                    pk: "incarnation".to_owned(),
                    sk: "key".to_owned(),
                },
                ApplicationWrite::Put {
                    pk: "incarnation".to_owned(),
                    sk: "key".to_owned(),
                    data: Bytes::from_static(b"fresh"),
                },
                ApplicationWrite::Put {
                    pk: "removed".to_owned(),
                    sk: "key".to_owned(),
                    data: Bytes::from_static(b"temporary"),
                },
                ApplicationWrite::Delete {
                    pk: "removed".to_owned(),
                    sk: "key".to_owned(),
                },
                ApplicationWrite::Put {
                    pk: "other".to_owned(),
                    sk: "key".to_owned(),
                    data: Bytes::from_static(b"other"),
                },
            ],
        )
        .unwrap();
    assert_eq!(result.commit_id, Some(7));
    let existing = engine
        .get_with_version("tenant-a", "existing", "key")
        .unwrap()
        .unwrap();
    assert_eq!(existing.data, Bytes::from_static(b"B"));
    assert_eq!(existing.version, 5);
    assert_eq!(
        engine
            .get_with_version("tenant-a", "incarnation", "key")
            .unwrap()
            .unwrap()
            .version,
        0
    );
    assert_eq!(engine.get("tenant-a", "removed", "key").unwrap(), None);
    assert_eq!(
        engine.get("tenant-a", "other", "key").unwrap(),
        Some(Bytes::from_static(b"other"))
    );
}

#[test]
fn conditional_batches_distinguish_conflicts_and_remain_atomic() {
    let (_directory, engine) = open_temporary();
    let create = ConditionalWrite::Create {
        pk: "pk".to_owned(),
        sk: "sk".to_owned(),
        data: Bytes::from_static(b"created"),
    };
    assert!(matches!(
        engine
            .conditional_write_batch("tenant-a", std::slice::from_ref(&create))
            .unwrap(),
        ConditionalWriteOutcome::Applied(_)
    ));
    let commit_after_create = engine.last_commit_id().unwrap();
    let create_conflict = engine
        .conditional_write_batch("tenant-a", &[create])
        .unwrap();
    assert!(matches!(
        create_conflict,
        ConditionalWriteOutcome::Conflict(_)
    ));
    assert_eq!(engine.last_commit_id().unwrap(), commit_after_create);

    assert!(matches!(
        engine
            .conditional_write_batch(
                "tenant-a",
                &[ConditionalWrite::Put {
                    pk: "pk".to_owned(),
                    sk: "sk".to_owned(),
                    expected_version: 0,
                    data: Bytes::from_static(b"updated"),
                }]
            )
            .unwrap(),
        ConditionalWriteOutcome::Applied(_)
    ));
    let stale_put = engine
        .conditional_write_batch(
            "tenant-a",
            &[ConditionalWrite::Put {
                pk: "pk".to_owned(),
                sk: "sk".to_owned(),
                expected_version: 0,
                data: Bytes::from_static(b"stale"),
            }],
        )
        .unwrap();
    assert_eq!(
        stale_put,
        ConditionalWriteOutcome::Conflict(vec![dibi::Conflict {
            pk: "pk".to_owned(),
            sk: "sk".to_owned(),
            expected_version: Some(0),
            actual_version: Some(1),
        }])
    );

    engine
        .put("tenant-a", "other", "key", Bytes::from_static(b"safe"))
        .unwrap();
    let commit_before_multi_conflict = engine.last_commit_id().unwrap();
    let multi_conflict = engine
        .conditional_write_batch(
            "tenant-a",
            &[
                ConditionalWrite::Put {
                    pk: "other".to_owned(),
                    sk: "key".to_owned(),
                    expected_version: 0,
                    data: Bytes::from_static(b"must not apply"),
                },
                ConditionalWrite::Delete {
                    pk: "pk".to_owned(),
                    sk: "sk".to_owned(),
                    expected_version: 0,
                },
            ],
        )
        .unwrap();
    assert!(matches!(
        multi_conflict,
        ConditionalWriteOutcome::Conflict(_)
    ));
    assert_eq!(
        engine.get("tenant-a", "other", "key").unwrap(),
        Some(Bytes::from_static(b"safe"))
    );
    assert_eq!(
        engine.last_commit_id().unwrap(),
        commit_before_multi_conflict
    );

    assert!(matches!(
        engine
            .conditional_write_batch(
                "tenant-a",
                &[ConditionalWrite::Delete {
                    pk: "pk".to_owned(),
                    sk: "sk".to_owned(),
                    expected_version: 1,
                }]
            )
            .unwrap(),
        ConditionalWriteOutcome::Applied(_)
    ));
    let stale_delete = engine
        .conditional_write_batch(
            "tenant-a",
            &[ConditionalWrite::Delete {
                pk: "pk".to_owned(),
                sk: "sk".to_owned(),
                expected_version: 1,
            }],
        )
        .unwrap();
    assert_eq!(
        stale_delete,
        ConditionalWriteOutcome::Conflict(vec![dibi::Conflict {
            pk: "pk".to_owned(),
            sk: "sk".to_owned(),
            expected_version: Some(1),
            actual_version: None,
        }])
    );

    let duplicate = engine.conditional_write_batch(
        "tenant-a",
        &[
            ConditionalWrite::Create {
                pk: "duplicate".to_owned(),
                sk: "key".to_owned(),
                data: Bytes::new(),
            },
            ConditionalWrite::Delete {
                pk: "duplicate".to_owned(),
                sk: "key".to_owned(),
                expected_version: 0,
            },
        ],
    );
    assert!(matches!(
        duplicate,
        Err(dibi::DibiError::DuplicateConditionalKey { .. })
    ));
}

#[test]
fn commit_log_contains_final_physical_mutations_in_order() {
    let (_directory, engine) = open_temporary();
    engine
        .put("tenant-a", "z", "key", Bytes::from_static(b"first"))
        .unwrap();
    engine.delete("tenant-a", "z", "key").unwrap();
    engine
        .application_write_batch(
            "tenant-a",
            &[
                ApplicationWrite::Put {
                    pk: "b".to_owned(),
                    sk: "key".to_owned(),
                    data: Bytes::from_static(b"old"),
                },
                ApplicationWrite::Put {
                    pk: "a".to_owned(),
                    sk: "key".to_owned(),
                    data: Bytes::from_static(b"final-a"),
                },
                ApplicationWrite::Put {
                    pk: "b".to_owned(),
                    sk: "key".to_owned(),
                    data: Bytes::from_static(b"final-b"),
                },
            ],
        )
        .unwrap();

    let records = engine.read_outbox(None, 10).unwrap();
    assert_eq!(
        records
            .iter()
            .map(|record| record.commit_id)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert!(matches!(
        records[0].mutations.as_slice(),
        [CommitMutation::Set { .. }]
    ));
    assert_eq!(
        records[1].mutations,
        vec![CommitMutation::Delete {
            encoded_key: encode_document_key("tenant-a", "z", "key"),
        }]
    );
    assert_eq!(records[2].mutations.len(), 2);
    let expected_a = encode_document_value(&dibi::StoredDocument {
        data: Bytes::from_static(b"final-a"),
        version: 0,
    });
    let expected_b = encode_document_value(&dibi::StoredDocument {
        data: Bytes::from_static(b"final-b"),
        version: 1,
    });
    assert_eq!(
        records[2].mutations,
        vec![
            CommitMutation::Set {
                encoded_key: encode_document_key("tenant-a", "a", "key"),
                encoded_value: expected_a,
            },
            CommitMutation::Set {
                encoded_key: encode_document_key("tenant-a", "b", "key"),
                encoded_value: expected_b,
            },
        ]
    );
    assert_eq!(
        engine
            .read_outbox(Some(1), 1)
            .unwrap()
            .into_iter()
            .map(|record| record.commit_id)
            .collect::<Vec<_>>(),
        vec![2]
    );
}

#[test]
fn admin_scan_filters_and_returns_versions_from_iterator_values() {
    let (_directory, engine) = open_temporary();
    engine
        .put("tenant-a", "keep-a", "1", Bytes::from_static(b"a0"))
        .unwrap();
    engine
        .put("tenant-a", "keep-a", "1", Bytes::from_static(b"a1"))
        .unwrap();
    engine
        .put("tenant-a", "keep-b", "2", Bytes::from_static(b"b0"))
        .unwrap();
    engine
        .put("tenant-a", "skip", "3", Bytes::from_static(b"s0"))
        .unwrap();

    let first = engine
        .admin_scan(
            "tenant-a",
            AdminScanRequest {
                after: None,
                limit: 1,
                pk_prefix: Some("keep-".to_owned()),
            },
        )
        .unwrap();
    assert_eq!(first.documents.len(), 1);
    assert_eq!(first.documents[0].data, Bytes::from_static(b"a1"));
    assert_eq!(first.documents[0].version, 1);
    let second = engine
        .admin_scan(
            "tenant-a",
            AdminScanRequest {
                after: first.next,
                limit: 10,
                pk_prefix: Some("keep-".to_owned()),
            },
        )
        .unwrap();
    assert_eq!(second.documents.len(), 1);
    assert_eq!(second.documents[0].pk, "keep-b");
    assert_eq!(second.documents[0].version, 0);
}

#[test]
fn concurrent_puts_have_no_lost_updates() {
    let (_directory, engine) = open_temporary();
    engine
        .put("tenant-a", "shared", "key", Bytes::from_static(b"seed"))
        .unwrap();
    let engine = Arc::new(engine);
    let thread_count = 8;
    let writes_per_thread = 20;
    let mut handles = Vec::new();
    for thread_number in 0..thread_count {
        let engine = Arc::clone(&engine);
        handles.push(thread::spawn(move || {
            for write_number in 0..writes_per_thread {
                engine
                    .put(
                        "tenant-a",
                        "shared",
                        "key",
                        Bytes::from(format!("{thread_number}-{write_number}")),
                    )
                    .unwrap();
            }
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let total_writes = thread_count * writes_per_thread;
    assert_eq!(
        engine
            .get_with_version("tenant-a", "shared", "key")
            .unwrap()
            .unwrap()
            .version,
        total_writes as i64
    );
    assert_eq!(engine.last_commit_id().unwrap(), total_writes as u64 + 1);
    assert_eq!(
        engine.read_outbox(None, total_writes + 10).unwrap().len(),
        total_writes + 1
    );
}
