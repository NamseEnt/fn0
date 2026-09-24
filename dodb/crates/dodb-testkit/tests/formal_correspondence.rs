use std::collections::VecDeque;
use std::time::Duration;

use dodb_core::{
    DocumentKey, Error, Lsn, PrimaryKey, Revision, RevisionState, SortKey, TransactionCondition,
    TransactionMutation, TransactionRequest, TransactionResult,
};
use dodb_storage::{
    AsyncShard, BTreeStore, BatchRequest, BatchResponse, CoordinatorConfig, DatabaseConfig,
};
use dodb_testkit::{CrashInjector, CrashableFile, Document, ReferenceDb};

fn key(pk: &[u8], sk: &[u8]) -> DocumentKey {
    DocumentKey::new(pk, sk)
}

fn open_store() -> BTreeStore<CrashableFile, CrashableFile> {
    BTreeStore::open_with_wal(
        CrashableFile::new(),
        CrashableFile::new(),
        DatabaseConfig::default().with_cache_capacity(0),
    )
    .unwrap()
}

fn assert_docs_equal(actual: &[dodb_storage::Document], expected: &[Document]) {
    assert_eq!(actual.len(), expected.len());
    for (actual_document, expected_document) in actual.iter().zip(expected) {
        assert_eq!(actual_document.key, expected_document.key);
        assert_eq!(actual_document.value, expected_document.value);
        assert_eq!(actual_document.revision, expected_document.revision);
    }
}

fn assert_store_matches_reference(
    store: &mut BTreeStore<CrashableFile, CrashableFile>,
    reference: &ReferenceDb,
    keys: &[DocumentKey],
) {
    for document_key in keys {
        assert_eq!(
            store.get(document_key).unwrap(),
            reference.get(document_key)
        );
    }
    assert_docs_equal(
        &store.scan(None, usize::MAX).unwrap(),
        &reference.scan(None, usize::MAX),
    );
}

fn assert_conflict_equal(actual: &Error, expected: &Error) {
    let (Error::Conflict(actual_conflict), Error::Conflict(expected_conflict)) = (actual, expected)
    else {
        panic!("expected structured conflicts, got {actual} and {expected}");
    };
    assert_eq!(actual_conflict.key, expected_conflict.key);
    assert_eq!(actual_conflict.expected, expected_conflict.expected);
    assert_eq!(actual_conflict.actual, expected_conflict.actual);
}

fn apply_result_to_reference(
    reference: &mut ReferenceDb,
    request: TransactionRequest,
    actual: &dodb_core::Result<TransactionResult>,
) {
    match actual {
        Ok(TransactionResult {
            commit_lsn: Some(actual_lsn),
        }) => assert_eq!(
            reference.transact_at(request, *actual_lsn).unwrap(),
            Some(*actual_lsn)
        ),
        Ok(TransactionResult { commit_lsn: None }) => {
            assert_eq!(reference.transact(request).unwrap(), None)
        }
        Err(Error::Conflict(_)) => {
            let expected = reference.transact(request).unwrap_err();
            assert_conflict_equal(actual.as_ref().unwrap_err(), &expected);
        }
        Err(Error::InvalidRequest(_)) => {
            assert!(matches!(
                reference.transact(request),
                Err(Error::InvalidRequest(_))
            ));
        }
        Err(error) => panic!("unexpected transaction result: {error}"),
    }
}

fn put_request(document_key: DocumentKey, value: &[u8]) -> TransactionRequest {
    TransactionRequest::new(
        Vec::new(),
        vec![TransactionMutation::Put {
            key: document_key,
            value: value.to_vec(),
        }],
    )
}

fn pipeline_requests(a: &DocumentKey, b: &DocumentKey) -> Vec<TransactionRequest> {
    vec![
        put_request(a.clone(), b"a"),
        TransactionRequest::new(
            vec![TransactionCondition::Exists { key: a.clone() }],
            Vec::new(),
        ),
        TransactionRequest::new(
            vec![TransactionCondition::NotExists { key: a.clone() }],
            vec![TransactionMutation::Put {
                key: b.clone(),
                value: b"must-not-appear".to_vec(),
            }],
        ),
        put_request(b.clone(), b"b"),
    ]
}

async fn run_fixed_pipeline(grouped: bool) -> (Vec<RevisionState>, Lsn, u64) {
    let a = key(b"pipeline", b"A");
    let b = key(b"pipeline", b"B");
    let store = open_store();
    let shard = AsyncShard::start_with_config(
        store,
        CoordinatorConfig {
            max_group_requests: if grouped { 4 } else { 1 },
            max_group_bytes: usize::MAX,
            max_collection_delay: if grouped {
                Duration::from_millis(10)
            } else {
                Duration::ZERO
            },
            ..CoordinatorConfig::default()
        },
    );
    let requests = pipeline_requests(&a, &b);
    let actual_results = if grouped {
        let first = shard.execute_transaction(requests[0].clone());
        let second = shard.execute_transaction(requests[1].clone());
        let third = shard.execute_transaction(requests[2].clone());
        let fourth = shard.execute_transaction(requests[3].clone());
        let (first, second, third, fourth) = tokio::join!(first, second, third, fourth);
        vec![first, second, third, fourth]
    } else {
        let mut results = Vec::new();
        for request in requests.iter().cloned() {
            results.push(shard.execute_transaction(request).await);
        }
        results
    };
    for (index, result) in actual_results.iter().enumerate() {
        match index {
            0 | 3 => assert!(result.as_ref().unwrap().commit_lsn.is_some()),
            1 => assert_eq!(result.as_ref().unwrap().commit_lsn, None),
            2 => assert!(matches!(result, Err(Error::Conflict(_)))),
            _ => unreachable!(),
        }
    }
    let metrics = shard.coordinator_metrics();
    assert_eq!(metrics.max_group_requests, if grouped { 4 } else { 1 });
    assert_eq!(metrics.logical_transactions, 4);
    assert_eq!(metrics.groups, if grouped { 1 } else { 4 });
    let wal_syncs = shard.wal_metrics().unwrap().wal_syncs;
    assert_eq!(wal_syncs, if grouped { 2 } else { 3 });

    let mut reference = ReferenceDb::new();
    for (request, result) in requests.into_iter().zip(&actual_results) {
        apply_result_to_reference(&mut reference, request, result);
    }
    let observed = shard
        .transact_get(vec![a.clone(), b.clone()])
        .await
        .unwrap();
    assert_eq!(observed, reference.transact_get(&[a.clone(), b.clone()]));
    assert!(observed.iter().all(|state| !state.is_missing()));
    let last_commit = actual_results[3].as_ref().unwrap().commit_lsn.unwrap();
    let checkpoint = shard.checkpoint().await.unwrap();
    assert_eq!(checkpoint.checkpoint_lsn, last_commit);
    assert_eq!(
        checkpoint.checkpoint_lsn,
        Lsn::new(observed[1].revision().get())
    );
    shard.shutdown().await.unwrap();
    (observed, last_commit, wal_syncs)
}

#[tokio::test(flavor = "current_thread")]
async fn formal_pipeline_matches_reference_with_split_groups() {
    let (states, _, syncs) = run_fixed_pipeline(false).await;
    assert_eq!(states.len(), 2);
    assert_eq!(syncs, 3);
}

#[tokio::test(flavor = "current_thread")]
async fn formal_pipeline_matches_reference_with_grouped_mutations() {
    let (states, _, syncs) = run_fixed_pipeline(true).await;
    assert_eq!(states.len(), 2);
    assert_eq!(syncs, 2);
}

#[tokio::test(flavor = "current_thread")]
async fn mutation_and_barriers_share_group_but_preserve_segment_order() {
    let a = key(b"segments", b"A");
    let b = key(b"segments", b"B");
    let shard = AsyncShard::start_with_config(
        open_store(),
        CoordinatorConfig {
            max_group_requests: 3,
            max_group_bytes: usize::MAX,
            max_collection_delay: Duration::from_millis(10),
            ..CoordinatorConfig::default()
        },
    );
    let prefix_a = shard
        .execute_transaction(put_request(a.clone(), b"a"))
        .await
        .unwrap()
        .commit_lsn
        .unwrap();
    let prefix_condition = TransactionRequest::new(
        vec![TransactionCondition::Exists { key: a.clone() }],
        Vec::new(),
    );
    assert_eq!(
        shard
            .execute_transaction(prefix_condition)
            .await
            .unwrap()
            .commit_lsn,
        None
    );
    let prefix_conflict = TransactionRequest::new(
        vec![TransactionCondition::NotExists { key: a.clone() }],
        vec![TransactionMutation::Put {
            key: b.clone(),
            value: b"never".to_vec(),
        }],
    );
    assert!(matches!(
        shard.execute_transaction(prefix_conflict).await,
        Err(Error::Conflict(_))
    ));
    let before_groups = shard.coordinator_metrics().groups;
    let put = shard.execute_transaction(put_request(b.clone(), b"b"));
    let read = shard.transact_get(vec![a.clone(), b.clone()]);
    let checkpoint = shard.checkpoint();
    let (put, read, checkpoint) = tokio::join!(put, read, checkpoint);
    let b_lsn = put.unwrap().commit_lsn.unwrap();
    let read = read.unwrap();
    assert_eq!(
        read,
        vec![
            RevisionState::present(b"a", prefix_a.into()),
            RevisionState::present(b"b", b_lsn.into())
        ]
    );
    assert_eq!(checkpoint.unwrap().checkpoint_lsn, b_lsn);
    let metrics = shard.coordinator_metrics();
    assert_eq!(metrics.groups - before_groups, 1);
    assert_eq!(metrics.max_group_requests, 3);
    shard.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn committed_read_view_never_exposes_half_of_a_grouped_publication() {
    let a = key(b"publication", b"A");
    let b = key(b"publication", b"B");
    let shard = std::sync::Arc::new(AsyncShard::start_with_config(
        open_store(),
        CoordinatorConfig {
            max_group_requests: 2,
            max_group_bytes: usize::MAX,
            max_collection_delay: Duration::from_millis(10),
            ..CoordinatorConfig::default()
        },
    ));
    let query_shard = std::sync::Arc::clone(&shard);
    let query_task = tokio::spawn(async move {
        let mut row_counts = Vec::new();
        for _ in 0..500 {
            let result = query_shard
                .execute(BatchRequest::Query {
                    pk: PrimaryKey::new(b"publication".to_vec()),
                    exclusive_after_sk: None,
                    limit: usize::MAX,
                })
                .await
                .unwrap();
            let BatchResponse::Query(rows) = result else {
                panic!("expected query response");
            };
            assert!(
                rows.is_empty() || rows.len() == 2,
                "partial committed view: {} rows",
                rows.len()
            );
            row_counts.push(rows.len());
            tokio::task::yield_now().await;
        }
        row_counts
    });
    let write_a = shard.execute(BatchRequest::Put {
        key: a.clone(),
        value: b"a".to_vec(),
    });
    let write_b = shard.execute(BatchRequest::Put {
        key: b.clone(),
        value: b"b".to_vec(),
    });
    let (write_a, write_b) = tokio::join!(write_a, write_b);
    assert!(matches!(write_a.unwrap(), BatchResponse::Put(_)));
    assert!(matches!(write_b.unwrap(), BatchResponse::Put(_)));
    let counts = query_task.await.unwrap();
    assert_eq!(counts.len(), 500);
    assert_eq!(shard.coordinator_metrics().max_group_requests, 2);
    let final_query = shard
        .execute(BatchRequest::Query {
            pk: PrimaryKey::new(b"publication".to_vec()),
            exclusive_after_sk: None,
            limit: usize::MAX,
        })
        .await
        .unwrap();
    let BatchResponse::Query(rows) = final_query else {
        panic!("expected final query response");
    };
    assert_eq!(rows.len(), 2);
    shard.shutdown().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn after_publish_error_is_visible_but_error() {
    let document_key = key(b"fault", b"after-publish");
    let mut store = open_store();
    store.set_fault_injector(CrashInjector::at("after_publish", 1));
    let shard = AsyncShard::start_with_config(store, CoordinatorConfig::default());
    assert!(
        shard
            .execute_transaction(put_request(document_key.clone(), b"visible"))
            .await
            .is_err()
    );
    let visible = shard
        .execute(BatchRequest::Get { key: document_key })
        .await
        .unwrap();
    let BatchResponse::Get(visible_state) = visible else {
        panic!("expected get response");
    };
    assert_eq!(visible_state.value(), Some(&b"visible"[..]));
    shard.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn before_publish_failure_breaks_live_read_view() {
    let document_key = key(b"fault", b"before-publish");
    let mut store = open_store();
    store.set_fault_injector(CrashInjector::at("before_publish", 1));
    let shard = AsyncShard::start_with_config(store, CoordinatorConfig::default());
    assert!(
        shard
            .execute_transaction(put_request(document_key.clone(), b"hidden"))
            .await
            .is_err()
    );
    assert!(matches!(
        shard.execute(BatchRequest::Get { key: document_key }).await,
        Err(Error::DurabilityFailure(_))
    ));
    shard.close().await.unwrap();
}

fn reopen_after_crash(
    store: BTreeStore<CrashableFile, CrashableFile>,
) -> BTreeStore<CrashableFile, CrashableFile> {
    let (mut data, mut wal) = store.into_files().unwrap();
    data.crash();
    wal.crash();
    BTreeStore::open_with_wal(data, wal, DatabaseConfig::default().with_cache_capacity(0)).unwrap()
}

#[test]
fn durable_post_sync_named_failures_recover_complete_transaction() {
    let a = key(b"post-sync", b"A");
    let b = key(b"post-sync", b"B");
    for point in ["after_wal_sync", "before_publish", "after_publish"] {
        let mut store = open_store();
        store.set_fault_injector(CrashInjector::at(point, 1));
        let request = TransactionRequest::new(
            Vec::new(),
            vec![
                TransactionMutation::Put {
                    key: a.clone(),
                    value: b"a".to_vec(),
                },
                TransactionMutation::Put {
                    key: b.clone(),
                    value: b"b".to_vec(),
                },
            ],
        );
        assert!(store.transact(request).is_err(), "{point}");
        let mut reopened = reopen_after_crash(store);
        let mut reference = ReferenceDb::new();
        let recovered_lsn = Lsn::new(reopened.get(&a).unwrap().revision().get());
        reference
            .transact_at(
                TransactionRequest::new(
                    Vec::new(),
                    vec![
                        TransactionMutation::Put {
                            key: a.clone(),
                            value: b"a".to_vec(),
                        },
                        TransactionMutation::Put {
                            key: b.clone(),
                            value: b"b".to_vec(),
                        },
                    ],
                ),
                recovered_lsn,
            )
            .unwrap();
        assert_store_matches_reference(&mut reopened, &reference, &[a.clone(), b.clone()]);
    }
}

#[test]
fn pre_sync_named_failures_do_not_invent_commits() {
    let a = key(b"pre-sync", b"A");
    let b = key(b"pre-sync", b"B");
    for point in [
        "before_wal_append",
        "before_commit_record",
        "after_group_records_written",
        "before_wal_sync",
        "during_wal_sync",
    ] {
        let mut store = open_store();
        store.set_fault_injector(CrashInjector::at(point, 1));
        let request = TransactionRequest::new(
            Vec::new(),
            vec![
                TransactionMutation::Put {
                    key: a.clone(),
                    value: b"a".to_vec(),
                },
                TransactionMutation::Put {
                    key: b.clone(),
                    value: b"b".to_vec(),
                },
            ],
        );
        assert!(store.transact(request).is_err(), "{point}");
        let mut reopened = reopen_after_crash(store);
        let reference = ReferenceDb::new();
        assert_store_matches_reference(&mut reopened, &reference, &[a.clone(), b.clone()]);
    }
}

fn next_random(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn random_request(
    rng: &mut u64,
    keys: &[DocumentKey],
    reference: &ReferenceDb,
) -> TransactionRequest {
    let choice = (next_random(rng) >> 32) % 12;
    let first_key = keys[((next_random(rng) >> 32) as usize) % keys.len()].clone();
    let second_key = keys[((next_random(rng) >> 32) as usize) % keys.len()].clone();
    match choice {
        0 => TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Put {
                key: first_key,
                value: vec![(next_random(rng) >> 24) as u8; 1 + (next_random(rng) % 64) as usize],
            }],
        ),
        1 => TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Delete { key: first_key }],
        ),
        2 => {
            let second_key = if second_key == first_key {
                keys[(keys.iter().position(|item| item == &first_key).unwrap() + 1) % keys.len()]
                    .clone()
            } else {
                second_key
            };
            TransactionRequest::new(
                Vec::new(),
                vec![
                    TransactionMutation::Put {
                        key: first_key,
                        value: vec![(next_random(rng) >> 32) as u8; 8],
                    },
                    TransactionMutation::Delete { key: second_key },
                ],
            )
        }
        3 | 4 => {
            let revision = reference.get(&first_key).revision();
            let expected_revision = if choice == 3 {
                revision
            } else if revision == Revision::ZERO {
                Revision::new(1)
            } else {
                Revision::new(revision.get() - 1)
            };
            TransactionRequest::new(
                vec![TransactionCondition::RevisionEquals {
                    key: first_key.clone(),
                    expected_revision,
                }],
                vec![TransactionMutation::Put {
                    key: second_key,
                    value: b"revision-conditioned".to_vec(),
                }],
            )
        }
        5 => TransactionRequest::new(
            vec![TransactionCondition::Exists {
                key: first_key.clone(),
            }],
            vec![TransactionMutation::Delete { key: second_key }],
        ),
        6 => TransactionRequest::new(
            vec![TransactionCondition::NotExists {
                key: first_key.clone(),
            }],
            vec![TransactionMutation::Put {
                key: second_key,
                value: b"absent-conditioned".to_vec(),
            }],
        ),
        7 => TransactionRequest::new(
            vec![TransactionCondition::Exists { key: first_key }],
            Vec::new(),
        ),
        8 => TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Put {
                key: first_key,
                value: b"one-key".to_vec(),
            }],
        ),
        9 => TransactionRequest::new(
            vec![
                TransactionCondition::Exists {
                    key: first_key.clone(),
                },
                TransactionCondition::NotExists {
                    key: first_key.clone(),
                },
            ],
            Vec::new(),
        ),
        10 => TransactionRequest::new(
            Vec::new(),
            vec![
                TransactionMutation::Delete {
                    key: first_key.clone(),
                },
                TransactionMutation::Put {
                    key: first_key,
                    value: b"duplicate".to_vec(),
                },
            ],
        ),
        _ => TransactionRequest::new(Vec::new(), Vec::new()),
    }
}

fn apply_random_group(
    seed: u64,
    group_index: usize,
    requests: &[TransactionRequest],
    actual: &[dodb_core::Result<TransactionResult>],
    reference: &mut ReferenceDb,
    trace: &mut VecDeque<String>,
) {
    for (request, result) in requests.iter().cloned().zip(actual) {
        let result_text = match result {
            Ok(result) => format!("Ok({:?})", result.commit_lsn),
            Err(error) => format!("Err({error})"),
        };
        trace.push_back(format!(
            "group={group_index} request={request:?} result={result_text}"
        ));
        while trace.len() > 8 {
            trace.pop_front();
        }
        if let Err(error) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply_result_to_reference(reference, request, result)
        })) {
            panic!(
                "seed={seed:#x} group={group_index} requests={requests:?} results={actual:?} recent={trace:?}: {error:?}"
            );
        }
    }
}

fn assert_with_context(context: String, assertion: impl FnOnce()) {
    if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(assertion)) {
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .unwrap_or("non-string assertion failure");
        panic!("{context}: {message}");
    }
}

fn assert_all_reads(
    store: &mut BTreeStore<CrashableFile, CrashableFile>,
    reference: &ReferenceDb,
    keys: &[DocumentKey],
) {
    let ordered = vec![keys[9].clone(), keys[2].clone(), keys[14].clone()];
    assert_eq!(
        store.transact_get(&ordered).unwrap(),
        reference.transact_get(&ordered)
    );
    let query_pk = PrimaryKey::new(vec![0, 1]);
    let cursor = Some(SortKey::new(vec![5]));
    assert_docs_equal(
        &store.query(&query_pk, cursor.as_ref(), 8).unwrap(),
        &reference.query(&query_pk, cursor.as_ref(), 8),
    );
    assert_docs_equal(
        &store.scan(Some(&keys[11]), 8).unwrap(),
        &reference.scan(Some(&keys[11]), 8),
    );
}

#[test]
fn formal_contract_randomized_transaction_groups_match_reference() {
    let seeds = [
        0xd0db_2026_1001,
        0xd0db_2026_1002,
        0xd0db_2026_1003,
        0xd0db_2026_1004,
        0xd0db_2026_1005,
        0xd0db_2026_1006,
        0xd0db_2026_1007,
        0xd0db_2026_1008,
    ];
    let mut total_requests = 0usize;
    let mut total_crash_reopens = 0usize;
    let mut total_checkpoint_reopens = 0usize;
    let mut total_condition_only = 0usize;
    let mut total_conflicts = 0usize;
    let mut total_invalid = 0usize;
    for seed in seeds {
        let keys = (0..24u8)
            .map(|index| key(&[0, index / 8], &[index % 8]))
            .collect::<Vec<_>>();
        let mut store = open_store();
        let mut reference = ReferenceDb::new();
        let mut rng = seed;
        let mut last_commit_lsn = Lsn::ZERO;
        let mut trace = VecDeque::new();
        let mut condition_only_count = 0usize;
        let mut conflict_count = 0usize;
        let mut invalid_count = 0usize;
        for group_index in 0..600usize {
            let group_size = (next_random(&mut rng) % 4 + 1) as usize;
            let requests = (0..group_size)
                .map(|_| random_request(&mut rng, &keys, &reference))
                .collect::<Vec<_>>();
            let actual = store
                .apply_transaction_group(&requests)
                .unwrap_or_else(|error| {
                    panic!("seed={seed:#x} group={group_index} requests={requests:?} outer group error: {error}")
                });
            for result in &actual {
                match result {
                    Ok(TransactionResult {
                        commit_lsn: Some(lsn),
                    }) => {
                        assert!(
                            *lsn > last_commit_lsn,
                            "seed={seed:#x} group={group_index} requests={requests:?} actual={actual:?} recent={trace:?}"
                        );
                        last_commit_lsn = *lsn;
                    }
                    Ok(TransactionResult { commit_lsn: None }) => condition_only_count += 1,
                    Err(Error::Conflict(_)) => conflict_count += 1,
                    Err(Error::InvalidRequest(_)) => invalid_count += 1,
                    Err(error) => {
                        panic!(
                            "seed={seed:#x} group={group_index} requests={requests:?} actual={actual:?}: unexpected result: {error}"
                        )
                    }
                }
            }
            apply_random_group(
                seed,
                group_index,
                &requests,
                &actual,
                &mut reference,
                &mut trace,
            );
            total_requests += group_size;
            let context = || {
                format!(
                    "seed={seed:#x} group={group_index} requests={requests:?} actual={actual:?} recent={trace:?}"
                )
            };
            if group_index % 31 == 30 {
                store = reopen_after_crash(store);
                assert_with_context(context(), || {
                    assert_store_matches_reference(&mut store, &reference, &keys)
                });
                total_crash_reopens += 1;
            }
            if group_index % 113 == 112 {
                let report = store
                    .checkpoint()
                    .unwrap_or_else(|error| panic!("{} checkpoint failed: {error}", context()));
                assert_eq!(
                    report.checkpoint_lsn,
                    last_commit_lsn,
                    "{} checkpoint LSN mismatch",
                    context()
                );
                store = reopen_after_crash(store);
                assert_with_context(context(), || {
                    assert_store_matches_reference(&mut store, &reference, &keys)
                });
                total_checkpoint_reopens += 1;
            }
            if group_index % 17 == 0 {
                assert_with_context(context(), || {
                    assert_store_matches_reference(&mut store, &reference, &keys);
                    assert_all_reads(&mut store, &reference, &keys);
                });
            }
        }
        assert_store_matches_reference(&mut store, &reference, &keys);
        assert!(condition_only_count > 0);
        assert!(conflict_count > 0);
        assert!(invalid_count > 0);
        total_condition_only += condition_only_count;
        total_conflicts += conflict_count;
        total_invalid += invalid_count;
    }
    assert!((8 * 600..=8 * 600 * 4).contains(&total_requests));
    assert_eq!(total_crash_reopens, 8 * (600 / 31));
    assert_eq!(total_checkpoint_reopens, 8 * (600 / 113));
    println!(
        "formal correspondence corpus: seeds=8 groups=4800 requests={total_requests} condition_only={total_condition_only} conflicts={total_conflicts} invalid={total_invalid} crash_reopens={total_crash_reopens} checkpoint_reopens={total_checkpoint_reopens}"
    );
}

#[test]
fn missing_revision_after_aba_matches_reference_through_group_and_recovery() {
    let document_key = key(b"aba", b"key");
    let mut store = open_store();
    let mut reference = ReferenceDb::new();
    let original_missing = RevisionState::missing(Revision::ZERO);
    let put = store
        .transact(put_request(document_key.clone(), b"present"))
        .unwrap();
    let put_lsn = put.commit_lsn.unwrap();
    reference
        .transact_at(put_request(document_key.clone(), b"present"), put_lsn)
        .unwrap();
    let delete = TransactionRequest::new(
        Vec::new(),
        vec![TransactionMutation::Delete {
            key: document_key.clone(),
        }],
    );
    let deleted = store.transact(delete.clone()).unwrap();
    reference
        .transact_at(delete, deleted.commit_lsn.unwrap())
        .unwrap();
    assert_ne!(store.get(&document_key).unwrap(), original_missing);
    assert_eq!(
        store.get(&document_key).unwrap(),
        reference.get(&document_key)
    );
    let mut reopened = reopen_after_crash(store);
    assert_eq!(
        reopened.get(&document_key).unwrap(),
        reference.get(&document_key)
    );
}
