use std::collections::BTreeSet;

use dodb_core::{
    DocumentKey, Error, Revision, RevisionState, TenantId, TransactionCondition,
    TransactionMutation, TransactionRequest,
};
use dodb_rocksdb::{RocksDbService, RocksDbServiceConfig};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use tempfile::TempDir;

const BUDGET: ExecutionBudget = ExecutionBudget::new(1 << 20);

async fn execute(
    service: &RocksDbService,
    tenant: u64,
    request: Request,
) -> Result<Response, Error> {
    service
        .execute(TenantId::new(tenant), request, BUDGET)
        .await
}

async fn get(service: &RocksDbService, tenant: u64, key: DocumentKey) -> RevisionState {
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

fn put(key: DocumentKey, value: Vec<u8>) -> TransactionMutation {
    TransactionMutation::Put { key, value }
}

fn delete(key: DocumentKey) -> TransactionMutation {
    TransactionMutation::Delete { key }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn rocksdb_service_r1a_contract_and_reopen() {
    let directory = TempDir::new().unwrap();
    let config = RocksDbServiceConfig::new(directory.path());
    let service = RocksDbService::new(config.clone()).unwrap();
    let binary_key = key(&[0, 255, 0, 1], &[255, 0, 2]);
    let binary_value = vec![0, 255, 1, 0, 128, 255];
    let absent = get(&service, 1, binary_key.clone()).await;
    assert_eq!(absent, RevisionState::missing(Revision::ZERO));

    let first_revision = match execute(
        &service,
        1,
        Request::Put {
            key: binary_key.clone(),
            value: binary_value.clone(),
        },
    )
    .await
    .unwrap()
    {
        Response::Put(revision) => revision,
        other => panic!("unexpected response: {other:?}"),
    };
    assert_eq!(
        get(&service, 1, binary_key.clone()).await,
        RevisionState::present(binary_value.clone(), first_revision)
    );

    let updated_revision = match execute(
        &service,
        1,
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
    let deleted_revision = match execute(
        &service,
        1,
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
    let repeated_delete_revision = match execute(
        &service,
        1,
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
    assert_eq!(repeated_delete_revision.get(), deleted_revision.get() + 1);
    assert_eq!(
        get(&service, 1, binary_key.clone()).await,
        RevisionState::missing(repeated_delete_revision)
    );

    let untouched = key(b"untouched", b"missing");
    let exists_key = key(b"present", b"exists");
    execute(
        &service,
        1,
        Request::Put {
            key: exists_key.clone(),
            value: vec![4, 0, 4],
        },
    )
    .await
    .unwrap();
    let exists_revision = get(&service, 1, exists_key.clone()).await.revision();
    let condition_only = TransactionRequest::new(
        vec![
            TransactionCondition::RevisionEquals {
                key: exists_key.clone(),
                expected_revision: exists_revision,
            },
            TransactionCondition::NotExists {
                key: untouched.clone(),
            },
        ],
        Vec::new(),
    );
    assert_eq!(
        execute(
            &service,
            1,
            Request::Transact {
                request: condition_only
            }
        )
        .await
        .unwrap(),
        Response::Transact(dodb_service::TransactionOutcome::conditions_satisfied())
    );
    assert_eq!(
        execute(
            &service,
            1,
            Request::Transact {
                request: TransactionRequest::new(
                    vec![TransactionCondition::Exists {
                        key: exists_key.clone(),
                    }],
                    Vec::new(),
                ),
            }
        )
        .await
        .unwrap(),
        Response::Transact(dodb_service::TransactionOutcome::conditions_satisfied())
    );
    let not_exists_existing = TransactionRequest::new(
        vec![TransactionCondition::NotExists {
            key: exists_key.clone(),
        }],
        vec![
            put(key(b"not-exists", b"put"), vec![1]),
            delete(key(b"not-exists", b"delete")),
        ],
    );
    assert!(matches!(
        execute(
            &service,
            1,
            Request::Transact {
                request: not_exists_existing,
            }
        )
        .await,
        Err(Error::Conflict(_))
    ));
    assert_eq!(
        get(&service, 1, key(b"not-exists", b"put")).await,
        RevisionState::missing(Revision::ZERO)
    );
    assert_eq!(
        get(&service, 1, key(b"not-exists", b"delete")).await,
        RevisionState::missing(Revision::ZERO)
    );
    let condition_failure = TransactionRequest::new(
        vec![TransactionCondition::RevisionEquals {
            key: exists_key.clone(),
            expected_revision: Revision::ZERO,
        }],
        vec![
            put(key(b"must-not", b"appear"), b"bad".to_vec()),
            delete(key(b"must-not", b"delete")),
        ],
    );
    assert!(matches!(
        execute(
            &service,
            1,
            Request::Transact {
                request: condition_failure
            }
        )
        .await,
        Err(Error::Conflict(_))
    ));
    assert_eq!(
        get(&service, 1, key(b"must-not", b"appear")).await,
        RevisionState::missing(Revision::ZERO)
    );
    assert_eq!(
        get(&service, 1, key(b"must-not", b"delete")).await,
        RevisionState::missing(Revision::ZERO)
    );
    assert!(matches!(
        execute(
            &service,
            1,
            Request::Transact {
                request: TransactionRequest::new(
                    vec![TransactionCondition::Exists {
                        key: key(b"condition-only", b"absent"),
                    }],
                    Vec::new(),
                ),
            }
        )
        .await,
        Err(Error::Conflict(_))
    ));

    for position in 2..4 {
        execute(
            &service,
            1,
            Request::Put {
                key: key(b"width-four", &[position]),
                value: vec![position, 9],
            },
        )
        .await
        .unwrap();
    }
    let width_four = vec![
        put(key(b"width-four", &[0]), vec![0, 0]),
        put(key(b"width-four", &[1]), vec![1, 0]),
        delete(key(b"width-four", &[2])),
        delete(key(b"width-four", &[3])),
    ];
    let width_four_response = execute(
        &service,
        1,
        Request::Transact {
            request: TransactionRequest::new(
                vec![TransactionCondition::NotExists {
                    key: untouched.clone(),
                }],
                width_four,
            ),
        },
    )
    .await
    .unwrap();
    let width_four_revision = match width_four_response {
        Response::Transact(outcome) => outcome.commit_lsn.unwrap().get(),
        other => panic!("unexpected response: {other:?}"),
    };
    for position in 0..2 {
        assert_eq!(
            get(&service, 1, key(b"width-four", &[position]))
                .await
                .revision()
                .get(),
            width_four_revision
        );
    }
    for position in 2..4 {
        assert_eq!(
            get(&service, 1, key(b"width-four", &[position])).await,
            RevisionState::missing(Revision::new(width_four_revision))
        );
    }

    let protected = key(b"condition-only", b"protects-write");
    let protected_revision = get(&service, 1, protected.clone()).await.revision();
    execute(
        &service,
        1,
        Request::Put {
            key: protected.clone(),
            value: b"inserted".to_vec(),
        },
    )
    .await
    .unwrap();
    let protected_transaction = TransactionRequest::new(
        vec![TransactionCondition::RevisionEquals {
            key: protected,
            expected_revision: protected_revision,
        }],
        vec![put(key(b"other", b"write"), b"must-not-appear".to_vec())],
    );
    assert!(matches!(
        execute(
            &service,
            1,
            Request::Transact {
                request: protected_transaction
            }
        )
        .await,
        Err(Error::Conflict(_))
    ));

    let concurrently_protected = key(b"concurrent-condition", b"untouched");
    let protected_write = key(b"concurrent-condition", b"separate-write");
    let observed_missing_revision = get(&service, 1, concurrently_protected.clone())
        .await
        .revision();
    let condition_service = &service;
    let write_service = &service;
    let condition_key = concurrently_protected.clone();
    let write_key = protected_write.clone();
    let mutation_key = concurrently_protected.clone();
    let (condition_result, unrelated_write_result) = tokio::join!(
        async move {
            execute(
                condition_service,
                1,
                Request::Transact {
                    request: TransactionRequest::new(
                        vec![TransactionCondition::RevisionEquals {
                            key: condition_key,
                            expected_revision: observed_missing_revision,
                        }],
                        vec![put(write_key, b"protected-target".to_vec())],
                    ),
                },
            )
            .await
        },
        async move {
            execute(
                write_service,
                1,
                Request::Put {
                    key: mutation_key,
                    value: b"new-state".to_vec(),
                },
            )
            .await
        }
    );
    assert!(unrelated_write_result.is_ok());
    assert!(condition_result.is_ok() || matches!(&condition_result, Err(Error::Conflict(_))));
    if condition_result.is_err() {
        assert_eq!(
            get(&service, 1, protected_write.clone()).await,
            RevisionState::missing(Revision::ZERO)
        );
    } else {
        assert!(matches!(
            get(&service, 1, protected_write.clone()).await,
            RevisionState::Present { value, .. } if value == b"protected-target"
        ));
    }

    let mut duplicate_condition = TransactionRequest::new(
        vec![
            TransactionCondition::NotExists {
                key: untouched.clone(),
            },
            TransactionCondition::Exists {
                key: untouched.clone(),
            },
        ],
        Vec::new(),
    );
    assert!(matches!(
        execute(
            &service,
            1,
            Request::Transact {
                request: duplicate_condition.clone()
            }
        )
        .await,
        Err(Error::InvalidRequest(_))
    ));
    duplicate_condition.conditions.clear();
    duplicate_condition.mutations =
        vec![put(untouched.clone(), vec![1]), delete(untouched.clone())];
    assert!(matches!(
        execute(
            &service,
            1,
            Request::Transact {
                request: duplicate_condition
            }
        )
        .await,
        Err(Error::InvalidRequest(_))
    ));
    assert!(matches!(
        execute(
            &service,
            1,
            Request::Transact {
                request: TransactionRequest::default()
            }
        )
        .await,
        Err(Error::InvalidRequest(_))
    ));
    assert!(matches!(
        execute(
            &service,
            1,
            Request::Query {
                pk: dodb_core::PrimaryKey::new(vec![]),
                exclusive_after_sk: None,
                limit: 1
            }
        )
        .await,
        Err(Error::InvalidRequest(_))
    ));

    let cold_tenant_writes = (0..4).map(|position| {
        let service = &service;
        async move {
            let cold_key = key(b"concurrent-cold-open", &[position]);
            execute(
                service,
                2,
                Request::Put {
                    key: cold_key,
                    value: vec![position, 0],
                },
            )
            .await
        }
    });
    let cold_open_results = futures::future::join_all(cold_tenant_writes).await;
    assert!(cold_open_results.iter().all(|result| result.is_ok()));
    let mut cold_open_revisions = BTreeSet::new();
    for position in 0..4 {
        let state = get(&service, 2, key(b"concurrent-cold-open", &[position])).await;
        assert!(matches!(
            &state,
            RevisionState::Present { value, .. } if value.as_slice() == [position, 0]
        ));
        cold_open_revisions.insert(state.revision().get());
    }
    assert_eq!(cold_open_revisions, BTreeSet::from([1, 2, 3, 4]));

    let aba_key = key(b"aba", b"missing");
    let missing_revision = get(&service, 1, aba_key.clone()).await.revision();
    execute(
        &service,
        1,
        Request::Put {
            key: aba_key.clone(),
            value: vec![2],
        },
    )
    .await
    .unwrap();
    execute(
        &service,
        1,
        Request::Delete {
            key: aba_key.clone(),
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        execute(
            &service,
            1,
            Request::Transact {
                request: TransactionRequest::new(
                    vec![TransactionCondition::RevisionEquals {
                        key: aba_key.clone(),
                        expected_revision: missing_revision,
                    }],
                    vec![put(key(b"aba", b"blocked"), vec![3])],
                ),
            }
        )
        .await,
        Err(Error::Conflict(_))
    ));
    assert!(
        execute(
            &service,
            1,
            Request::Transact {
                request: TransactionRequest::new(
                    vec![TransactionCondition::NotExists {
                        key: aba_key.clone()
                    }],
                    Vec::new(),
                ),
            },
        )
        .await
        .is_ok()
    );

    let race_key = key(b"race", b"revision");
    let race_revision = match execute(
        &service,
        1,
        Request::Put {
            key: race_key.clone(),
            value: vec![0],
        },
    )
    .await
    .unwrap()
    {
        Response::Put(revision) => revision,
        other => panic!("unexpected response: {other:?}"),
    };
    let left_service = &service;
    let right_service = &service;
    let left_key = race_key.clone();
    let right_key = race_key.clone();
    let (left, right) = tokio::join!(
        async move {
            execute(
                left_service,
                1,
                Request::Transact {
                    request: TransactionRequest::new(
                        vec![TransactionCondition::RevisionEquals {
                            key: left_key.clone(),
                            expected_revision: race_revision,
                        }],
                        vec![put(left_key, vec![1])],
                    ),
                },
            )
            .await
        },
        async move {
            execute(
                right_service,
                1,
                Request::Transact {
                    request: TransactionRequest::new(
                        vec![TransactionCondition::RevisionEquals {
                            key: right_key.clone(),
                            expected_revision: race_revision,
                        }],
                        vec![put(right_key, vec![2])],
                    ),
                },
            )
            .await
        }
    );
    assert_ne!(left.is_ok(), right.is_ok());
    assert!(left.is_ok() || matches!(&left, Err(Error::Conflict(_))));
    assert!(right.is_ok() || matches!(&right, Err(Error::Conflict(_))));

    let current_race_revision = get(&service, 1, race_key.clone()).await.revision();
    let conditional_service = &service;
    let unconditional_service = &service;
    let conditional_key = race_key.clone();
    let unconditional_key = race_key.clone();
    let (conditional_put, unconditional_put) = tokio::join!(
        async move {
            execute(
                conditional_service,
                1,
                Request::Transact {
                    request: TransactionRequest::new(
                        vec![TransactionCondition::RevisionEquals {
                            key: conditional_key.clone(),
                            expected_revision: current_race_revision,
                        }],
                        vec![put(conditional_key, b"conditional".to_vec())],
                    ),
                },
            )
            .await
        },
        async move {
            execute(
                unconditional_service,
                1,
                Request::Put {
                    key: unconditional_key,
                    value: b"unconditional".to_vec(),
                },
            )
            .await
        }
    );
    assert!(unconditional_put.is_ok());
    assert!(conditional_put.is_ok() || matches!(conditional_put, Err(Error::Conflict(_))));
    assert!(matches!(
        get(&service, 1, race_key.clone()).await,
        RevisionState::Present { value, .. } if value.as_slice() == b"unconditional"
    ));

    let delete_revision = get(&service, 1, race_key.clone()).await.revision();
    let conditional_delete_service = &service;
    let unconditional_delete_service = &service;
    let conditional_delete_key = race_key.clone();
    let unconditional_delete_key = race_key.clone();
    let (conditional_write, unconditional_delete) = tokio::join!(
        async move {
            execute(
                conditional_delete_service,
                1,
                Request::Transact {
                    request: TransactionRequest::new(
                        vec![TransactionCondition::RevisionEquals {
                            key: conditional_delete_key.clone(),
                            expected_revision: delete_revision,
                        }],
                        vec![put(conditional_delete_key, b"before-delete".to_vec())],
                    ),
                },
            )
            .await
        },
        async move {
            execute(
                unconditional_delete_service,
                1,
                Request::Delete {
                    key: unconditional_delete_key,
                },
            )
            .await
        }
    );
    assert!(unconditional_delete.is_ok());
    assert!(conditional_write.is_ok() || matches!(conditional_write, Err(Error::Conflict(_))));
    assert!(matches!(
        get(&service, 1, race_key.clone()).await,
        RevisionState::Missing { .. }
    ));

    let tenant_requests = (10..14).map(|tenant| {
        let service = &service;
        async move {
            let shared_key = key(b"shared-document", b"same");
            execute(
                service,
                tenant,
                Request::Put {
                    key: shared_key.clone(),
                    value: tenant.to_be_bytes().to_vec(),
                },
            )
            .await
            .unwrap();
            get(service, tenant, shared_key).await
        }
    });
    let tenant_states = futures::future::join_all(tenant_requests).await;
    for (tenant_index, state) in tenant_states.iter().enumerate() {
        assert_eq!(
            state,
            &RevisionState::present(
                (tenant_index as u64 + 10).to_be_bytes().to_vec(),
                Revision::new(1)
            )
        );
    }
    for left_tenant in 10..14 {
        for right_tenant in (left_tenant + 1)..14 {
            assert_ne!(
                get(&service, left_tenant, key(b"shared-document", b"same")).await,
                get(&service, right_tenant, key(b"shared-document", b"same")).await
            );
        }
    }

    service.shutdown().await;
    let reopened = RocksDbService::new(config).unwrap();
    assert_eq!(
        get(&reopened, 1, binary_key.clone()).await,
        RevisionState::missing(repeated_delete_revision)
    );
    assert_eq!(
        get(&reopened, 1, exists_key.clone()).await.revision(),
        exists_revision
    );
    assert!(
        execute(
            &reopened,
            1,
            Request::Transact {
                request: TransactionRequest::new(
                    vec![TransactionCondition::RevisionEquals {
                        key: binary_key.clone(),
                        expected_revision: repeated_delete_revision,
                    }],
                    Vec::new(),
                ),
            },
        )
        .await
        .is_ok()
    );
    assert!(
        execute(
            &reopened,
            1,
            Request::Transact {
                request: TransactionRequest::new(
                    vec![TransactionCondition::NotExists {
                        key: binary_key.clone(),
                    }],
                    Vec::new(),
                ),
            },
        )
        .await
        .is_ok()
    );
    let after_reopen = match execute(
        &reopened,
        1,
        Request::Put {
            key: binary_key,
            value: vec![0, 0, 9],
        },
    )
    .await
    .unwrap()
    {
        Response::Put(revision) => revision,
        other => panic!("unexpected response: {other:?}"),
    };
    assert!(after_reopen > repeated_delete_revision);
    reopened.shutdown().await;
}
