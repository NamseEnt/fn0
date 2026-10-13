use std::collections::BTreeMap;

use dodb_core::{
    DocumentKey, Error, Revision, RevisionState, TenantId, TransactionCondition,
    TransactionMutation, TransactionRequest,
};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use tokio::sync::Barrier;

const BUDGET: ExecutionBudget = ExecutionBudget::new(1 << 20);

async fn execute<S: DodbService>(
    service: &S,
    tenant: u64,
    request: Request,
) -> Result<Response, Error> {
    service
        .execute(TenantId::new(tenant), request, BUDGET)
        .await
}

async fn get<S: DodbService>(service: &S, tenant: u64, key: DocumentKey) -> RevisionState {
    match execute(service, tenant, Request::Get { key })
        .await
        .unwrap()
    {
        Response::Get(state) => state,
        other => panic!("unexpected response: {other:?}"),
    }
}

fn key(pk: &[u8], sk: &[u8]) -> DocumentKey {
    DocumentKey::new(pk.to_vec(), sk.to_vec())
}

fn put(key: DocumentKey, value: &[u8]) -> TransactionMutation {
    TransactionMutation::Put {
        key,
        value: value.to_vec(),
    }
}

fn delete(key: DocumentKey) -> TransactionMutation {
    TransactionMutation::Delete { key }
}

fn condition_revision(key: DocumentKey, expected_revision: Revision) -> TransactionCondition {
    TransactionCondition::RevisionEquals {
        key,
        expected_revision,
    }
}

async fn transact<S: DodbService>(
    service: &S,
    tenant: u64,
    request: TransactionRequest,
) -> Result<Response, Error> {
    execute(service, tenant, Request::Transact { request }).await
}

async fn assert_present<S: DodbService>(
    service: &S,
    tenant: u64,
    document_key: DocumentKey,
    expected_value: &[u8],
) -> Revision {
    match get(service, tenant, document_key).await {
        RevisionState::Present { value, revision } => {
            assert_eq!(value, expected_value);
            revision
        }
        state => panic!("expected present state, got {state:?}"),
    }
}

async fn assert_missing<S: DodbService>(
    service: &S,
    tenant: u64,
    document_key: DocumentKey,
) -> Revision {
    match get(service, tenant, document_key).await {
        RevisionState::Missing { revision } => revision,
        state => panic!("expected missing state, got {state:?}"),
    }
}

pub async fn check_transaction_contract<S, F>(service: S, reopen: F)
where
    S: DodbService,
    F: Fn() -> S,
{
    let tenant = 101;
    let binary_key = key(&[0, 255, 0], &[255, 0, 128]);
    let binary_value = [0, 255, 1, 0, 128, 255];
    let absent_revision = assert_missing(&service, tenant, binary_key.clone()).await;
    assert_eq!(absent_revision, Revision::ZERO);

    assert!(matches!(
        execute(
            &service,
            tenant,
            Request::Put {
                key: binary_key.clone(),
                value: binary_value.to_vec(),
            },
        )
        .await,
        Ok(Response::Put(_))
    ));
    let first_revision = assert_present(&service, tenant, binary_key.clone(), &binary_value).await;
    let updated_revision = match execute(
        &service,
        tenant,
        Request::Put {
            key: binary_key.clone(),
            value: vec![9, 0, 8],
        },
    )
    .await
    .unwrap()
    {
        Response::Put(revision) => revision,
        other => panic!("unexpected response: {other:?}"),
    };
    assert!(updated_revision > first_revision);
    let binary_deleted_revision = match execute(
        &service,
        tenant,
        Request::Delete {
            key: binary_key.clone(),
        },
    )
    .await
    .unwrap()
    {
        Response::Delete(revision) => revision,
        other => panic!("unexpected response: {other:?}"),
    };
    assert_eq!(
        assert_missing(&service, tenant, binary_key.clone()).await,
        binary_deleted_revision
    );

    let deleted_missing_key = key(b"delete-missing", b"once");
    let before_missing_delete = assert_missing(&service, tenant, deleted_missing_key.clone()).await;
    let deleted_missing_revision = match execute(
        &service,
        tenant,
        Request::Delete {
            key: deleted_missing_key.clone(),
        },
    )
    .await
    .unwrap()
    {
        Response::Delete(revision) => revision,
        other => panic!("unexpected response: {other:?}"),
    };
    assert!(deleted_missing_revision > before_missing_delete);
    assert_eq!(
        assert_missing(&service, tenant, deleted_missing_key.clone()).await,
        deleted_missing_revision
    );

    let empty_request = transact(&service, tenant, TransactionRequest::default()).await;
    assert!(matches!(empty_request, Err(Error::InvalidRequest(_))));

    let exists_key = key(b"conditions", b"exists");
    let exists_revision = match execute(
        &service,
        tenant,
        Request::Put {
            key: exists_key.clone(),
            value: b"exists-value".to_vec(),
        },
    )
    .await
    .unwrap()
    {
        Response::Put(revision) => revision,
        other => panic!("unexpected response: {other:?}"),
    };
    assert!(
        transact(
            &service,
            tenant,
            TransactionRequest::new(
                vec![TransactionCondition::Exists {
                    key: exists_key.clone(),
                }],
                vec![put(key(b"conditions", b"exists-mutated"), b"exists-ok")],
            ),
        )
        .await
        .is_ok()
    );
    let exists_condition_mutation = key(b"conditions", b"not-exists-must-not-write");
    assert!(matches!(
        transact(
            &service,
            tenant,
            TransactionRequest::new(
                vec![TransactionCondition::NotExists {
                    key: exists_key.clone(),
                }],
                vec![put(exists_condition_mutation.clone(), b"must-not-appear")],
            ),
        )
        .await,
        Err(Error::Conflict(_))
    ));
    assert_eq!(
        assert_missing(&service, tenant, exists_condition_mutation.clone()).await,
        Revision::ZERO
    );

    let not_exists_key = key(b"conditions", b"not-exists");
    assert!(
        transact(
            &service,
            tenant,
            TransactionRequest::new(
                vec![TransactionCondition::NotExists {
                    key: not_exists_key.clone(),
                }],
                vec![put(not_exists_key.clone(), b"not-exists-ok")],
            ),
        )
        .await
        .is_ok()
    );
    let same_key_revision =
        assert_present(&service, tenant, not_exists_key.clone(), b"not-exists-ok").await;
    assert!(
        transact(
            &service,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(
                    not_exists_key.clone(),
                    same_key_revision
                )],
                vec![put(not_exists_key.clone(), b"same-key-ok")],
            ),
        )
        .await
        .is_ok()
    );
    assert!(
        transact(
            &service,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(exists_key.clone(), exists_revision)],
                vec![put(key(b"conditions", b"revision-equals"), b"revision-ok")],
            ),
        )
        .await
        .is_ok()
    );

    let condition_only = TransactionRequest::new(
        vec![
            TransactionCondition::Exists {
                key: exists_key.clone(),
            },
            TransactionCondition::NotExists {
                key: key(b"conditions", b"untouched-missing"),
            },
        ],
        Vec::new(),
    );
    assert_eq!(
        transact(&service, tenant, condition_only).await.unwrap(),
        Response::Transact(dodb_service::TransactionOutcome::conditions_satisfied())
    );
    assert!(matches!(
        transact(
            &service,
            tenant,
            TransactionRequest::new(
                vec![TransactionCondition::Exists {
                    key: key(b"conditions", b"absent"),
                }],
                Vec::new(),
            ),
        )
        .await,
        Err(Error::Conflict(_))
    ));
    assert!(
        transact(
            &service,
            tenant,
            TransactionRequest::new(
                vec![TransactionCondition::NotExists {
                    key: key(b"conditions", b"empty"),
                }],
                Vec::new(),
            ),
        )
        .await
        .is_ok()
    );

    let duplicate_condition = TransactionRequest::new(
        vec![
            TransactionCondition::NotExists {
                key: not_exists_key.clone(),
            },
            TransactionCondition::Exists {
                key: not_exists_key.clone(),
            },
        ],
        Vec::new(),
    );
    assert!(matches!(
        transact(&service, tenant, duplicate_condition).await,
        Err(Error::InvalidRequest(_))
    ));
    let duplicate_mutation = TransactionRequest::new(
        Vec::new(),
        vec![
            put(key(b"validation", b"duplicate"), b"first"),
            delete(key(b"validation", b"duplicate")),
        ],
    );
    assert!(matches!(
        transact(&service, tenant, duplicate_mutation).await,
        Err(Error::InvalidRequest(_))
    ));

    let atomic_first = key(b"atomic", b"first");
    let atomic_second = key(b"atomic", b"second");
    let atomic_deleted = key(b"atomic", b"deleted");
    execute(
        &service,
        tenant,
        Request::Put {
            key: atomic_deleted.clone(),
            value: b"old".to_vec(),
        },
    )
    .await
    .unwrap();
    let atomic_previous = [
        assert_missing(&service, tenant, atomic_first.clone()).await,
        assert_missing(&service, tenant, atomic_second.clone()).await,
        assert_present(&service, tenant, atomic_deleted.clone(), b"old").await,
    ];
    let mixed_request = TransactionRequest::new(
        vec![TransactionCondition::NotExists {
            key: key(b"atomic", b"condition-only"),
        }],
        vec![
            put(atomic_first.clone(), b"new-first"),
            put(atomic_second.clone(), b"new-second"),
            delete(atomic_deleted.clone()),
        ],
    );
    let mixed_outcome = transact(&service, tenant, mixed_request).await.unwrap();
    assert!(matches!(mixed_outcome, Response::Transact(outcome) if outcome.commit_lsn.is_some()));
    let first_after = assert_present(&service, tenant, atomic_first.clone(), b"new-first").await;
    let second_after = assert_present(&service, tenant, atomic_second.clone(), b"new-second").await;
    let deleted_after = assert_missing(&service, tenant, atomic_deleted.clone()).await;
    assert_eq!(first_after, second_after);
    assert_eq!(first_after, deleted_after);
    assert!(first_after > atomic_previous.into_iter().max().unwrap());

    for width in [1, 4, 8] {
        let mutations = (0..width)
            .map(|position| {
                put(
                    key(b"width", &[width as u8, position as u8]),
                    &[width as u8, position as u8, 0],
                )
            })
            .collect();
        let response = transact(
            &service,
            tenant,
            TransactionRequest::new(Vec::new(), mutations),
        )
        .await
        .unwrap();
        assert!(matches!(response, Response::Transact(outcome) if outcome.commit_lsn.is_some()));
        let mut transaction_revisions = Vec::new();
        for position in 0..width {
            transaction_revisions.push(
                assert_present(
                    &service,
                    tenant,
                    key(b"width", &[width as u8, position as u8]),
                    &[width as u8, position as u8, 0],
                )
                .await,
            );
        }
        assert!(
            transaction_revisions
                .iter()
                .all(|revision| *revision == transaction_revisions[0])
        );
        assert!(transaction_revisions[0] > Revision::ZERO);
    }

    let failed_atomic_put = key(b"atomic-failure", b"put");
    let failed_atomic_delete = key(b"atomic-failure", b"delete");
    execute(
        &service,
        tenant,
        Request::Put {
            key: failed_atomic_delete.clone(),
            value: b"keep".to_vec(),
        },
    )
    .await
    .unwrap();
    let retained_revision =
        assert_present(&service, tenant, failed_atomic_delete.clone(), b"keep").await;
    let valid_condition_key = key(b"atomic-failure", b"valid-condition");
    execute(
        &service,
        tenant,
        Request::Put {
            key: valid_condition_key.clone(),
            value: b"valid".to_vec(),
        },
    )
    .await
    .unwrap();
    let stale_condition_key = key(b"atomic-failure", b"stale-condition");
    let stale_revision = assert_missing(&service, tenant, stale_condition_key.clone()).await;
    execute(
        &service,
        tenant,
        Request::Put {
            key: stale_condition_key.clone(),
            value: b"now-present".to_vec(),
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        transact(
            &service,
            tenant,
            TransactionRequest::new(
                vec![
                    TransactionCondition::Exists {
                        key: valid_condition_key.clone(),
                    },
                    condition_revision(stale_condition_key.clone(), stale_revision),
                ],
                vec![
                    put(failed_atomic_put.clone(), b"must-not-appear"),
                    delete(failed_atomic_delete.clone())
                ],
            ),
        )
        .await,
        Err(Error::Conflict(_))
    ));
    assert_eq!(
        assert_missing(&service, tenant, failed_atomic_put.clone()).await,
        Revision::ZERO
    );
    assert_eq!(
        assert_present(&service, tenant, failed_atomic_delete.clone(), b"keep").await,
        retained_revision
    );

    let unmodified_condition = key(b"protected", b"condition");
    let dependent_write = key(b"protected", b"dependent-write");
    let observed_missing = assert_missing(&service, tenant, unmodified_condition.clone()).await;
    execute(
        &service,
        tenant,
        Request::Put {
            key: unmodified_condition.clone(),
            value: b"changed".to_vec(),
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        transact(
            &service,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(
                    unmodified_condition.clone(),
                    observed_missing
                )],
                vec![put(dependent_write.clone(), b"must-not-appear")],
            ),
        )
        .await,
        Err(Error::Conflict(_))
    ));
    assert_eq!(
        assert_missing(&service, tenant, dependent_write.clone()).await,
        Revision::ZERO
    );

    let race_key = key(b"race", b"same-revision");
    execute(
        &service,
        tenant,
        Request::Put {
            key: race_key.clone(),
            value: b"initial".to_vec(),
        },
    )
    .await
    .unwrap();
    let shared_revision = assert_present(&service, tenant, race_key.clone(), b"initial").await;
    let start_barrier = std::sync::Arc::new(Barrier::new(3));
    let left_barrier = start_barrier.clone();
    let right_barrier = start_barrier.clone();
    let left_key = race_key.clone();
    let right_key = race_key.clone();
    let left_service = &service;
    let right_service = &service;
    let left_task = async move {
        left_barrier.wait().await;
        transact(
            left_service,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(left_key.clone(), shared_revision)],
                vec![put(left_key, b"left")],
            ),
        )
        .await
    };
    let right_task = async move {
        right_barrier.wait().await;
        transact(
            right_service,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(right_key.clone(), shared_revision)],
                vec![put(right_key, b"right")],
            ),
        )
        .await
    };
    let (left_result, right_result, _) = tokio::join!(left_task, right_task, async {
        start_barrier.wait().await;
    });
    assert_ne!(left_result.is_ok(), right_result.is_ok());
    assert!(matches!(
        &left_result,
        Ok(Response::Transact(_)) | Err(Error::Conflict(_))
    ));
    assert!(matches!(
        &right_result,
        Ok(Response::Transact(_)) | Err(Error::Conflict(_))
    ));
    let race_state = get(&service, tenant, race_key.clone()).await;
    assert!(matches!(
        race_state,
        RevisionState::Present { value, .. } if value == b"left" || value == b"right"
    ));

    let competing_condition = key(b"protected-race", b"condition");
    let competing_target = key(b"protected-race", b"target");
    let protected_revision = assert_missing(&service, tenant, competing_condition.clone()).await;
    let start_barrier = std::sync::Arc::new(Barrier::new(3));
    let transaction_barrier = start_barrier.clone();
    let condition_writer_barrier = start_barrier.clone();
    let transaction_service = &service;
    let condition_writer_service = &service;
    let transaction_condition = competing_condition.clone();
    let writer_condition = competing_condition.clone();
    let transaction_target = competing_target.clone();
    let transaction_task = async move {
        transaction_barrier.wait().await;
        transact(
            transaction_service,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(
                    transaction_condition,
                    protected_revision,
                )],
                vec![put(transaction_target, b"conditional-target")],
            ),
        )
        .await
    };
    let condition_writer_task = async move {
        condition_writer_barrier.wait().await;
        execute(
            condition_writer_service,
            tenant,
            Request::Put {
                key: writer_condition,
                value: b"writer-value".to_vec(),
            },
        )
        .await
    };
    let (protected_result, writer_result, _) =
        tokio::join!(transaction_task, condition_writer_task, async {
            start_barrier.wait().await
        });
    assert!(writer_result.is_ok());
    assert_present(
        &service,
        tenant,
        competing_condition.clone(),
        b"writer-value",
    )
    .await;
    match protected_result {
        Ok(Response::Transact(_)) => {
            assert_present(
                &service,
                tenant,
                competing_target.clone(),
                b"conditional-target",
            )
            .await;
        }
        Err(Error::Conflict(_)) => assert_eq!(
            assert_missing(&service, tenant, competing_target.clone()).await,
            Revision::ZERO
        ),
        other => panic!("unexpected protected transaction result: {other:?}"),
    }

    let unconditional_key = key(b"unconditional", b"same-key");
    execute(
        &service,
        tenant,
        Request::Put {
            key: unconditional_key.clone(),
            value: b"start".to_vec(),
        },
    )
    .await
    .unwrap();
    let conditional_revision =
        assert_present(&service, tenant, unconditional_key.clone(), b"start").await;
    let start_barrier = std::sync::Arc::new(Barrier::new(3));
    let conditional_barrier = start_barrier.clone();
    let unconditional_barrier = start_barrier.clone();
    let conditional_service = &service;
    let unconditional_service = &service;
    let conditional_key_copy = unconditional_key.clone();
    let unconditional_key_copy = unconditional_key.clone();
    let conditional_task = async move {
        conditional_barrier.wait().await;
        transact(
            conditional_service,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(
                    conditional_key_copy.clone(),
                    conditional_revision,
                )],
                vec![put(conditional_key_copy, b"conditional")],
            ),
        )
        .await
    };
    let unconditional_task = async move {
        unconditional_barrier.wait().await;
        execute(
            unconditional_service,
            tenant,
            Request::Put {
                key: unconditional_key_copy,
                value: b"unconditional".to_vec(),
            },
        )
        .await
    };
    let (conditional_result, unconditional_result, _) =
        tokio::join!(conditional_task, unconditional_task, async {
            start_barrier.wait().await
        });
    assert!(unconditional_result.is_ok());
    assert!(matches!(
        conditional_result,
        Ok(Response::Transact(_)) | Err(Error::Conflict(_))
    ));
    assert_eq!(
        assert_present(
            &service,
            tenant,
            unconditional_key.clone(),
            b"unconditional"
        )
        .await,
        get(&service, tenant, unconditional_key.clone())
            .await
            .revision()
    );

    let delete_race_key = key(b"unconditional", b"delete-race");
    execute(
        &service,
        tenant,
        Request::Put {
            key: delete_race_key.clone(),
            value: b"start".to_vec(),
        },
    )
    .await
    .unwrap();
    let delete_race_revision =
        assert_present(&service, tenant, delete_race_key.clone(), b"start").await;
    let start_barrier = std::sync::Arc::new(Barrier::new(3));
    let conditional_barrier = start_barrier.clone();
    let delete_barrier = start_barrier.clone();
    let conditional_service = &service;
    let delete_service = &service;
    let conditional_key_copy = delete_race_key.clone();
    let delete_key_copy = delete_race_key.clone();
    let conditional_task = async move {
        conditional_barrier.wait().await;
        transact(
            conditional_service,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(
                    conditional_key_copy.clone(),
                    delete_race_revision,
                )],
                vec![put(conditional_key_copy, b"conditional")],
            ),
        )
        .await
    };
    let delete_task = async move {
        delete_barrier.wait().await;
        execute(
            delete_service,
            tenant,
            Request::Delete {
                key: delete_key_copy,
            },
        )
        .await
    };
    let (conditional_result, delete_result, _) =
        tokio::join!(conditional_task, delete_task, async {
            start_barrier.wait().await
        });
    assert!(delete_result.is_ok());
    assert!(matches!(
        conditional_result,
        Ok(Response::Transact(_)) | Err(Error::Conflict(_))
    ));
    assert!(
        get(&service, tenant, delete_race_key.clone())
            .await
            .is_missing()
    );

    let aba_key = key(b"aba", b"missing-create-delete-recreate");
    let first_missing_revision = assert_missing(&service, tenant, aba_key.clone()).await;
    execute(
        &service,
        tenant,
        Request::Put {
            key: aba_key.clone(),
            value: b"created".to_vec(),
        },
    )
    .await
    .unwrap();
    let created_revision = assert_present(&service, tenant, aba_key.clone(), b"created").await;
    assert!(created_revision > first_missing_revision);
    execute(
        &service,
        tenant,
        Request::Delete {
            key: aba_key.clone(),
        },
    )
    .await
    .unwrap();
    let deleted_revision = assert_missing(&service, tenant, aba_key.clone()).await;
    assert!(deleted_revision > created_revision);
    assert!(matches!(
        transact(
            &service,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(aba_key.clone(), first_missing_revision)],
                vec![put(key(b"aba", b"blocked"), b"stale")],
            ),
        )
        .await,
        Err(Error::Conflict(_))
    ));
    assert!(
        transact(
            &service,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(aba_key.clone(), deleted_revision)],
                vec![put(key(b"aba", b"current-token"), b"current")],
            ),
        )
        .await
        .is_ok()
    );
    execute(
        &service,
        tenant,
        Request::Delete {
            key: aba_key.clone(),
        },
    )
    .await
    .unwrap();
    let recreated_missing_revision = assert_missing(&service, tenant, aba_key.clone()).await;
    assert!(recreated_missing_revision > deleted_revision);
    execute(
        &service,
        tenant,
        Request::Put {
            key: aba_key.clone(),
            value: b"recreated".to_vec(),
        },
    )
    .await
    .unwrap();
    let recreated_revision = assert_present(&service, tenant, aba_key.clone(), b"recreated").await;
    assert!(recreated_revision > recreated_missing_revision);
    execute(
        &service,
        tenant,
        Request::Delete {
            key: aba_key.clone(),
        },
    )
    .await
    .unwrap();
    let final_missing_revision = assert_missing(&service, tenant, aba_key.clone()).await;
    assert!(final_missing_revision > recreated_revision);

    let snapshot_first = key(b"snapshot-live", b"first");
    let snapshot_second = key(b"snapshot-live", b"second");
    let start_barrier = std::sync::Arc::new(Barrier::new(3));
    let transaction_barrier = start_barrier.clone();
    let read_barrier = start_barrier.clone();
    let transaction_service = &service;
    let read_service = &service;
    let transaction_first = snapshot_first.clone();
    let transaction_second = snapshot_second.clone();
    let transaction_task = async move {
        transaction_barrier.wait().await;
        transact(
            transaction_service,
            tenant,
            TransactionRequest::new(
                Vec::new(),
                vec![
                    put(transaction_first, b"complete-first"),
                    put(transaction_second, b"complete-second"),
                ],
            ),
        )
        .await
    };
    let read_task = async move {
        read_barrier.wait().await;
        execute(
            read_service,
            tenant,
            Request::Query {
                pk: dodb_core::PrimaryKey::new(b"snapshot-live".to_vec()),
                exclusive_after_sk: None,
                limit: 10,
            },
        )
        .await
    };
    let (write_result, read_result, _) = tokio::join!(transaction_task, read_task, async {
        start_barrier.wait().await
    });
    assert!(write_result.is_ok());
    let snapshot_rows = match read_result.unwrap() {
        Response::Query(rows) => rows,
        other => panic!("unexpected snapshot response: {other:?}"),
    };
    assert!(snapshot_rows.is_empty() || snapshot_rows.len() == 2);
    if !snapshot_rows.is_empty() {
        let snapshot_values = snapshot_rows
            .into_iter()
            .map(|document| (document.key, document.value))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            snapshot_values.get(&snapshot_first),
            Some(&b"complete-first".to_vec())
        );
        assert_eq!(
            snapshot_values.get(&snapshot_second),
            Some(&b"complete-second".to_vec())
        );
    }

    let shared_tenant_key = key(b"same-key", b"same-sort-key");
    let tenant_writes = (201_u64..205).map(|tenant_id| {
        let service = &service;
        let shared_tenant_key = shared_tenant_key.clone();
        async move {
            let value = tenant_id.to_be_bytes().to_vec();
            let result = transact(
                service,
                tenant_id,
                TransactionRequest::new(
                    vec![TransactionCondition::NotExists {
                        key: shared_tenant_key.clone(),
                    }],
                    vec![put(shared_tenant_key.clone(), &value)],
                ),
            )
            .await;
            (tenant_id, value, result)
        }
    });
    let tenant_results = futures::future::join_all(tenant_writes).await;
    let mut independent_expectations = BTreeMap::new();
    for (tenant_id, expected_value, result) in tenant_results {
        assert!(result.is_ok());
        independent_expectations.insert(tenant_id, expected_value);
    }
    for (&tenant_id, expected_value) in &independent_expectations {
        assert!(
            assert_present(
                &service,
                tenant_id,
                shared_tenant_key.clone(),
                &expected_value
            )
            .await
            .get()
                > 0
        );
    }

    service.shutdown().await;
    drop(service);
    let reopened = reopen();
    for (&tenant_id, expected_value) in &independent_expectations {
        assert_present(
            &reopened,
            tenant_id,
            shared_tenant_key.clone(),
            expected_value,
        )
        .await;
    }
    assert_eq!(
        assert_missing(&reopened, tenant, binary_key.clone()).await,
        binary_deleted_revision
    );
    assert_eq!(
        assert_present(&reopened, tenant, snapshot_first.clone(), b"complete-first").await,
        assert_present(
            &reopened,
            tenant,
            snapshot_second.clone(),
            b"complete-second"
        )
        .await
    );
    assert_eq!(
        assert_missing(&reopened, tenant, aba_key.clone()).await,
        final_missing_revision
    );
    assert!(matches!(
        transact(
            &reopened,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(aba_key.clone(), first_missing_revision)],
                vec![put(key(b"aba", b"reopen-stale"), b"must-not-appear")],
            ),
        )
        .await,
        Err(Error::Conflict(_))
    ));
    assert!(
        transact(
            &reopened,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(aba_key.clone(), final_missing_revision)],
                vec![put(key(b"aba", b"reopen-current"), b"current")],
            ),
        )
        .await
        .is_ok()
    );
    execute(
        &reopened,
        tenant,
        Request::Delete {
            key: aba_key.clone(),
        },
    )
    .await
    .unwrap();
    let post_reopen_missing_revision = assert_missing(&reopened, tenant, aba_key.clone()).await;
    assert!(post_reopen_missing_revision > final_missing_revision);
    assert!(matches!(
        transact(
            &reopened,
            tenant,
            TransactionRequest::new(
                vec![condition_revision(aba_key.clone(), final_missing_revision)],
                vec![put(key(b"aba", b"post-reopen-stale"), b"stale")],
            ),
        )
        .await,
        Err(Error::Conflict(_))
    ));
    reopened.shutdown().await;
}
