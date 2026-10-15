use std::sync::Arc;

use dodb_core::{
    DocumentKey, Error, Revision, RevisionState, TenantId, TransactionCondition,
    TransactionMutation, TransactionRequest,
};
use dodb_rocksdb::{RevisionAllocatorMode, RocksDbService, RocksDbServiceConfig};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use tempfile::TempDir;

const BUDGET: ExecutionBudget = ExecutionBudget::new(1 << 20);

fn key(partition: &[u8], sort: &[u8]) -> DocumentKey {
    DocumentKey::new(partition.to_vec(), sort.to_vec())
}

async fn execute(
    service: &RocksDbService,
    tenant: u64,
    request: Request,
) -> Result<Response, Error> {
    service
        .execute(TenantId::new(tenant), request, BUDGET)
        .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn research_metrics_are_bounded_phase_separated_and_counted_once() {
    let directory = TempDir::new().unwrap();
    let mut config = RocksDbServiceConfig::new(directory.path());
    config.research_metrics = true;
    let service = Arc::new(RocksDbService::new(config).unwrap());
    let metrics = service.research_metrics_handle().unwrap();
    let warmup_key = key(b"phase", b"seed");
    execute(
        &service,
        1,
        Request::Put {
            key: warmup_key.clone(),
            value: b"initial".to_vec(),
        },
    )
    .await
    .unwrap();
    service.set_research_warmup_phase();
    assert!(matches!(
        execute(&service, 1, Request::Get { key: warmup_key }).await,
        Ok(Response::Get(RevisionState::Present { .. }))
    ));
    service.set_research_measurement_phase();
    let target_key = key(b"measured", b"target");
    assert!(matches!(
        dodb_rocksdb::with_research_read_kind(
            dodb_rocksdb::ResearchReadKind::PreparatoryGet,
            execute(
                &service,
                1,
                Request::Get {
                    key: target_key.clone(),
                },
            ),
        )
        .await,
        Ok(Response::Get(RevisionState::Missing { .. }))
    ));
    assert!(matches!(
        execute(
            &service,
            1,
            Request::Put {
                key: target_key.clone(),
                value: b"value".to_vec(),
            },
        )
        .await,
        Ok(Response::Put(revision)) if revision == Revision::new(2)
    ));
    let rejected = TransactionRequest::new(
        vec![TransactionCondition::RevisionEquals {
            key: target_key.clone(),
            expected_revision: Revision::new(999),
        }],
        vec![TransactionMutation::Put {
            key: target_key.clone(),
            value: b"rejected".to_vec(),
        }],
    );
    assert!(matches!(
        execute(&service, 1, Request::Transact { request: rejected }).await,
        Err(Error::Conflict(_))
    ));
    assert!(matches!(
        execute(&service, 1, Request::Get { key: target_key.clone() }).await,
        Ok(Response::Get(RevisionState::Present { value, revision })) if value == b"value" && revision == Revision::new(2)
    ));
    assert!(matches!(
        execute(
            &service,
            1,
            Request::Query {
                pk: dodb_core::PrimaryKey::new(b"measured".to_vec()),
                exclusive_after_sk: None,
                limit: 16,
            },
        )
        .await,
        Ok(Response::Query(rows)) if rows.len() == 1 && rows[0].value == b"value"
    ));

    let concurrent_writes = 32;
    let mut tasks = tokio::task::JoinSet::new();
    for request_number in 0..concurrent_writes {
        let service = Arc::clone(&service);
        tasks.spawn(async move {
            execute(
                &service,
                request_number % 2 + 1,
                Request::Put {
                    key: key(b"parallel", &request_number.to_be_bytes()),
                    value: request_number.to_be_bytes().to_vec(),
                },
            )
            .await
        });
    }
    while let Some(result) = tasks.join_next().await {
        assert!(matches!(result.unwrap(), Ok(Response::Put(_))));
    }
    service.shutdown().await;

    let snapshot = metrics.snapshot();
    let tenant_one = &snapshot["tenants"]["tenant_1"];
    let tenant_two = &snapshot["tenants"]["tenant_2"];
    assert_eq!(tenant_one["warmup"]["service_request_duration"]["count"], 1);
    assert_eq!(
        tenant_one["read_stages"]["warmup"]["get"]["service_request_total"]["count"],
        1
    );
    assert_eq!(
        tenant_one["measurement"]["service_request_duration"]["count"],
        21
    );
    assert_eq!(
        tenant_one["measurement"]["write_semaphore_wait"]["count"],
        18
    );
    assert_eq!(tenant_one["measurement"]["read_semaphore_wait"]["count"], 3);
    assert_eq!(
        tenant_one["read_stages"]["measurement"]["get"]["service_request_total"]["count"],
        1
    );
    assert_eq!(
        tenant_one["read_stages"]["measurement"]["get"]["rocksdb_get_call"]["count"],
        1
    );
    assert_eq!(
        tenant_one["read_stages"]["measurement"]["get"]["get_copy_decode"]["count"],
        1
    );
    assert_eq!(
        tenant_one["read_stages"]["measurement"]["preparatory_get"]["service_request_total"]["count"],
        1
    );
    assert_eq!(
        tenant_one["read_stages"]["measurement"]["query16"]["service_request_total"]["count"],
        1
    );
    assert_eq!(
        tenant_one["read_stages"]["measurement"]["get"]["blocking_closure_duration"]["count"],
        1
    );
    for metric in [
        "semaphore_wait",
        "blocking_dispatch_wait",
        "blocking_completion_handoff",
    ] {
        assert_eq!(
            tenant_one["read_stages"]["measurement"]["get"][metric]["count"],
            1
        );
        assert_eq!(
            tenant_one["read_stages"]["measurement"]["query16"][metric]["count"],
            1
        );
    }
    assert_eq!(
        tenant_one["read_stages"]["measurement"]["query16"]["query_iterator_calls_sum"]["count"],
        1
    );
    assert_eq!(
        tenant_one["read_stages"]["measurement"]["query16"]["query_snapshot_iterator_setup"]["count"],
        1
    );
    assert_eq!(
        tenant_one["read_stages"]["measurement"]["query16"]["query_copy_decode_budget"]["count"],
        1
    );
    assert_eq!(
        tenant_one["measurement"]["user_key_get_for_update_duration"]["count"],
        18
    );
    assert_eq!(
        tenant_one["measurement"]["revision_counter_get_for_update_duration"]["count"],
        17
    );
    assert_eq!(tenant_one["measurement"]["commit_duration"]["count"], 17);
    assert_eq!(tenant_one["measurement"]["rollback_duration"]["count"], 1);
    assert_eq!(
        tenant_one["outcomes"]["measurement"]["successful_writes"],
        17
    );
    assert_eq!(
        tenant_one["outcomes"]["measurement"]["condition_rejections"],
        1
    );
    assert_eq!(tenant_one["outcomes"]["measurement"]["other_errors"], 0);
    assert_eq!(
        tenant_two["measurement"]["service_request_duration"]["count"],
        16
    );
    assert_eq!(
        tenant_two["outcomes"]["measurement"]["successful_writes"],
        16
    );
    assert_eq!(
        tenant_one["measurement"]["service_request_duration"]["count"]
            .as_u64()
            .unwrap()
            + tenant_two["measurement"]["service_request_duration"]["count"]
                .as_u64()
                .unwrap(),
        (concurrent_writes + 5) as u64
    );
}

#[tokio::test]
async fn disabled_research_metrics_are_explicitly_absent() {
    let directory = TempDir::new().unwrap();
    let service = RocksDbService::new(RocksDbServiceConfig::new(directory.path())).unwrap();
    assert!(service.research_metrics_snapshot().is_none());
    service.shutdown().await;
}

#[tokio::test]
async fn metric_toggle_preserves_logical_results() {
    async fn run(metrics_enabled: bool) -> (Response, bool, Response) {
        let directory = TempDir::new().unwrap();
        let mut config = RocksDbServiceConfig::new(directory.path());
        config.research_metrics = metrics_enabled;
        let service = RocksDbService::new(config).unwrap();
        let target_key = key(b"toggle", b"same-result");
        let put = execute(
            &service,
            1,
            Request::Put {
                key: target_key.clone(),
                value: b"same-value".to_vec(),
            },
        )
        .await
        .unwrap();
        let rejected = TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: target_key.clone(),
                expected_revision: Revision::new(999),
            }],
            vec![TransactionMutation::Delete {
                key: target_key.clone(),
            }],
        );
        let condition_rejected = matches!(
            execute(&service, 1, Request::Transact { request: rejected }).await,
            Err(Error::Conflict(_))
        );
        let get = execute(&service, 1, Request::Get { key: target_key })
            .await
            .unwrap();
        service.shutdown().await;
        (put, condition_rejected, get)
    }

    assert_eq!(run(false).await, run(true).await);
}

#[tokio::test]
async fn range_reservations_are_separate_from_user_commit_metrics() {
    let directory = TempDir::new().unwrap();
    let mut config = RocksDbServiceConfig::new(directory.path());
    config.research_metrics = true;
    config.revision_allocator = RevisionAllocatorMode::ReservedRanges;
    let service = RocksDbService::new(config).unwrap();
    service.set_research_measurement_phase();
    for ordinal in 0..3 {
        execute(
            &service,
            1,
            Request::Put {
                key: key(b"range-metrics", &[ordinal]),
                value: vec![ordinal],
            },
        )
        .await
        .unwrap();
    }
    let tenant = service.research_metrics_snapshot().unwrap()["tenants"]["tenant_1"].clone();
    let outcomes = &tenant["outcomes"]["measurement"];
    assert_eq!(outcomes["user_commits"], 3);
    assert_eq!(outcomes["range_reservations"], 1);
    assert_eq!(outcomes["reservation_commits"], 1);
    assert_eq!(outcomes["user_transaction_counter_accesses"], 0);
    assert_eq!(outcomes["reservation_counter_accesses"], 1);
    assert_eq!(tenant["measurement"]["user_commit_duration"]["count"], 3);
    assert_eq!(
        tenant["measurement"]["revision_range_reservation_duration"]["count"],
        1
    );
    assert_eq!(tenant["measurement"]["revision_allocator_wait"]["count"], 3);
    service.shutdown().await;
}

#[tokio::test]
async fn write_reads_account_for_roles_payloads_rejections_and_phases() {
    let directory = TempDir::new().unwrap();
    let mut config = RocksDbServiceConfig::new(directory.path());
    config.research_metrics = true;
    config.revision_allocator = RevisionAllocatorMode::ReservedRanges;
    let service = RocksDbService::new(config).unwrap();
    let mutation_key = key(b"write-reads", b"mutation");
    let condition_key = key(b"write-reads", b"condition");
    let tombstone_key = key(b"write-reads", b"tombstone");
    for (target, payload_size) in [(&mutation_key, 512), (&condition_key, 1024)] {
        execute(
            &service,
            1,
            Request::Put {
                key: target.clone(),
                value: vec![7; payload_size],
            },
        )
        .await
        .unwrap();
    }
    execute(
        &service,
        1,
        Request::Delete {
            key: tombstone_key.clone(),
        },
    )
    .await
    .unwrap();
    service.set_research_warmup_phase();
    execute(
        &service,
        1,
        Request::Put {
            key: mutation_key.clone(),
            value: vec![8; 512],
        },
    )
    .await
    .unwrap();
    service.set_research_measurement_phase();
    execute(
        &service,
        1,
        Request::Put {
            key: mutation_key.clone(),
            value: vec![9; 512],
        },
    )
    .await
    .unwrap();
    execute(
        &service,
        1,
        Request::Transact {
            request: TransactionRequest::new(
                vec![TransactionCondition::Exists {
                    key: condition_key.clone(),
                }],
                vec![
                    TransactionMutation::Put {
                        key: mutation_key.clone(),
                        value: vec![10; 512],
                    },
                    TransactionMutation::Delete { key: tombstone_key },
                ],
            ),
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
                    vec![TransactionCondition::NotExists { key: condition_key }],
                    vec![TransactionMutation::Put {
                        key: mutation_key,
                        value: vec![11; 512]
                    }],
                )
            }
        )
        .await,
        Err(Error::Conflict(_))
    ));
    service.shutdown().await;
    let snapshot = service.research_metrics_snapshot().unwrap();
    let reads = &snapshot["tenants"]["tenant_1"]["write_reads"];
    let warmup = &reads["warmup"]["mutation_only_key"];
    assert_eq!(warmup["keys"], 1);
    assert_eq!(warmup["payload_bytes_fetched"], 512);
    let condition = &reads["measurement"]["condition_key"];
    let mutation = &reads["measurement"]["mutation_only_key"];
    assert_eq!(condition["keys"], 2);
    assert_eq!(condition["found_keys"], 2);
    assert_eq!(condition["encoded_bytes_fetched"], 2068);
    assert_eq!(condition["payload_bytes_fetched"], 2048);
    assert_eq!(mutation["keys"], 4);
    assert_eq!(mutation["found_keys"], 4);
    assert_eq!(mutation["encoded_bytes_fetched"], 1576);
    assert_eq!(mutation["payload_bytes_fetched"], 1536);
    for role in [condition, mutation] {
        for timing in [
            "native_get_for_update",
            "native_lock_wait",
            "shim_value_copy",
            "rust_value_copy",
            "state_decode",
        ] {
            assert_eq!(role["timings"][timing]["count"], role["keys"]);
        }
    }
}
