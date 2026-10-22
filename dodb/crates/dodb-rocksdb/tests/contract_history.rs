use dodb_core::{
    DocumentKey, Error, Lsn, PrimaryKey, Revision, RevisionState, TenantId, TransactionCondition,
    TransactionMutation, TransactionRequest,
};
use dodb_rocksdb::{
    ConditionKeyLockMode, RevisionAllocatorMode, RocksDbService, RocksDbServiceConfig,
};
use dodb_server::{BlinkCheckpointPolicy, BlinkTenantService, LocalTenantServiceConfig};
use dodb_service::Document;
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::TempDir;
use tokio::sync::Barrier;

const RESPONSE_BUDGET: ExecutionBudget = ExecutionBudget::new(1024 * 1024);

fn temporary_db_directory() -> TempDir {
    match std::env::var_os("DODB_TEST_DATA_DIR") {
        Some(path) => TempDir::new_in(path).unwrap(),
        None => TempDir::new().unwrap(),
    }
}

fn key(partition: &[u8], sort: &[u8]) -> DocumentKey {
    DocumentKey::new(partition.to_vec(), sort.to_vec())
}

async fn execute<S: DodbService>(service: &S, request: Request) -> Result<Response, Error> {
    service
        .execute(TenantId::new(73), request, RESPONSE_BUDGET)
        .await
}

async fn check_conditional_history<S: DodbService>(service: Arc<S>) {
    let first_key = key(b"history-x", b"row");
    let second_key = key(b"history-y", b"row");
    let first_revision = match execute(
        service.as_ref(),
        Request::Put {
            key: first_key.clone(),
            value: b"old-x".to_vec(),
        },
    )
    .await
    .unwrap()
    {
        Response::Put(revision) => revision,
        other => panic!("unexpected response: {other:?}"),
    };
    let second_revision = match execute(
        service.as_ref(),
        Request::Put {
            key: second_key.clone(),
            value: b"old-y".to_vec(),
        },
    )
    .await
    .unwrap()
    {
        Response::Put(revision) => revision,
        other => panic!("unexpected response: {other:?}"),
    };

    let first_request = Request::Transact {
        request: TransactionRequest::new(
            vec![
                TransactionCondition::RevisionEquals {
                    key: first_key.clone(),
                    expected_revision: first_revision,
                },
                TransactionCondition::RevisionEquals {
                    key: second_key.clone(),
                    expected_revision: second_revision,
                },
            ],
            vec![TransactionMutation::Put {
                key: first_key.clone(),
                value: b"new-x".to_vec(),
            }],
        ),
    };
    let second_request = Request::Transact {
        request: TransactionRequest::new(
            vec![
                TransactionCondition::RevisionEquals {
                    key: first_key.clone(),
                    expected_revision: first_revision,
                },
                TransactionCondition::RevisionEquals {
                    key: second_key.clone(),
                    expected_revision: second_revision,
                },
            ],
            vec![TransactionMutation::Put {
                key: second_key.clone(),
                value: b"new-y".to_vec(),
            }],
        ),
    };
    let first_future = execute(service.as_ref(), first_request);
    let second_future = execute(service.as_ref(), second_request);
    let (first_result, second_result) = tokio::join!(first_future, second_future);
    let first_commit = match first_result {
        Ok(Response::Transact(outcome)) => outcome
            .commit_lsn
            .map(|commit_lsn| Revision::new(commit_lsn.get())),
        Err(Error::Conflict(_)) => None,
        other => panic!("unexpected first transaction result: {other:?}"),
    };
    let second_commit = match second_result {
        Ok(Response::Transact(outcome)) => outcome
            .commit_lsn
            .map(|commit_lsn| Revision::new(commit_lsn.get())),
        Err(Error::Conflict(_)) => None,
        other => panic!("unexpected second transaction result: {other:?}"),
    };
    assert_ne!(first_commit.is_some(), second_commit.is_some());
    let winner_revision = first_commit.or(second_commit).unwrap();

    let first_state = match execute(
        service.as_ref(),
        Request::Get {
            key: first_key.clone(),
        },
    )
    .await
    .unwrap()
    {
        Response::Get(RevisionState::Present { value, revision }) => (value, revision),
        other => panic!("unexpected Get response: {other:?}"),
    };
    let second_state = match execute(
        service.as_ref(),
        Request::Get {
            key: second_key.clone(),
        },
    )
    .await
    .unwrap()
    {
        Response::Get(RevisionState::Present { value, revision }) => (value, revision),
        other => panic!("unexpected Get response: {other:?}"),
    };

    let first_won = first_state.0 == b"new-x";
    let second_won = second_state.0 == b"new-y";
    assert_ne!(first_won, second_won);
    if first_won {
        assert_eq!(first_state.1, winner_revision);
        assert_eq!(second_state, (b"old-y".to_vec(), second_revision));
    } else {
        assert_eq!(second_state.1, winner_revision);
        assert_eq!(first_state, (b"old-x".to_vec(), first_revision));
    }

    let first_query = match execute(
        service.as_ref(),
        Request::Query {
            pk: PrimaryKey::new(b"history-x".to_vec()),
            exclusive_after_sk: None,
            limit: 8,
        },
    )
    .await
    .unwrap()
    {
        Response::Query(rows) => rows,
        other => panic!("unexpected first Query response: {other:?}"),
    };
    let second_query = match execute(
        service.as_ref(),
        Request::Query {
            pk: PrimaryKey::new(b"history-y".to_vec()),
            exclusive_after_sk: None,
            limit: 8,
        },
    )
    .await
    .unwrap()
    {
        Response::Query(rows) => rows,
        other => panic!("unexpected second Query response: {other:?}"),
    };
    assert_eq!(first_query.len(), 1);
    assert_eq!(second_query.len(), 1);
    assert_eq!(first_query[0].key, first_key);
    assert_eq!(second_query[0].key, second_key);
    assert_eq!(first_query[0].value, first_state.0);
    assert_eq!(second_query[0].value, second_state.0);
    assert_eq!(first_query[0].revision, first_state.1);
    assert_eq!(second_query[0].revision, second_state.1);

    let scan = match execute(
        service.as_ref(),
        Request::Scan {
            exclusive_after_key: None,
            limit: 8,
        },
    )
    .await
    .unwrap()
    {
        Response::Scan(rows) => rows,
        other => panic!("unexpected Scan response: {other:?}"),
    };
    assert_eq!(scan.len(), 2);
    for row in scan {
        let expected = if row.key == first_key {
            &first_state
        } else {
            assert_eq!(row.key, second_key);
            &second_state
        };
        assert_eq!(row.value, expected.0);
        assert_eq!(row.revision, expected.1);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn conditional_multi_key_history_agrees_for_blink_and_rocksdb_allocators() {
    let blink_directory = temporary_db_directory();
    let blink_service = BlinkTenantService::new(LocalTenantServiceConfig {
        data_dir: blink_directory.path().to_path_buf(),
        blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 1 },
        ..LocalTenantServiceConfig::default()
    })
    .unwrap();
    check_conditional_history(Arc::new(blink_service)).await;

    for condition_key_lock_mode in [
        ConditionKeyLockMode::Exclusive,
        ConditionKeyLockMode::Shared,
    ] {
        for allocator in [
            RevisionAllocatorMode::Serialized,
            RevisionAllocatorMode::ReservedRanges,
        ] {
            let rocks_directory = temporary_db_directory();
            let mut config = RocksDbServiceConfig::new(rocks_directory.path());
            config.condition_key_lock_mode = condition_key_lock_mode;
            config.revision_allocator = allocator;
            check_conditional_history(Arc::new(RocksDbService::new(config).unwrap())).await;
        }
    }
}

#[derive(Clone, Debug)]
struct ModelEntry {
    value: Option<Vec<u8>>,
    revision: Revision,
}

#[derive(Clone, Debug)]
enum RecordedOutcome {
    Response(Response),
    Conflict,
    OtherError,
}

#[derive(Clone, Debug)]
struct RecordedOperation {
    operation_index: usize,
    invocation_order: u64,
    response_order: u64,
    request: Request,
    outcome: RecordedOutcome,
}

fn record_outcome(result: Result<Response, Error>) -> RecordedOutcome {
    match result {
        Ok(response) => RecordedOutcome::Response(response),
        Err(Error::Conflict(_)) => RecordedOutcome::Conflict,
        Err(_) => RecordedOutcome::OtherError,
    }
}

fn revision_matches_model(
    model: &BTreeMap<DocumentKey, ModelEntry>,
    key: &DocumentKey,
) -> Revision {
    model
        .get(key)
        .map_or(Revision::ZERO, |entry| entry.revision)
}

fn conditions_match_model(
    model: &BTreeMap<DocumentKey, ModelEntry>,
    conditions: &[TransactionCondition],
) -> bool {
    conditions.iter().all(|condition| match condition {
        TransactionCondition::RevisionEquals {
            key,
            expected_revision,
        } => revision_matches_model(model, key) == *expected_revision,
        TransactionCondition::Exists { key } => {
            model.get(key).is_some_and(|entry| entry.value.is_some())
        }
        TransactionCondition::NotExists { key } => {
            model.get(key).is_none_or(|entry| entry.value.is_none())
        }
    })
}

fn mutation_keys(request: &Request) -> Vec<DocumentKey> {
    match request {
        Request::Put { key, .. } | Request::Delete { key } => vec![key.clone()],
        Request::Transact { request } => request
            .mutations
            .iter()
            .map(|mutation| match mutation {
                TransactionMutation::Put { key, .. } | TransactionMutation::Delete { key } => {
                    key.clone()
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn conflicting_mutation_overlaps(history: &[RecordedOperation], operation_index: usize) -> bool {
    let operation = &history[operation_index];
    let operation_keys: BTreeSet<_> = mutation_keys(&operation.request).into_iter().collect();
    history.iter().enumerate().any(|(other_index, other)| {
        if other_index == operation_index
            || operation.response_order <= other.invocation_order
            || other.response_order <= operation.invocation_order
        {
            return false;
        }
        let other_keys: BTreeSet<_> = mutation_keys(&other.request).into_iter().collect();
        !operation_keys.is_disjoint(&other_keys)
            && matches!(
                &other.request,
                Request::Put { .. } | Request::Delete { .. } | Request::Transact { .. }
            )
    })
}

fn document_rows_match(
    actual: &[Document],
    model: &BTreeMap<DocumentKey, ModelEntry>,
    partition: Option<&PrimaryKey>,
    limit: usize,
) -> bool {
    let expected: Vec<_> = model
        .iter()
        .filter(|(key, entry)| {
            entry.value.is_some() && partition.is_none_or(|partition| key.pk == *partition)
        })
        .take(limit)
        .map(|(key, entry)| {
            (
                key.clone(),
                entry.value.clone().unwrap_or_default(),
                entry.revision,
            )
        })
        .collect();
    actual.len() == expected.len()
        && actual.iter().zip(expected).all(|(actual, expected)| {
            actual.key == expected.0 && actual.value == expected.1 && actual.revision == expected.2
        })
}

fn result_matches_model(
    history: &[RecordedOperation],
    operation_index: usize,
    model: &mut BTreeMap<DocumentKey, ModelEntry>,
) -> bool {
    let operation = &history[operation_index];
    match (&operation.request, &operation.outcome) {
        (Request::Get { key }, RecordedOutcome::Response(Response::Get(actual))) => {
            match model.get(key) {
                Some(entry) => {
                    actual
                        == &match &entry.value {
                            Some(value) => RevisionState::present(value.clone(), entry.revision),
                            None => RevisionState::missing(entry.revision),
                        }
                }
                None => actual == &RevisionState::missing(Revision::ZERO),
            }
        }
        (Request::Put { key, value }, RecordedOutcome::Response(Response::Put(revision))) => {
            let previous_revision = revision_matches_model(model, key);
            if revision.get() == 0 || revision.get() <= previous_revision.get() {
                return false;
            }
            model.insert(
                key.clone(),
                ModelEntry {
                    value: Some(value.clone()),
                    revision: *revision,
                },
            );
            true
        }
        (Request::Delete { key }, RecordedOutcome::Response(Response::Delete(revision))) => {
            let previous_revision = revision_matches_model(model, key);
            if revision.get() == 0 || revision.get() <= previous_revision.get() {
                return false;
            }
            model.insert(
                key.clone(),
                ModelEntry {
                    value: None,
                    revision: *revision,
                },
            );
            true
        }
        (Request::Transact { request }, RecordedOutcome::Response(Response::Transact(outcome))) => {
            if !conditions_match_model(model, &request.conditions) {
                return false;
            }
            if request.mutations.is_empty() {
                return outcome.commit_lsn.is_none();
            }
            let Some(commit_lsn) = outcome.commit_lsn else {
                return false;
            };
            let revision = Revision::new(commit_lsn.get());
            if revision.get() == 0
                || request.mutations.iter().any(|mutation| {
                    let key = match mutation {
                        TransactionMutation::Put { key, .. }
                        | TransactionMutation::Delete { key } => key,
                    };
                    revision.get() <= revision_matches_model(model, key).get()
                })
            {
                return false;
            }
            for mutation in &request.mutations {
                match mutation {
                    TransactionMutation::Put { key, value } => {
                        model.insert(
                            key.clone(),
                            ModelEntry {
                                value: Some(value.clone()),
                                revision,
                            },
                        );
                    }
                    TransactionMutation::Delete { key } => {
                        model.insert(
                            key.clone(),
                            ModelEntry {
                                value: None,
                                revision,
                            },
                        );
                    }
                }
            }
            true
        }
        (Request::Query { pk, limit, .. }, RecordedOutcome::Response(Response::Query(actual))) => {
            document_rows_match(actual, model, Some(pk), *limit)
        }
        (Request::Scan { limit, .. }, RecordedOutcome::Response(Response::Scan(actual))) => {
            document_rows_match(actual, model, None, *limit)
        }
        (Request::Transact { request }, RecordedOutcome::Conflict) => {
            !conditions_match_model(model, &request.conditions)
                || conflicting_mutation_overlaps(history, operation_index)
        }
        (Request::Put { .. } | Request::Delete { .. }, RecordedOutcome::Conflict) => {
            conflicting_mutation_overlaps(history, operation_index)
        }
        (_, RecordedOutcome::OtherError) => false,
        _ => false,
    }
}

fn is_linearizable_history(
    history: &[RecordedOperation],
    initial_model: &BTreeMap<DocumentKey, ModelEntry>,
) -> bool {
    if history.len() > 8 {
        return false;
    }
    let predecessor_masks: Vec<usize> = history
        .iter()
        .enumerate()
        .map(|(operation_index, operation)| {
            history
                .iter()
                .enumerate()
                .fold(0usize, |mask, (other_index, other)| {
                    if operation_index != other_index
                        && other.response_order < operation.invocation_order
                    {
                        mask | (1usize << other_index)
                    } else {
                        mask
                    }
                })
        })
        .collect();
    let full_mask = (1usize << history.len()) - 1;

    fn search(
        history: &[RecordedOperation],
        predecessor_masks: &[usize],
        full_mask: usize,
        completed_mask: usize,
        model: &BTreeMap<DocumentKey, ModelEntry>,
    ) -> bool {
        if completed_mask == full_mask {
            return true;
        }
        for operation_index in 0..history.len() {
            let operation_mask = 1usize << operation_index;
            if completed_mask & operation_mask != 0
                || predecessor_masks[operation_index] & completed_mask
                    != predecessor_masks[operation_index]
            {
                continue;
            }
            let mut next_model = model.clone();
            if result_matches_model(history, operation_index, &mut next_model)
                && search(
                    history,
                    predecessor_masks,
                    full_mask,
                    completed_mask | operation_mask,
                    &next_model,
                )
            {
                return true;
            }
        }
        false
    }

    search(history, &predecessor_masks, full_mask, 0, initial_model)
}

async fn check_bounded_read_write_history<S: DodbService + 'static>(service: Arc<S>) {
    const HISTORY_FIXTURE_SEED: u64 = 20261019;
    const PARTITION: &[u8] = b"bounded-history";
    let first_key = key(PARTITION, b"a");
    let second_key = key(PARTITION, b"b");
    let condition_only_key = key(PARTITION, b"condition");
    let mut initial_model = BTreeMap::new();
    for (fixture_index, seeded_key, value_suffix) in [
        (0u64, first_key.clone(), "old-a"),
        (1u64, second_key.clone(), "old-b"),
        (2u64, condition_only_key.clone(), "stable"),
    ] {
        let seeded_value = format!("{HISTORY_FIXTURE_SEED}:{value_suffix}").into_bytes();
        let revision = match execute(
            service.as_ref(),
            Request::Put {
                key: seeded_key.clone(),
                value: seeded_value.clone(),
            },
        )
        .await
        .unwrap()
        {
            Response::Put(revision) => revision,
            other => panic!("unexpected fixture response {fixture_index}: {other:?}"),
        };
        initial_model.insert(
            seeded_key,
            ModelEntry {
                value: Some(seeded_value),
                revision,
            },
        );
    }

    let operations = vec![
        Request::Put {
            key: first_key.clone(),
            value: b"put-a".to_vec(),
        },
        Request::Put {
            key: second_key.clone(),
            value: b"put-b".to_vec(),
        },
        Request::Transact {
            request: TransactionRequest::new(
                vec![TransactionCondition::Exists {
                    key: condition_only_key,
                }],
                vec![
                    TransactionMutation::Put {
                        key: first_key.clone(),
                        value: b"txn-a".to_vec(),
                    },
                    TransactionMutation::Put {
                        key: second_key.clone(),
                        value: b"txn-b".to_vec(),
                    },
                ],
            ),
        },
        Request::Query {
            pk: PrimaryKey::new(PARTITION.to_vec()),
            exclusive_after_sk: None,
            limit: 3,
        },
        Request::Get {
            key: first_key.clone(),
        },
        Request::Scan {
            exclusive_after_key: None,
            limit: 3,
        },
    ];
    assert!(operations.len() <= 8);

    let event_clock = Arc::new(AtomicU64::new(0));
    let start_barrier = Arc::new(Barrier::new(operations.len() + 1));
    let mut tasks = tokio::task::JoinSet::new();
    for (operation_index, request) in operations.into_iter().enumerate() {
        let service = Arc::clone(&service);
        let event_clock = Arc::clone(&event_clock);
        let start_barrier = Arc::clone(&start_barrier);
        tasks.spawn(async move {
            start_barrier.wait().await;
            let invocation_order = event_clock.fetch_add(1, Ordering::SeqCst);
            let result = execute(service.as_ref(), request.clone()).await;
            let response_order = event_clock.fetch_add(1, Ordering::SeqCst);
            RecordedOperation {
                operation_index,
                invocation_order,
                response_order,
                request,
                outcome: record_outcome(result),
            }
        });
    }
    start_barrier.wait().await;
    let mut history = Vec::new();
    while let Some(result) = tasks.join_next().await {
        history.push(result.unwrap());
    }
    history.sort_by_key(|operation| operation.invocation_order);
    assert_eq!(history.len(), 6);
    assert_eq!(
        history
            .iter()
            .map(|operation| operation.operation_index)
            .collect::<BTreeSet<_>>()
            .len(),
        6
    );
    assert!(
        is_linearizable_history(&history, &initial_model),
        "fixture_seed={HISTORY_FIXTURE_SEED}; schedule_seed=none; operations={history:#?}; initial_model={initial_model:#?}"
    );
    service.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn bounded_read_write_histories_replay_for_blink_and_rocksdb_modes() {
    let blink_directory = temporary_db_directory();
    let blink_service = BlinkTenantService::new(LocalTenantServiceConfig {
        data_dir: blink_directory.path().to_path_buf(),
        blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 1 },
        ..LocalTenantServiceConfig::default()
    })
    .unwrap();
    check_bounded_read_write_history(Arc::new(blink_service)).await;

    for condition_key_lock_mode in [
        ConditionKeyLockMode::Exclusive,
        ConditionKeyLockMode::Shared,
    ] {
        for allocator in [
            RevisionAllocatorMode::Serialized,
            RevisionAllocatorMode::ReservedRanges,
        ] {
            let rocks_directory = temporary_db_directory();
            let mut config = RocksDbServiceConfig::new(rocks_directory.path());
            config.condition_key_lock_mode = condition_key_lock_mode;
            config.revision_allocator = allocator;
            check_bounded_read_write_history(Arc::new(RocksDbService::new(config).unwrap())).await;
        }
    }
}

#[test]
fn bounded_history_checker_rejects_stale_read_and_partial_atomic_snapshot() {
    let partition = b"synthetic-history";
    let first_key = key(partition, b"a");
    let second_key = key(partition, b"b");
    let first_revision = Revision::new(1);
    let second_revision = Revision::new(1);
    let committed_revision = Revision::new(2);
    let mut initial_model = BTreeMap::new();
    initial_model.insert(
        first_key.clone(),
        ModelEntry {
            value: Some(b"old-a".to_vec()),
            revision: first_revision,
        },
    );
    initial_model.insert(
        second_key.clone(),
        ModelEntry {
            value: Some(b"old-b".to_vec()),
            revision: second_revision,
        },
    );

    let stale_read_history = vec![
        RecordedOperation {
            operation_index: 0,
            invocation_order: 0,
            response_order: 1,
            request: Request::Put {
                key: first_key.clone(),
                value: b"new-a".to_vec(),
            },
            outcome: RecordedOutcome::Response(Response::Put(committed_revision)),
        },
        RecordedOperation {
            operation_index: 1,
            invocation_order: 2,
            response_order: 3,
            request: Request::Get {
                key: first_key.clone(),
            },
            outcome: RecordedOutcome::Response(Response::Get(RevisionState::present(
                b"old-a".to_vec(),
                first_revision,
            ))),
        },
    ];
    assert!(!is_linearizable_history(
        &stale_read_history,
        &initial_model
    ));

    let atomic_transaction = Request::Transact {
        request: TransactionRequest::new(
            Vec::new(),
            vec![
                TransactionMutation::Put {
                    key: first_key.clone(),
                    value: b"new-a".to_vec(),
                },
                TransactionMutation::Put {
                    key: second_key.clone(),
                    value: b"new-b".to_vec(),
                },
            ],
        ),
    };
    let partial_query_history = vec![
        RecordedOperation {
            operation_index: 0,
            invocation_order: 0,
            response_order: 3,
            request: atomic_transaction,
            outcome: RecordedOutcome::Response(Response::Transact(
                dodb_service::TransactionOutcome::committed(Lsn::new(2)),
            )),
        },
        RecordedOperation {
            operation_index: 1,
            invocation_order: 1,
            response_order: 2,
            request: Request::Query {
                pk: PrimaryKey::new(partition.to_vec()),
                exclusive_after_sk: None,
                limit: 2,
            },
            outcome: RecordedOutcome::Response(Response::Query(vec![
                Document {
                    key: first_key,
                    value: b"new-a".to_vec(),
                    revision: committed_revision,
                },
                Document {
                    key: second_key,
                    value: b"old-b".to_vec(),
                    revision: second_revision,
                },
            ])),
        },
    ];
    assert!(!is_linearizable_history(
        &partial_query_history,
        &initial_model
    ));
}

#[test]
fn bounded_history_checker_accepts_reverse_global_revision_order_for_independent_keys() {
    let first_key = key(b"synthetic-independent", b"a");
    let second_key = key(b"synthetic-independent", b"b");
    let first_revision = Revision::new(3);
    let second_revision = Revision::new(2);
    let history = vec![
        RecordedOperation {
            operation_index: 0,
            invocation_order: 0,
            response_order: 2,
            request: Request::Put {
                key: first_key.clone(),
                value: b"value-a".to_vec(),
            },
            outcome: RecordedOutcome::Response(Response::Put(first_revision)),
        },
        RecordedOperation {
            operation_index: 1,
            invocation_order: 1,
            response_order: 3,
            request: Request::Put {
                key: second_key.clone(),
                value: b"value-b".to_vec(),
            },
            outcome: RecordedOutcome::Response(Response::Put(second_revision)),
        },
        RecordedOperation {
            operation_index: 2,
            invocation_order: 4,
            response_order: 5,
            request: Request::Get {
                key: first_key.clone(),
            },
            outcome: RecordedOutcome::Response(Response::Get(RevisionState::present(
                b"value-a".to_vec(),
                first_revision,
            ))),
        },
        RecordedOperation {
            operation_index: 3,
            invocation_order: 6,
            response_order: 7,
            request: Request::Get {
                key: second_key.clone(),
            },
            outcome: RecordedOutcome::Response(Response::Get(RevisionState::present(
                b"value-b".to_vec(),
                second_revision,
            ))),
        },
    ];
    assert!(is_linearizable_history(&history, &BTreeMap::new()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn segmented_sync_and_background_preserve_conditional_and_linearizable_histories() {
    for execution in [
        dodb_server::BlinkCheckpointExecution::Sync,
        dodb_server::BlinkCheckpointExecution::Background,
    ] {
        for conditional in [true, false] {
            let directory = temporary_db_directory();
            let service = Arc::new(
                BlinkTenantService::new(LocalTenantServiceConfig {
                    data_dir: directory.path().to_path_buf(),
                    blink_wal_layout: dodb_server::BlinkWalLayout::Segmented {
                        max_segment_bytes: 1,
                    },
                    blink_checkpoint_execution: execution,
                    blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold {
                        threshold_bytes: 1,
                    },
                    ..LocalTenantServiceConfig::default()
                })
                .unwrap(),
            );
            if conditional {
                check_conditional_history(Arc::clone(&service)).await;
            } else {
                check_bounded_read_write_history(Arc::clone(&service)).await;
            }
            service.shutdown_checked().await.unwrap();
            assert_eq!(service.checkpoint_metrics_snapshot().checkpoints_failed, 0);
        }
    }
}
