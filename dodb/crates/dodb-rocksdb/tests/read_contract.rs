use dodb_core::{
    DocumentKey, Error, PrimaryKey, RevisionState, SortKey, TenantId, TransactionMutation,
    TransactionRequest,
};
use dodb_protocol::{ProtocolLimits, ResponseEnvelope, encode_response};
use dodb_rocksdb::{
    ConditionKeyLockMode, RevisionAllocatorMode, RocksDbService, RocksDbServiceConfig,
};
use dodb_server::{BlinkTenantService, LocalTenantService, LocalTenantServiceConfig};
use dodb_service::{Document, DodbService, ExecutionBudget, Request, Response};
use std::path::PathBuf;
use tempfile::TempDir;

const MAXIMUM_BUDGET: ExecutionBudget = ExecutionBudget::new(2 * 1024 * 1024);

fn temporary_db_directory() -> TempDir {
    match std::env::var_os("DODB_TEST_DATA_DIR") {
        Some(path) => TempDir::new_in(path).unwrap(),
        None => TempDir::new().unwrap(),
    }
}

fn key(pk: &[u8], sk: &[u8]) -> DocumentKey {
    DocumentKey::new(pk.to_vec(), sk.to_vec())
}

async fn execute<S: DodbService>(
    service: &S,
    tenant: u64,
    request: Request,
    budget: ExecutionBudget,
) -> Result<Response, Error> {
    service
        .execute(TenantId::new(tenant), request, budget)
        .await
}

fn projections(documents: &[Document]) -> Vec<(DocumentKey, Vec<u8>)> {
    documents
        .iter()
        .map(|document| (document.key.clone(), document.value.clone()))
        .collect()
}

fn query_response(response: Response) -> Vec<Document> {
    match response {
        Response::Query(documents) => documents,
        other => panic!("unexpected response: {other:?}"),
    }
}

fn scan_response(response: Response) -> Vec<Document> {
    match response {
        Response::Scan(documents) => documents,
        other => panic!("unexpected response: {other:?}"),
    }
}

async fn check_read_contract<S: DodbService>(service: &S) {
    let empty_scan = execute(
        service,
        90,
        Request::Scan {
            exclusive_after_key: None,
            limit: 10,
        },
        ExecutionBudget::new(17),
    )
    .await
    .unwrap();
    assert_eq!(empty_scan, Response::Scan(Vec::new()));
    assert!(matches!(
        execute(
            service,
            90,
            Request::Scan {
                exclusive_after_key: None,
                limit: 10,
            },
            ExecutionBudget::new(16),
        )
        .await,
        Err(Error::ResponseTooLarge(_))
    ));
    assert_eq!(
        query_response(
            execute(
                service,
                90,
                Request::Query {
                    pk: PrimaryKey::new(b"absent".to_vec()),
                    exclusive_after_sk: None,
                    limit: 10,
                },
                MAXIMUM_BUDGET,
            )
            .await
            .unwrap()
        ),
        Vec::<Document>::new()
    );

    let deleted_key = key(b"p1", &[0x7f]);
    let documents = [
        (key(b"p1", &[0]), vec![0, 255, 1]),
        (key(b"p1", &[0, 0]), vec![2, 0, 254]),
        (deleted_key.clone(), b"deleted".to_vec()),
        (key(b"p1", &[0xff]), vec![255, 0, 128]),
        (key(b"p1", &[0xff, 0]), b"partition-end".to_vec()),
        (key(b"p2", &[0]), b"next-partition".to_vec()),
        (key(&[0, 255, 0], &[255]), b"binary-partition".to_vec()),
    ];
    for (document_key, value) in &documents {
        execute(
            service,
            1,
            Request::Put {
                key: document_key.clone(),
                value: value.clone(),
            },
            MAXIMUM_BUDGET,
        )
        .await
        .unwrap();
    }
    execute(
        service,
        1,
        Request::Delete {
            key: deleted_key.clone(),
        },
        MAXIMUM_BUDGET,
    )
    .await
    .unwrap();
    assert!(
        projections(&query_response(
            execute(
                service,
                1,
                Request::Query {
                    pk: PrimaryKey::new(b"p1".to_vec()),
                    exclusive_after_sk: None,
                    limit: usize::MAX,
                },
                MAXIMUM_BUDGET,
            )
            .await
            .unwrap()
        ))
        .iter()
        .all(|(document_key, _)| *document_key != deleted_key)
    );
    let reinserted_key = key(b"p1", &[0x7f]);
    execute(
        service,
        1,
        Request::Put {
            key: reinserted_key.clone(),
            value: b"reinserted".to_vec(),
        },
        MAXIMUM_BUDGET,
    )
    .await
    .unwrap();

    let query_expected = vec![
        (key(b"p1", &[0]), vec![0, 255, 1]),
        (key(b"p1", &[0, 0]), vec![2, 0, 254]),
        (key(b"p1", &[0x7f]), b"reinserted".to_vec()),
        (key(b"p1", &[0xff]), vec![255, 0, 128]),
        (key(b"p1", &[0xff, 0]), b"partition-end".to_vec()),
    ];
    let query = |cursor: Option<SortKey>, limit| Request::Query {
        pk: PrimaryKey::new(b"p1".to_vec()),
        exclusive_after_sk: cursor,
        limit,
    };
    assert_eq!(
        projections(&query_response(
            execute(service, 1, query(None, usize::MAX), MAXIMUM_BUDGET)
                .await
                .unwrap()
        )),
        query_expected
    );
    assert_eq!(
        projections(&query_response(
            execute(
                service,
                1,
                query(Some(SortKey::new(vec![0])), usize::MAX),
                MAXIMUM_BUDGET,
            )
            .await
            .unwrap()
        )),
        query_expected[1..]
    );
    assert_eq!(
        projections(&query_response(
            execute(
                service,
                1,
                query(Some(SortKey::new(vec![0x80])), usize::MAX),
                MAXIMUM_BUDGET,
            )
            .await
            .unwrap()
        )),
        query_expected[3..]
    );
    assert_eq!(
        query_response(
            execute(
                service,
                1,
                query(Some(SortKey::new(vec![0xff, 0])), usize::MAX),
                MAXIMUM_BUDGET,
            )
            .await
            .unwrap()
        ),
        Vec::<Document>::new()
    );
    assert_eq!(
        query_response(
            execute(service, 1, query(None, 0), MAXIMUM_BUDGET)
                .await
                .unwrap()
        ),
        Vec::<Document>::new()
    );

    let mut scan_expected = vec![
        (key(&[0, 255, 0], &[255]), b"binary-partition".to_vec()),
        (key(b"p1", &[0]), vec![0, 255, 1]),
        (key(b"p1", &[0, 0]), vec![2, 0, 254]),
        (key(b"p1", &[0x7f]), b"reinserted".to_vec()),
        (key(b"p1", &[0xff]), vec![255, 0, 128]),
        (key(b"p1", &[0xff, 0]), b"partition-end".to_vec()),
        (key(b"p2", &[0]), b"next-partition".to_vec()),
    ];
    scan_expected.sort_by(|left, right| left.0.cmp(&right.0));
    let scan = |cursor: Option<DocumentKey>, limit| Request::Scan {
        exclusive_after_key: cursor,
        limit,
    };
    let all_scan = scan_response(
        execute(service, 1, scan(None, usize::MAX), MAXIMUM_BUDGET)
            .await
            .unwrap(),
    );
    assert_eq!(projections(&all_scan), scan_expected);
    assert_eq!(
        projections(&scan_response(
            execute(
                service,
                1,
                scan(Some(scan_expected[0].0.clone()), usize::MAX),
                MAXIMUM_BUDGET,
            )
            .await
            .unwrap()
        )),
        scan_expected[1..]
    );
    let partition_boundary = key(b"p1", &[0xff, 0]);
    let partition_boundary_index = scan_expected
        .iter()
        .position(|(document_key, _)| *document_key == partition_boundary)
        .unwrap();
    assert_eq!(
        projections(&scan_response(
            execute(
                service,
                1,
                scan(Some(partition_boundary), usize::MAX),
                MAXIMUM_BUDGET,
            )
            .await
            .unwrap()
        )),
        scan_expected[partition_boundary_index + 1..]
    );
    assert_eq!(
        projections(&scan_response(
            execute(
                service,
                1,
                scan(Some(key(b"p1", &[0x80])), usize::MAX),
                MAXIMUM_BUDGET,
            )
            .await
            .unwrap()
        )),
        scan_expected
            .iter()
            .filter(|(document_key, _)| *document_key > key(b"p1", &[0x80]))
            .cloned()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        projections(&scan_response(
            execute(
                service,
                1,
                scan(Some(scan_expected.last().unwrap().0.clone()), usize::MAX),
                MAXIMUM_BUDGET,
            )
            .await
            .unwrap()
        )),
        Vec::<(DocumentKey, Vec<u8>)>::new()
    );
    assert_eq!(
        scan_response(
            execute(service, 1, scan(None, 0), MAXIMUM_BUDGET)
                .await
                .unwrap()
        ),
        Vec::<Document>::new()
    );

    let mut pages = Vec::new();
    let mut cursor = None;
    loop {
        let page = scan_response(
            execute(service, 1, scan(cursor.clone(), 2), MAXIMUM_BUDGET)
                .await
                .unwrap(),
        );
        if page.is_empty() {
            break;
        }
        cursor = page.last().map(|document| document.key.clone());
        pages.extend(projections(&page));
    }
    assert_eq!(pages, scan_expected);

    let large_document = vec![0x5a; 2_000];
    let large_key = key(b"budget", b"single");
    execute(
        service,
        1,
        Request::Put {
            key: large_key.clone(),
            value: large_document.clone(),
        },
        MAXIMUM_BUDGET,
    )
    .await
    .unwrap();
    assert!(matches!(
        execute(
            service,
            1,
            Request::Get {
                key: large_key.clone(),
            },
            ExecutionBudget::new(100),
        )
        .await,
        Err(Error::ResponseTooLarge(_))
    ));
    assert!(matches!(
        execute(
            service,
            1,
            Request::Get { key: large_key },
            MAXIMUM_BUDGET,
        )
        .await,
        Ok(Response::Get(RevisionState::Present { value, .. })) if value == large_document
    ));
    let exact_response = Response::Query(vec![Document {
        key: key(b"budget", b"single"),
        value: large_document,
        revision: dodb_core::Revision::new(1),
    }]);
    let exact_budget = encode_response(
        &ResponseEnvelope::Success(exact_response),
        ProtocolLimits::default(),
    )
    .unwrap()
    .len();
    assert_eq!(
        projections(&query_response(
            execute(
                service,
                1,
                Request::Query {
                    pk: PrimaryKey::new(b"budget".to_vec()),
                    exclusive_after_sk: None,
                    limit: 1,
                },
                ExecutionBudget::new(exact_budget),
            )
            .await
            .unwrap()
        )),
        vec![(key(b"budget", b"single"), vec![0x5a; 2_000])]
    );
    assert!(matches!(
        execute(
            service,
            1,
            Request::Query {
                pk: PrimaryKey::new(b"budget".to_vec()),
                exclusive_after_sk: None,
                limit: 1,
            },
            ExecutionBudget::new(exact_budget - 1),
        )
        .await,
        Err(Error::ResponseTooLarge(_))
    ));
    assert!(matches!(
        execute(
            service,
            1,
            Request::Query {
                pk: PrimaryKey::new(b"budget".to_vec()),
                exclusive_after_sk: None,
                limit: 1,
            },
            ExecutionBudget::new(100),
        )
        .await,
        Err(Error::ResponseTooLarge(_))
    ));

    let shared_key = key(b"same", b"key");
    for tenant in 10..14 {
        execute(
            service,
            tenant,
            Request::Put {
                key: shared_key.clone(),
                value: tenant.to_be_bytes().to_vec(),
            },
            MAXIMUM_BUDGET,
        )
        .await
        .unwrap();
        let tenant_scan = scan_response(
            execute(service, tenant, scan(None, 10), MAXIMUM_BUDGET)
                .await
                .unwrap(),
        );
        assert_eq!(
            projections(&tenant_scan),
            vec![(shared_key.clone(), tenant.to_be_bytes().to_vec())]
        );
    }

    let atomic_keys = [key(b"atomic-read", b"a"), key(b"atomic-read", b"b")];
    let old_values = [b"old-a".to_vec(), b"old-b".to_vec()];
    let new_values = [b"new-a".to_vec(), b"new-b".to_vec()];
    for (document_key, value) in atomic_keys.iter().zip(&old_values) {
        execute(
            service,
            50,
            Request::Put {
                key: document_key.clone(),
                value: value.clone(),
            },
            MAXIMUM_BUDGET,
        )
        .await
        .unwrap();
    }
    let barrier = tokio::sync::Barrier::new(2);
    let transaction = async {
        barrier.wait().await;
        execute(
            service,
            50,
            Request::Transact {
                request: TransactionRequest::new(
                    Vec::new(),
                    atomic_keys
                        .iter()
                        .zip(&new_values)
                        .map(|(document_key, value)| TransactionMutation::Put {
                            key: document_key.clone(),
                            value: value.clone(),
                        })
                        .collect(),
                ),
            },
            MAXIMUM_BUDGET,
        )
        .await
    };
    let read = async {
        barrier.wait().await;
        execute(
            service,
            50,
            Request::Query {
                pk: PrimaryKey::new(b"atomic-read".to_vec()),
                exclusive_after_sk: None,
                limit: 10,
            },
            MAXIMUM_BUDGET,
        )
        .await
    };
    let (transaction_result, read_result) = tokio::join!(transaction, read);
    assert!(matches!(transaction_result, Ok(Response::Transact(_))));
    let observed = projections(&query_response(read_result.unwrap()));
    let old_state = atomic_keys
        .iter()
        .cloned()
        .zip(old_values.iter().cloned())
        .collect::<Vec<_>>();
    let new_state = atomic_keys
        .iter()
        .cloned()
        .zip(new_values.iter().cloned())
        .collect::<Vec<_>>();
    assert!(observed == old_state || observed == new_state);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn btree_blink_and_rocksdb_share_the_same_read_contract() {
    let native_directory = temporary_db_directory();
    let blink_directory = temporary_db_directory();
    let native = LocalTenantService::new(LocalTenantServiceConfig {
        data_dir: PathBuf::from(native_directory.path()),
        ..LocalTenantServiceConfig::default()
    })
    .unwrap();
    let blink = BlinkTenantService::new(LocalTenantServiceConfig {
        data_dir: PathBuf::from(blink_directory.path()),
        ..LocalTenantServiceConfig::default()
    })
    .unwrap();
    check_read_contract(&native).await;
    check_read_contract(&blink).await;

    for condition_key_lock_mode in [
        ConditionKeyLockMode::Exclusive,
        ConditionKeyLockMode::Shared,
    ] {
        for revision_allocator in [
            RevisionAllocatorMode::Serialized,
            RevisionAllocatorMode::ReservedRanges,
        ] {
            let rocks_directory = temporary_db_directory();
            let mut config = RocksDbServiceConfig::new(rocks_directory.path());
            config.condition_key_lock_mode = condition_key_lock_mode;
            config.revision_allocator = revision_allocator;
            let rocksdb = RocksDbService::new(config).unwrap();
            check_read_contract(&rocksdb).await;
            rocksdb.shutdown().await;
        }
    }

    native.shutdown().await;
    blink.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn rocksdb_query_and_scan_survive_shutdown_and_reopen() {
    let directory = temporary_db_directory();
    let config = RocksDbServiceConfig::new(directory.path());
    let service = RocksDbService::new(config.clone()).unwrap();
    let expected_keys = [key(b"reopen", &[0]), key(b"reopen", &[255])];
    for (position, document_key) in expected_keys.iter().enumerate() {
        execute(
            &service,
            7,
            Request::Put {
                key: document_key.clone(),
                value: vec![position as u8, 0, 255],
            },
            MAXIMUM_BUDGET,
        )
        .await
        .unwrap();
    }
    let before_query = projections(&query_response(
        execute(
            &service,
            7,
            Request::Query {
                pk: PrimaryKey::new(b"reopen".to_vec()),
                exclusive_after_sk: None,
                limit: 10,
            },
            MAXIMUM_BUDGET,
        )
        .await
        .unwrap(),
    ));
    let before_scan = projections(&scan_response(
        execute(
            &service,
            7,
            Request::Scan {
                exclusive_after_key: None,
                limit: 10,
            },
            MAXIMUM_BUDGET,
        )
        .await
        .unwrap(),
    ));
    service.shutdown().await;

    let reopened = RocksDbService::new(config).unwrap();
    let after_query = projections(&query_response(
        execute(
            &reopened,
            7,
            Request::Query {
                pk: PrimaryKey::new(b"reopen".to_vec()),
                exclusive_after_sk: None,
                limit: 10,
            },
            MAXIMUM_BUDGET,
        )
        .await
        .unwrap(),
    ));
    let after_scan = projections(&scan_response(
        execute(
            &reopened,
            7,
            Request::Scan {
                exclusive_after_key: None,
                limit: 10,
            },
            MAXIMUM_BUDGET,
        )
        .await
        .unwrap(),
    ));
    assert_eq!(after_query, before_query);
    assert_eq!(after_scan, before_scan);
    reopened.shutdown().await;
}
