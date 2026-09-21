use std::{
    fs,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bytes::Bytes;
use dibi_protocol::{
    AdminItem, ExecuteOperation, ExecuteResult, Key, QueryItem, RequestOperation, ResponsePayload,
    ScanItem, Status, TransactWriteOperation, VersionedItem, WriteOperation, decode_request_frame,
    encode_response_frame, validate_tenant,
};
use rustls::pki_types::CertificateDer;
use tokio::{task::JoinSet, time::timeout};
use tracing::{error, warn};

use crate::{
    AdminScanRequest, ApplicationWrite, ConditionalWrite, ConditionalWriteOutcome, DibiEngine,
    DibiError, Document, StoredDocument,
};

const ALPN_PROTOCOL: &[u8] = b"dibi/2";
const DEFAULT_REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_REQUEST_PROCESSING_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_PAYLOAD_SIZE: usize = dibi_protocol::MAX_FRAME_SIZE - dibi_protocol::FRAME_HEADER_SIZE;

#[derive(Clone, Debug)]
pub struct DibiServerConfig {
    pub data_dir: PathBuf,
    pub listen: SocketAddr,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub worker_token: Vec<u8>,
    pub request_read_timeout: Duration,
    pub request_processing_timeout: Duration,
}

impl DibiServerConfig {
    pub fn new(
        data_dir: impl Into<PathBuf>,
        listen: SocketAddr,
        cert_path: impl Into<PathBuf>,
        key_path: impl Into<PathBuf>,
        worker_token: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            data_dir: data_dir.into(),
            listen,
            cert_path: cert_path.into(),
            key_path: key_path.into(),
            worker_token: worker_token.into(),
            request_read_timeout: DEFAULT_REQUEST_READ_TIMEOUT,
            request_processing_timeout: DEFAULT_REQUEST_PROCESSING_TIMEOUT,
        }
    }
}

pub struct DibiServer {
    endpoint: quinn::Endpoint,
    state: Arc<ServerState>,
    request_read_timeout: Duration,
    request_processing_timeout: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TLS configuration error: {0}")]
    Tls(String),
    #[error("QUIC configuration error: {0}")]
    Quic(String),
    #[error("database error: {0}")]
    Database(#[from] DibiError),
}

struct ServerState {
    engine: DibiEngine,
    worker_token: Arc<[u8]>,
}

enum OperationError {
    Database(DibiError),
    Conflict(Vec<dibi_protocol::Conflict>),
    Internal,
}

impl DibiServer {
    pub fn bind(config: DibiServerConfig) -> Result<Self, ServerError> {
        if config.worker_token.is_empty() {
            return Err(ServerError::Quic(
                "DIBI_WORKER_TOKEN must be configured and non-empty".to_owned(),
            ));
        }
        let engine = DibiEngine::open(&config.data_dir)?;
        let server_config = load_server_config(&config.cert_path, &config.key_path)?;
        let endpoint = quinn::Endpoint::server(server_config, config.listen)?;
        Ok(Self {
            endpoint,
            state: Arc::new(ServerState {
                engine,
                worker_token: Arc::from(config.worker_token),
            }),
            request_read_timeout: config.request_read_timeout,
            request_processing_timeout: config.request_processing_timeout,
        })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, ServerError> {
        Ok(self.endpoint.local_addr()?)
    }

    pub async fn run(
        self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<(), ServerError> {
        let mut connection_tasks = JoinSet::new();
        tokio::pin!(shutdown_signal);
        loop {
            tokio::select! {
                _ = &mut shutdown_signal => break,
                incoming = self.endpoint.accept() => {
                    let Some(incoming) = incoming else {
                        break;
                    };
                    match incoming.await {
                        Ok(connection) => {
                            let state = Arc::clone(&self.state);
                            let read_timeout = self.request_read_timeout;
                            let processing_timeout = self.request_processing_timeout;
                            connection_tasks.spawn(async move {
                                handle_connection(connection, state, read_timeout, processing_timeout).await;
                            });
                        }
                        Err(connection_error) => {
                            warn!(%connection_error, "QUIC connection handshake failed");
                        }
                    }
                }
            }
        }
        self.endpoint.close(0u32.into(), b"server shutting down");
        while connection_tasks.join_next().await.is_some() {}
        Ok(())
    }
}

fn load_server_config(
    cert_path: &std::path::Path,
    key_path: &std::path::Path,
) -> Result<quinn::ServerConfig, ServerError> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let certificate_bytes = fs::read(cert_path)?;
    let certificates = rustls_pemfile::certs(&mut certificate_bytes.as_slice())
        .collect::<std::result::Result<Vec<CertificateDer<'static>>, _>>()
        .map_err(|error| ServerError::Tls(format!("invalid certificate PEM: {error}")))?;
    if certificates.is_empty() {
        return Err(ServerError::Tls("certificate chain is empty".to_owned()));
    }
    let key_bytes = fs::read(key_path)?;
    let private_key = rustls_pemfile::private_key(&mut key_bytes.as_slice())
        .map_err(|error| ServerError::Tls(format!("invalid private key PEM: {error}")))?
        .ok_or_else(|| ServerError::Tls("private key is missing".to_owned()))?;
    let mut crypto_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificates, private_key)
        .map_err(|error| ServerError::Tls(error.to_string()))?;
    crypto_config.alpn_protocols = vec![ALPN_PROTOCOL.to_vec()];
    let crypto_config = quinn::crypto::rustls::QuicServerConfig::try_from(crypto_config)
        .map_err(|error| ServerError::Quic(error.to_string()))?;
    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(crypto_config));
    let transport_config = Arc::get_mut(&mut server_config.transport)
        .ok_or_else(|| ServerError::Quic("transport configuration is shared".to_owned()))?;
    transport_config.max_concurrent_uni_streams(0_u8.into());
    transport_config.max_concurrent_bidi_streams(1024_u32.into());
    Ok(server_config)
}

async fn handle_connection(
    connection: quinn::Connection,
    state: Arc<ServerState>,
    request_read_timeout: Duration,
    request_processing_timeout: Duration,
) {
    let authenticated = Arc::new(AtomicBool::new(false));
    let mut stream_tasks = JoinSet::new();
    loop {
        tokio::select! {
            stream = connection.accept_bi() => {
                match stream {
                    Ok((send, receive)) => {
                        let state = Arc::clone(&state);
                        let authenticated = Arc::clone(&authenticated);
                        stream_tasks.spawn(async move {
                            handle_stream(
                                send,
                                receive,
                                state,
                                authenticated,
                                request_read_timeout,
                                request_processing_timeout,
                            )
                            .await;
                        });
                    }
                    Err(_) => break,
                }
            }
            task = stream_tasks.join_next(), if !stream_tasks.is_empty() => {
                if let Some(Err(task_error)) = task {
                    error!(%task_error, "stream task failed");
                }
            }
        }
    }
    while let Some(task_result) = stream_tasks.join_next().await {
        if let Err(task_error) = task_result {
            error!(%task_error, "stream task failed");
        }
    }
}

async fn handle_stream(
    mut send: quinn::SendStream,
    mut receive: quinn::RecvStream,
    state: Arc<ServerState>,
    authenticated: Arc<AtomicBool>,
    request_read_timeout: Duration,
    request_processing_timeout: Duration,
) {
    let read_result = timeout(request_read_timeout, read_request_frame(&mut receive)).await;
    let (request_id, frame) = match read_result {
        Ok(Ok((request_id, frame))) => (request_id, frame),
        Ok(Err((request_id, _error))) => {
            let _ = send_error(&mut send, request_id, Status::InvalidRequest).await;
            return;
        }
        Err(_) => {
            let _ = send_error(&mut send, 0, Status::InvalidRequest).await;
            return;
        }
    };
    let decoded = decode_request_frame(&frame);
    let request = match decoded {
        Ok(request) => request,
        Err(_) => {
            let _ = send_error(&mut send, request_id, Status::InvalidRequest).await;
            return;
        }
    };
    let request_id = request.request_id;
    let tenant = request.tenant;
    let is_ping = matches!(&request.operation, RequestOperation::Ping);
    let is_auth = matches!(&request.operation, RequestOperation::Auth { .. });
    if !is_ping && !is_auth && !authenticated.load(Ordering::Acquire) {
        let _ = send_error(&mut send, request_id, Status::Unauthorized).await;
        return;
    }
    if request.operation.opcode().is_tenant_scoped() && validate_tenant(&tenant).is_err() {
        let _ = send_error(&mut send, request_id, Status::InvalidRequest).await;
        return;
    }
    if let RequestOperation::Auth { worker_token } = &request.operation {
        if !constant_time_equal(worker_token, &state.worker_token) {
            let _ = send_error(&mut send, request_id, Status::Unauthorized).await;
            return;
        }
        authenticated.store(true, Ordering::Release);
    }
    let result = timeout(
        request_processing_timeout,
        execute_request(Arc::clone(&state), tenant, request.operation),
    )
    .await;
    let (status, payload) = match result {
        Ok(Ok(payload)) => (Status::Ok, payload),
        Ok(Err(error)) => operation_error_response(error),
        Err(_) => (Status::InternalError, generic_error_payload()),
    };
    if let Err(error) = send_response(&mut send, request_id, status, &payload).await {
        error!(%error, "failed to send Dibi response");
    }
}

async fn read_request_frame(
    receive: &mut quinn::RecvStream,
) -> Result<(u64, Vec<u8>), (u64, std::io::Error)> {
    let mut header = [0_u8; dibi_protocol::FRAME_HEADER_SIZE];
    receive.read_exact(&mut header).await.map_err(|error| {
        (
            0,
            std::io::Error::new(std::io::ErrorKind::UnexpectedEof, error.to_string()),
        )
    })?;
    let request_id = u64::from_be_bytes(header[8..16].try_into().unwrap_or_default());
    let payload_len = u32::from_be_bytes(header[16..20].try_into().unwrap_or_default()) as usize;
    let frame_len = dibi_protocol::FRAME_HEADER_SIZE
        .checked_add(payload_len)
        .ok_or_else(|| {
            (
                request_id,
                std::io::Error::new(std::io::ErrorKind::InvalidData, "frame length overflow"),
            )
        })?;
    if payload_len > MAX_PAYLOAD_SIZE || frame_len > dibi_protocol::MAX_FRAME_SIZE {
        return Err((
            request_id,
            std::io::Error::new(std::io::ErrorKind::InvalidData, "frame is too large"),
        ));
    }
    let mut frame = Vec::with_capacity(frame_len);
    frame.extend_from_slice(&header);
    let mut payload = vec![0_u8; payload_len];
    receive.read_exact(&mut payload).await.map_err(|error| {
        (
            request_id,
            std::io::Error::new(std::io::ErrorKind::UnexpectedEof, error.to_string()),
        )
    })?;
    frame.extend_from_slice(&payload);
    let mut trailing = [0_u8; 1];
    match receive.read(&mut trailing).await.map_err(|error| {
        (
            request_id,
            std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()),
        )
    })? {
        Some(0) | None => Ok((request_id, frame)),
        Some(_) => Err((
            request_id,
            std::io::Error::new(std::io::ErrorKind::InvalidData, "trailing stream bytes"),
        )),
    }
}

async fn send_error(
    send: &mut quinn::SendStream,
    request_id: u64,
    status: Status,
) -> Result<(), String> {
    send_response(
        send,
        request_id,
        status,
        &ResponsePayload::ErrorMessage("invalid request".to_owned()),
    )
    .await
}

async fn send_response(
    send: &mut quinn::SendStream,
    request_id: u64,
    status: Status,
    payload: &ResponsePayload,
) -> Result<(), String> {
    let frame =
        encode_response_frame(request_id, status, payload).map_err(|error| error.to_string())?;
    send.write_all(&frame)
        .await
        .map_err(|error| error.to_string())?;
    send.finish().map_err(|error| error.to_string())
}

async fn execute_request(
    state: Arc<ServerState>,
    tenant: String,
    operation: RequestOperation,
) -> Result<ResponsePayload, OperationError> {
    match operation {
        RequestOperation::Get { pk, sk } => {
            let engine = state.engine.clone();
            let document = run_engine(move || engine.get(&tenant, &pk, &sk)).await?;
            Ok(found_payload(document))
        }
        RequestOperation::Put { pk, sk, data } => {
            let engine = state.engine.clone();
            let commit_id =
                run_engine(move || engine.put(&tenant, &pk, &sk, Bytes::from(data))).await?;
            Ok(ResponsePayload::CommitId(commit_id))
        }
        RequestOperation::Delete { pk, sk } => {
            let engine = state.engine.clone();
            let commit_id = run_engine(move || engine.delete(&tenant, &pk, &sk)).await?;
            Ok(ResponsePayload::CommitId(commit_id))
        }
        RequestOperation::Query {
            pk,
            after_sk,
            limit,
        } => {
            let engine = state.engine.clone();
            let documents =
                run_engine(move || engine.query(&tenant, &pk, after_sk.as_deref(), limit as usize))
                    .await?;
            Ok(ResponsePayload::QueryItems(query_items(documents)))
        }
        RequestOperation::Scan { cursor, limit } => {
            let engine = state.engine.clone();
            let documents = run_engine(move || {
                engine.scan(
                    &tenant,
                    cursor
                        .as_ref()
                        .map(|key| (key.pk.as_str(), key.sk.as_str())),
                    limit as usize,
                )
            })
            .await?;
            Ok(ResponsePayload::ScanItems(scan_items(documents)))
        }
        RequestOperation::Batch(operations) => {
            let engine = state.engine.clone();
            let operations = application_writes(operations);
            let result =
                run_engine(move || engine.application_write_batch(&tenant, &operations)).await?;
            Ok(ResponsePayload::OptionalCommitId(result.commit_id))
        }
        RequestOperation::ExecuteOps(operations) => {
            let mut results = Vec::with_capacity(operations.len());
            for operation in operations {
                match operation {
                    ExecuteOperation::Get { pk, sk } => {
                        let engine = state.engine.clone();
                        let operation_tenant = tenant.clone();
                        let data =
                            run_engine(move || engine.get(&operation_tenant, &pk, &sk)).await?;
                        results.push(ExecuteResult::Single {
                            found: data.is_some(),
                            data: data.map(|value| value.to_vec()),
                        });
                    }
                    ExecuteOperation::Query {
                        pk,
                        after_sk,
                        limit,
                    } => {
                        let engine = state.engine.clone();
                        let operation_tenant = tenant.clone();
                        let documents = run_engine(move || {
                            engine.query(
                                &operation_tenant,
                                &pk,
                                after_sk.as_deref(),
                                limit as usize,
                            )
                        })
                        .await?;
                        results.push(ExecuteResult::Multiple(query_items(documents)));
                    }
                    ExecuteOperation::Put { pk, sk, data } => {
                        let engine = state.engine.clone();
                        let operation_tenant = tenant.clone();
                        run_engine(move || {
                            engine.put(&operation_tenant, &pk, &sk, Bytes::from(data))
                        })
                        .await?;
                        results.push(ExecuteResult::Done);
                    }
                    ExecuteOperation::Delete { pk, sk } => {
                        let engine = state.engine.clone();
                        let operation_tenant = tenant.clone();
                        run_engine(move || engine.delete(&operation_tenant, &pk, &sk)).await?;
                        results.push(ExecuteResult::Done);
                    }
                }
            }
            Ok(ResponsePayload::ExecuteResults(results))
        }
        RequestOperation::GetWithVersion { pk, sk } => {
            let engine = state.engine.clone();
            let document = run_engine(move || engine.get_with_version(&tenant, &pk, &sk)).await?;
            Ok(versioned_payload(document))
        }
        RequestOperation::BatchGetWithVersion(keys) => {
            let engine = state.engine.clone();
            let documents = run_engine(move || {
                keys.into_iter()
                    .map(|key| engine.get_with_version(&tenant, &key.pk, &key.sk))
                    .collect::<crate::Result<Vec<_>>>()
            })
            .await?;
            Ok(ResponsePayload::VersionedItems(versioned_items(documents)))
        }
        RequestOperation::TransactWriteItems(operations)
        | RequestOperation::AdminTransactWriteItems(operations) => {
            let engine = state.engine.clone();
            let operations = conditional_writes(operations);
            match run_engine(move || engine.conditional_write_batch(&tenant, &operations)).await? {
                ConditionalWriteOutcome::Applied(result) => {
                    Ok(ResponsePayload::OptionalCommitId(result.commit_id))
                }
                ConditionalWriteOutcome::Conflict(conflicts) => {
                    Err(OperationError::Conflict(protocol_conflicts(conflicts)))
                }
            }
        }
        RequestOperation::AdminScan {
            cursor,
            limit,
            pk_prefix,
        } => {
            let engine = state.engine.clone();
            let page = run_engine(move || {
                engine.admin_scan(
                    &tenant,
                    AdminScanRequest {
                        after: cursor.map(|key| (key.pk, key.sk)),
                        limit: limit as usize,
                        pk_prefix,
                    },
                )
            })
            .await?;
            let crate::AdminScanPage { documents, next } = page;
            let items = documents
                .into_iter()
                .map(|document| AdminItem {
                    pk: document.pk,
                    sk: document.sk,
                    version: document.version,
                    data: document.data.to_vec(),
                })
                .collect();
            Ok(ResponsePayload::AdminScan {
                items,
                next_cursor: next.map(|(pk, sk)| Key { pk, sk }),
            })
        }
        RequestOperation::Ping | RequestOperation::Auth { .. } => Ok(ResponsePayload::Empty),
        RequestOperation::Status => {
            let engine = state.engine.clone();
            let status = run_engine(move || {
                Ok::<_, DibiError>((engine.db_uuid()?, engine.last_commit_id()?))
            })
            .await?;
            Ok(ResponsePayload::Status {
                db_uuid: *status.0.as_bytes(),
                last_commit_id: status.1,
            })
        }
    }
}

async fn run_engine<T, Operation>(operation: Operation) -> Result<T, OperationError>
where
    T: Send + 'static,
    Operation: FnOnce() -> crate::Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| OperationError::Internal)?
        .map_err(OperationError::Database)
}

fn found_payload(document: Option<Bytes>) -> ResponsePayload {
    ResponsePayload::Found {
        found: document.is_some(),
        data: document.map(|value| value.to_vec()),
    }
}

fn versioned_payload(document: Option<StoredDocument>) -> ResponsePayload {
    ResponsePayload::Versioned {
        found: document.is_some(),
        version: document.as_ref().map(|value| value.version),
        data: document.map(|value| value.data.to_vec()),
    }
}

fn versioned_items(documents: Vec<Option<StoredDocument>>) -> Vec<VersionedItem> {
    documents
        .into_iter()
        .map(|document| VersionedItem {
            found: document.is_some(),
            version: document.as_ref().map(|value| value.version),
            data: document.map(|value| value.data.to_vec()),
        })
        .collect()
}

fn query_items(documents: Vec<Document>) -> Vec<QueryItem> {
    documents
        .into_iter()
        .map(|document| QueryItem {
            sk: document.sk,
            data: document.data.to_vec(),
        })
        .collect()
}

fn scan_items(documents: Vec<Document>) -> Vec<ScanItem> {
    documents
        .into_iter()
        .map(|document| ScanItem {
            pk: document.pk,
            sk: document.sk,
            data: document.data.to_vec(),
        })
        .collect()
}

fn application_writes(operations: Vec<WriteOperation>) -> Vec<ApplicationWrite> {
    operations
        .into_iter()
        .map(|operation| match operation {
            WriteOperation::Put { pk, sk, data } => ApplicationWrite::Put {
                pk,
                sk,
                data: Bytes::from(data),
            },
            WriteOperation::Delete { pk, sk } => ApplicationWrite::Delete { pk, sk },
        })
        .collect()
}

fn conditional_writes(operations: Vec<TransactWriteOperation>) -> Vec<ConditionalWrite> {
    operations
        .into_iter()
        .map(|operation| match operation {
            TransactWriteOperation::Create { pk, sk, data } => ConditionalWrite::Create {
                pk,
                sk,
                data: Bytes::from(data),
            },
            TransactWriteOperation::Put {
                pk,
                sk,
                expected_version,
                data,
            } => ConditionalWrite::Put {
                pk,
                sk,
                expected_version,
                data: Bytes::from(data),
            },
            TransactWriteOperation::Delete {
                pk,
                sk,
                expected_version,
            } => ConditionalWrite::Delete {
                pk,
                sk,
                expected_version,
            },
        })
        .collect()
}

fn protocol_conflicts(conflicts: Vec<crate::Conflict>) -> Vec<dibi_protocol::Conflict> {
    conflicts
        .into_iter()
        .map(|conflict| dibi_protocol::Conflict {
            pk: conflict.pk,
            sk: conflict.sk,
            expected_version: conflict.expected_version,
            actual_version: conflict.actual_version,
        })
        .collect()
}

fn operation_error_response(error: OperationError) -> (Status, ResponsePayload) {
    match error {
        OperationError::Conflict(conflicts) => (
            Status::Conflict,
            ResponsePayload::ConditionalConflicts(conflicts),
        ),
        OperationError::Database(error) => {
            if matches!(error, DibiError::DuplicateConditionalKey { .. }) {
                (Status::InvalidRequest, generic_error_payload())
            } else {
                error!(%error, "Dibi engine operation failed");
                (Status::InternalError, generic_error_payload())
            }
        }
        OperationError::Internal => {
            error!("Dibi operation task failed");
            (Status::InternalError, generic_error_payload())
        }
    }
}

fn generic_error_payload() -> ResponsePayload {
    ResponsePayload::ErrorMessage("internal server error".to_owned())
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    for byte_index in 0..left.len().max(right.len()) {
        let left_byte = left.get(byte_index).copied().unwrap_or(0);
        let right_byte = right.get(byte_index).copied().unwrap_or(0);
        difference |= usize::from(left_byte ^ right_byte);
    }
    difference == 0
}
