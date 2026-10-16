use dodb_core::{
    DocumentKey, Error, PrimaryKey, Revision, RevisionState, TenantId, TransactionCondition,
    TransactionMutation, TransactionRequest,
};
use dodb_rocksdb::{RevisionAllocatorMode, RocksDbService, RocksDbServiceConfig};
use dodb_server::{BlinkTenantService, LocalTenantServiceConfig};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use std::sync::Arc;
use tempfile::TempDir;

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
        Ok(Response::Transact(outcome)) => outcome.commit_lsn.map(Revision::new),
        Err(Error::Conflict(_)) => None,
        other => panic!("unexpected first transaction result: {other:?}"),
    };
    let second_commit = match second_result {
        Ok(Response::Transact(outcome)) => outcome.commit_lsn.map(Revision::new),
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
        ..LocalTenantServiceConfig::default()
    })
    .unwrap();
    check_conditional_history(Arc::new(blink_service)).await;

    for allocator in [
        RevisionAllocatorMode::Serialized,
        RevisionAllocatorMode::ReservedRanges,
    ] {
        let rocks_directory = temporary_db_directory();
        let mut config = RocksDbServiceConfig::new(rocks_directory.path());
        config.revision_allocator = allocator;
        check_conditional_history(Arc::new(RocksDbService::new(config).unwrap())).await;
    }
}
