use std::sync::Arc;

use dodb_core::{
    DocumentKey, Error, Revision, RevisionState, TenantId, TransactionCondition,
    TransactionMutation, TransactionRequest,
};
use dodb_rocksdb::{RocksDbService, RocksDbServiceConfig};
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
        tenant_one["measurement"]["service_request_duration"]["count"],
        19
    );
    assert_eq!(
        tenant_one["measurement"]["write_semaphore_wait"]["count"],
        18
    );
    assert_eq!(tenant_one["measurement"]["read_semaphore_wait"]["count"], 2);
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
        (concurrent_writes + 3) as u64
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
