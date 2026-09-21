use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use dibi::{
    DibiServer, DibiServerConfig,
    dibi_protocol::{
        AdminItem, ExecuteOperation, ExecuteResult, Key, QueryItem, RequestOperation,
        ResponsePayload, Status, TransactWriteOperation, VersionedItem, WriteOperation,
        decode_response_frame, decode_response_payload, encode_request_frame,
    },
};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::CertificateDer;
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle};

struct TestServer {
    address: std::net::SocketAddr,
    shutdown_sender: Option<oneshot::Sender<()>>,
    task: JoinHandle<Result<(), dibi::ServerError>>,
}

struct TestClient {
    endpoint: quinn::Endpoint,
    connection: quinn::Connection,
    next_request_id: AtomicU64,
}

fn create_certificate(directory: &Path) -> (PathBuf, PathBuf) {
    let certificate = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
    let cert_path = directory.join("cert.pem");
    let key_path = directory.join("key.pem");
    fs::write(&cert_path, certificate.cert.pem()).unwrap();
    fs::write(&key_path, certificate.signing_key.serialize_pem()).unwrap();
    (cert_path, key_path)
}

async fn start_server(data_dir: &Path, cert_path: &Path, key_path: &Path) -> TestServer {
    let config = DibiServerConfig::new(
        data_dir,
        "127.0.0.1:0".parse().unwrap(),
        cert_path,
        key_path,
    );
    let server = DibiServer::bind(config).unwrap();
    let address = server.local_addr().unwrap();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel();
    let task = tokio::spawn(server.run(async move {
        let _ = shutdown_receiver.await;
    }));
    TestServer {
        address,
        shutdown_sender: Some(shutdown_sender),
        task,
    }
}

async fn connect_client(address: std::net::SocketAddr, cert_path: &Path) -> TestClient {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let certificate_bytes = fs::read(cert_path).unwrap();
    let certificates = rustls_pemfile::certs(&mut certificate_bytes.as_slice())
        .collect::<Result<Vec<CertificateDer<'static>>, _>>()
        .unwrap();
    let mut roots = rustls::RootCertStore::empty();
    for certificate in certificates {
        roots.add(certificate).unwrap();
    }
    let mut crypto_config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    crypto_config.alpn_protocols = vec![b"dibi/1".to_vec()];
    let crypto_config = quinn::crypto::rustls::QuicClientConfig::try_from(crypto_config).unwrap();
    let mut endpoint = quinn::Endpoint::client("127.0.0.1:0".parse().unwrap()).unwrap();
    endpoint.set_default_client_config(quinn::ClientConfig::new(Arc::new(crypto_config)));
    let connection = endpoint
        .connect(address, "localhost")
        .unwrap()
        .await
        .unwrap();
    TestClient {
        endpoint,
        connection,
        next_request_id: AtomicU64::new(1),
    }
}

impl TestClient {
    async fn request(&self, operation: RequestOperation) -> (Status, ResponsePayload) {
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let opcode = operation.opcode();
        let frame = encode_request_frame(request_id, &operation);
        let (mut send, mut receive) = self.connection.open_bi().await.unwrap();
        send.write_all(&frame).await.unwrap();
        send.finish().unwrap();

        let mut header = [0_u8; dibi_protocol::FRAME_HEADER_SIZE];
        receive.read_exact(&mut header).await.unwrap();
        let payload_len = u32::from_be_bytes(header[16..20].try_into().unwrap()) as usize;
        let mut encoded_response = header.to_vec();
        let mut payload = vec![0_u8; payload_len];
        receive.read_exact(&mut payload).await.unwrap();
        encoded_response.extend_from_slice(&payload);
        let mut trailing = [0_u8; 1];
        assert!(matches!(
            receive.read(&mut trailing).await.unwrap(),
            Some(0) | None
        ));
        let response = decode_response_frame(&encoded_response).unwrap();
        assert_eq!(response.request_id, request_id);
        let payload = decode_response_payload(opcode, response.status, &response.payload).unwrap();
        (response.status, payload)
    }

    async fn request_ok(&self, operation: RequestOperation) -> ResponsePayload {
        let (status, payload) = self.request(operation).await;
        assert_eq!(status, Status::Ok);
        payload
    }

    fn close(&self) {
        self.endpoint.close(0_u32.into(), b"test complete");
    }
}

impl TestServer {
    async fn shutdown(mut self) {
        self.shutdown_sender.take().unwrap().send(()).unwrap();
        self.task.await.unwrap().unwrap();
    }
}

fn found_payload(payload: ResponsePayload) -> (bool, Option<Vec<u8>>) {
    match payload {
        ResponsePayload::Found { found, data } => (found, data),
        other => panic!("unexpected payload: {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn basic_operations_and_status() {
    let directory = tempfile::tempdir().unwrap();
    let (cert_path, key_path) = create_certificate(directory.path());
    let server = start_server(directory.path(), &cert_path, &key_path).await;
    let client = connect_client(server.address, &cert_path).await;

    let status = client.request_ok(RequestOperation::Status).await;
    let (db_uuid, last_commit_id) = match status {
        ResponsePayload::Status {
            db_uuid,
            last_commit_id,
        } => (db_uuid, last_commit_id),
        other => panic!("unexpected status payload: {other:?}"),
    };
    assert_ne!(db_uuid, [0; 16]);
    assert_eq!(last_commit_id, 0);
    assert_eq!(
        client.request_ok(RequestOperation::Ping).await,
        ResponsePayload::Empty
    );

    assert_eq!(
        found_payload(
            client
                .request_ok(RequestOperation::Get {
                    pk: "missing".to_owned(),
                    sk: "key".to_owned(),
                })
                .await
        ),
        (false, None)
    );
    let binary_data = vec![0, 1, 2, 255];
    assert!(matches!(
        client
            .request_ok(RequestOperation::Put {
                pk: "partition\0".to_owned(),
                sk: "a\0".to_owned(),
                data: binary_data.clone(),
            })
            .await,
        ResponsePayload::CommitId(1)
    ));
    assert_eq!(
        found_payload(
            client
                .request_ok(RequestOperation::Get {
                    pk: "partition\0".to_owned(),
                    sk: "a\0".to_owned(),
                })
                .await
        ),
        (true, Some(binary_data))
    );
    assert!(matches!(
        client
            .request_ok(RequestOperation::GetWithVersion {
                pk: "partition\0".to_owned(),
                sk: "a\0".to_owned(),
            })
            .await,
        ResponsePayload::Versioned {
            found: true,
            version: Some(0),
            ..
        }
    ));
    assert!(matches!(
        client
            .request_ok(RequestOperation::Delete {
                pk: "partition\0".to_owned(),
                sk: "a\0".to_owned(),
            })
            .await,
        ResponsePayload::CommitId(2)
    ));
    assert_eq!(
        found_payload(
            client
                .request_ok(RequestOperation::Get {
                    pk: "partition\0".to_owned(),
                    sk: "a\0".to_owned(),
                })
                .await
        ),
        (false, None)
    );

    let batch = client
        .request_ok(RequestOperation::Batch(vec![
            WriteOperation::Put {
                pk: "partition".to_owned(),
                sk: "z".to_owned(),
                data: b"z".to_vec(),
            },
            WriteOperation::Put {
                pk: "partition".to_owned(),
                sk: "b".to_owned(),
                data: b"b".to_vec(),
            },
        ]))
        .await;
    assert_eq!(batch, ResponsePayload::OptionalCommitId(Some(3)));
    assert_eq!(
        client
            .request_ok(RequestOperation::Query {
                pk: "partition".to_owned(),
                after_sk: None,
                limit: 10,
            })
            .await,
        ResponsePayload::QueryItems(vec![
            QueryItem {
                sk: "b".to_owned(),
                data: b"b".to_vec(),
            },
            QueryItem {
                sk: "z".to_owned(),
                data: b"z".to_vec(),
            },
        ])
    );
    assert_eq!(
        client
            .request_ok(RequestOperation::Scan {
                cursor: None,
                limit: 10,
            })
            .await,
        ResponsePayload::ScanItems(vec![
            dibi_protocol::ScanItem {
                pk: "partition".to_owned(),
                sk: "b".to_owned(),
                data: b"b".to_vec(),
            },
            dibi_protocol::ScanItem {
                pk: "partition".to_owned(),
                sk: "z".to_owned(),
                data: b"z".to_vec(),
            },
        ])
    );

    let execute_results = client
        .request_ok(RequestOperation::ExecuteOps(vec![
            ExecuteOperation::Put {
                pk: "execute".to_owned(),
                sk: "key".to_owned(),
                data: b"one".to_vec(),
            },
            ExecuteOperation::Get {
                pk: "execute".to_owned(),
                sk: "key".to_owned(),
            },
            ExecuteOperation::Put {
                pk: "execute".to_owned(),
                sk: "key".to_owned(),
                data: b"two".to_vec(),
            },
            ExecuteOperation::Get {
                pk: "execute".to_owned(),
                sk: "key".to_owned(),
            },
        ]))
        .await;
    assert_eq!(
        execute_results,
        ResponsePayload::ExecuteResults(vec![
            ExecuteResult::Done,
            ExecuteResult::Single {
                found: true,
                data: Some(b"one".to_vec()),
            },
            ExecuteResult::Done,
            ExecuteResult::Single {
                found: true,
                data: Some(b"two".to_vec()),
            },
        ])
    );

    client.close();
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn transact_write_items_admin_and_duplicate_validation() {
    let directory = tempfile::tempdir().unwrap();
    let (cert_path, key_path) = create_certificate(directory.path());
    let server = start_server(directory.path(), &cert_path, &key_path).await;
    let client = connect_client(server.address, &cert_path).await;

    assert_eq!(
        client
            .request_ok(RequestOperation::TransactWriteItems(vec![
                TransactWriteOperation::Create {
                    pk: "conditional".to_owned(),
                    sk: "key".to_owned(),
                    data: b"old".to_vec(),
                },
            ]))
            .await,
        ResponsePayload::OptionalCommitId(Some(1))
    );
    let (conflict_status, conflict_payload) = client
        .request(RequestOperation::TransactWriteItems(vec![
            TransactWriteOperation::Create {
                pk: "conditional".to_owned(),
                sk: "key".to_owned(),
                data: b"conflict".to_vec(),
            },
        ]))
        .await;
    assert_eq!(conflict_status, Status::Conflict);
    assert_eq!(
        conflict_payload,
        ResponsePayload::ConditionalConflicts(vec![dibi_protocol::Conflict {
            pk: "conditional".to_owned(),
            sk: "key".to_owned(),
            expected_version: None,
            actual_version: Some(0),
        }])
    );
    assert_eq!(
        client
            .request_ok(RequestOperation::AdminScan {
                cursor: None,
                limit: 10,
                pk_prefix: Some("conditional".to_owned()),
            })
            .await,
        ResponsePayload::AdminScan {
            items: vec![AdminItem {
                pk: "conditional".to_owned(),
                sk: "key".to_owned(),
                version: 0,
                data: b"old".to_vec(),
            }],
            next_cursor: None,
        }
    );
    assert_eq!(
        client
            .request_ok(RequestOperation::AdminTransactWriteItems(vec![
                TransactWriteOperation::Put {
                    pk: "conditional".to_owned(),
                    sk: "key".to_owned(),
                    expected_version: 0,
                    data: b"new".to_vec(),
                },
            ]))
            .await,
        ResponsePayload::OptionalCommitId(Some(2))
    );
    let (duplicate_status, _) = client
        .request(RequestOperation::TransactWriteItems(vec![
            TransactWriteOperation::Create {
                pk: "duplicate".to_owned(),
                sk: "key".to_owned(),
                data: b"first".to_vec(),
            },
            TransactWriteOperation::Delete {
                pk: "duplicate".to_owned(),
                sk: "key".to_owned(),
                expected_version: 0,
            },
        ]))
        .await;
    assert_eq!(duplicate_status, Status::InvalidRequest);
    client.close();
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn batch_get_with_version_preserves_order() {
    let directory = tempfile::tempdir().unwrap();
    let (cert_path, key_path) = create_certificate(directory.path());
    let server = start_server(directory.path(), &cert_path, &key_path).await;
    let client = connect_client(server.address, &cert_path).await;

    client
        .request_ok(RequestOperation::TransactWriteItems(vec![
            TransactWriteOperation::Create {
                pk: "present".to_owned(),
                sk: "a".to_owned(),
                data: b"a".to_vec(),
            },
            TransactWriteOperation::Create {
                pk: "present".to_owned(),
                sk: "b".to_owned(),
                data: b"b".to_vec(),
            },
        ]))
        .await;
    assert_eq!(
        client
            .request_ok(RequestOperation::BatchGetWithVersion(vec![
                Key {
                    pk: "missing".to_owned(),
                    sk: "key".to_owned(),
                },
                Key {
                    pk: "present".to_owned(),
                    sk: "b".to_owned(),
                },
                Key {
                    pk: "present".to_owned(),
                    sk: "a".to_owned(),
                },
            ]))
            .await,
        ResponsePayload::VersionedItems(vec![
            VersionedItem {
                found: false,
                version: None,
                data: None,
            },
            VersionedItem {
                found: true,
                version: Some(0),
                data: Some(b"b".to_vec()),
            },
            VersionedItem {
                found: true,
                version: Some(0),
                data: Some(b"a".to_vec()),
            },
        ])
    );

    client.close();
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn transact_write_version_correctness() {
    let directory = tempfile::tempdir().unwrap();
    let (cert_path, key_path) = create_certificate(directory.path());
    let server = start_server(directory.path(), &cert_path, &key_path).await;
    let client = connect_client(server.address, &cert_path).await;

    assert_eq!(
        client
            .request_ok(RequestOperation::TransactWriteItems(vec![
                TransactWriteOperation::Create {
                    pk: "versioned".to_owned(),
                    sk: "key".to_owned(),
                    data: b"zero".to_vec(),
                },
            ]))
            .await,
        ResponsePayload::OptionalCommitId(Some(1))
    );
    assert_eq!(
        client
            .request_ok(RequestOperation::TransactWriteItems(vec![
                TransactWriteOperation::Put {
                    pk: "versioned".to_owned(),
                    sk: "key".to_owned(),
                    expected_version: 0,
                    data: b"one".to_vec(),
                },
            ]))
            .await,
        ResponsePayload::OptionalCommitId(Some(2))
    );
    let (status, payload) = client
        .request(RequestOperation::TransactWriteItems(vec![
            TransactWriteOperation::Put {
                pk: "versioned".to_owned(),
                sk: "key".to_owned(),
                expected_version: 0,
                data: b"stale".to_vec(),
            },
        ]))
        .await;
    assert_eq!(status, Status::Conflict);
    assert_eq!(
        payload,
        ResponsePayload::ConditionalConflicts(vec![dibi_protocol::Conflict {
            pk: "versioned".to_owned(),
            sk: "key".to_owned(),
            expected_version: Some(0),
            actual_version: Some(1),
        }])
    );

    client.close();
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn same_version_race_has_one_success_and_one_conflict() {
    let directory = tempfile::tempdir().unwrap();
    let (cert_path, key_path) = create_certificate(directory.path());
    let server = start_server(directory.path(), &cert_path, &key_path).await;
    let first_client = Arc::new(connect_client(server.address, &cert_path).await);
    let second_client = Arc::new(connect_client(server.address, &cert_path).await);

    first_client
        .request_ok(RequestOperation::TransactWriteItems(vec![
            TransactWriteOperation::Create {
                pk: "race".to_owned(),
                sk: "key".to_owned(),
                data: b"initial".to_vec(),
            },
        ]))
        .await;
    for client in [&first_client, &second_client] {
        assert_eq!(
            client
                .request_ok(RequestOperation::GetWithVersion {
                    pk: "race".to_owned(),
                    sk: "key".to_owned(),
                })
                .await,
            ResponsePayload::Versioned {
                found: true,
                version: Some(0),
                data: Some(b"initial".to_vec()),
            }
        );
    }

    let first_operation = RequestOperation::TransactWriteItems(vec![TransactWriteOperation::Put {
        pk: "race".to_owned(),
        sk: "key".to_owned(),
        expected_version: 0,
        data: b"first".to_vec(),
    }]);
    let second_operation =
        RequestOperation::TransactWriteItems(vec![TransactWriteOperation::Put {
            pk: "race".to_owned(),
            sk: "key".to_owned(),
            expected_version: 0,
            data: b"second".to_vec(),
        }]);
    let (first_result, second_result) = tokio::join!(
        first_client.request(first_operation),
        second_client.request(second_operation),
    );
    let statuses = [first_result.0, second_result.0];
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == Status::Ok)
            .count(),
        1
    );
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == Status::Conflict)
            .count(),
        1
    );

    let final_value = first_client
        .request_ok(RequestOperation::GetWithVersion {
            pk: "race".to_owned(),
            sk: "key".to_owned(),
        })
        .await;
    assert!(matches!(
        final_value,
        ResponsePayload::Versioned {
            found: true,
            version: Some(1),
            data: Some(_),
        }
    ));

    first_client.close();
    second_client.close();
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn multi_item_transact_write_is_atomic() {
    let directory = tempfile::tempdir().unwrap();
    let (cert_path, key_path) = create_certificate(directory.path());
    let server = start_server(directory.path(), &cert_path, &key_path).await;
    let client = connect_client(server.address, &cert_path).await;

    client
        .request_ok(RequestOperation::TransactWriteItems(vec![
            TransactWriteOperation::Create {
                pk: "atomic".to_owned(),
                sk: "a".to_owned(),
                data: b"a0".to_vec(),
            },
            TransactWriteOperation::Create {
                pk: "atomic".to_owned(),
                sk: "b".to_owned(),
                data: b"b0".to_vec(),
            },
        ]))
        .await;
    for expected_version in 0_i64..3 {
        client
            .request_ok(RequestOperation::TransactWriteItems(vec![
                TransactWriteOperation::Put {
                    pk: "atomic".to_owned(),
                    sk: "a".to_owned(),
                    expected_version,
                    data: format!("a{expected_version}").into_bytes(),
                },
            ]))
            .await;
    }
    for expected_version in 0_i64..7 {
        client
            .request_ok(RequestOperation::TransactWriteItems(vec![
                TransactWriteOperation::Put {
                    pk: "atomic".to_owned(),
                    sk: "b".to_owned(),
                    expected_version,
                    data: format!("b{expected_version}").into_bytes(),
                },
            ]))
            .await;
    }
    assert!(matches!(
        client
            .request_ok(RequestOperation::TransactWriteItems(vec![
                TransactWriteOperation::Put {
                    pk: "atomic".to_owned(),
                    sk: "a".to_owned(),
                    expected_version: 3,
                    data: b"a3".to_vec(),
                },
                TransactWriteOperation::Delete {
                    pk: "atomic".to_owned(),
                    sk: "b".to_owned(),
                    expected_version: 7,
                },
                TransactWriteOperation::Create {
                    pk: "atomic".to_owned(),
                    sk: "c".to_owned(),
                    data: b"c0".to_vec(),
                },
            ]))
            .await,
        ResponsePayload::OptionalCommitId(Some(_))
    ));
    assert_eq!(
        client
            .request_ok(RequestOperation::BatchGetWithVersion(vec![
                Key {
                    pk: "atomic".to_owned(),
                    sk: "a".to_owned(),
                },
                Key {
                    pk: "atomic".to_owned(),
                    sk: "b".to_owned(),
                },
                Key {
                    pk: "atomic".to_owned(),
                    sk: "c".to_owned(),
                },
            ]))
            .await,
        ResponsePayload::VersionedItems(vec![
            VersionedItem {
                found: true,
                version: Some(4),
                data: Some(b"a3".to_vec()),
            },
            VersionedItem {
                found: false,
                version: None,
                data: None,
            },
            VersionedItem {
                found: true,
                version: Some(0),
                data: Some(b"c0".to_vec()),
            },
        ])
    );

    client
        .request_ok(RequestOperation::TransactWriteItems(vec![
            TransactWriteOperation::Create {
                pk: "atomic-failure".to_owned(),
                sk: "a".to_owned(),
                data: b"a3".to_vec(),
            },
            TransactWriteOperation::Create {
                pk: "atomic-failure".to_owned(),
                sk: "b".to_owned(),
                data: b"b7".to_vec(),
            },
        ]))
        .await;
    for expected_version in 0_i64..3 {
        client
            .request_ok(RequestOperation::TransactWriteItems(vec![
                TransactWriteOperation::Put {
                    pk: "atomic-failure".to_owned(),
                    sk: "a".to_owned(),
                    expected_version,
                    data: format!("a{expected_version}").into_bytes(),
                },
            ]))
            .await;
    }
    for expected_version in 0_i64..7 {
        client
            .request_ok(RequestOperation::TransactWriteItems(vec![
                TransactWriteOperation::Put {
                    pk: "atomic-failure".to_owned(),
                    sk: "b".to_owned(),
                    expected_version,
                    data: format!("b{expected_version}").into_bytes(),
                },
            ]))
            .await;
    }
    let (status, _) = client
        .request(RequestOperation::TransactWriteItems(vec![
            TransactWriteOperation::Put {
                pk: "atomic-failure".to_owned(),
                sk: "a".to_owned(),
                expected_version: 2,
                data: b"must-not-apply".to_vec(),
            },
            TransactWriteOperation::Delete {
                pk: "atomic-failure".to_owned(),
                sk: "b".to_owned(),
                expected_version: 7,
            },
            TransactWriteOperation::Create {
                pk: "atomic-failure".to_owned(),
                sk: "c".to_owned(),
                data: b"must-not-create".to_vec(),
            },
        ]))
        .await;
    assert_eq!(status, Status::Conflict);
    assert_eq!(
        client
            .request_ok(RequestOperation::BatchGetWithVersion(vec![
                Key {
                    pk: "atomic-failure".to_owned(),
                    sk: "a".to_owned(),
                },
                Key {
                    pk: "atomic-failure".to_owned(),
                    sk: "b".to_owned(),
                },
                Key {
                    pk: "atomic-failure".to_owned(),
                    sk: "c".to_owned(),
                },
            ]))
            .await,
        ResponsePayload::VersionedItems(vec![
            VersionedItem {
                found: true,
                version: Some(3),
                data: Some(b"a2".to_vec()),
            },
            VersionedItem {
                found: true,
                version: Some(7),
                data: Some(b"b6".to_vec()),
            },
            VersionedItem {
                found: false,
                version: None,
                data: None,
            },
        ])
    );

    client.close();
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_streams_are_supported() {
    let directory = tempfile::tempdir().unwrap();
    let (cert_path, key_path) = create_certificate(directory.path());
    let server = start_server(directory.path(), &cert_path, &key_path).await;
    let client = Arc::new(connect_client(server.address, &cert_path).await);
    let mut stream_tasks = Vec::new();
    for request_index in 0..8 {
        let client = Arc::clone(&client);
        stream_tasks.push(tokio::spawn(async move {
            client
                .request_ok(RequestOperation::Put {
                    pk: "concurrent".to_owned(),
                    sk: format!("{request_index:02}"),
                    data: vec![request_index],
                })
                .await
        }));
    }
    for stream_task in stream_tasks {
        assert!(matches!(
            stream_task.await.unwrap(),
            ResponsePayload::CommitId(_)
        ));
    }
    assert_eq!(
        client
            .request_ok(RequestOperation::Query {
                pk: "concurrent".to_owned(),
                after_sk: None,
                limit: 10,
            })
            .await,
        ResponsePayload::QueryItems(
            (0..8)
                .map(|request_index| dibi_protocol::QueryItem {
                    sk: format!("{request_index:02}"),
                    data: vec![request_index],
                })
                .collect(),
        )
    );
    client.close();
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_streams_and_restart_preserve_data() {
    let directory = TempDir::new().unwrap();
    let (cert_path, key_path) = create_certificate(directory.path());
    let first_server = start_server(directory.path(), &cert_path, &key_path).await;
    let client = connect_client(first_server.address, &cert_path).await;
    let initial_status = client.request_ok(RequestOperation::Status).await;
    let initial_uuid = match initial_status {
        ResponsePayload::Status { db_uuid, .. } => db_uuid,
        other => panic!("unexpected status payload: {other:?}"),
    };
    client
        .request_ok(RequestOperation::Put {
            pk: "persistent".to_owned(),
            sk: "key".to_owned(),
            data: b"value".to_vec(),
        })
        .await;
    client.close();
    first_server.shutdown().await;

    let second_server = start_server(directory.path(), &cert_path, &key_path).await;
    let reopened_client = connect_client(second_server.address, &cert_path).await;
    let reopened_status = reopened_client.request_ok(RequestOperation::Status).await;
    assert_eq!(
        reopened_status,
        ResponsePayload::Status {
            db_uuid: initial_uuid,
            last_commit_id: 1,
        }
    );
    let reopened_document = reopened_client
        .request_ok(RequestOperation::GetWithVersion {
            pk: "persistent".to_owned(),
            sk: "key".to_owned(),
        })
        .await;
    assert_eq!(
        reopened_document,
        ResponsePayload::Versioned {
            found: true,
            version: Some(0),
            data: Some(b"value".to_vec()),
        }
    );
    reopened_client.close();
    second_server.shutdown().await;
}
