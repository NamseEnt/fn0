use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use dodb_client::{ClientError, ClientTlsConfig, DodbClient, DodbConnection};
use dodb_core::{
    ConditionExpectation, DocumentKey, Error, ObservedState, Revision, RevisionState, TenantId,
    TransactionCondition, TransactionMutation, TransactionRequest,
};
use dodb_server::{
    DodbServer, DodbServerConfig, LocalTenantService, LocalTenantServiceConfig, ServerTlsConfig,
};
use dodb_service::{
    DodbService, ExecutionBudget, Request, Response, ServiceFuture, TransactionOutcome,
};
use dodb_storage::FaultInjector;
use quinn::rustls::pki_types::CertificateDer;
use quinn::{ClientConfig as QuinnClientConfig, Endpoint};
use rcgen::generate_simple_self_signed;

struct TestTls {
    certificate: Vec<u8>,
    private_key: Vec<u8>,
}

fn test_tls() -> TestTls {
    let certified = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
    TestTls {
        certificate: certified.cert.der().to_vec(),
        private_key: certified.signing_key.serialize_der(),
    }
}

async fn start_server(
    data_dir: std::path::PathBuf,
    tls: &TestTls,
) -> (
    Arc<DodbServer<LocalTenantService>>,
    tokio::task::JoinHandle<Result<(), dodb_server::ServerError>>,
) {
    start_server_with_limits(data_dir, tls, dodb_protocol::ProtocolLimits::default(), 64).await
}

async fn start_server_with_limits(
    data_dir: std::path::PathBuf,
    tls: &TestTls,
    protocol_limits: dodb_protocol::ProtocolLimits,
    max_concurrent_requests: usize,
) -> (
    Arc<DodbServer<LocalTenantService>>,
    tokio::task::JoinHandle<Result<(), dodb_server::ServerError>>,
) {
    start_server_with_limits_at(
        data_dir,
        tls,
        protocol_limits,
        max_concurrent_requests,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
}

async fn start_server_with_limits_at(
    data_dir: std::path::PathBuf,
    tls: &TestTls,
    protocol_limits: dodb_protocol::ProtocolLimits,
    max_concurrent_requests: usize,
    listen_addr: std::net::SocketAddr,
) -> (
    Arc<DodbServer<LocalTenantService>>,
    tokio::task::JoinHandle<Result<(), dodb_server::ServerError>>,
) {
    let service = Arc::new(
        LocalTenantService::new(LocalTenantServiceConfig {
            data_dir,
            ..LocalTenantServiceConfig::default()
        })
        .unwrap(),
    );
    let server = Arc::new(
        DodbServer::bind(
            service,
            DodbServerConfig {
                listen_addr,
                tls: ServerTlsConfig::from_der(
                    vec![tls.certificate.clone()],
                    tls.private_key.clone(),
                )
                .unwrap(),
                protocol_limits,
                max_connections: 8,
                max_concurrent_streams: 64,
                max_concurrent_requests,
            },
        )
        .unwrap(),
    );
    let task_server = Arc::clone(&server);
    let task = tokio::spawn(async move { task_server.run().await });
    (server, task)
}

async fn connect_client<S: DodbService + 'static>(
    server: &DodbServer<S>,
    tls: &TestTls,
    tenant: TenantId,
) -> DodbClient {
    connect_client_with_limits(
        server,
        tls,
        tenant,
        dodb_protocol::ProtocolLimits::default(),
    )
    .await
}

async fn connect_client_with_limits<S: DodbService + 'static>(
    server: &DodbServer<S>,
    tls: &TestTls,
    tenant: TenantId,
    limits: dodb_protocol::ProtocolLimits,
) -> DodbClient {
    DodbClient::connect(
        "0.0.0.0:0".parse().unwrap(),
        server.local_addr().unwrap(),
        "localhost",
        tenant,
        ClientTlsConfig::from_der(vec![tls.certificate.clone()]).unwrap(),
        limits,
    )
    .await
    .unwrap()
}

fn key(pk: &[u8], sk: &[u8]) -> DocumentKey {
    DocumentKey::new(pk.to_vec(), sk.to_vec())
}

struct BlockingService {
    entered: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Notify>,
    first_request: AtomicBool,
}

struct BlockingMutationService {
    entered: Arc<tokio::sync::Notify>,
    release: Arc<tokio::sync::Notify>,
    calls: AtomicUsize,
}

impl DodbService for BlockingMutationService {
    fn execute<'service>(
        &'service self,
        _tenant: TenantId,
        request: Request,
        _budget: ExecutionBudget,
    ) -> ServiceFuture<'service> {
        Box::pin(async move {
            match request {
                Request::Put { .. } => {
                    self.calls.fetch_add(1, Ordering::SeqCst);
                    self.entered.notify_one();
                    self.release.notified().await;
                    Ok(Response::Put(Revision::new(1)))
                }
                _ => Ok(Response::Get(RevisionState::missing(Revision::ZERO))),
            }
        })
    }
}

impl DodbService for BlockingService {
    fn execute<'service>(
        &'service self,
        _tenant: TenantId,
        _request: Request,
        _budget: ExecutionBudget,
    ) -> ServiceFuture<'service> {
        Box::pin(async move {
            if !self.first_request.swap(true, Ordering::SeqCst) {
                self.entered.notify_one();
                self.release.notified().await;
            }
            Ok(Response::Get(RevisionState::missing(Revision::ZERO)))
        })
    }
}

async fn stop_server<S: DodbService + 'static>(
    client: DodbClient,
    server: Arc<DodbServer<S>>,
    task: tokio::task::JoinHandle<Result<(), dodb_server::ServerError>>,
) {
    client.close();
    server.shutdown().await;
    task.await.unwrap().unwrap();
}

async fn wait_for_active_connections<S: DodbService + 'static>(
    server: &DodbServer<S>,
    expected: u64,
) {
    for _ in 0..1_000 {
        if server.metrics().snapshot().active_connections >= expected {
            return;
        }
        tokio::task::yield_now().await;
    }
    panic!("server did not observe {expected} active connections");
}

async fn abruptly_disconnect(
    server: &DodbServer<LocalTenantService>,
    tls: &TestTls,
    request: Option<dodb_service::Request>,
) {
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap()).unwrap();
    let mut roots = quinn::rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from(tls.certificate.clone()))
        .unwrap();
    endpoint.set_default_client_config(
        QuinnClientConfig::with_root_certificates(Arc::new(roots)).unwrap(),
    );
    let connection = endpoint
        .connect(server.local_addr().unwrap(), "localhost")
        .unwrap()
        .await
        .unwrap();
    let (mut send, _receive) = connection.open_bi().await.unwrap();
    if let Some(request) = request {
        let frame = dodb_protocol::encode_request(
            TenantId::new(91),
            &request,
            dodb_protocol::ProtocolLimits::default(),
        )
        .unwrap();
        send.write_all(&frame).await.unwrap();
        send.finish().unwrap();
    }
    connection.close(quinn::VarInt::from_u32(0), b"abrupt test disconnect");
    endpoint.close(quinn::VarInt::from_u32(0), b"abrupt test disconnect");
}

#[tokio::test]
async fn tenant_handles_share_one_connection_and_close_independently() {
    let directory = tempfile::tempdir().unwrap();
    let tls = test_tls();
    let (server, task) = start_server(directory.path().to_owned(), &tls).await;
    let connection = DodbConnection::connect(
        "0.0.0.0:0".parse().unwrap(),
        server.local_addr().unwrap(),
        "localhost",
        ClientTlsConfig::from_der(vec![tls.certificate.clone()]).unwrap(),
        dodb_protocol::ProtocolLimits::default(),
    )
    .await
    .unwrap();
    let first = connection.for_tenant(TenantId::new(101));
    let second = connection.for_tenant(TenantId::new(102));
    wait_for_active_connections(&server, 1).await;
    assert_eq!(server.metrics().snapshot().active_connections, 1);

    let shared_key = key(b"shared", b"key");
    first.put(shared_key.clone(), vec![1]).await.unwrap();
    assert_eq!(
        second.get(shared_key.clone()).await.unwrap(),
        RevisionState::missing(Revision::ZERO)
    );

    first.close();
    drop(first);
    assert_eq!(second.get(shared_key).await.unwrap().value(), None);
    assert_eq!(server.metrics().snapshot().active_connections, 1);

    connection.close();
    server.shutdown().await;
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn loopback_protocol_preserves_storage_semantics_and_lazy_creation() {
    let directory = tempfile::tempdir().unwrap();
    let tls = test_tls();
    let (server, task) = start_server(directory.path().to_owned(), &tls).await;
    let tenant = TenantId::new(41);
    let client = connect_client(&server, &tls, tenant).await;
    let missing_key = key(&[], &[0xff]);

    assert_eq!(
        client.get(missing_key.clone()).await.unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    assert!(
        client
            .query(dodb_core::PrimaryKey::new(Vec::new()), None, 10)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(client.scan(None, 10).await.unwrap().is_empty());
    assert_eq!(
        client.get(missing_key.clone()).await.unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    let condition_only = client
        .transact(TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: missing_key.clone(),
                expected_revision: Revision::ZERO,
            }],
            Vec::new(),
        ))
        .await
        .unwrap();
    assert_eq!(condition_only, TransactionOutcome::conditions_satisfied());
    assert!(
        std::fs::read_dir(directory.path())
            .unwrap()
            .next()
            .is_none()
    );

    let first_revision = client
        .put(missing_key.clone(), vec![0, 0xff, 1])
        .await
        .unwrap();
    assert!(first_revision > Revision::ZERO);
    let existing_condition_only = client
        .transact(TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: missing_key.clone(),
                expected_revision: first_revision,
            }],
            Vec::new(),
        ))
        .await
        .unwrap();
    assert_eq!(
        existing_condition_only,
        TransactionOutcome::conditions_satisfied()
    );
    let stale_condition_only = client
        .transact(TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: missing_key.clone(),
                expected_revision: Revision::ZERO,
            }],
            Vec::new(),
        ))
        .await
        .unwrap_err();
    assert!(matches!(
        stale_condition_only,
        ClientError::Application(error)
            if error.kind == dodb_protocol::ApplicationErrorKind::Conflict
    ));
    assert_eq!(
        client.get(missing_key.clone()).await.unwrap(),
        RevisionState::present(vec![0, 0xff, 1], first_revision)
    );
    let second_key = key(&[1], &[2]);
    let transaction = client
        .transact(TransactionRequest::new(
            Vec::new(),
            vec![
                TransactionMutation::Put {
                    key: second_key.clone(),
                    value: vec![3, 4],
                },
                TransactionMutation::Delete {
                    key: key(&[5], &[6]),
                },
            ],
        ))
        .await
        .unwrap();
    assert!(transaction.commit_lsn.is_some());
    let rows = client
        .query(dodb_core::PrimaryKey::new(vec![1]), None, 10)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].value, vec![3, 4]);
    assert_eq!(client.scan(None, 10).await.unwrap().len(), 2);

    let transaction = client
        .transact(TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: missing_key.clone(),
                expected_revision: first_revision,
            }],
            vec![TransactionMutation::Put {
                key: missing_key.clone(),
                value: vec![9],
            }],
        ))
        .await
        .unwrap();
    assert!(transaction.commit_lsn.is_some());
    let conflict = client
        .transact(TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: missing_key.clone(),
                expected_revision: first_revision,
            }],
            vec![TransactionMutation::Delete {
                key: missing_key.clone(),
            }],
        ))
        .await
        .unwrap_err();
    match conflict {
        ClientError::Application(error) => {
            assert_eq!(error.kind, dodb_protocol::ApplicationErrorKind::Conflict);
            assert_eq!(
                error.mutation_outcome,
                dodb_protocol::MutationOutcome::NotApplied
            );
            assert_eq!(
                error.conflict.as_ref().unwrap().expected,
                ConditionExpectation::RevisionEquals(first_revision)
            );
        }
        other => panic!("unexpected conflict error: {other}"),
    }

    let deleted_revision = client.delete(missing_key.clone()).await.unwrap();
    let aba_conflict = client
        .transact(TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: missing_key.clone(),
                expected_revision: Revision::ZERO,
            }],
            vec![TransactionMutation::Put {
                key: missing_key,
                value: vec![10],
            }],
        ))
        .await
        .unwrap_err();
    match aba_conflict {
        ClientError::Application(error) => {
            let conflict = error.conflict.as_ref().unwrap();
            assert_eq!(conflict.actual, ObservedState::missing(deleted_revision));
        }
        other => panic!("unexpected ABA error: {other}"),
    }

    let invalid = client
        .transact(TransactionRequest::new(
            Vec::new(),
            vec![
                TransactionMutation::Put {
                    key: key(b"invalid", b"key"),
                    value: vec![1],
                },
                TransactionMutation::Delete {
                    key: key(b"invalid", b"key"),
                },
            ],
        ))
        .await
        .unwrap_err();
    match invalid {
        ClientError::Application(error) => {
            assert_eq!(
                error.kind,
                dodb_protocol::ApplicationErrorKind::InvalidRequest
            );
            assert_eq!(
                error.mutation_outcome,
                dodb_protocol::MutationOutcome::NotApplied
            );
        }
        other => panic!("unexpected invalid request error: {other}"),
    }

    stop_server(client, server, task).await;
}

#[tokio::test]
async fn lazy_connection_dials_on_first_use_and_recovers_after_initial_failure() {
    let directory = tempfile::tempdir().unwrap();
    let tls = test_tls();
    let (server, task) = start_server(directory.path().to_owned(), &tls).await;
    let address = server.local_addr().unwrap();
    let connection = DodbConnection::connect_lazy(
        "0.0.0.0:0".parse().unwrap(),
        address,
        "localhost",
        ClientTlsConfig::from_der(vec![tls.certificate.clone()]).unwrap(),
        dodb_protocol::ProtocolLimits::default(),
    )
    .unwrap();
    let client = connection.for_tenant(TenantId::new(42));

    assert_eq!(server.metrics().snapshot().connections_total, 0);
    assert_eq!(
        client.get(key(b"lazy", b"available")).await.unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    let connections_after_first_request = server.metrics().snapshot().connections_total;
    assert_eq!(connections_after_first_request, 1);
    assert_eq!(
        client.get(key(b"lazy", b"reused")).await.unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    assert_eq!(
        server.metrics().snapshot().connections_total,
        connections_after_first_request
    );

    connection.close();
    server.shutdown().await;
    task.await.unwrap().unwrap();

    let wrong_tls = test_tls();
    let initial_failure_directory = tempfile::tempdir().unwrap();
    let (initial_failure_server, initial_failure_task) = start_server_with_limits(
        initial_failure_directory.path().to_owned(),
        &wrong_tls,
        dodb_protocol::ProtocolLimits::default(),
        64,
    )
    .await;
    let unavailable_address = initial_failure_server.local_addr().unwrap();
    let retry_connection = DodbConnection::connect_lazy(
        "0.0.0.0:0".parse().unwrap(),
        unavailable_address,
        "localhost",
        ClientTlsConfig::from_der(vec![tls.certificate.clone()]).unwrap(),
        dodb_protocol::ProtocolLimits::default(),
    )
    .unwrap();
    let retry_client = retry_connection.for_tenant(TenantId::new(43));
    let initial_failure = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        retry_client.get(key(b"lazy", b"initial-failure")),
    )
    .await
    .expect("initial lazy dial did not finish promptly")
    .unwrap_err();
    assert!(matches!(
        initial_failure,
        ClientError::Transport(_) | ClientError::Endpoint(_)
    ));
    initial_failure_server.shutdown().await;
    initial_failure_task.await.unwrap().unwrap();
    drop(initial_failure_server);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let retry_directory = tempfile::tempdir().unwrap();
    let (retry_server, retry_task) = start_server_with_limits_at(
        retry_directory.path().to_owned(),
        &tls,
        dodb_protocol::ProtocolLimits::default(),
        64,
        unavailable_address,
    )
    .await;
    assert_eq!(
        retry_client
            .get(key(b"lazy", b"after-recovery"))
            .await
            .unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    stop_server(retry_client, retry_server, retry_task).await;
}

#[tokio::test]
async fn concurrent_streams_and_tenant_isolation_are_preserved() {
    let directory = tempfile::tempdir().unwrap();
    let tls = test_tls();
    let (server, task) = start_server(directory.path().to_owned(), &tls).await;
    let first_client = connect_client(&server, &tls, TenantId::new(1)).await;
    let second_client = connect_client(&server, &tls, TenantId::new(2)).await;
    let shared_key = key(&[7], &[8]);
    let mut write_tasks = Vec::new();
    for value in 0..16u8 {
        let client = first_client.clone();
        let key = shared_key.clone();
        write_tasks.push(tokio::spawn(
            async move { client.put(key, vec![value]).await },
        ));
    }
    for task in write_tasks {
        task.await.unwrap().unwrap();
    }
    let connections_before_reads = server.metrics().snapshot().connections_total;
    let mut read_tasks = Vec::new();
    for _ in 0..64 {
        let client = first_client.clone();
        let key = shared_key.clone();
        read_tasks.push(tokio::spawn(async move { client.get(key).await }));
    }
    for task in read_tasks {
        assert!(matches!(
            task.await.unwrap().unwrap(),
            RevisionState::Present { .. }
        ));
    }
    assert_eq!(
        server.metrics().snapshot().connections_total,
        connections_before_reads,
        "concurrent independent Gets must reuse the shared QUIC connection"
    );
    assert_eq!(
        second_client.get(shared_key).await.unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    stop_server(first_client, server, task).await;
    second_client.close();
}

#[tokio::test]
async fn global_request_backpressure_preserves_unrelated_in_flight_streams() {
    let tls = test_tls();
    let service = Arc::new(BlockingService {
        entered: Arc::new(tokio::sync::Notify::new()),
        release: Arc::new(tokio::sync::Notify::new()),
        first_request: AtomicBool::new(false),
    });
    let server = Arc::new(
        DodbServer::bind(
            Arc::clone(&service),
            DodbServerConfig {
                listen_addr: "127.0.0.1:0".parse().unwrap(),
                tls: ServerTlsConfig::from_der(
                    vec![tls.certificate.clone()],
                    tls.private_key.clone(),
                )
                .unwrap(),
                protocol_limits: dodb_protocol::ProtocolLimits::default(),
                max_connections: 2,
                max_concurrent_streams: 8,
                max_concurrent_requests: 1,
            },
        )
        .unwrap(),
    );
    let task_server = Arc::clone(&server);
    let task = tokio::spawn(async move { task_server.run().await });
    let client = connect_client(&server, &tls, TenantId::new(71)).await;

    let entered = service.entered.notified();
    let first_client = client.clone();
    let first = tokio::spawn(async move { first_client.get(key(b"blocking", b"first")).await });
    entered.await;

    let second_client = client.clone();
    let second = tokio::spawn(async move { second_client.get(key(b"blocking", b"second")).await });
    tokio::task::yield_now().await;
    service.release.notify_one();

    assert_eq!(
        first.await.unwrap().unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    assert_eq!(
        second.await.unwrap().unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    stop_server(client, server, task).await;
}

#[tokio::test]
async fn idle_connection_does_not_hoard_global_request_capacity() {
    let tls = test_tls();
    let service = Arc::new(BlockingService {
        entered: Arc::new(tokio::sync::Notify::new()),
        release: Arc::new(tokio::sync::Notify::new()),
        first_request: AtomicBool::new(false),
    });
    let server = Arc::new(
        DodbServer::bind(
            Arc::clone(&service),
            DodbServerConfig {
                listen_addr: "127.0.0.1:0".parse().unwrap(),
                tls: ServerTlsConfig::from_der(
                    vec![tls.certificate.clone()],
                    tls.private_key.clone(),
                )
                .unwrap(),
                protocol_limits: dodb_protocol::ProtocolLimits::default(),
                max_connections: 2,
                max_concurrent_streams: 8,
                max_concurrent_requests: 1,
            },
        )
        .unwrap(),
    );
    let task_server = Arc::clone(&server);
    let task = tokio::spawn(async move { task_server.run().await });
    let idle_client = connect_client(&server, &tls, TenantId::new(74)).await;
    wait_for_active_connections(&server, 1).await;
    tokio::task::yield_now().await;

    let request_client = connect_client(&server, &tls, TenantId::new(75)).await;
    let entered = service.entered.notified();
    let request = tokio::spawn(async move {
        request_client
            .get(key(b"cross-connection", b"request"))
            .await
    });
    tokio::time::timeout(std::time::Duration::from_secs(5), entered)
        .await
        .expect("request on the second connection was blocked by the idle first connection");
    service.release.notify_one();
    assert_eq!(
        request.await.unwrap().unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    stop_server(idle_client, server, task).await;
}

#[tokio::test]
async fn aggregate_read_budget_returns_structured_query_and_scan_errors() {
    let directory = tempfile::tempdir().unwrap();
    let tls = test_tls();
    let limits = dodb_protocol::ProtocolLimits {
        max_request_frame_size: 3 * 1024 * 1024,
        max_response_frame_size: 3 * 1024 * 1024,
        ..dodb_protocol::ProtocolLimits::default()
    };
    let (server, task) =
        start_server_with_limits(directory.path().to_owned(), &tls, limits, 8).await;
    let client = connect_client_with_limits(&server, &tls, TenantId::new(72), limits).await;
    let first = key(b"aggregate", b"first");
    let second = key(b"aggregate", b"second");
    let value = vec![5; 2 * 1024 * 1024];
    client.put(first.clone(), value.clone()).await.unwrap();
    client.put(second.clone(), value).await.unwrap();

    let aggregate_results = vec![
        client
            .query(dodb_core::PrimaryKey::new(b"aggregate".to_vec()), None, 2)
            .await,
        client.scan(None, 2).await,
    ];
    for result in aggregate_results {
        match result.unwrap_err() {
            ClientError::Application(error) => {
                assert_eq!(
                    error.kind,
                    dodb_protocol::ApplicationErrorKind::ResponseTooLarge
                );
            }
            other => panic!("unexpected aggregate read error: {other}"),
        }
    }
    stop_server(client, server, task).await;
}

#[tokio::test]
async fn conflict_does_not_materialize_large_value_or_exceed_small_response_budget() {
    let directory = tempfile::tempdir().unwrap();
    let tls = test_tls();
    let limits = dodb_protocol::ProtocolLimits {
        max_response_frame_size: 256,
        ..dodb_protocol::ProtocolLimits::default()
    };
    let (server, task) =
        start_server_with_limits(directory.path().to_owned(), &tls, limits, 8).await;
    let client = connect_client_with_limits(&server, &tls, TenantId::new(76), limits).await;
    let conflict_key = key(b"large", b"value");
    let revision = client
        .put(conflict_key.clone(), vec![7; 2 * 1024 * 1024])
        .await
        .unwrap();

    let error = client
        .transact(TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: conflict_key.clone(),
                expected_revision: Revision::ZERO,
            }],
            vec![TransactionMutation::Delete { key: conflict_key }],
        ))
        .await
        .unwrap_err();
    match error {
        ClientError::Application(error) => {
            assert_eq!(error.kind, dodb_protocol::ApplicationErrorKind::Conflict);
            assert_eq!(
                error.mutation_outcome,
                dodb_protocol::MutationOutcome::NotApplied
            );
            assert_eq!(
                error.conflict.as_ref().unwrap().actual,
                ObservedState::present(revision)
            );
        }
        other => panic!("unexpected large-value conflict error: {other}"),
    }
    stop_server(client, server, task).await;
}

#[tokio::test]
async fn persisted_state_is_available_after_server_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let tls = test_tls();
    let tenant = TenantId::new(55);
    let persisted_key = key(&[0, 1], &[2, 3]);
    {
        let (server, task) = start_server(directory.path().to_owned(), &tls).await;
        let client = connect_client(&server, &tls, tenant).await;
        client
            .put(persisted_key.clone(), vec![4, 5, 6])
            .await
            .unwrap();
        stop_server(client, server, task).await;
    }
    {
        let (server, task) = start_server(directory.path().to_owned(), &tls).await;
        let client = connect_client(&server, &tls, tenant).await;
        assert_eq!(
            client.get(persisted_key).await.unwrap().value(),
            Some(&[4, 5, 6][..])
        );
        stop_server(client, server, task).await;
    }
}

#[tokio::test]
async fn shared_client_reconnects_after_same_address_restart() {
    let directory = tempfile::tempdir().unwrap();
    let tls = test_tls();
    let tenant = TenantId::new(56);
    let persisted_key = key(b"reconnect", b"value");
    let (server, task) = start_server(directory.path().to_owned(), &tls).await;
    let address = server.local_addr().unwrap();
    let connection = DodbConnection::connect(
        "0.0.0.0:0".parse().unwrap(),
        address,
        "localhost",
        ClientTlsConfig::from_der(vec![tls.certificate.clone()]).unwrap(),
        dodb_protocol::ProtocolLimits::default(),
    )
    .await
    .unwrap();
    let client = connection.for_tenant(tenant);
    client
        .put(persisted_key.clone(), vec![1, 2, 3])
        .await
        .unwrap();

    server.close();
    let first_failure = client.get(persisted_key.clone()).await.unwrap_err();
    assert!(matches!(
        first_failure,
        ClientError::Transport(_) | ClientError::Endpoint(_)
    ));
    server.shutdown().await;
    task.await.unwrap().unwrap();
    drop(server);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let (restarted, restarted_task) = start_server_with_limits_at(
        directory.path().to_owned(),
        &tls,
        dodb_protocol::ProtocolLimits::default(),
        64,
        address,
    )
    .await;
    let connections_before_recovery = restarted.metrics().snapshot().connections_total;
    let mut requests = Vec::new();
    for _ in 0..16 {
        let shared_client = client.clone();
        let persisted_key = persisted_key.clone();
        requests.push(tokio::spawn(async move {
            shared_client.get(persisted_key).await
        }));
    }
    for request in requests {
        assert_eq!(
            request.await.unwrap().unwrap().value(),
            Some(&[1, 2, 3][..])
        );
    }
    assert_eq!(
        restarted.metrics().snapshot().connections_total - connections_before_recovery,
        1,
        "concurrent future requests must share one reconnect dial"
    );

    connection.close();
    match client.get(key(b"reconnect", b"after-close")).await {
        Err(ClientError::Endpoint(detail)) => {
            assert!(detail.contains("explicitly closed"));
        }
        other => panic!("explicit close unexpectedly allowed reconnect: {other:?}"),
    }

    restarted.shutdown().await;
    restarted_task.await.unwrap().unwrap();
}

#[tokio::test]
async fn ambiguous_mutation_is_not_replayed_after_connection_loss() {
    let tls = test_tls();
    let service = Arc::new(BlockingMutationService {
        entered: Arc::new(tokio::sync::Notify::new()),
        release: Arc::new(tokio::sync::Notify::new()),
        calls: AtomicUsize::new(0),
    });
    let server = Arc::new(
        DodbServer::bind(
            Arc::clone(&service),
            DodbServerConfig {
                listen_addr: "127.0.0.1:0".parse().unwrap(),
                tls: ServerTlsConfig::from_der(
                    vec![tls.certificate.clone()],
                    tls.private_key.clone(),
                )
                .unwrap(),
                protocol_limits: dodb_protocol::ProtocolLimits::default(),
                max_connections: 2,
                max_concurrent_streams: 8,
                max_concurrent_requests: 8,
            },
        )
        .unwrap(),
    );
    let task_server = Arc::clone(&server);
    let task = tokio::spawn(async move { task_server.run().await });
    let client = connect_client(&server, &tls, TenantId::new(57)).await;
    let entered = service.entered.notified();
    let request_client = client.clone();
    let request = tokio::spawn(async move {
        request_client
            .put(key(b"ambiguous", b"mutation"), vec![1, 2, 3])
            .await
    });
    entered.await;
    server.close();
    service.release.notify_one();
    match request.await.unwrap().unwrap_err() {
        ClientError::UnknownMutationOutcome { cause: None, .. } => {}
        other => panic!("ambiguous mutation returned the wrong error: {other:?}"),
    }
    assert_eq!(service.calls.load(Ordering::SeqCst), 1);
    server.shutdown().await;
    task.await.unwrap().unwrap();
}

struct FailWalSync;

impl FaultInjector for FailWalSync {
    fn hit(&mut self, point: &str) -> dodb_core::Result<()> {
        if point == "during_wal_sync" {
            return Err(Error::durability("injected WAL sync failure"));
        }
        Ok(())
    }
}

#[tokio::test]
async fn wal_durability_failure_is_unknown_over_the_real_client_path() {
    let directory = tempfile::tempdir().unwrap();
    let tls = test_tls();
    let mut local_service = LocalTenantService::new(LocalTenantServiceConfig {
        data_dir: directory.path().to_owned(),
        ..LocalTenantServiceConfig::default()
    })
    .unwrap();
    local_service.set_fault_injector_factory(Arc::new(|_, _| Box::new(FailWalSync)));
    let server = Arc::new(
        DodbServer::bind(
            Arc::new(local_service),
            DodbServerConfig {
                listen_addr: "127.0.0.1:0".parse().unwrap(),
                tls: ServerTlsConfig::from_der(
                    vec![tls.certificate.clone()],
                    tls.private_key.clone(),
                )
                .unwrap(),
                protocol_limits: dodb_protocol::ProtocolLimits::default(),
                max_connections: 2,
                max_concurrent_streams: 8,
                max_concurrent_requests: 8,
            },
        )
        .unwrap(),
    );
    let task_server = Arc::clone(&server);
    let task = tokio::spawn(async move { task_server.run().await });
    let client = connect_client(&server, &tls, TenantId::new(73)).await;
    let error = client
        .put(key(b"durability", b"failure"), vec![1, 2, 3])
        .await
        .unwrap_err();
    match error {
        ClientError::UnknownMutationOutcome {
            cause: Some(cause), ..
        } => {
            assert_eq!(
                cause.kind,
                dodb_protocol::ApplicationErrorKind::DurabilityFailure
            );
            assert_eq!(
                cause.mutation_outcome,
                dodb_protocol::MutationOutcome::Unknown
            );
        }
        other => panic!("unexpected durability error: {other}"),
    }
    stop_server(client, server, task).await;
}

#[tokio::test]
async fn abrupt_disconnects_do_not_trigger_implicit_mutation_retries() {
    let directory = tempfile::tempdir().unwrap();
    let tls = test_tls();
    let (server, task) = start_server(directory.path().to_owned(), &tls).await;
    let tenant = TenantId::new(91);
    let key = key(&[1], &[2]);

    abruptly_disconnect(&server, &tls, None).await;
    abruptly_disconnect(
        &server,
        &tls,
        Some(dodb_service::Request::Get { key: key.clone() }),
    )
    .await;
    let client = connect_client(&server, &tls, tenant).await;
    assert_eq!(
        client.get(key.clone()).await.unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    client.close();

    abruptly_disconnect(
        &server,
        &tls,
        Some(dodb_service::Request::Put {
            key: key.clone(),
            value: vec![8, 9],
        }),
    )
    .await;
    let reconnect = connect_client(&server, &tls, tenant).await;
    let state = reconnect.get(key).await.unwrap();
    match state {
        RevisionState::Missing { .. } => {}
        RevisionState::Present { value, .. } => assert_eq!(value, vec![8, 9]),
    }
    stop_server(reconnect, server, task).await;
}
