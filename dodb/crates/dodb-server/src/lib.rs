use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fmt;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
#[cfg(feature = "blink-demand-paging")]
use std::sync::RwLock as StdRwLock;
#[cfg(feature = "blink-demand-paging")]
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

mod physical_checkpoint;

pub const REQUEST_LATENCY_BUCKETS_SECONDS: [f64; 18] = [
    0.000025, 0.00005, 0.0001, 0.00025, 0.0005, 0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25,
    0.5, 1.0, 2.5, 5.0, 10.0,
];
pub const REQUEST_OPERATIONS: [(&str, usize); 6] = [
    ("get", 0),
    ("put", 1),
    ("delete", 2),
    ("query", 3),
    ("scan", 4),
    ("transact", 7),
];

const REQUEST_LATENCY_BUCKET_COUNT: usize = REQUEST_LATENCY_BUCKETS_SECONDS.len() + 1;
const TRANSPORT_STAGE_COUNT: usize = 5;
const TRANSPORT_REASON_COUNT: usize = 14;
const TRANSPORT_OUTCOME_COUNT: usize = 2;
const TRANSPORT_EVENT_COUNT: usize =
    TRANSPORT_STAGE_COUNT * TRANSPORT_REASON_COUNT * TRANSPORT_OUTCOME_COUNT;

use dodb_core::{
    Error, ObservedState, RevisionState, ShardId, TenantId, TransactionCondition,
    TransactionConflict, TransactionMutation, TransactionRequest, TransactionResult,
};
use dodb_protocol::{
    ApplicationError, ProtocolError, ProtocolLimits, ResponseEnvelope, decode_header,
    decode_request_parts, encode_response,
};
use dodb_service::{
    Document, DodbService, ExecutionBudget, Request, Response, ServiceFuture, ShutdownFuture,
    TransactionOutcome,
};
use dodb_storage::{
    AsyncShard, BTreeStore, BatchRequest, BatchResponse, CheckpointReport, CoordinatorConfig,
    DatabaseConfig, FaultInjector, InvariantReport, ProductionFile,
};
use quinn::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use quinn::{
    Connection, ConnectionError, Endpoint, Incoming, ReadError, ReadExactError,
    ServerConfig as QuinnServerConfig, VarInt, WriteError,
};
use tokio::sync::{Mutex, Notify, OwnedSemaphorePermit, Semaphore};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TransportStage {
    Handshake,
    AcceptStream,
    ReadHeader,
    WriteResponse,
    FinishResponse,
}

impl TransportStage {
    const ALL: [Self; TRANSPORT_STAGE_COUNT] = [
        Self::Handshake,
        Self::AcceptStream,
        Self::ReadHeader,
        Self::WriteResponse,
        Self::FinishResponse,
    ];

    const fn index(self) -> usize {
        match self {
            Self::Handshake => 0,
            Self::AcceptStream => 1,
            Self::ReadHeader => 2,
            Self::WriteResponse => 3,
            Self::FinishResponse => 4,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Handshake => "handshake",
            Self::AcceptStream => "accept_stream",
            Self::ReadHeader => "read_header",
            Self::WriteResponse => "write_response",
            Self::FinishResponse => "finish_response",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TransportReason {
    ApplicationClosed,
    LocallyClosed,
    TimedOut,
    Reset,
    ConnectionClosed,
    TransportError,
    VersionMismatch,
    CidsExhausted,
    PeerReset,
    PeerStopped,
    PeerFinishedEarly,
    ClosedStream,
    ZeroRttRejected,
    IllegalOrderedRead,
}

impl TransportReason {
    const ALL: [Self; TRANSPORT_REASON_COUNT] = [
        Self::ApplicationClosed,
        Self::LocallyClosed,
        Self::TimedOut,
        Self::Reset,
        Self::ConnectionClosed,
        Self::TransportError,
        Self::VersionMismatch,
        Self::CidsExhausted,
        Self::PeerReset,
        Self::PeerStopped,
        Self::PeerFinishedEarly,
        Self::ClosedStream,
        Self::ZeroRttRejected,
        Self::IllegalOrderedRead,
    ];

    const fn index(self) -> usize {
        match self {
            Self::ApplicationClosed => 0,
            Self::LocallyClosed => 1,
            Self::TimedOut => 2,
            Self::Reset => 3,
            Self::ConnectionClosed => 4,
            Self::TransportError => 5,
            Self::VersionMismatch => 6,
            Self::CidsExhausted => 7,
            Self::PeerReset => 8,
            Self::PeerStopped => 9,
            Self::PeerFinishedEarly => 10,
            Self::ClosedStream => 11,
            Self::ZeroRttRejected => 12,
            Self::IllegalOrderedRead => 13,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::ApplicationClosed => "application_closed",
            Self::LocallyClosed => "locally_closed",
            Self::TimedOut => "timed_out",
            Self::Reset => "reset",
            Self::ConnectionClosed => "connection_closed",
            Self::TransportError => "transport_error",
            Self::VersionMismatch => "version_mismatch",
            Self::CidsExhausted => "cids_exhausted",
            Self::PeerReset => "peer_reset",
            Self::PeerStopped => "peer_stopped",
            Self::PeerFinishedEarly => "peer_finished_early",
            Self::ClosedStream => "closed_stream",
            Self::ZeroRttRejected => "zero_rtt_rejected",
            Self::IllegalOrderedRead => "illegal_ordered_read",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TransportOutcome {
    Benign,
    Error,
}

impl TransportOutcome {
    const fn index(self) -> usize {
        match self {
            Self::Benign => 0,
            Self::Error => 1,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Benign => "benign",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TransportClassification {
    reason: TransportReason,
    outcome: TransportOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConnectionFailureKind {
    ApplicationClosed,
    LocallyClosed,
    TimedOut,
    Reset,
    ConnectionClosed,
    TransportError,
    VersionMismatch,
    CidsExhausted,
}

#[derive(Debug)]
pub enum ServerError {
    Io(std::io::Error),
    Tls(String),
    Endpoint(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Tls(error) => write!(formatter, "TLS configuration error: {error}"),
            Self::Endpoint(error) => write!(formatter, "QUIC endpoint error: {error}"),
        }
    }
}

impl std::error::Error for ServerError {}

impl From<std::io::Error> for ServerError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerTlsConfig {
    certificate_chain: Vec<Vec<u8>>,
    private_key: Vec<u8>,
}

impl ServerTlsConfig {
    pub fn from_der(
        certificate_chain: Vec<Vec<u8>>,
        private_key: Vec<u8>,
    ) -> Result<Self, ServerError> {
        if certificate_chain.is_empty() || private_key.is_empty() {
            return Err(ServerError::Tls(
                "certificate chain and private key are required".to_owned(),
            ));
        }
        Ok(Self {
            certificate_chain,
            private_key,
        })
    }

    pub fn from_pem(certificate_pem: &[u8], private_key_pem: &[u8]) -> Result<Self, ServerError> {
        let mut certificate_reader = Cursor::new(certificate_pem);
        let certificates = rustls_pemfile::certs(&mut certificate_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| ServerError::Tls(error.to_string()))?
            .into_iter()
            .map(|certificate| certificate.to_vec())
            .collect::<Vec<_>>();
        let mut key_reader = Cursor::new(private_key_pem);
        let private_key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|error| ServerError::Tls(error.to_string()))?
            .ok_or_else(|| ServerError::Tls("private key PEM section is missing".to_owned()))?
            .secret_der()
            .to_vec();
        Self::from_der(certificates, private_key)
    }

    fn to_quinn_config(
        &self,
        max_concurrent_streams: usize,
    ) -> Result<QuinnServerConfig, ServerError> {
        let certificates = self
            .certificate_chain
            .iter()
            .cloned()
            .map(CertificateDer::from)
            .collect::<Vec<_>>();
        let private_key = PrivateKeyDer::try_from(self.private_key.clone())
            .map_err(|error| ServerError::Tls(error.to_string()))?;
        let mut config = QuinnServerConfig::with_single_cert(certificates, private_key)
            .map_err(|error| ServerError::Tls(error.to_string()))?;
        let mut transport = quinn::TransportConfig::default();
        transport.max_concurrent_bidi_streams(
            VarInt::try_from(max_concurrent_streams)
                .map_err(|error| ServerError::Tls(error.to_string()))?,
        );
        config.transport_config(Arc::new(transport));
        Ok(config)
    }
}

#[derive(Clone, Debug)]
pub struct DodbServerConfig {
    pub listen_addr: std::net::SocketAddr,
    pub tls: ServerTlsConfig,
    pub protocol_limits: ProtocolLimits,
    pub max_connections: usize,
    /// Maximum bidirectional request streams advertised per QUIC connection.
    pub max_concurrent_streams: usize,
    /// Global active service requests across all connections.
    pub max_concurrent_requests: usize,
}

impl DodbServerConfig {
    pub fn validate(&self) -> Result<(), ServerError> {
        self.protocol_limits
            .validate()
            .map_err(|error| ServerError::Endpoint(error.to_string()))?;
        if self.max_connections == 0
            || self.max_concurrent_streams == 0
            || self.max_concurrent_requests == 0
        {
            return Err(ServerError::Endpoint(
                "connection, stream, and request limits must be nonzero".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct ServerMetrics {
    inner: Arc<ServerMetricsInner>,
}

#[derive(Debug)]
struct ServerMetricsInner {
    connections_total: AtomicU64,
    active_connections: AtomicU64,
    active_streams: AtomicU64,
    requests_total: AtomicU64,
    request_bytes: AtomicU64,
    response_bytes: AtomicU64,
    protocol_errors: AtomicU64,
    transport_errors: AtomicU64,
    transport_events: [AtomicU64; TRANSPORT_EVENT_COUNT],
    application_errors: AtomicU64,
    overloaded_responses: AtomicU64,
    request_latency_nanos: AtomicU64,
    operations: [AtomicU64; 8],
    request_latency_buckets: [[AtomicU64; REQUEST_LATENCY_BUCKET_COUNT]; 8],
}

impl Default for ServerMetricsInner {
    fn default() -> Self {
        Self {
            connections_total: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            active_streams: AtomicU64::new(0),
            requests_total: AtomicU64::new(0),
            request_bytes: AtomicU64::new(0),
            response_bytes: AtomicU64::new(0),
            protocol_errors: AtomicU64::new(0),
            transport_errors: AtomicU64::new(0),
            transport_events: std::array::from_fn(|_| AtomicU64::new(0)),
            application_errors: AtomicU64::new(0),
            overloaded_responses: AtomicU64::new(0),
            request_latency_nanos: AtomicU64::new(0),
            operations: std::array::from_fn(|_| AtomicU64::new(0)),
            request_latency_buckets: std::array::from_fn(|_| {
                std::array::from_fn(|_| AtomicU64::new(0))
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerMetricsSnapshot {
    pub connections_total: u64,
    pub active_connections: u64,
    pub active_streams: u64,
    pub requests_total: u64,
    pub request_bytes: u64,
    pub response_bytes: u64,
    pub protocol_errors: u64,
    pub transport_errors: u64,
    pub transport_events: Vec<TransportEventSnapshot>,
    pub application_errors: u64,
    pub overloaded_responses: u64,
    pub request_latency_nanos: u64,
    pub operations: [u64; 8],
    pub request_latency_buckets: [[u64; REQUEST_LATENCY_BUCKET_COUNT]; 8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportEventSnapshot {
    pub stage: &'static str,
    pub reason: &'static str,
    pub outcome: &'static str,
    pub value: u64,
}

impl ServerMetrics {
    pub fn snapshot(&self) -> ServerMetricsSnapshot {
        let inner = &self.inner;
        ServerMetricsSnapshot {
            connections_total: inner.connections_total.load(Ordering::Relaxed),
            active_connections: inner.active_connections.load(Ordering::Relaxed),
            active_streams: inner.active_streams.load(Ordering::Relaxed),
            requests_total: inner.requests_total.load(Ordering::Relaxed),
            request_bytes: inner.request_bytes.load(Ordering::Relaxed),
            response_bytes: inner.response_bytes.load(Ordering::Relaxed),
            protocol_errors: inner.protocol_errors.load(Ordering::Relaxed),
            transport_errors: inner.transport_errors.load(Ordering::Relaxed),
            transport_events: snapshot_transport_events(inner),
            application_errors: inner.application_errors.load(Ordering::Relaxed),
            overloaded_responses: inner.overloaded_responses.load(Ordering::Relaxed),
            request_latency_nanos: inner.request_latency_nanos.load(Ordering::Relaxed),
            operations: std::array::from_fn(|operation_index| {
                inner.operations[operation_index].load(Ordering::Relaxed)
            }),
            request_latency_buckets: std::array::from_fn(|operation_index| {
                std::array::from_fn(|bucket_index| {
                    inner.request_latency_buckets[operation_index][bucket_index]
                        .load(Ordering::Relaxed)
                })
            }),
        }
    }

    fn record_request_latency(&self, operation_index: usize, elapsed_nanos: u64) {
        let Some(operation_buckets) = self.inner.request_latency_buckets.get(operation_index)
        else {
            return;
        };
        let elapsed_seconds = elapsed_nanos as f64 / 1_000_000_000.0;
        for (bucket_index, upper_bound) in REQUEST_LATENCY_BUCKETS_SECONDS.iter().enumerate() {
            if elapsed_seconds <= *upper_bound {
                operation_buckets[bucket_index].fetch_add(1, Ordering::Relaxed);
            }
        }
        operation_buckets[REQUEST_LATENCY_BUCKETS_SECONDS.len()].fetch_add(1, Ordering::Relaxed);
    }

    fn record_transport_event(
        &self,
        stage: TransportStage,
        classification: TransportClassification,
        error_detail: &str,
    ) {
        let TransportClassification { reason, outcome } = classification;
        self.inner.transport_events[transport_event_index(stage, reason, outcome)]
            .fetch_add(1, Ordering::Relaxed);
        if outcome == TransportOutcome::Error {
            self.inner.transport_errors.fetch_add(1, Ordering::Relaxed);
            eprintln!(
                "WARN dodb QUIC transport failure stage={} reason={} error={error_detail}",
                stage.as_str(),
                reason.as_str()
            );
        }
    }
}

fn snapshot_transport_events(inner: &ServerMetricsInner) -> Vec<TransportEventSnapshot> {
    let mut events = Vec::new();
    for stage in TransportStage::ALL {
        for reason in TransportReason::ALL {
            for outcome in [TransportOutcome::Benign, TransportOutcome::Error] {
                let value = inner.transport_events[transport_event_index(stage, reason, outcome)]
                    .load(Ordering::Relaxed);
                if value > 0 {
                    events.push(TransportEventSnapshot {
                        stage: stage.as_str(),
                        reason: reason.as_str(),
                        outcome: outcome.as_str(),
                        value,
                    });
                }
            }
        }
    }
    events
}

fn transport_event_index(
    stage: TransportStage,
    reason: TransportReason,
    outcome: TransportOutcome,
) -> usize {
    (stage.index() * TRANSPORT_REASON_COUNT + reason.index()) * TRANSPORT_OUTCOME_COUNT
        + outcome.index()
}

// Quinn reports peer closes and stream cancellation as errors, so classify peer-driven shutdowns as diagnostic events.
fn classify_connection_error(
    stage: TransportStage,
    error: &ConnectionError,
) -> TransportClassification {
    let kind = match error {
        ConnectionError::ApplicationClosed(_) => ConnectionFailureKind::ApplicationClosed,
        ConnectionError::LocallyClosed => ConnectionFailureKind::LocallyClosed,
        ConnectionError::TimedOut => ConnectionFailureKind::TimedOut,
        ConnectionError::Reset => ConnectionFailureKind::Reset,
        ConnectionError::ConnectionClosed(_) => ConnectionFailureKind::ConnectionClosed,
        ConnectionError::TransportError(_) => ConnectionFailureKind::TransportError,
        ConnectionError::VersionMismatch => ConnectionFailureKind::VersionMismatch,
        ConnectionError::CidsExhausted => ConnectionFailureKind::CidsExhausted,
    };
    classify_connection_failure(stage, kind)
}

fn classify_connection_failure(
    stage: TransportStage,
    kind: ConnectionFailureKind,
) -> TransportClassification {
    let (reason, outcome) = match kind {
        ConnectionFailureKind::ApplicationClosed => {
            (TransportReason::ApplicationClosed, TransportOutcome::Benign)
        }
        ConnectionFailureKind::LocallyClosed => {
            (TransportReason::LocallyClosed, TransportOutcome::Benign)
        }
        ConnectionFailureKind::TimedOut
            if matches!(
                stage,
                TransportStage::Handshake | TransportStage::AcceptStream
            ) =>
        {
            (TransportReason::TimedOut, TransportOutcome::Benign)
        }
        ConnectionFailureKind::TimedOut => (TransportReason::TimedOut, TransportOutcome::Error),
        ConnectionFailureKind::Reset => (TransportReason::Reset, TransportOutcome::Benign),
        ConnectionFailureKind::ConnectionClosed => {
            (TransportReason::ConnectionClosed, TransportOutcome::Error)
        }
        ConnectionFailureKind::TransportError => {
            (TransportReason::TransportError, TransportOutcome::Error)
        }
        ConnectionFailureKind::VersionMismatch => {
            (TransportReason::VersionMismatch, TransportOutcome::Error)
        }
        ConnectionFailureKind::CidsExhausted => {
            (TransportReason::CidsExhausted, TransportOutcome::Error)
        }
    };
    TransportClassification { reason, outcome }
}

fn classify_read_error(stage: TransportStage, error: &ReadExactError) -> TransportClassification {
    match error {
        ReadExactError::FinishedEarly(_) => TransportClassification {
            reason: TransportReason::PeerFinishedEarly,
            outcome: TransportOutcome::Benign,
        },
        ReadExactError::ReadError(ReadError::Reset(_)) => TransportClassification {
            reason: TransportReason::PeerReset,
            outcome: TransportOutcome::Benign,
        },
        ReadExactError::ReadError(ReadError::ConnectionLost(error)) => {
            classify_connection_error(stage, error)
        }
        ReadExactError::ReadError(ReadError::ClosedStream) => TransportClassification {
            reason: TransportReason::ClosedStream,
            outcome: TransportOutcome::Error,
        },
        ReadExactError::ReadError(ReadError::IllegalOrderedRead) => TransportClassification {
            reason: TransportReason::IllegalOrderedRead,
            outcome: TransportOutcome::Error,
        },
        ReadExactError::ReadError(ReadError::ZeroRttRejected) => TransportClassification {
            reason: TransportReason::ZeroRttRejected,
            outcome: TransportOutcome::Error,
        },
    }
}

fn classify_write_error(stage: TransportStage, error: &WriteError) -> TransportClassification {
    match error {
        WriteError::Stopped(_) => TransportClassification {
            reason: TransportReason::PeerStopped,
            outcome: TransportOutcome::Benign,
        },
        WriteError::ConnectionLost(error) => classify_connection_error(stage, error),
        WriteError::ClosedStream => TransportClassification {
            reason: TransportReason::ClosedStream,
            outcome: TransportOutcome::Error,
        },
        WriteError::ZeroRttRejected => TransportClassification {
            reason: TransportReason::ZeroRttRejected,
            outcome: TransportOutcome::Error,
        },
    }
}

pub struct DodbServer<S> {
    endpoint: Endpoint,
    service: Arc<S>,
    protocol_limits: ProtocolLimits,
    connection_slots: Arc<Semaphore>,
    request_slots: Arc<Semaphore>,
    metrics: ServerMetrics,
}

impl<S: DodbService + 'static> DodbServer<S> {
    pub fn bind(service: Arc<S>, config: DodbServerConfig) -> Result<Self, ServerError> {
        config.validate()?;
        let quinn_config = config.tls.to_quinn_config(config.max_concurrent_streams)?;
        let endpoint = Endpoint::server(quinn_config, config.listen_addr)?;
        Ok(Self {
            endpoint,
            service,
            protocol_limits: config.protocol_limits,
            connection_slots: Arc::new(Semaphore::new(config.max_connections)),
            request_slots: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            metrics: ServerMetrics::default(),
        })
    }

    pub fn local_addr(&self) -> Result<std::net::SocketAddr, ServerError> {
        self.endpoint.local_addr().map_err(ServerError::Io)
    }

    pub fn metrics(&self) -> ServerMetrics {
        self.metrics.clone()
    }

    pub fn close(&self) {
        self.endpoint.close(VarInt::from_u32(0), b"server shutdown");
    }

    /// Closes the endpoint, waits for established connections to drain, and
    /// then awaits service-owned coordinator shutdown.
    pub async fn shutdown(&self) {
        self.close();
        self.endpoint.wait_idle().await;
        self.service.shutdown().await;
    }

    pub async fn run(&self) -> Result<(), ServerError> {
        while let Some(incoming) = self.endpoint.accept().await {
            let Ok(connection_permit) = self.connection_slots.clone().try_acquire_owned() else {
                incoming.refuse();
                self.metrics
                    .inner
                    .overloaded_responses
                    .fetch_add(1, Ordering::Relaxed);
                continue;
            };
            let service = Arc::clone(&self.service);
            let request_slots = Arc::clone(&self.request_slots);
            let limits = self.protocol_limits;
            let metrics = self.metrics.clone();
            tokio::spawn(async move {
                serve_connection(
                    incoming,
                    service,
                    request_slots,
                    limits,
                    metrics,
                    connection_permit,
                )
                .await;
            });
        }
        Ok(())
    }
}

async fn serve_connection<S: DodbService + 'static>(
    incoming: Incoming,
    service: Arc<S>,
    request_slots: Arc<Semaphore>,
    limits: ProtocolLimits,
    metrics: ServerMetrics,
    _connection_permit: OwnedSemaphorePermit,
) {
    let connection = match incoming.await {
        Ok(connection) => connection,
        Err(error) => {
            let classification = classify_connection_error(TransportStage::Handshake, &error);
            metrics.record_transport_event(
                TransportStage::Handshake,
                classification,
                &error.to_string(),
            );
            return;
        }
    };
    metrics
        .inner
        .connections_total
        .fetch_add(1, Ordering::Relaxed);
    metrics
        .inner
        .active_connections
        .fetch_add(1, Ordering::Relaxed);
    accept_streams(connection, service, request_slots, limits, metrics.clone()).await;
    metrics
        .inner
        .active_connections
        .fetch_sub(1, Ordering::Relaxed);
}

async fn accept_streams<S: DodbService + 'static>(
    connection: Connection,
    service: Arc<S>,
    request_slots: Arc<Semaphore>,
    limits: ProtocolLimits,
    metrics: ServerMetrics,
) {
    loop {
        let stream = match tokio::select! {
        stream = connection.accept_bi() => stream,
            _ = connection.closed() => {
                if let Some(error) = connection.close_reason() {
                    let classification =
                        classify_connection_error(TransportStage::AcceptStream, &error);
                    metrics.record_transport_event(
                        TransportStage::AcceptStream,
                        classification,
                        &error.to_string(),
                    );
                }
                return;
            },
        } {
            Ok(stream) => stream,
            Err(error) => {
                let classification =
                    classify_connection_error(TransportStage::AcceptStream, &error);
                metrics.record_transport_event(
                    TransportStage::AcceptStream,
                    classification,
                    &error.to_string(),
                );
                return;
            }
        };
        let stream_permit = match tokio::select! {
            permit = request_slots.clone().acquire_owned() => permit,
            _ = connection.closed() => {
                if let Some(error) = connection.close_reason() {
                    let classification =
                        classify_connection_error(TransportStage::AcceptStream, &error);
                    metrics.record_transport_event(
                        TransportStage::AcceptStream,
                        classification,
                        &error.to_string(),
                    );
                }
                return;
            },
        } {
            Ok(permit) => permit,
            Err(_) => return,
        };
        let service = Arc::clone(&service);
        let metrics = metrics.clone();
        tokio::spawn(async move {
            metrics.inner.active_streams.fetch_add(1, Ordering::Relaxed);
            serve_stream(stream, service, limits, metrics.clone()).await;
            metrics.inner.active_streams.fetch_sub(1, Ordering::Relaxed);
            drop(stream_permit);
        });
    }
}

async fn serve_stream<S: DodbService + 'static>(
    (mut send, mut receive): (quinn::SendStream, quinn::RecvStream),
    service: Arc<S>,
    limits: ProtocolLimits,
    metrics: ServerMetrics,
) {
    let mut header_bytes = [0u8; dodb_protocol::HEADER_SIZE];
    if let Err(error) = receive.read_exact(&mut header_bytes).await {
        let classification = classify_read_error(TransportStage::ReadHeader, &error);
        metrics.record_transport_event(
            TransportStage::ReadHeader,
            classification,
            &error.to_string(),
        );
        return;
    }
    let header = match decode_header(&header_bytes) {
        Ok(header) => header,
        Err(error) => {
            metrics
                .inner
                .protocol_errors
                .fetch_add(1, Ordering::Relaxed);
            let _ = send_protocol_error(&mut send, error, limits).await;
            return;
        }
    };
    let frame_length = dodb_protocol::HEADER_SIZE.saturating_add(header.payload_length);
    if frame_length > limits.max_request_frame_size {
        metrics
            .inner
            .protocol_errors
            .fetch_add(1, Ordering::Relaxed);
        let _ = send_protocol_error(
            &mut send,
            ProtocolError::PayloadTooLarge {
                length: frame_length,
                maximum: limits.max_request_frame_size,
            },
            limits,
        )
        .await;
        return;
    }
    let mut payload = vec![0u8; header.payload_length];
    if receive.read_exact(&mut payload).await.is_err() {
        metrics
            .inner
            .protocol_errors
            .fetch_add(1, Ordering::Relaxed);
        return;
    }
    let trailing = receive.read_to_end(1).await;
    if trailing.as_ref().is_err() || trailing.as_ref().is_ok_and(|bytes| !bytes.is_empty()) {
        metrics
            .inner
            .protocol_errors
            .fetch_add(1, Ordering::Relaxed);
        return;
    }
    let (tenant, request) = match decode_request_parts(header, &payload, limits) {
        Ok(request) => request,
        Err(error) => {
            metrics
                .inner
                .protocol_errors
                .fetch_add(1, Ordering::Relaxed);
            let _ = send_protocol_error(&mut send, error, limits).await;
            return;
        }
    };
    let operation_index = usize::from(dodb_protocol::request_opcode(&request).saturating_sub(1));
    let is_mutation = request.is_mutation();
    metrics.inner.requests_total.fetch_add(1, Ordering::Relaxed);
    metrics.inner.request_bytes.fetch_add(
        (dodb_protocol::HEADER_SIZE + payload.len()) as u64,
        Ordering::Relaxed,
    );
    if let Some(counter) = metrics.inner.operations.get(operation_index) {
        counter.fetch_add(1, Ordering::Relaxed);
    }
    let started = Instant::now();
    let envelope = match service
        .execute(
            tenant,
            request,
            ExecutionBudget::new(limits.max_response_frame_size),
        )
        .await
    {
        Ok(response) => ResponseEnvelope::Success(response),
        Err(error) => {
            let application_error = ApplicationError::from_core_for_request(&error, is_mutation);
            if application_error.kind == dodb_protocol::ApplicationErrorKind::Overloaded {
                metrics
                    .inner
                    .overloaded_responses
                    .fetch_add(1, Ordering::Relaxed);
            }
            ResponseEnvelope::Error(application_error)
        }
    };
    if matches!(envelope, ResponseEnvelope::Error(_)) {
        metrics
            .inner
            .application_errors
            .fetch_add(1, Ordering::Relaxed);
    }
    let encoded = match encode_response(&envelope, limits) {
        Ok(encoded) => encoded,
        Err(error) => {
            metrics
                .inner
                .protocol_errors
                .fetch_add(1, Ordering::Relaxed);
            let _ = send_protocol_error(&mut send, error, limits).await;
            return;
        }
    };
    if let Err(error) = send.write_all(&encoded).await {
        let classification = classify_write_error(TransportStage::WriteResponse, &error);
        metrics.record_transport_event(
            TransportStage::WriteResponse,
            classification,
            &error.to_string(),
        );
        return;
    }
    if let Err(error) = send.finish() {
        metrics.record_transport_event(
            TransportStage::FinishResponse,
            TransportClassification {
                reason: TransportReason::ClosedStream,
                outcome: TransportOutcome::Error,
            },
            &error.to_string(),
        );
        return;
    }
    metrics
        .inner
        .response_bytes
        .fetch_add(encoded.len() as u64, Ordering::Relaxed);
    let elapsed_nanos = started.elapsed().as_nanos() as u64;
    metrics
        .inner
        .request_latency_nanos
        .fetch_add(elapsed_nanos, Ordering::Relaxed);
    metrics.record_request_latency(operation_index, elapsed_nanos);
}

async fn send_protocol_error(
    send: &mut quinn::SendStream,
    error: ProtocolError,
    limits: ProtocolLimits,
) -> Result<(), quinn::WriteError> {
    let response = encode_response(
        &ResponseEnvelope::Error(ApplicationError::from_protocol(&error)),
        limits,
    )
    .map_err(|_| quinn::WriteError::Stopped(VarInt::from_u32(1)))?;
    send.write_all(&response).await?;
    send.finish()
        .map_err(|_| quinn::WriteError::Stopped(VarInt::from_u32(1)))
}

#[derive(Clone, Debug)]
pub struct LocalTenantServiceConfig {
    pub data_dir: PathBuf,
    pub max_open_shards: usize,
    pub blink_open_worker_count: usize,
    pub blink_open_waiter_limit: usize,
    pub blink_open_shutdown_timeout: std::time::Duration,
    pub database_config: DatabaseConfig,
    pub coordinator_config: CoordinatorConfig,
    pub blink_checkpoint_policy: BlinkCheckpointPolicy,
    pub blink_wal_layout: BlinkWalLayout,
    pub blink_checkpoint_execution: BlinkCheckpointExecution,
    pub blink_checkpoint_retention_bytes: u64,
    #[cfg(feature = "blink-demand-paging")]
    pub blink_shared_resources: Option<BlinkSharedResourceConfig>,
}

#[cfg(feature = "blink-demand-paging")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlinkSharedResourceConfig {
    pub memory: dodb_storage::blink::resource::ResourceLimits,
    pub disk: dodb_storage::blink::resource::DiskResourceLimits,
    pub open_tenant_metadata_bytes: u64,
    pub paged_read_only: bool,
    pub paged_writer_enabled: bool,
    pub maximum_working_pages: usize,
    pub page_loader: dodb_storage::blink::page_cache::PageLoaderConfig,
    pub maximum_cached_index_nodes: usize,
    pub maximum_open_source_handles: usize,
    pub maximum_roots_per_tenant: usize,
    pub source_handle_charge_bytes: u64,
    pub source_gc_reservation_bytes: u64,
    pub checkpoint_capacity_wait_timeout: std::time::Duration,
    pub maximum_checkpoint_debt_bytes_global: u64,
    pub maximum_checkpoint_debt_bytes_per_tenant: u64,
}

#[cfg(feature = "blink-demand-paging")]
fn valid_paged_open_config(config: BlinkSharedResourceConfig) -> bool {
    let Some(metadata_bytes) = u64::try_from(config.maximum_cached_index_nodes)
        .ok()
        .and_then(|nodes| nodes.checked_mul(dodb_storage::PAGE_SIZE as u64))
        .and_then(|bytes| {
            u64::try_from(config.maximum_open_source_handles)
                .ok()
                .and_then(|handles| handles.checked_mul(config.source_handle_charge_bytes))
                .and_then(|handle_bytes| bytes.checked_add(handle_bytes))
        })
        .and_then(|bytes| {
            u64::try_from(config.maximum_roots_per_tenant)
                .ok()
                .and_then(|roots| {
                    roots.checked_mul(std::mem::size_of::<
                        dodb_storage::blink::version_index::VersionIndexRoot,
                    >() as u64)
                })
                .and_then(|root_bytes| bytes.checked_add(root_bytes))
        })
        .and_then(|bytes| bytes.checked_add(config.open_tenant_metadata_bytes))
    else {
        return false;
    };
    let memory_category_limit = |category| {
        config.memory.global_category_bytes[category as usize]
            .min(config.memory.per_tenant_category_bytes[category as usize])
    };
    let disk_category_limit = |category| {
        config.disk.global_category_bytes[category as usize]
            .min(config.disk.per_tenant_category_bytes[category as usize])
    };
    let Some(working_bytes) = u64::try_from(config.maximum_working_pages)
        .ok()
        .and_then(|pages| pages.checked_mul(dodb_storage::PAGE_SIZE as u64))
    else {
        return false;
    };
    config.page_loader.validate().is_ok()
        && config.maximum_working_pages > 0
        && config.maximum_cached_index_nodes > 0
        && config.maximum_open_source_handles > 0
        && config.maximum_roots_per_tenant > 0
        && config.source_handle_charge_bytes > 0
        && config.source_gc_reservation_bytes > 0
        && !config.checkpoint_capacity_wait_timeout.is_zero()
        && config.maximum_checkpoint_debt_bytes_global > 0
        && config.maximum_checkpoint_debt_bytes_per_tenant > 0
        && config.maximum_checkpoint_debt_bytes_per_tenant
            <= config.maximum_checkpoint_debt_bytes_global
        && config.maximum_checkpoint_debt_bytes_global <= config.disk.global_bytes
        && config.maximum_checkpoint_debt_bytes_per_tenant <= config.disk.per_tenant_bytes
        && dodb_storage::blink::maximum_paged_transaction_wal_append_bytes(
            config.maximum_working_pages,
        )
        .is_ok_and(|maximum_append_bytes| {
            maximum_append_bytes <= config.maximum_checkpoint_debt_bytes_per_tenant
        })
        && metadata_bytes
            <= memory_category_limit(dodb_storage::blink::resource::ResourceCategory::Metadata)
        && config.page_loader.maximum_cache_bytes
            <= memory_category_limit(
                dodb_storage::blink::resource::ResourceCategory::DecodedPayload,
            )
        && config.page_loader.maximum_load_reconstruction_bytes
            <= memory_category_limit(
                dodb_storage::blink::resource::ResourceCategory::LoadReconstruction,
            )
        && config.source_gc_reservation_bytes
            <= disk_category_limit(dodb_storage::blink::resource::DiskCategory::Scratch)
        && working_bytes
            <= memory_category_limit(
                dodb_storage::blink::resource::ResourceCategory::WritePreparation,
            )
        && working_bytes
            <= memory_category_limit(dodb_storage::blink::resource::ResourceCategory::DirtyImages)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlinkCheckpointExecution {
    #[default]
    Sync,
    Background,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlinkWalLayout {
    #[default]
    Legacy,
    Segmented {
        max_segment_bytes: u64,
    },
    SegmentedBases {
        max_segment_bytes: u64,
        coalesce: bool,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlinkCheckpointPolicy {
    #[default]
    Disabled,
    WalBytesThreshold {
        threshold_bytes: u64,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlinkCheckpointEvent {
    pub phase: String,
    pub task_id: u64,
    pub target_lsn: u64,
    pub writes_during_checkpoint: u64,
    pub preparation_nanos: u64,
    pub page_write_nanos: u64,
    pub data_sync_nanos: u64,
    pub metadata_sync_nanos: u64,
    pub completion_nanos: u64,
    pub started_unix_nanos: u128,
    pub ended_unix_nanos: u128,
    pub wal_bytes_before: u64,
    pub wal_bytes_after: u64,
    pub reclaimed_bytes: u64,
    pub duration_nanos: u64,
    pub succeeded: bool,
    pub checkpoint_bytes_written: u64,
    pub base_bytes_written: u64,
    pub base_bytes_retained: u64,
    pub base_bytes_reclaimed: u64,
    pub full_base_snapshots: u64,
    pub checkpoint_write_requests: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlinkTenantCheckpointMetrics {
    pub wal_bytes_current: u64,
    pub checkpoint_debt_bytes_current: u64,
    pub checkpoint_debt_bytes_max: u64,
    pub threshold_excess_total_nanos: u64,
    pub threshold_excess_since_unix_nanos: Option<u128>,
    pub checkpoint_requests_queued: u64,
    pub checkpoint_requests_pending: u64,
    pub checkpoint_requests_cancelled: u64,
    pub checkpoint_request_wait_nanos_total: u64,
    pub checkpoint_request_wait_nanos_max: u64,
    pub service_slices: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BlinkWalCheckpointMetrics {
    pub policy: BlinkCheckpointPolicy,
    pub threshold_bytes: Option<u64>,
    pub checkpoints_started: u64,
    pub checkpoints_completed: u64,
    pub checkpoints_failed: u64,
    pub checkpoint_total_nanos: u64,
    pub checkpoint_max_nanos: u64,
    pub wal_bytes_current: u64,
    pub wal_bytes_at_measurement_start: Option<u64>,
    pub wal_bytes_at_measurement_end: Option<u64>,
    pub wal_bytes_max: u64,
    pub checkpoint_debt_bytes_current: u64,
    pub checkpoint_debt_bytes_max: u64,
    pub threshold_excess_max_bytes: u64,
    pub threshold_excess_total_nanos: u64,
    pub threshold_excess_current_nanos: u64,
    pub threshold_excess_since_unix_nanos: Option<u128>,
    pub successful_mutation_transactions: u64,
    pub mutation_commit_groups: u64,
    pub transaction_group_histogram: Vec<u64>,
    pub transaction_group_count: u64,
    pub wal_syncs: u64,
    pub commit_wal_syncs: u64,
    pub checkpoint_wal_syncs: u64,
    pub omitted_checkpoint_events: u64,
    pub checkpoint_events: Vec<BlinkCheckpointEvent>,
    pub failed_message: Option<String>,
    pub execution: BlinkCheckpointExecution,
    pub writes_during_checkpoint: u64,
    pub merged_checkpoint_requests: u64,
    pub checkpoint_requests_queued: u64,
    pub checkpoint_requests_pending: u64,
    pub checkpoint_requests_cancelled: u64,
    pub checkpoint_request_wait_nanos_total: u64,
    pub checkpoint_request_wait_nanos_max: u64,
    pub capacity_wait_count: u64,
    pub capacity_wait_nanos: u64,
    pub coordinator_pause_nanos: u64,
    pub retained_image_bytes: u64,
    pub peak_retained_image_bytes: u64,
    pub tasks_running: u64,
    pub peak_tasks_running: u64,
    pub per_tenant: BTreeMap<u64, BlinkTenantCheckpointMetrics>,
}

#[derive(Clone)]
pub struct BlinkWalCheckpointMetricsHandle {
    inner: Arc<StdMutex<BlinkWalCheckpointMetrics>>,
    activity: Arc<CoordinatorActivity>,
}

impl BlinkWalCheckpointMetricsHandle {
    pub fn snapshot(&self) -> BlinkWalCheckpointMetrics {
        self.inner
            .lock()
            .map(|metrics| metrics.snapshot_at(current_unix_nanos()))
            .unwrap_or_default()
    }

    pub fn reset(&self) {
        if let Ok(mut metrics) = self.inner.lock() {
            metrics.reset();
        }
    }

    pub async fn wait_for_coordinators_idle(&self) {
        self.activity.wait_until_idle().await;
    }
}

fn current_unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

impl BlinkWalCheckpointMetrics {
    fn observe_tenant_wal_bytes(
        &mut self,
        tenant_id: u64,
        wal_bytes: u64,
        observed_unix_nanos: u128,
    ) {
        let debt_bytes = wal_bytes.saturating_sub(self.threshold_bytes.unwrap_or(0));
        self.observe_tenant_checkpoint_state(tenant_id, wal_bytes, debt_bytes, observed_unix_nanos);
    }

    fn observe_tenant_checkpoint_state(
        &mut self,
        tenant_id: u64,
        wal_bytes: u64,
        debt_bytes: u64,
        observed_unix_nanos: u128,
    ) {
        let tenant_debt_max = {
            let tenant = self.per_tenant.entry(tenant_id).or_default();
            tenant.wal_bytes_current = wal_bytes;
            tenant.checkpoint_debt_bytes_current = debt_bytes;
            tenant.checkpoint_debt_bytes_max = tenant.checkpoint_debt_bytes_max.max(debt_bytes);
            if debt_bytes > 0 {
                tenant
                    .threshold_excess_since_unix_nanos
                    .get_or_insert(observed_unix_nanos);
            } else if let Some(started_unix_nanos) = tenant.threshold_excess_since_unix_nanos.take()
            {
                let elapsed_nanos = observed_unix_nanos
                    .saturating_sub(started_unix_nanos)
                    .min(u64::MAX as u128) as u64;
                tenant.threshold_excess_total_nanos = tenant
                    .threshold_excess_total_nanos
                    .saturating_add(elapsed_nanos);
            }
            tenant.checkpoint_debt_bytes_max
        };
        self.wal_bytes_current = self
            .per_tenant
            .values()
            .map(|entry| entry.wal_bytes_current)
            .fold(0, u64::saturating_add);
        self.checkpoint_debt_bytes_current = self
            .per_tenant
            .values()
            .map(|entry| entry.checkpoint_debt_bytes_current)
            .fold(0, u64::saturating_add);
        self.wal_bytes_max = self.wal_bytes_max.max(self.wal_bytes_current);
        self.checkpoint_debt_bytes_max = self
            .checkpoint_debt_bytes_max
            .max(self.checkpoint_debt_bytes_current);
        self.threshold_excess_max_bytes = self.threshold_excess_max_bytes.max(tenant_debt_max);
        self.threshold_excess_current_nanos = self
            .per_tenant
            .values()
            .map(|entry| entry.threshold_excess_total_nanos)
            .fold(0, u64::saturating_add);
        self.threshold_excess_total_nanos = self.threshold_excess_current_nanos;
        self.threshold_excess_since_unix_nanos = self
            .per_tenant
            .values()
            .filter_map(|entry| entry.threshold_excess_since_unix_nanos)
            .min();
    }

    fn tenant_checkpoint_request_queued(&mut self, tenant_id: u64) {
        self.checkpoint_request_queued();
        let tenant = self.per_tenant.entry(tenant_id).or_default();
        tenant.checkpoint_requests_queued = tenant.checkpoint_requests_queued.saturating_add(1);
        tenant.checkpoint_requests_pending = tenant.checkpoint_requests_pending.saturating_add(1);
    }

    fn tenant_checkpoint_request_started(&mut self, tenant_id: u64, wait_nanos: u64) {
        self.checkpoint_request_started(wait_nanos);
        let tenant = self.per_tenant.entry(tenant_id).or_default();
        tenant.checkpoint_requests_pending = tenant.checkpoint_requests_pending.saturating_sub(1);
        tenant.checkpoint_request_wait_nanos_total = tenant
            .checkpoint_request_wait_nanos_total
            .saturating_add(wait_nanos);
        tenant.checkpoint_request_wait_nanos_max =
            tenant.checkpoint_request_wait_nanos_max.max(wait_nanos);
    }

    fn tenant_checkpoint_request_cancelled(&mut self, tenant_id: u64, wait_nanos: u64) {
        self.checkpoint_request_cancelled(wait_nanos);
        let tenant = self.per_tenant.entry(tenant_id).or_default();
        tenant.checkpoint_requests_pending = tenant.checkpoint_requests_pending.saturating_sub(1);
        tenant.checkpoint_requests_cancelled =
            tenant.checkpoint_requests_cancelled.saturating_add(1);
        tenant.checkpoint_request_wait_nanos_total = tenant
            .checkpoint_request_wait_nanos_total
            .saturating_add(wait_nanos);
        tenant.checkpoint_request_wait_nanos_max =
            tenant.checkpoint_request_wait_nanos_max.max(wait_nanos);
    }

    fn tenant_checkpoint_slice_completed(&mut self, tenant_id: u64) {
        let tenant = self.per_tenant.entry(tenant_id).or_default();
        tenant.service_slices = tenant.service_slices.saturating_add(1);
    }

    fn observe_wal_bytes(&mut self, wal_bytes: u64, observed_unix_nanos: u128) {
        self.wal_bytes_current = wal_bytes;
        self.wal_bytes_max = self.wal_bytes_max.max(wal_bytes);
        let debt_bytes = self.threshold_bytes.map_or(0, |threshold_bytes| {
            wal_bytes.saturating_sub(threshold_bytes)
        });
        self.checkpoint_debt_bytes_current = debt_bytes;
        self.checkpoint_debt_bytes_max = self.checkpoint_debt_bytes_max.max(debt_bytes);
        self.threshold_excess_max_bytes = self.threshold_excess_max_bytes.max(debt_bytes);
        if debt_bytes > 0 {
            self.threshold_excess_since_unix_nanos
                .get_or_insert(observed_unix_nanos);
        } else if let Some(started_unix_nanos) = self.threshold_excess_since_unix_nanos.take() {
            let elapsed_nanos = observed_unix_nanos
                .saturating_sub(started_unix_nanos)
                .min(u64::MAX as u128) as u64;
            self.threshold_excess_total_nanos = self
                .threshold_excess_total_nanos
                .saturating_add(elapsed_nanos);
        }
        self.threshold_excess_current_nanos = self.threshold_excess_total_nanos;
        if let Some(started_unix_nanos) = self.threshold_excess_since_unix_nanos {
            let elapsed_nanos = observed_unix_nanos
                .saturating_sub(started_unix_nanos)
                .min(u64::MAX as u128) as u64;
            self.threshold_excess_current_nanos = self
                .threshold_excess_current_nanos
                .saturating_add(elapsed_nanos);
        }
    }

    fn snapshot_at(&self, observed_unix_nanos: u128) -> BlinkWalCheckpointMetrics {
        let mut snapshot = self.clone();
        if snapshot.per_tenant.is_empty() {
            snapshot.threshold_excess_current_nanos = snapshot.threshold_excess_total_nanos;
            if let Some(started_unix_nanos) = snapshot.threshold_excess_since_unix_nanos {
                let elapsed_nanos = observed_unix_nanos
                    .saturating_sub(started_unix_nanos)
                    .min(u64::MAX as u128) as u64;
                snapshot.threshold_excess_current_nanos = snapshot
                    .threshold_excess_current_nanos
                    .saturating_add(elapsed_nanos);
            }
        } else {
            snapshot.threshold_excess_current_nanos = 0;
            for tenant in snapshot.per_tenant.values() {
                let mut tenant_current = tenant.threshold_excess_total_nanos;
                if let Some(started_unix_nanos) = tenant.threshold_excess_since_unix_nanos {
                    tenant_current = tenant_current.saturating_add(
                        observed_unix_nanos
                            .saturating_sub(started_unix_nanos)
                            .min(u64::MAX as u128) as u64,
                    );
                }
                snapshot.threshold_excess_current_nanos = snapshot
                    .threshold_excess_current_nanos
                    .saturating_add(tenant_current);
            }
        }
        snapshot
    }

    fn checkpoint_request_queued(&mut self) {
        self.checkpoint_requests_queued = self.checkpoint_requests_queued.saturating_add(1);
        self.checkpoint_requests_pending = self.checkpoint_requests_pending.saturating_add(1);
    }

    fn checkpoint_request_started(&mut self, wait_nanos: u64) {
        self.checkpoint_requests_pending = self.checkpoint_requests_pending.saturating_sub(1);
        self.checkpoint_request_wait_nanos_total = self
            .checkpoint_request_wait_nanos_total
            .saturating_add(wait_nanos);
        self.checkpoint_request_wait_nanos_max =
            self.checkpoint_request_wait_nanos_max.max(wait_nanos);
    }

    fn checkpoint_request_cancelled(&mut self, wait_nanos: u64) {
        self.checkpoint_requests_pending = self.checkpoint_requests_pending.saturating_sub(1);
        self.checkpoint_requests_cancelled = self.checkpoint_requests_cancelled.saturating_add(1);
        self.checkpoint_request_wait_nanos_total = self
            .checkpoint_request_wait_nanos_total
            .saturating_add(wait_nanos);
        self.checkpoint_request_wait_nanos_max =
            self.checkpoint_request_wait_nanos_max.max(wait_nanos);
    }

    fn reset(&mut self) {
        let policy = self.policy;
        let execution = self.execution;
        let retained_image_bytes = self.retained_image_bytes;
        let tasks_running = self.tasks_running;
        let checkpoint_debt_bytes_current = self.checkpoint_debt_bytes_current;
        let threshold_excess_since_unix_nanos = self.threshold_excess_since_unix_nanos;
        let checkpoint_requests_pending = self.checkpoint_requests_pending;
        let mut per_tenant = self.per_tenant.clone();
        for tenant in per_tenant.values_mut() {
            tenant.checkpoint_debt_bytes_max = tenant.checkpoint_debt_bytes_current;
            tenant.threshold_excess_total_nanos = 0;
            tenant.checkpoint_requests_queued = 0;
            tenant.checkpoint_requests_cancelled = 0;
            tenant.checkpoint_request_wait_nanos_total = 0;
            tenant.checkpoint_request_wait_nanos_max = 0;
            tenant.service_slices = 0;
        }
        *self = Self {
            policy,
            execution,
            retained_image_bytes,
            peak_retained_image_bytes: retained_image_bytes,
            tasks_running,
            peak_tasks_running: tasks_running,
            checkpoint_debt_bytes_current,
            threshold_excess_since_unix_nanos,
            checkpoint_requests_pending,
            per_tenant,
            threshold_bytes: match policy {
                BlinkCheckpointPolicy::Disabled => None,
                BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes } => {
                    Some(threshold_bytes)
                }
            },
            ..Self::default()
        };
    }

    fn record_event(&mut self, event: BlinkCheckpointEvent) {
        if self.checkpoint_events.len() < 1_024 {
            self.checkpoint_events.push(event);
        } else {
            self.omitted_checkpoint_events = self.omitted_checkpoint_events.saturating_add(1);
        }
    }
}

impl Default for LocalTenantServiceConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("."),
            max_open_shards: 1_024,
            blink_open_worker_count: 4,
            blink_open_waiter_limit: 128,
            blink_open_shutdown_timeout: std::time::Duration::from_secs(30),
            database_config: DatabaseConfig::default(),
            coordinator_config: CoordinatorConfig::default(),
            blink_checkpoint_policy: BlinkCheckpointPolicy::Disabled,
            blink_wal_layout: BlinkWalLayout::Legacy,
            blink_checkpoint_execution: BlinkCheckpointExecution::Sync,
            blink_checkpoint_retention_bytes: 512 * 1024 * 1024,
            #[cfg(feature = "blink-demand-paging")]
            blink_shared_resources: None,
        }
    }
}

pub trait TenantShardResolver: Send + Sync {
    fn resolve(&self, tenant: TenantId) -> ShardId;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OneTenantOneShard;

impl TenantShardResolver for OneTenantOneShard {
    fn resolve(&self, tenant: TenantId) -> ShardId {
        ShardId::new(tenant.get())
    }
}

pub struct LocalTenantService {
    data_dir: PathBuf,
    max_open_shards: usize,
    database_config: DatabaseConfig,
    coordinator_config: CoordinatorConfig,
    resolver: Arc<dyn TenantShardResolver>,
    shards: Mutex<BTreeMap<TenantId, Arc<AsyncShard<ProductionFile, ProductionFile>>>>,
    fault_injector_factory: Mutex<Option<FaultInjectorFactory>>,
}

pub struct BlinkTenantService {
    data_dir: PathBuf,
    max_open_shards: usize,
    open_workers: Arc<Semaphore>,
    open_waiters: Arc<Semaphore>,
    open_changed: Arc<Notify>,
    open_shutdown_timeout: std::time::Duration,
    database_config: DatabaseConfig,
    coordinator_config: CoordinatorConfig,
    checkpoint_policy: BlinkCheckpointPolicy,
    wal_layout: BlinkWalLayout,
    checkpoint_execution: BlinkCheckpointExecution,
    checkpoint_retention_bytes: u64,
    checkpoint_metrics: Arc<StdMutex<BlinkWalCheckpointMetrics>>,
    coordinator_activity: Arc<CoordinatorActivity>,
    #[cfg(feature = "blink-demand-paging")]
    shared_resources: Option<dodb_storage::blink::resource::BlinkResourceManager>,
    #[cfg(feature = "blink-demand-paging")]
    open_tenant_metadata_bytes: u64,
    #[cfg(feature = "blink-demand-paging")]
    paged_open_config: Option<BlinkSharedResourceConfig>,
    #[cfg(feature = "blink-demand-paging")]
    checkpoint_debt_ledger: Option<Arc<CheckpointDebtLedger>>,
    shards: Arc<StdMutex<BlinkShardRegistry>>,
    fault_injector_factory: Mutex<Option<FaultInjectorFactory>>,
    #[cfg(test)]
    open_test_gate: StdMutex<Option<(TenantId, Arc<Semaphore>)>>,
    #[cfg(all(test, feature = "blink-demand-paging"))]
    page_read_test_gate: StdMutex<Option<Arc<OneShotTestGate>>>,
    #[cfg(all(test, feature = "blink-demand-paging"))]
    commit_test_gate: StdMutex<Option<Arc<OneShotTestGate>>>,
}

struct BlinkShardRegistry {
    shards: BTreeMap<TenantId, Arc<AsyncBlinkShard>>,
    opening: HashSet<TenantId>,
    shutting_down: bool,
}

struct BlinkOpenAttempt {
    tenant: TenantId,
    registry: Arc<StdMutex<BlinkShardRegistry>>,
    changed: Arc<Notify>,
    active: bool,
}

impl BlinkOpenAttempt {
    fn finish(mut self, shard: Option<Arc<AsyncBlinkShard>>) -> Result<(), Error> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|_| Error::invariant("B-link tenant registry lock poisoned"))?;
        registry.opening.remove(&self.tenant);
        let result = if registry.shutting_down {
            Err(Error::overloaded("tenant service is shutting down"))
        } else if let Some(shard) = shard {
            registry.shards.insert(self.tenant, shard);
            Ok(())
        } else {
            Ok(())
        };
        self.active = false;
        drop(registry);
        self.changed.notify_waiters();
        result
    }
}

impl Drop for BlinkOpenAttempt {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if let Ok(mut registry) = self.registry.lock() {
            registry.opening.remove(&self.tenant);
        }
        self.changed.notify_waiters();
    }
}

struct CoordinatorActivity {
    active: AtomicU64,
    idle: tokio::sync::Notify,
}

struct CoordinatorActivityGuard {
    activity: Arc<CoordinatorActivity>,
}

impl CoordinatorActivity {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            active: AtomicU64::new(0),
            idle: tokio::sync::Notify::new(),
        })
    }

    fn enter(self: &Arc<Self>) -> CoordinatorActivityGuard {
        self.active.fetch_add(1, Ordering::SeqCst);
        CoordinatorActivityGuard {
            activity: Arc::clone(self),
        }
    }

    async fn wait_until_idle(&self) {
        loop {
            let notified = self.idle.notified();
            if self.active.load(Ordering::SeqCst) == 0 {
                return;
            }
            notified.await;
        }
    }
}

impl Drop for CoordinatorActivityGuard {
    fn drop(&mut self) {
        if self.activity.active.fetch_sub(1, Ordering::SeqCst) == 1 {
            self.activity.idle.notify_waiters();
        }
    }
}

struct AsyncBlinkShard {
    read_handle: BlinkShardReadHandle,
    request_tx: Mutex<Option<tokio::sync::mpsc::Sender<QueuedBlinkTransaction>>>,
    coordinator_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    failed_message: Arc<StdMutex<Option<String>>>,
    activity: Arc<CoordinatorActivity>,
    #[cfg(feature = "blink-demand-paging")]
    tenant_id: u64,
    #[cfg(feature = "blink-demand-paging")]
    foreground_resources: Option<dodb_storage::blink::resource::BlinkResourceManager>,
    #[cfg(feature = "blink-demand-paging")]
    _metadata_reservation: Option<dodb_storage::blink::resource::Reservation>,
}

enum BlinkShardReadHandle {
    Resident(dodb_storage::BlinkReadHandle),
    #[cfg(feature = "blink-demand-paging")]
    Paged {
        handle: Option<Arc<dodb_storage::blink::PagedReadHandle>>,
        writable: Option<Arc<PagedWritableShard>>,
        blocking_workers: Arc<Semaphore>,
        activity: Arc<CoordinatorActivity>,
        #[cfg(test)]
        read_test_gate: Option<Arc<OneShotTestGate>>,
    },
}

#[cfg(feature = "blink-demand-paging")]
#[derive(Default)]
struct CheckpointDebtState {
    actual_by_tenant: BTreeMap<u64, u64>,
    reserved_by_tenant: BTreeMap<u64, u64>,
}

#[cfg(feature = "blink-demand-paging")]
struct CheckpointDebtLedger {
    global_limit: u64,
    per_tenant_limit: u64,
    state: StdMutex<CheckpointDebtState>,
}

#[cfg(feature = "blink-demand-paging")]
struct CheckpointDebtReservation {
    ledger: Arc<CheckpointDebtLedger>,
    tenant_id: u64,
    bytes: u64,
    active: bool,
}

#[cfg(feature = "blink-demand-paging")]
impl CheckpointDebtLedger {
    fn new(global_limit: u64, per_tenant_limit: u64) -> Arc<Self> {
        Arc::new(Self {
            global_limit,
            per_tenant_limit,
            state: StdMutex::new(CheckpointDebtState::default()),
        })
    }

    fn reserve(
        self: &Arc<Self>,
        tenant_id: u64,
        bytes: u64,
    ) -> Result<CheckpointDebtReservation, Error> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| Error::invariant("checkpoint debt ledger lock poisoned"))?;
        let actual_tenant = state.actual_by_tenant.get(&tenant_id).copied().unwrap_or(0);
        let reserved_tenant = state
            .reserved_by_tenant
            .get(&tenant_id)
            .copied()
            .unwrap_or(0);
        let next_tenant = actual_tenant
            .checked_add(reserved_tenant)
            .and_then(|current| current.checked_add(bytes))
            .ok_or_else(|| Error::overloaded("per-tenant checkpoint debt overflows"))?;
        let current_global = state
            .actual_by_tenant
            .values()
            .chain(state.reserved_by_tenant.values())
            .try_fold(0u64, |total, tenant_bytes| total.checked_add(*tenant_bytes))
            .ok_or_else(|| Error::overloaded("global checkpoint debt overflows"))?;
        let next_global = current_global
            .checked_add(bytes)
            .ok_or_else(|| Error::overloaded("global checkpoint debt overflows"))?;
        if next_tenant > self.per_tenant_limit || next_global > self.global_limit {
            return Err(Error::overloaded(
                "B-link checkpoint debt admission limit is exhausted",
            ));
        }
        state.reserved_by_tenant.insert(
            tenant_id,
            reserved_tenant
                .checked_add(bytes)
                .ok_or_else(|| Error::overloaded("per-tenant checkpoint debt overflows"))?,
        );
        Ok(CheckpointDebtReservation {
            ledger: Arc::clone(self),
            tenant_id,
            bytes,
            active: true,
        })
    }

    fn set_tenant_debt(&self, tenant_id: u64, bytes: u64) -> Result<(), Error> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| Error::invariant("checkpoint debt ledger lock poisoned"))?;
        if bytes == 0 {
            state.actual_by_tenant.remove(&tenant_id);
        } else {
            state.actual_by_tenant.insert(tenant_id, bytes);
        }
        Ok(())
    }
}

#[cfg(feature = "blink-demand-paging")]
impl CheckpointDebtReservation {
    fn commit(mut self, actual_debt_bytes: u64) -> Result<(), Error> {
        let mut state = self
            .ledger
            .state
            .lock()
            .map_err(|_| Error::invariant("checkpoint debt ledger lock poisoned"))?;
        let reserved_tenant = state
            .reserved_by_tenant
            .get(&self.tenant_id)
            .copied()
            .unwrap_or(0);
        let remaining_reserved = reserved_tenant
            .checked_sub(self.bytes)
            .ok_or_else(|| Error::invariant("checkpoint debt reservation underflow"))?;
        if remaining_reserved == 0 {
            state.reserved_by_tenant.remove(&self.tenant_id);
        } else {
            state
                .reserved_by_tenant
                .insert(self.tenant_id, remaining_reserved);
        }
        if actual_debt_bytes == 0 {
            state.actual_by_tenant.remove(&self.tenant_id);
        } else {
            state
                .actual_by_tenant
                .insert(self.tenant_id, actual_debt_bytes);
        }
        self.active = false;
        Ok(())
    }
}

#[cfg(feature = "blink-demand-paging")]
impl Drop for CheckpointDebtReservation {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let Ok(mut state) = self.ledger.state.lock() else {
            return;
        };
        let reserved_tenant = state
            .reserved_by_tenant
            .get(&self.tenant_id)
            .copied()
            .unwrap_or(0);
        let remaining_reserved = reserved_tenant.saturating_sub(self.bytes);
        if remaining_reserved == 0 {
            state.reserved_by_tenant.remove(&self.tenant_id);
        } else {
            state
                .reserved_by_tenant
                .insert(self.tenant_id, remaining_reserved);
        }
    }
}

#[cfg(feature = "blink-demand-paging")]
struct PagedWritableShard {
    writer: StdMutex<dodb_storage::blink::PagedTransactionWriter<ProductionFile>>,
    manifest_store: StdMutex<dodb_storage::blink::paged_manifest::PagedManifestStore>,
    checkpoint_directory: PathBuf,
    wal_directory: PathBuf,
    wal_identity: dodb_storage::WalIdentity,
    tenant_id: u64,
    checkpoint_policy: BlinkCheckpointPolicy,
    checkpoint_execution: BlinkCheckpointExecution,
    checkpoint_metrics: Arc<StdMutex<BlinkWalCheckpointMetrics>>,
    activity: Arc<CoordinatorActivity>,
    checkpoint_capacity_wait_timeout: std::time::Duration,
    maximum_transaction_wal_append_bytes: u64,
    checkpoint_debt_ledger: Arc<CheckpointDebtLedger>,
    checkpoint_pending: AtomicBool,
    checkpoint_shutdown: AtomicBool,
    checkpoint_shutdown_notify: tokio::sync::Notify,
    latest: StdRwLock<Arc<dodb_storage::blink::PagedReadHandle>>,
    transaction_worker: Arc<Semaphore>,
    #[cfg(all(test, feature = "blink-demand-paging"))]
    commit_test_gate: Option<Arc<OneShotTestGate>>,
}

#[cfg(feature = "blink-demand-paging")]
struct PagedWritableShardOpenConfig {
    writer: dodb_storage::blink::PagedTransactionWriter<ProductionFile>,
    manifest_store: dodb_storage::blink::paged_manifest::PagedManifestStore,
    checkpoint_directory: PathBuf,
    wal_directory: PathBuf,
    wal_identity: dodb_storage::WalIdentity,
    tenant_id: u64,
    checkpoint_policy: BlinkCheckpointPolicy,
    checkpoint_execution: BlinkCheckpointExecution,
    checkpoint_metrics: Arc<StdMutex<BlinkWalCheckpointMetrics>>,
    activity: Arc<CoordinatorActivity>,
    checkpoint_capacity_wait_timeout: std::time::Duration,
    maximum_transaction_wal_append_bytes: u64,
    checkpoint_debt_ledger: Arc<CheckpointDebtLedger>,
}

#[cfg(feature = "blink-demand-paging")]
struct AtomicFlagReset<'flag>(&'flag AtomicBool);

#[cfg(feature = "blink-demand-paging")]
impl Drop for AtomicFlagReset<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[cfg(feature = "blink-demand-paging")]
struct CheckpointTaskGuard(Arc<StdMutex<BlinkWalCheckpointMetrics>>);

#[cfg(feature = "blink-demand-paging")]
impl Drop for CheckpointTaskGuard {
    fn drop(&mut self) {
        if let Ok(mut metrics) = self.0.lock() {
            metrics.tasks_running = metrics.tasks_running.saturating_sub(1);
        }
    }
}

#[cfg(feature = "blink-demand-paging")]
struct CheckpointRequestMetricsGuard {
    metrics: Arc<StdMutex<BlinkWalCheckpointMetrics>>,
    tenant_id: u64,
    queued_at: Instant,
    pending: bool,
}

#[cfg(feature = "blink-demand-paging")]
impl CheckpointRequestMetricsGuard {
    fn started(&mut self) {
        if !self.pending {
            return;
        }
        let wait_nanos = self.queued_at.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.tenant_checkpoint_request_started(self.tenant_id, wait_nanos);
        }
        self.pending = false;
    }
}

#[cfg(feature = "blink-demand-paging")]
impl Drop for CheckpointRequestMetricsGuard {
    fn drop(&mut self) {
        if !self.pending {
            return;
        }
        let wait_nanos = self.queued_at.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.tenant_checkpoint_request_cancelled(self.tenant_id, wait_nanos);
        }
    }
}

#[cfg(feature = "blink-demand-paging")]
impl PagedWritableShard {
    fn new(
        config: PagedWritableShardOpenConfig,
        #[cfg(all(test, feature = "blink-demand-paging"))] commit_test_gate: Option<
            Arc<OneShotTestGate>,
        >,
    ) -> Result<Arc<Self>, Error> {
        let PagedWritableShardOpenConfig {
            writer,
            manifest_store,
            checkpoint_directory,
            wal_directory,
            wal_identity,
            tenant_id,
            checkpoint_policy,
            checkpoint_execution,
            checkpoint_metrics,
            activity,
            checkpoint_capacity_wait_timeout,
            maximum_transaction_wal_append_bytes,
            checkpoint_debt_ledger,
        } = config;
        let wal_bytes = writer.wal_metrics()?.wal_bytes;
        let has_uncheckpointed_commits =
            writer.has_commits_after(manifest_store.current().committed_prefix_lsn);
        let initial_debt_bytes = match checkpoint_policy {
            BlinkCheckpointPolicy::Disabled => 0,
            BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes }
                if has_uncheckpointed_commits =>
            {
                wal_bytes.saturating_sub(threshold_bytes)
            }
            BlinkCheckpointPolicy::WalBytesThreshold { .. } => 0,
        };
        checkpoint_debt_ledger.set_tenant_debt(tenant_id, initial_debt_bytes)?;
        let latest = Arc::new(writer.snapshot()?);
        Ok(Arc::new(Self {
            writer: StdMutex::new(writer),
            manifest_store: StdMutex::new(manifest_store),
            checkpoint_directory,
            wal_directory,
            wal_identity,
            tenant_id,
            checkpoint_policy,
            checkpoint_execution,
            checkpoint_metrics,
            activity,
            checkpoint_capacity_wait_timeout,
            maximum_transaction_wal_append_bytes,
            checkpoint_debt_ledger,
            checkpoint_pending: AtomicBool::new(false),
            checkpoint_shutdown: AtomicBool::new(false),
            checkpoint_shutdown_notify: tokio::sync::Notify::new(),
            latest: StdRwLock::new(latest),
            transaction_worker: Arc::new(Semaphore::new(1)),
            #[cfg(all(test, feature = "blink-demand-paging"))]
            commit_test_gate,
        }))
    }

    fn snapshot(&self) -> Result<Arc<dodb_storage::blink::PagedReadHandle>, Error> {
        self.latest
            .read()
            .map(|current| Arc::clone(&current))
            .map_err(|_| Error::invariant("paged writer snapshot lock poisoned"))
    }

    fn schedule_checkpoint_after_commit(
        self: &Arc<Self>,
        wal_bytes: u64,
        has_uncheckpointed_commits: bool,
        runtime: &tokio::runtime::Handle,
    ) -> Option<tokio::sync::oneshot::Receiver<Result<(), Error>>> {
        let BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes } = self.checkpoint_policy
        else {
            return None;
        };
        let debt_bytes = if has_uncheckpointed_commits {
            wal_bytes.saturating_sub(threshold_bytes)
        } else {
            0
        };
        if let Ok(mut metrics) = self.checkpoint_metrics.lock() {
            metrics.observe_tenant_checkpoint_state(
                self.tenant_id,
                wal_bytes,
                debt_bytes,
                current_unix_nanos(),
            );
        }
        if !has_uncheckpointed_commits || wal_bytes < threshold_bytes {
            return None;
        }
        if self
            .checkpoint_pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return None;
        }
        if let Ok(mut metrics) = self.checkpoint_metrics.lock() {
            metrics.tenant_checkpoint_request_queued(self.tenant_id);
        }
        let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
        let writable = Arc::clone(self);
        let activity_guard = self.activity.enter();
        runtime.spawn(async move {
            let _pending_guard = AtomicFlagReset(&writable.checkpoint_pending);
            let _activity_guard = activity_guard;
            let result = writable.run_threshold_checkpoint(threshold_bytes).await;
            let _ = result_sender.send(result);
        });
        Some(result_receiver)
    }

    async fn reschedule_checkpoint_for_debt_pressure(
        self: &Arc<Self>,
    ) -> Result<Option<tokio::sync::oneshot::Receiver<Result<(), Error>>>, Error> {
        let writable = Arc::clone(self);
        let (wal_bytes, has_uncheckpointed_commits) = tokio::task::spawn_blocking(move || {
            let _activity_guard = writable.activity.enter();
            let writer = writable
                .writer
                .lock()
                .map_err(|_| Error::invariant("paged writer lock poisoned"))?;
            let manifest_store = writable
                .manifest_store
                .lock()
                .map_err(|_| Error::invariant("paged manifest lock poisoned"))?;
            let wal_bytes = writer.wal_metrics()?.wal_bytes;
            let has_uncheckpointed =
                writer.has_commits_after(manifest_store.current().committed_prefix_lsn);
            Ok::<_, Error>((wal_bytes, has_uncheckpointed))
        })
        .await
        .map_err(|error| {
            Error::durability(format!("paged debt-pressure worker failed: {error}"))
        })??;
        Ok(self.schedule_checkpoint_after_commit(
            wal_bytes,
            has_uncheckpointed_commits,
            &tokio::runtime::Handle::current(),
        ))
    }

    async fn run_threshold_checkpoint(self: &Arc<Self>, threshold_bytes: u64) -> Result<(), Error> {
        let _activity_guard = self.activity.enter();
        let checkpoint_started = Instant::now();
        let (current_wal_bytes, has_uncheckpointed_commits) = {
            let writable = Arc::clone(self);
            tokio::task::spawn_blocking(move || {
                let _activity_guard = writable.activity.enter();
                let writer = writable
                    .writer
                    .lock()
                    .map_err(|_| Error::invariant("paged writer lock poisoned"))?;
                let manifest_store = writable
                    .manifest_store
                    .lock()
                    .map_err(|_| Error::invariant("paged manifest lock poisoned"))?;
                let wal_bytes = writer.wal_metrics()?.wal_bytes;
                let has_uncheckpointed =
                    writer.has_commits_after(manifest_store.current().committed_prefix_lsn);
                Ok::<_, Error>((wal_bytes, has_uncheckpointed))
            })
            .await
            .map_err(|error| {
                Error::durability(format!("paged WAL metric worker failed: {error}"))
            })??
        };
        let current_debt_bytes = if has_uncheckpointed_commits {
            current_wal_bytes.saturating_sub(threshold_bytes)
        } else {
            0
        };
        self.checkpoint_debt_ledger
            .set_tenant_debt(self.tenant_id, current_debt_bytes)?;
        if let Ok(mut metrics) = self.checkpoint_metrics.lock() {
            metrics.observe_tenant_checkpoint_state(
                self.tenant_id,
                current_wal_bytes,
                current_debt_bytes,
                current_unix_nanos(),
            );
        }
        if !has_uncheckpointed_commits || current_wal_bytes < threshold_bytes {
            return Ok(());
        }
        let request_metrics_guard = CheckpointRequestMetricsGuard {
            metrics: Arc::clone(&self.checkpoint_metrics),
            tenant_id: self.tenant_id,
            queued_at: Instant::now(),
            pending: true,
        };
        let checkpoint_result = self.run_checkpoint_slices(request_metrics_guard).await;
        if let Ok(Some((wal_bytes, has_uncheckpointed))) = &checkpoint_result {
            let debt_bytes = if *has_uncheckpointed {
                wal_bytes.saturating_sub(threshold_bytes)
            } else {
                0
            };
            self.checkpoint_debt_ledger
                .set_tenant_debt(self.tenant_id, debt_bytes)?;
        }
        let duration_nanos = checkpoint_started
            .elapsed()
            .as_nanos()
            .min(u64::MAX as u128) as u64;
        if let Ok(mut metrics) = self.checkpoint_metrics.lock() {
            metrics.checkpoint_total_nanos = metrics
                .checkpoint_total_nanos
                .saturating_add(duration_nanos);
            metrics.checkpoint_max_nanos = metrics.checkpoint_max_nanos.max(duration_nanos);
            match &checkpoint_result {
                Ok(Some((wal_bytes, has_uncheckpointed))) => {
                    metrics.checkpoints_completed = metrics.checkpoints_completed.saturating_add(1);
                    let debt_bytes = if *has_uncheckpointed {
                        wal_bytes.saturating_sub(threshold_bytes)
                    } else {
                        0
                    };
                    metrics.observe_tenant_checkpoint_state(
                        self.tenant_id,
                        *wal_bytes,
                        debt_bytes,
                        current_unix_nanos(),
                    );
                }
                Ok(None) => {}
                Err(error) => {
                    metrics.checkpoints_failed = metrics.checkpoints_failed.saturating_add(1);
                    metrics.failed_message = Some(error.to_string());
                }
            }
        }
        checkpoint_result.map(|_| ())
    }

    async fn run_checkpoint_slices(
        self: &Arc<Self>,
        mut request_metrics_guard: CheckpointRequestMetricsGuard,
    ) -> Result<Option<(u64, bool)>, Error> {
        let Some(initial_permit) = self.acquire_checkpoint_slice().await? else {
            return Ok(None);
        };
        let writable = Arc::clone(self);
        let pack_number = next_paged_checkpoint_pack_number(&self.checkpoint_directory)?;
        let mut job = physical_checkpoint::submit_checkpoint_progress(move || {
            let _activity_guard = writable.activity.enter();
            let result = (|| {
                let mut writer = writable
                    .writer
                    .lock()
                    .map_err(|_| Error::invariant("paged writer lock poisoned"))?;
                let manifest_store = writable
                    .manifest_store
                    .lock()
                    .map_err(|_| Error::invariant("paged manifest lock poisoned"))?;
                let previous_manifest = manifest_store.current();
                writer.begin_checkpoint(
                    &manifest_store,
                    dodb_storage::blink::PagedCheckpointPackRequest {
                        directory: writable.checkpoint_directory.clone(),
                        file_number: pack_number,
                        identity: writable.wal_identity.clone(),
                        checkpoint_lsn: dodb_core::Lsn::ZERO,
                        previous_checkpoint_lsn: Some(previous_manifest.committed_prefix_lsn),
                        coalesce: true,
                    },
                    true,
                )
            })();
            drop(initial_permit);
            result
        })?
        .await
        .map_err(|error| {
            Error::durability(format!("paged checkpoint begin worker failed: {error}"))
        })??;
        let Some(mut job) = job.take() else {
            return Ok(None);
        };
        if !self.reserve_checkpoint_disk_capacity(&mut job).await? {
            return Ok(None);
        }
        request_metrics_guard.started();
        if let Ok(mut metrics) = self.checkpoint_metrics.lock() {
            metrics.tenant_checkpoint_slice_completed(self.tenant_id);
        }
        if let Ok(mut metrics) = self.checkpoint_metrics.lock() {
            metrics.checkpoints_started = metrics.checkpoints_started.saturating_add(1);
            metrics.tasks_running = metrics.tasks_running.saturating_add(1);
            metrics.peak_tasks_running = metrics.peak_tasks_running.max(metrics.tasks_running);
        }
        let _task_guard = CheckpointTaskGuard(Arc::clone(&self.checkpoint_metrics));
        self.complete_checkpoint_job(job).await
    }

    async fn acquire_checkpoint_slice(
        &self,
    ) -> Result<Option<physical_checkpoint::CheckpointCapacityPermit>, Error> {
        if self.checkpoint_shutdown.load(Ordering::Acquire) {
            return Ok(None);
        }
        let shutdown = self.checkpoint_shutdown_notify.notified();
        tokio::pin!(shutdown);
        shutdown.as_mut().enable();
        if self.checkpoint_shutdown.load(Ordering::Acquire) {
            return Ok(None);
        }
        tokio::select! {
            permit = physical_checkpoint::acquire_fair_checkpoint_slice(self.tenant_id) => {
                permit.map(Some)
            }
            _ = shutdown => Ok(None),
        }
    }

    async fn reserve_checkpoint_disk_capacity(
        &self,
        job: &mut dodb_storage::blink::PagedCheckpointJob,
    ) -> Result<bool, Error> {
        if self.checkpoint_shutdown.load(Ordering::Acquire) {
            return Ok(false);
        }
        let shutdown = self.checkpoint_shutdown_notify.notified();
        tokio::pin!(shutdown);
        shutdown.as_mut().enable();
        if self.checkpoint_shutdown.load(Ordering::Acquire) {
            return Ok(false);
        }
        tokio::select! {
            result = job.reserve_disk_capacity(self.checkpoint_capacity_wait_timeout) => {
                result.map(|()| true)
            }
            _ = shutdown => Ok(false),
        }
    }

    fn cancel_checkpoints(&self) {
        self.checkpoint_shutdown.store(true, Ordering::Release);
        self.checkpoint_shutdown_notify.notify_waiters();
    }

    async fn complete_checkpoint_job(
        self: &Arc<Self>,
        mut job: dodb_storage::blink::PagedCheckpointJob,
    ) -> Result<Option<(u64, bool)>, Error> {
        loop {
            let Some(permit) = self.acquire_checkpoint_slice().await? else {
                return Ok(None);
            };
            let activity = Arc::clone(&self.activity);
            let (next_job, complete) =
                physical_checkpoint::submit_checkpoint_progress(move || {
                    let _activity_guard = activity.enter();
                    let mut job = job;
                    let result = job.write_pack_slice(128, None);
                    drop(permit);
                    result.map(|complete| (job, complete))
                })?
                .await
                .map_err(|error| {
                    Error::durability(format!("paged checkpoint slice worker failed: {error}"))
                })??;
            job = next_job;
            if let Ok(mut metrics) = self.checkpoint_metrics.lock() {
                metrics.tenant_checkpoint_slice_completed(self.tenant_id);
            }
            if complete {
                break;
            }
        }
        let Some(publish_permit) = self.acquire_checkpoint_slice().await? else {
            return Ok(None);
        };
        let writable = Arc::clone(self);
        let (wal_bytes, has_uncheckpointed, next_snapshot) =
            physical_checkpoint::submit_checkpoint_progress(move || {
                let _activity_guard = writable.activity.enter();
                let result: Result<(u64, bool, Arc<dodb_storage::blink::PagedReadHandle>), Error> =
                    (|| {
                        let mut writer = writable
                            .writer
                            .lock()
                            .map_err(|_| Error::invariant("paged writer lock poisoned"))?;
                        let mut manifest_store = writable
                            .manifest_store
                            .lock()
                            .map_err(|_| Error::invariant("paged manifest lock poisoned"))?;
                        let previous_manifest = manifest_store.current();
                        let (next_wal_number, next_wal_file) = create_next_paged_wal_segment(
                            &writable.wal_directory,
                            previous_manifest.recovery_wal_file_number,
                        )?;
                        writer.publish_checkpoint(
                            &mut manifest_store,
                            job,
                            next_wal_number,
                            next_wal_file,
                        )?;
                        let wal_bytes = writer.wal_metrics()?.wal_bytes;
                        let has_uncheckpointed =
                            writer.has_commits_after(manifest_store.current().committed_prefix_lsn);
                        let next_snapshot = Arc::new(writer.snapshot()?);
                        Ok((wal_bytes, has_uncheckpointed, next_snapshot))
                    })();
                drop(publish_permit);
                result
            })?
            .await
            .map_err(|error| {
                Error::durability(format!("paged checkpoint publish worker failed: {error}"))
            })??;
        *self
            .latest
            .write()
            .map_err(|_| Error::durability("paged committed root lock is poisoned"))? =
            next_snapshot;
        if let Ok(mut metrics) = self.checkpoint_metrics.lock() {
            metrics.tenant_checkpoint_slice_completed(self.tenant_id);
        }
        Ok(Some((wal_bytes, has_uncheckpointed)))
    }

    async fn transact(
        self: &Arc<Self>,
        request: TransactionRequest,
        queued_reservation: Option<dodb_storage::blink::resource::Reservation>,
        preparation_reservation: Option<dodb_storage::blink::resource::Reservation>,
        blocking_workers: Arc<Semaphore>,
        activity: Arc<CoordinatorActivity>,
    ) -> Result<TransactionResult, Error> {
        let debt_reservation = if request.mutations.is_empty()
            || self.checkpoint_policy == BlinkCheckpointPolicy::Disabled
        {
            None
        } else {
            match self
                .checkpoint_debt_ledger
                .reserve(self.tenant_id, self.maximum_transaction_wal_append_bytes)
            {
                Ok(reservation) => Some(reservation),
                Err(debt_limit_error) => {
                    let checkpoint_waiter = self.reschedule_checkpoint_for_debt_pressure().await?;
                    if self.checkpoint_execution == BlinkCheckpointExecution::Sync
                        && let Some(checkpoint_waiter) = checkpoint_waiter
                    {
                        checkpoint_waiter.await.map_err(|error| {
                            Error::durability(format!(
                                "paged debt-pressure checkpoint task dropped its result: {error}"
                            ))
                        })??;
                        Some(
                            self.checkpoint_debt_ledger.reserve(
                                self.tenant_id,
                                self.maximum_transaction_wal_append_bytes,
                            )?,
                        )
                    } else {
                        return Err(debt_limit_error);
                    }
                }
            }
        };
        let worker_permit = blocking_workers
            .try_acquire_owned()
            .map_err(|_| Error::overloaded("paged writer worker capacity is full"))?;
        let transaction_permit = self
            .transaction_worker
            .clone()
            .try_acquire_owned()
            .map_err(|_| Error::overloaded("paged transaction capacity is full"))?;
        let activity_guard = activity.enter();
        let writable = Arc::clone(self);
        let runtime = tokio::runtime::Handle::current();
        let (transaction_result, checkpoint_waiter) =
            tokio::task::spawn_blocking(move || {
                let _worker_permit = worker_permit;
                let _transaction_permit = transaction_permit;
                let _activity_guard = activity_guard;
                let _queued_reservation = queued_reservation;
                let _preparation_reservation = preparation_reservation;
                let mut debt_reservation = debt_reservation;
                #[cfg(all(test, feature = "blink-demand-paging"))]
                wait_for_one_shot_test_gate(writable.commit_test_gate.clone());
                let mut writer = writable
                    .writer
                    .lock()
                    .map_err(|_| Error::invariant("paged writer lock poisoned"))?;
                let revision = writer.next_revision()?;
                let transaction_results = writer.transact_group(
                    std::slice::from_ref(&request),
                    &dodb_storage::StorageLimits::default(),
                    &[revision],
                )?;
                let transaction_result =
                    transaction_results.into_iter().next().ok_or_else(|| {
                        Error::invariant("paged writer returned no transaction result")
                    })?;
                let checkpoint_waiter = if transaction_result
                    .as_ref()
                    .is_ok_and(|result| result.commit_lsn.is_some())
                {
                    let wal_bytes = writer
                        .wal_metrics()
                        .map_err(|error| {
                            Error::durability(format!(
                                "paged transaction committed but WAL accounting failed: {error}"
                            ))
                        })?
                        .wal_bytes;
                    let manifest_store = writable.manifest_store.lock().map_err(|_| {
                        Error::durability(
                            "paged transaction committed but manifest state is unavailable",
                        )
                    })?;
                    let has_uncheckpointed_commits =
                        writer.has_commits_after(manifest_store.current().committed_prefix_lsn);
                    let debt_bytes = match writable.checkpoint_policy {
                        BlinkCheckpointPolicy::Disabled => 0,
                        BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes }
                            if has_uncheckpointed_commits =>
                        {
                            wal_bytes.saturating_sub(threshold_bytes)
                        }
                        BlinkCheckpointPolicy::WalBytesThreshold { .. } => 0,
                    };
                    if let Some(reservation) = debt_reservation.take() {
                        reservation.commit(debt_bytes)?;
                    }
                    writable
                        .checkpoint_debt_ledger
                        .set_tenant_debt(writable.tenant_id, debt_bytes)?;
                    writable.schedule_checkpoint_after_commit(
                        wal_bytes,
                        has_uncheckpointed_commits,
                        &runtime,
                    )
                } else {
                    None
                };
                if transaction_result
                    .as_ref()
                    .is_ok_and(|result| result.commit_lsn.is_some())
                {
                    let next_snapshot = Arc::new(writer.snapshot().map_err(|error| {
                    Error::durability(format!(
                        "paged transaction committed but its read root could not be pinned: {error}"
                    ))
                })?);
                    *writable.latest.write().map_err(|_| {
                        Error::durability("paged committed root lock is poisoned")
                    })? = next_snapshot;
                }
                Ok::<_, Error>((transaction_result, checkpoint_waiter))
            })
            .await
            .map_err(|error| {
                Error::durability(format!("paged transaction worker failed: {error}"))
            })??;
        if self.checkpoint_execution == BlinkCheckpointExecution::Sync
            && let Some(checkpoint_waiter) = checkpoint_waiter
        {
            checkpoint_waiter.await.map_err(|error| {
                Error::durability(format!("paged checkpoint task dropped its result: {error}"))
            })??;
        }
        transaction_result
    }
}

impl BlinkShardReadHandle {
    async fn get_with_response_budget(
        &self,
        key: &dodb_core::DocumentKey,
        maximum_response_bytes: usize,
    ) -> Result<RevisionState, Error> {
        match self {
            Self::Resident(handle) => handle.get_with_response_budget(key, maximum_response_bytes),
            #[cfg(feature = "blink-demand-paging")]
            Self::Paged {
                handle,
                writable,
                blocking_workers,
                activity,
                #[cfg(test)]
                read_test_gate,
            } => {
                let worker_permit = blocking_workers
                    .clone()
                    .try_acquire_owned()
                    .map_err(|_| Error::overloaded("paged read worker capacity is full"))?;
                let handle =
                    match writable {
                        Some(writable) => writable.snapshot()?,
                        None => Arc::clone(handle.as_ref().ok_or_else(|| {
                            Error::invariant("read-only paged handle is missing")
                        })?),
                    };
                let key = key.clone();
                let activity = Arc::clone(activity);
                let activity_guard = activity.enter();
                #[cfg(test)]
                let read_test_gate = read_test_gate.clone();
                tokio::task::spawn_blocking(move || {
                    let _worker_permit = worker_permit;
                    let _activity_guard = activity_guard;
                    #[cfg(test)]
                    wait_for_one_shot_test_gate(read_test_gate);
                    handle.get_with_response_budget(&key, maximum_response_bytes)
                })
                .await
                .map_err(|error| Error::invariant(format!("paged read worker failed: {error}")))?
            }
        }
    }

    async fn query_with_response_budget(
        &self,
        primary_key: &dodb_core::PrimaryKey,
        exclusive_after_sort_key: Option<&dodb_core::SortKey>,
        limit: usize,
        maximum_response_bytes: usize,
    ) -> Result<Vec<dodb_storage::Document>, Error> {
        match self {
            Self::Resident(handle) => handle.query_with_response_budget(
                primary_key,
                exclusive_after_sort_key,
                limit,
                maximum_response_bytes,
            ),
            #[cfg(feature = "blink-demand-paging")]
            Self::Paged {
                handle,
                writable,
                blocking_workers,
                activity,
                #[cfg(test)]
                read_test_gate,
            } => {
                let worker_permit = blocking_workers
                    .clone()
                    .try_acquire_owned()
                    .map_err(|_| Error::overloaded("paged read worker capacity is full"))?;
                let handle =
                    match writable {
                        Some(writable) => writable.snapshot()?,
                        None => Arc::clone(handle.as_ref().ok_or_else(|| {
                            Error::invariant("read-only paged handle is missing")
                        })?),
                    };
                let primary_key = primary_key.clone();
                let exclusive_after_sort_key = exclusive_after_sort_key.cloned();
                let activity = Arc::clone(activity);
                let activity_guard = activity.enter();
                #[cfg(test)]
                let read_test_gate = read_test_gate.clone();
                tokio::task::spawn_blocking(move || {
                    let _worker_permit = worker_permit;
                    let _activity_guard = activity_guard;
                    #[cfg(test)]
                    wait_for_one_shot_test_gate(read_test_gate);
                    handle.query_with_response_budget(
                        &primary_key,
                        exclusive_after_sort_key.as_ref(),
                        limit,
                        maximum_response_bytes,
                    )
                })
                .await
                .map_err(|error| Error::invariant(format!("paged read worker failed: {error}")))?
            }
        }
    }

    async fn scan_with_response_budget(
        &self,
        exclusive_after: Option<&dodb_core::DocumentKey>,
        limit: usize,
        maximum_response_bytes: usize,
    ) -> Result<Vec<dodb_storage::Document>, Error> {
        match self {
            Self::Resident(handle) => {
                handle.scan_with_response_budget(exclusive_after, limit, maximum_response_bytes)
            }
            #[cfg(feature = "blink-demand-paging")]
            Self::Paged {
                handle,
                writable,
                blocking_workers,
                activity,
                #[cfg(test)]
                read_test_gate,
            } => {
                let worker_permit = blocking_workers
                    .clone()
                    .try_acquire_owned()
                    .map_err(|_| Error::overloaded("paged read worker capacity is full"))?;
                let handle =
                    match writable {
                        Some(writable) => writable.snapshot()?,
                        None => Arc::clone(handle.as_ref().ok_or_else(|| {
                            Error::invariant("read-only paged handle is missing")
                        })?),
                    };
                let exclusive_after = exclusive_after.cloned();
                let activity = Arc::clone(activity);
                let activity_guard = activity.enter();
                #[cfg(test)]
                let read_test_gate = read_test_gate.clone();
                tokio::task::spawn_blocking(move || {
                    let _worker_permit = worker_permit;
                    let _activity_guard = activity_guard;
                    #[cfg(test)]
                    wait_for_one_shot_test_gate(read_test_gate);
                    handle.scan_with_response_budget(
                        exclusive_after.as_ref(),
                        limit,
                        maximum_response_bytes,
                    )
                })
                .await
                .map_err(|error| Error::invariant(format!("paged read worker failed: {error}")))?
            }
        }
    }

    fn is_read_only(&self) -> bool {
        #[cfg(feature = "blink-demand-paging")]
        if matches!(self, Self::Paged { writable: None, .. }) {
            return true;
        }
        false
    }
}

#[cfg(all(test, feature = "blink-demand-paging"))]
struct OneShotTestGate {
    barrier: std::sync::Barrier,
    blocks_next_worker: std::sync::atomic::AtomicBool,
}

#[cfg(all(test, feature = "blink-demand-paging"))]
struct FailAtOnce {
    point: &'static str,
    fired: bool,
}

#[cfg(all(test, feature = "blink-demand-paging"))]
impl dodb_storage::FaultInjector for FailAtOnce {
    fn hit(&mut self, point: &str) -> Result<(), Error> {
        if point == self.point && !self.fired {
            self.fired = true;
            return Err(Error::durability(format!("injected failure at {point}")));
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "blink-demand-paging"))]
fn wait_for_one_shot_test_gate(gate: Option<Arc<OneShotTestGate>>) {
    if let Some(gate) = gate
        && gate.blocks_next_worker.swap(false, Ordering::SeqCst)
    {
        gate.barrier.wait();
    }
}

struct QueuedBlinkTransaction {
    request: TransactionRequest,
    response_tx: tokio::sync::oneshot::Sender<Result<TransactionResult, Error>>,
    #[cfg(feature = "blink-demand-paging")]
    _queued_reservation: Option<dodb_storage::blink::resource::Reservation>,
    #[cfg(feature = "blink-demand-paging")]
    _preparation_reservation: Option<dodb_storage::blink::resource::Reservation>,
}

impl BlinkTenantService {
    pub fn new(config: LocalTenantServiceConfig) -> Result<Self, Error> {
        if config.max_open_shards == 0 {
            return Err(Error::invalid_request("max_open_shards must be nonzero"));
        }
        if config.blink_open_worker_count == 0
            || config.blink_open_waiter_limit == 0
            || config.blink_open_shutdown_timeout.is_zero()
        {
            return Err(Error::invalid_request(
                "B-link open worker, waiter, and shutdown limits must be nonzero",
            ));
        }
        if matches!(
            config.blink_checkpoint_policy,
            BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 0 }
        ) {
            return Err(Error::invalid_request(
                "B-link WAL checkpoint threshold must be positive",
            ));
        }
        if matches!(
            config.blink_wal_layout,
            BlinkWalLayout::Segmented {
                max_segment_bytes: 0
            } | BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 0,
                ..
            }
        ) {
            return Err(Error::invalid_request(
                "B-link WAL segment size must be positive",
            ));
        }
        #[cfg(feature = "blink-demand-paging")]
        if config.blink_shared_resources.is_some()
            && !matches!(
                config.blink_wal_layout,
                BlinkWalLayout::SegmentedBases { .. }
            )
        {
            return Err(Error::invalid_request(
                "B-link demand paging requires segmented WAL with durable bases",
            ));
        }
        if config.blink_checkpoint_execution == BlinkCheckpointExecution::Background
            && config.blink_wal_layout == BlinkWalLayout::Legacy
        {
            return Err(Error::invalid_request(
                "background checkpoints require segmented WAL",
            ));
        }
        if !(4 * dodb_storage::PAGE_SIZE as u64..=512 * 1024 * 1024)
            .contains(&config.blink_checkpoint_retention_bytes)
        {
            return Err(Error::invalid_request(
                "checkpoint retention budget is too small",
            ));
        }
        #[cfg(feature = "blink-demand-paging")]
        let (shared_resources, open_tenant_metadata_bytes, paged_open_config) = match config
            .blink_shared_resources
        {
            Some(resource_config) => {
                if resource_config.open_tenant_metadata_bytes == 0
                    || config.coordinator_config.queue_capacity == 0
                    || config.coordinator_config.max_group_requests == 0
                    || config.coordinator_config.max_group_bytes == 0
                    || resource_config.memory.max_tenants < config.max_open_shards
                    || resource_config.memory.max_pending_waiters < config.blink_open_waiter_limit
                    || resource_config.disk.max_tenants < config.max_open_shards
                    || (resource_config.paged_read_only || resource_config.paged_writer_enabled)
                        && !valid_paged_open_config(resource_config)
                {
                    return Err(Error::invalid_request(
                        "B-link shared resource limits do not cover configured tenant opens",
                    ));
                }
                let manager =
                    dodb_storage::blink::resource::BlinkResourceManager::new_with_disk_limits(
                        resource_config.memory,
                        resource_config.disk,
                    )
                    .map_err(|_| {
                        Error::invalid_request("B-link shared resource limits are invalid")
                    })?;
                (
                    Some(manager),
                    resource_config.open_tenant_metadata_bytes,
                    (resource_config.paged_read_only || resource_config.paged_writer_enabled)
                        .then_some(resource_config),
                )
            }
            None => (None, 0, None),
        };
        std::fs::create_dir_all(config.data_dir.join("blink"))?;
        let checkpoint_metrics = Arc::new(StdMutex::new(BlinkWalCheckpointMetrics {
            policy: config.blink_checkpoint_policy,
            execution: config.blink_checkpoint_execution,
            threshold_bytes: match config.blink_checkpoint_policy {
                BlinkCheckpointPolicy::Disabled => None,
                BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes } => {
                    Some(threshold_bytes)
                }
            },
            ..BlinkWalCheckpointMetrics::default()
        }));
        let coordinator_activity = CoordinatorActivity::new();
        Ok(Self {
            data_dir: config.data_dir,
            max_open_shards: config.max_open_shards,
            open_workers: Arc::new(Semaphore::new(config.blink_open_worker_count)),
            open_waiters: Arc::new(Semaphore::new(config.blink_open_waiter_limit)),
            open_changed: Arc::new(Notify::new()),
            open_shutdown_timeout: config.blink_open_shutdown_timeout,
            database_config: config.database_config,
            coordinator_config: config.coordinator_config,
            checkpoint_policy: config.blink_checkpoint_policy,
            wal_layout: config.blink_wal_layout,
            checkpoint_execution: config.blink_checkpoint_execution,
            checkpoint_retention_bytes: config.blink_checkpoint_retention_bytes,
            checkpoint_metrics,
            coordinator_activity,
            #[cfg(feature = "blink-demand-paging")]
            shared_resources,
            #[cfg(feature = "blink-demand-paging")]
            open_tenant_metadata_bytes,
            #[cfg(feature = "blink-demand-paging")]
            paged_open_config,
            #[cfg(feature = "blink-demand-paging")]
            checkpoint_debt_ledger: paged_open_config.map(|resource_config| {
                CheckpointDebtLedger::new(
                    resource_config.maximum_checkpoint_debt_bytes_global,
                    resource_config.maximum_checkpoint_debt_bytes_per_tenant,
                )
            }),
            shards: Arc::new(StdMutex::new(BlinkShardRegistry {
                shards: BTreeMap::new(),
                opening: HashSet::new(),
                shutting_down: false,
            })),
            fault_injector_factory: Mutex::new(None),
            #[cfg(test)]
            open_test_gate: StdMutex::new(None),
            #[cfg(all(test, feature = "blink-demand-paging"))]
            page_read_test_gate: StdMutex::new(None),
            #[cfg(all(test, feature = "blink-demand-paging"))]
            commit_test_gate: StdMutex::new(None),
        })
    }

    pub fn set_fault_injector_factory(&mut self, factory: FaultInjectorFactory) {
        if let Ok(mut current) = self.fault_injector_factory.try_lock() {
            *current = Some(factory);
        }
    }

    pub fn checkpoint_metrics_snapshot(&self) -> BlinkWalCheckpointMetrics {
        self.checkpoint_metrics
            .lock()
            .map(|metrics| metrics.snapshot_at(current_unix_nanos()))
            .unwrap_or_default()
    }

    pub fn checkpoint_metrics_handle(&self) -> BlinkWalCheckpointMetricsHandle {
        BlinkWalCheckpointMetricsHandle {
            inner: Arc::clone(&self.checkpoint_metrics),
            activity: Arc::clone(&self.coordinator_activity),
        }
    }

    pub fn reset_checkpoint_metrics(&self) {
        if let Ok(mut metrics) = self.checkpoint_metrics.lock() {
            metrics.reset();
        }
    }

    pub async fn wait_for_coordinators_idle(&self) {
        let shards = self
            .shards
            .lock()
            .ok()
            .map(|registry| registry.shards.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        for shard in shards {
            shard.activity.wait_until_idle().await;
        }
    }

    pub async fn shutdown(&self) {
        let _ = self.shutdown_checked().await;
    }

    pub async fn shutdown_checked(&self) -> Result<(), Error> {
        self.shards
            .lock()
            .map_err(|_| Error::invariant("B-link tenant registry lock poisoned"))?
            .shutting_down = true;
        let deadline = tokio::time::Instant::now() + self.open_shutdown_timeout;
        loop {
            let changed = self.open_changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            let opening_count = self
                .shards
                .lock()
                .map_err(|_| Error::invariant("B-link tenant registry lock poisoned"))?
                .opening
                .len();
            if opening_count == 0 {
                break;
            }
            tokio::time::timeout_at(deadline, changed)
                .await
                .map_err(|_| {
                    Error::overloaded("B-link tenant opens did not drain before shutdown")
                })?;
        }
        let shards = {
            let mut registry = self
                .shards
                .lock()
                .map_err(|_| Error::invariant("B-link tenant registry lock poisoned"))?;
            std::mem::take(&mut registry.shards)
                .into_values()
                .collect::<Vec<_>>()
        };
        let mut failure = None;
        for shard in shards {
            if let Err(error) = shard.shutdown().await {
                failure.get_or_insert(error);
            }
        }
        if tokio::time::timeout(
            self.open_shutdown_timeout,
            self.coordinator_activity.wait_until_idle(),
        )
        .await
        .is_err()
        {
            failure.get_or_insert_with(|| {
                Error::overloaded("B-link page reads did not drain before shutdown")
            });
        }
        self.shards
            .lock()
            .map_err(|_| Error::invariant("B-link tenant registry lock poisoned"))?
            .shutting_down = false;
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    async fn execute_request(
        &self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> Result<Response, Error> {
        #[cfg(feature = "blink-demand-paging")]
        let request_reservation = self.reserve_request(tenant, &request)?;
        #[cfg(feature = "blink-demand-paging")]
        let _response_reservation =
            self.reserve_response_budget(tenant, budget.max_response_bytes)?;
        match request {
            Request::Get { key } => {
                let state = match self.read_shard(tenant).await? {
                    Some(shard) => {
                        shard
                            .read_handle
                            .get_with_response_budget(&key, budget.max_response_bytes)
                            .await?
                    }
                    None => RevisionState::missing(dodb_core::Revision::ZERO),
                };
                let response = Response::Get(state);
                ensure_synthesized_response_budget(&response, budget.max_response_bytes)?;
                Ok(response)
            }
            Request::Put { key, value } => {
                let shard = self.open_shard(tenant).await?;
                let result = shard
                    .transact(
                        TransactionRequest::new(
                            Vec::new(),
                            vec![TransactionMutation::Put { key, value }],
                        ),
                        #[cfg(feature = "blink-demand-paging")]
                        request_reservation,
                    )
                    .await?;
                let response =
                    Response::Put(result.revision.ok_or_else(|| {
                        Error::invariant("Blink put transaction has no revision")
                    })?);
                ensure_synthesized_response_budget(&response, budget.max_response_bytes)?;
                Ok(response)
            }
            Request::Delete { key } => {
                let shard = self.open_shard(tenant).await?;
                let result = shard
                    .transact(
                        TransactionRequest::new(
                            Vec::new(),
                            vec![TransactionMutation::Delete { key }],
                        ),
                        #[cfg(feature = "blink-demand-paging")]
                        request_reservation,
                    )
                    .await?;
                let response =
                    Response::Delete(result.revision.ok_or_else(|| {
                        Error::invariant("Blink delete transaction has no revision")
                    })?);
                ensure_synthesized_response_budget(&response, budget.max_response_bytes)?;
                Ok(response)
            }
            Request::Query {
                pk,
                exclusive_after_sk,
                limit,
            } => {
                let rows = match self.read_shard(tenant).await? {
                    Some(shard) => {
                        shard
                            .read_handle
                            .query_with_response_budget(
                                &pk,
                                exclusive_after_sk.as_ref(),
                                limit,
                                budget.max_response_bytes,
                            )
                            .await?
                    }
                    None => Vec::new(),
                };
                let response = Response::Query(rows.into_iter().map(storage_document).collect());
                ensure_synthesized_response_budget(&response, budget.max_response_bytes)?;
                Ok(response)
            }
            Request::Scan {
                exclusive_after_key,
                limit,
            } => {
                let rows = match self.read_shard(tenant).await? {
                    Some(shard) => {
                        shard
                            .read_handle
                            .scan_with_response_budget(
                                exclusive_after_key.as_ref(),
                                limit,
                                budget.max_response_bytes,
                            )
                            .await?
                    }
                    None => Vec::new(),
                };
                let response = Response::Scan(rows.into_iter().map(storage_document).collect());
                ensure_synthesized_response_budget(&response, budget.max_response_bytes)?;
                Ok(response)
            }
            Request::Transact { request } => {
                request.validate()?;
                if request.mutations.is_empty() {
                    let Some(shard) = self.read_shard(tenant).await? else {
                        let actual = ObservedState::missing(dodb_core::Revision::ZERO);
                        for condition in &request.conditions {
                            if !condition_matches(condition, &actual) {
                                return Err(Error::conflict(TransactionConflict {
                                    key: condition.key().clone(),
                                    expected: condition.expectation(),
                                    actual,
                                }));
                            }
                        }
                        let response =
                            Response::Transact(TransactionOutcome::conditions_satisfied());
                        ensure_synthesized_response_budget(&response, budget.max_response_bytes)?;
                        return Ok(response);
                    };
                    let result = shard
                        .transact(
                            request,
                            #[cfg(feature = "blink-demand-paging")]
                            request_reservation,
                        )
                        .await?;
                    let response = Response::Transact(match result.commit_lsn {
                        Some(commit_lsn) => TransactionOutcome::committed(commit_lsn),
                        None => TransactionOutcome::conditions_satisfied(),
                    });
                    ensure_synthesized_response_budget(&response, budget.max_response_bytes)?;
                    return Ok(response);
                }
                let result = self
                    .open_shard(tenant)
                    .await?
                    .transact(
                        request,
                        #[cfg(feature = "blink-demand-paging")]
                        request_reservation,
                    )
                    .await?;
                let response = Response::Transact(match result.commit_lsn {
                    Some(commit_lsn) => TransactionOutcome::committed(commit_lsn),
                    None => TransactionOutcome::conditions_satisfied(),
                });
                ensure_synthesized_response_budget(&response, budget.max_response_bytes)?;
                Ok(response)
            }
        }
    }

    #[cfg(feature = "blink-demand-paging")]
    fn reserve_request(
        &self,
        tenant: TenantId,
        request: &Request,
    ) -> Result<Option<dodb_storage::blink::resource::Reservation>, Error> {
        let Some(resources) = &self.shared_resources else {
            return Ok(None);
        };
        let bytes = queued_request_allocation_bytes(request)?;
        resources
            .reserve(
                tenant.get(),
                dodb_storage::blink::resource::ResourceCategory::QueuedForeground,
                dodb_storage::blink::resource::ReservationClass::Foreground,
                bytes,
            )
            .map(Some)
            .map_err(|_| Error::overloaded("B-link queued request budget is exhausted"))
    }

    #[cfg(feature = "blink-demand-paging")]
    fn reserve_response_budget(
        &self,
        tenant: TenantId,
        maximum_response_bytes: usize,
    ) -> Result<Option<dodb_storage::blink::resource::Reservation>, Error> {
        let Some(resources) = &self.shared_resources else {
            return Ok(None);
        };
        let bytes = u64::try_from(maximum_response_bytes)
            .map_err(|_| Error::overloaded("B-link response reservation exceeds u64"))?;
        resources
            .reserve(
                tenant.get(),
                dodb_storage::blink::resource::ResourceCategory::LoadReconstruction,
                dodb_storage::blink::resource::ReservationClass::Foreground,
                bytes,
            )
            .map(Some)
            .map_err(|_| Error::overloaded("B-link response budget is exhausted"))
    }

    async fn read_shard(&self, tenant: TenantId) -> Result<Option<Arc<AsyncBlinkShard>>, Error> {
        let (shard, opening) = {
            let registry = self
                .shards
                .lock()
                .map_err(|_| Error::invariant("B-link tenant registry lock poisoned"))?;
            (
                registry.shards.get(&tenant).cloned(),
                registry.opening.contains(&tenant),
            )
        };
        if let Some(shard) = shard {
            return Ok(Some(shard));
        }
        if opening {
            return self.open_shard(tenant).await.map(Some);
        }
        let path = self.database_path(tenant);
        #[cfg(feature = "blink-demand-paging")]
        let paged_marker =
            dodb_storage::blink::paged_manifest::PagedManifestStore::has_format_marker(&path);
        #[cfg(not(feature = "blink-demand-paging"))]
        let paged_marker = false;
        if !path.exists() && !path.with_extension("wal").exists() && !paged_marker {
            return Ok(None);
        }
        Ok(Some(self.open_shard(tenant).await?))
    }

    async fn open_shard(&self, tenant: TenantId) -> Result<Arc<AsyncBlinkShard>, Error> {
        let mut waiter_permit = None;
        loop {
            let changed = self.open_changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            let reservation = {
                let mut registry = self
                    .shards
                    .lock()
                    .map_err(|_| Error::invariant("B-link tenant registry lock poisoned"))?;
                if registry.shutting_down {
                    return Err(Error::overloaded("tenant service is shutting down"));
                }
                if let Some(shard) = registry.shards.get(&tenant) {
                    return Ok(Arc::clone(shard));
                }
                if registry.opening.contains(&tenant) {
                    None
                } else {
                    if registry.shards.len() + registry.opening.len() >= self.max_open_shards {
                        return Err(Error::overloaded("open shard cache is full"));
                    }
                    registry.opening.insert(tenant);
                    Some(BlinkOpenAttempt {
                        tenant,
                        registry: Arc::clone(&self.shards),
                        changed: Arc::clone(&self.open_changed),
                        active: true,
                    })
                }
            };
            if let Some(attempt) = reservation {
                let worker_permit = self
                    .open_workers
                    .clone()
                    .try_acquire_owned()
                    .map_err(|_| Error::overloaded("B-link open worker capacity is full"))?;
                #[cfg(feature = "blink-demand-paging")]
                let metadata_reservation = match &self.shared_resources {
                    Some(resources) => Some(
                        resources
                            .reserve(
                                tenant.get(),
                                dodb_storage::blink::resource::ResourceCategory::Metadata,
                                dodb_storage::blink::resource::ReservationClass::Foreground,
                                self.open_tenant_metadata_bytes,
                            )
                            .map_err(|_| {
                                Error::overloaded("B-link shared metadata budget is exhausted")
                            })?,
                    ),
                    None => None,
                };
                #[cfg(test)]
                self.wait_for_test_open_gate(tenant).await?;
                let shard_id = ShardId::new(tenant.get());
                let database_path = self.database_path(tenant);
                let mut database_config = self.database_config.clone();
                database_config.tenant_id = tenant;
                database_config.shard_id = shard_id;
                database_config.database_uuid = database_uuid(tenant, shard_id);
                let wal_layout = self.wal_layout;
                let checkpoint_policy = self.checkpoint_policy;
                let checkpoint_retention_bytes = self.checkpoint_retention_bytes;
                let checkpoint_execution = self.checkpoint_execution;
                #[cfg(feature = "blink-demand-paging")]
                let paged_open_config = self.paged_open_config;
                #[cfg(feature = "blink-demand-paging")]
                let shared_resources = self.shared_resources.clone();
                let factory = self.fault_injector_factory.lock().await.clone();
                let open_factory = factory.clone();
                let open_result = tokio::task::spawn_blocking(move || {
                    let _worker_permit = worker_permit;
                    #[cfg(feature = "blink-demand-paging")]
                    if dodb_storage::blink::paged_manifest::PagedManifestStore::has_format_marker(
                        &database_path,
                    ) {
                        let Some(paged_config) = paged_open_config else {
                            return Err(Error::unsupported_format(
                                "paged Blink database requires the explicit paging service route",
                            ));
                        };
                        let Some(resources) = shared_resources else {
                            return Err(Error::invalid_request(
                                "paged Blink service route requires shared resource limits",
                            ));
                        };
                        return open_paged_blink_store(
                            database_path,
                            database_config,
                            tenant,
                            paged_config,
                            resources,
                        );
                    }
                    #[cfg(feature = "blink-demand-paging")]
                    if paged_open_config.is_some() {
                        return Err(Error::unsupported_format(
                            "paged Blink service route requires a paged format marker",
                        ));
                    }
                    open_blink_store(ResidentBlinkOpenConfig {
                        database_path,
                        database_config,
                        wal_layout,
                        checkpoint_policy,
                        checkpoint_retention_bytes,
                        factory: open_factory,
                        tenant,
                        shard_id,
                    })
                    .map(|store| OpenedBlinkStore::Resident(Box::new(store)))
                })
                .await
                .map_err(|error| Error::invariant(format!("B-link open worker failed: {error}")))?;
                let store = match open_result {
                    Ok(store) => store,
                    Err(error) => {
                        attempt.finish(None)?;
                        return Err(error);
                    }
                };
                let shard = match store {
                    OpenedBlinkStore::Resident(store) => {
                        #[cfg(all(test, feature = "blink-demand-paging"))]
                        let commit_test_gate = self
                            .commit_test_gate
                            .lock()
                            .ok()
                            .and_then(|current| current.clone());
                        Arc::new(AsyncBlinkShard::start(AsyncBlinkShardStartConfig {
                            store: *store,
                            coordinator_config: self.coordinator_config,
                            checkpoint_policy: self.checkpoint_policy,
                            physical_config: match wal_layout {
                                BlinkWalLayout::Legacy => None,
                                BlinkWalLayout::Segmented { .. }
                                | BlinkWalLayout::SegmentedBases { .. } => {
                                    Some(physical_checkpoint::PhysicalCheckpointConfig {
                                        execution: checkpoint_execution,
                                        injector_factory: factory
                                            .map(|factory| (factory, tenant, shard_id)),
                                    })
                                }
                            },
                            checkpoint_metrics: Arc::clone(&self.checkpoint_metrics),
                            activity: Arc::clone(&self.coordinator_activity),
                            tenant_id: tenant.get(),
                            #[cfg(feature = "blink-demand-paging")]
                            foreground_resources: self.shared_resources.clone(),
                            #[cfg(all(test, feature = "blink-demand-paging"))]
                            commit_test_gate,
                            #[cfg(feature = "blink-demand-paging")]
                            metadata_reservation,
                        }))
                    }
                    #[cfg(feature = "blink-demand-paging")]
                    OpenedBlinkStore::PagedReadOnly(read_handle) => {
                        #[cfg(test)]
                        let page_read_test_gate = self
                            .page_read_test_gate
                            .lock()
                            .ok()
                            .and_then(|current| current.clone());
                        Arc::new(AsyncBlinkShard::start_paged(
                            AsyncBlinkShardPagedStartConfig {
                                read_handle: Arc::new(*read_handle),
                                writable: None,
                                activity: Arc::clone(&self.coordinator_activity),
                                blocking_workers: Arc::clone(&self.open_workers),
                                #[cfg(test)]
                                read_test_gate: page_read_test_gate,
                                tenant_id: tenant.get(),
                                foreground_resources: self.shared_resources.clone(),
                                metadata_reservation,
                            },
                        ))
                    }
                    #[cfg(feature = "blink-demand-paging")]
                    OpenedBlinkStore::PagedWritable(opened) => {
                        #[cfg(all(test, feature = "blink-demand-paging"))]
                        let commit_test_gate = self
                            .commit_test_gate
                            .lock()
                            .ok()
                            .and_then(|current| current.clone());
                        let writable = PagedWritableShard::new(
                            PagedWritableShardOpenConfig {
                                writer: opened.writer,
                                manifest_store: opened.manifest_store,
                                checkpoint_directory: opened.checkpoint_directory,
                                wal_directory: opened.wal_directory,
                                wal_identity: opened.wal_identity,
                                tenant_id: tenant.get(),
                                checkpoint_policy: self.checkpoint_policy,
                                checkpoint_execution: self.checkpoint_execution,
                                checkpoint_metrics: Arc::clone(&self.checkpoint_metrics),
                                activity: Arc::clone(&self.coordinator_activity),
                                checkpoint_capacity_wait_timeout: self
                                    .paged_open_config
                                    .ok_or_else(|| {
                                        Error::invariant(
                                            "paged checkpoint capacity wait limit is missing",
                                        )
                                    })?
                                    .checkpoint_capacity_wait_timeout,
                                maximum_transaction_wal_append_bytes:
                                    dodb_storage::blink::maximum_paged_transaction_wal_append_bytes(
                                        self.paged_open_config
                                            .ok_or_else(|| {
                                                Error::invariant(
                                                    "paged checkpoint debt limits are missing",
                                                )
                                            })?
                                            .maximum_working_pages,
                                    )?,
                                checkpoint_debt_ledger: Arc::clone(
                                    self.checkpoint_debt_ledger.as_ref().ok_or_else(|| {
                                        Error::invariant("paged checkpoint debt ledger is missing")
                                    })?,
                                ),
                            },
                            #[cfg(all(test, feature = "blink-demand-paging"))]
                            commit_test_gate,
                        )?;
                        let read_handle = writable.snapshot()?;
                        Arc::new(AsyncBlinkShard::start_paged(
                            AsyncBlinkShardPagedStartConfig {
                                read_handle,
                                writable: Some(writable),
                                activity: Arc::clone(&self.coordinator_activity),
                                blocking_workers: Arc::clone(&self.open_workers),
                                #[cfg(test)]
                                read_test_gate: None,
                                tenant_id: tenant.get(),
                                foreground_resources: self.shared_resources.clone(),
                                metadata_reservation,
                            },
                        ))
                    }
                };
                attempt.finish(Some(Arc::clone(&shard)))?;
                return Ok(shard);
            }
            if waiter_permit.is_none() {
                waiter_permit = Some(
                    self.open_waiters
                        .clone()
                        .try_acquire_owned()
                        .map_err(|_| Error::overloaded("B-link open waiter capacity is full"))?,
                );
            }
            changed.await;
        }
    }

    #[cfg(test)]
    fn set_open_test_gate(&self, tenant: TenantId, gate: Arc<Semaphore>) {
        if let Ok(mut current) = self.open_test_gate.lock() {
            *current = Some((tenant, gate));
        }
    }

    #[cfg(all(test, feature = "blink-demand-paging"))]
    fn set_page_read_test_gate(&self, gate: Arc<OneShotTestGate>) {
        if let Ok(mut current) = self.page_read_test_gate.lock() {
            *current = Some(gate);
        }
    }

    #[cfg(all(test, feature = "blink-demand-paging"))]
    fn set_commit_test_gate(&self, gate: Arc<OneShotTestGate>) {
        if let Ok(mut current) = self.commit_test_gate.lock() {
            *current = Some(gate);
        }
    }

    #[cfg(test)]
    async fn wait_for_test_open_gate(&self, tenant: TenantId) -> Result<(), Error> {
        let gate = self
            .open_test_gate
            .lock()
            .map_err(|_| Error::invariant("B-link test gate lock poisoned"))?
            .as_ref()
            .filter(|(gated_tenant, _)| *gated_tenant == tenant)
            .map(|(_, gate)| Arc::clone(gate));
        if let Some(gate) = gate {
            gate.acquire()
                .await
                .map_err(|_| Error::invariant("B-link test gate closed"))?
                .forget();
        }
        Ok(())
    }

    fn database_path(&self, tenant: TenantId) -> PathBuf {
        self.data_dir.join("blink").join(format!(
            "tenant-{}-shard-{}.db",
            tenant.get(),
            tenant.get()
        ))
    }
}

enum OpenedBlinkStore {
    Resident(Box<dodb_storage::BlinkStore<ProductionFile, ProductionFile>>),
    #[cfg(feature = "blink-demand-paging")]
    PagedReadOnly(Box<dodb_storage::blink::PagedReadHandle>),
    #[cfg(feature = "blink-demand-paging")]
    PagedWritable(Box<OpenedPagedWritableStore>),
}

#[cfg(feature = "blink-demand-paging")]
struct OpenedPagedWritableStore {
    writer: dodb_storage::blink::PagedTransactionWriter<ProductionFile>,
    manifest_store: dodb_storage::blink::paged_manifest::PagedManifestStore,
    checkpoint_directory: PathBuf,
    wal_directory: PathBuf,
    wal_identity: dodb_storage::WalIdentity,
}

#[cfg(feature = "blink-demand-paging")]
fn next_paged_checkpoint_pack_number(directory: &std::path::Path) -> Result<u64, Error> {
    let mut maximum_number = 0u64;
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Some(number) = file_name
            .strip_prefix("base-")
            .and_then(|name| name.strip_suffix(".pages"))
            .and_then(|number| number.parse::<u64>().ok())
        else {
            continue;
        };
        maximum_number = maximum_number.max(number);
    }
    maximum_number
        .checked_add(1)
        .ok_or_else(|| Error::durability("paged checkpoint pack number is exhausted"))
}

#[cfg(feature = "blink-demand-paging")]
fn create_next_paged_wal_segment(
    directory: &std::path::Path,
    current_number: u64,
) -> Result<(u64, ProductionFile), Error> {
    let mut candidate = current_number
        .checked_add(1)
        .ok_or_else(|| Error::durability("paged recovery WAL segment number is exhausted"))?;
    loop {
        let path = directory.join(format!("segment-{candidate:020}.wal"));
        match std::fs::OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(path)
        {
            Ok(file) => return Ok((candidate, ProductionFile::from_file(file))),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                candidate = candidate.checked_add(1).ok_or_else(|| {
                    Error::durability("paged recovery WAL segment number is exhausted")
                })?;
            }
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(feature = "blink-demand-paging")]
fn open_paged_blink_store(
    database_path: PathBuf,
    database_config: DatabaseConfig,
    tenant: TenantId,
    paging_config: BlinkSharedResourceConfig,
    resources: dodb_storage::blink::resource::BlinkResourceManager,
) -> Result<OpenedBlinkStore, Error> {
    use dodb_storage::blink::paged_manifest::PagedManifestStore;
    use dodb_storage::blink::source_manager::{SourceManager, SourceManagerConfig};
    use dodb_storage::blink::version_index::{StoreIdentity, VersionIndex};

    let identity = StoreIdentity {
        database_uuid: database_config.database_uuid,
        wal_epoch: database_config.shard_epoch.get(),
    };
    PagedManifestStore::validate_format_marker(&database_path, identity)?;
    let selected_manifest =
        PagedManifestStore::validate_current_manifest(&database_path, identity)?;
    let sidecar_directory = PagedManifestStore::sidecar_directory(&database_path);
    let checkpoint_directory = sidecar_directory.join("packs");
    let wal_directory = sidecar_directory.join("segments");
    let scratch_directory = sidecar_directory.join("scratch");
    let index = Arc::new(VersionIndex::open(
        PagedManifestStore::version_index_directory(&database_path).as_path(),
        identity,
        resources.clone(),
        tenant.get(),
        paging_config.maximum_cached_index_nodes,
    )?);
    let sources = SourceManager::new(
        SourceManagerConfig {
            store_identity: identity,
            checkpoint_directory,
            wal_directory: wal_directory.clone(),
            scratch_directory,
            tenant_id: tenant.get(),
            maximum_open_handles: paging_config.maximum_open_source_handles,
            maximum_roots: paging_config.maximum_roots_per_tenant,
            handle_charge_bytes: paging_config.source_handle_charge_bytes,
            gc_reservation_bytes: paging_config.source_gc_reservation_bytes,
        },
        resources.clone(),
    )?;
    let recovery_wal_file_number = selected_manifest.recovery_wal_file_number;
    let recovery_wal_path =
        wal_directory.join(format!("segment-{recovery_wal_file_number:020}.wal"));
    let recovery_source =
        PagedManifestStore::recovery_source_identity(&recovery_wal_path, recovery_wal_file_number)?;
    let wal_identity = dodb_storage::WalIdentity::new(
        database_config.database_uuid,
        tenant,
        database_config.shard_id,
        database_config.shard_epoch,
    );
    let (manifest_store, read_handle) = PagedManifestStore::open_with_recovered_pages(
        &database_path,
        identity,
        dodb_storage::blink::paged_manifest::PagedRecoveryOpenConfig {
            index,
            tenant_id: tenant.get(),
            resources,
            sources,
            loader: paging_config.page_loader,
            header_length: |source: dodb_storage::blink::version_index::SourceIdentity| {
                Ok(
                    if source.kind == dodb_storage::blink::version_index::SourceFileKind::WalSegment
                    {
                        dodb_storage::wal::INIT_FRAME_SIZE
                    } else {
                        dodb_storage::PAGE_SIZE
                    },
                )
            },
            recovery_wal_path: recovery_wal_path.clone(),
            recovery_source,
            wal_identity: wal_identity.clone(),
        },
    )?;
    if paging_config.paged_writer_enabled {
        return Ok(OpenedBlinkStore::PagedWritable(Box::new(
            OpenedPagedWritableStore {
                writer: dodb_storage::blink::PagedTransactionWriter::open_from_recovered_manifest(
                    read_handle,
                    &manifest_store,
                    &recovery_wal_path,
                    wal_identity.clone(),
                    &database_config,
                    paging_config.maximum_working_pages,
                )?,
                manifest_store,
                checkpoint_directory: sidecar_directory.join("packs"),
                wal_directory,
                wal_identity,
            },
        )));
    }
    Ok(OpenedBlinkStore::PagedReadOnly(Box::new(read_handle)))
}

struct ResidentBlinkOpenConfig {
    database_path: PathBuf,
    database_config: DatabaseConfig,
    wal_layout: BlinkWalLayout,
    checkpoint_policy: BlinkCheckpointPolicy,
    checkpoint_retention_bytes: u64,
    factory: Option<FaultInjectorFactory>,
    tenant: TenantId,
    shard_id: ShardId,
}

fn open_blink_store(
    config: ResidentBlinkOpenConfig,
) -> Result<dodb_storage::BlinkStore<ProductionFile, ProductionFile>, Error> {
    let ResidentBlinkOpenConfig {
        database_path,
        database_config,
        wal_layout,
        checkpoint_policy,
        checkpoint_retention_bytes,
        factory,
        tenant,
        shard_id,
    } = config;
    let mut store = match wal_layout {
        BlinkWalLayout::Legacy => {
            dodb_storage::BlinkStore::<ProductionFile, ProductionFile>::open_path(
                &database_path,
                database_config,
            )?
        }
        BlinkWalLayout::Segmented { max_segment_bytes } => {
            dodb_storage::BlinkStore::<ProductionFile, ProductionFile>::open_segmented_path(
                &database_path,
                database_config,
                max_segment_bytes,
            )?
        }
        BlinkWalLayout::SegmentedBases {
            max_segment_bytes,
            coalesce,
        } => dodb_storage::BlinkStore::<ProductionFile, ProductionFile>::open_segmented_base_path(
            &database_path,
            database_config,
            max_segment_bytes,
            coalesce,
        )?,
    };
    if let Some(factory) = &factory {
        store.set_boxed_fault_injector(factory(tenant, shard_id));
    }
    if !matches!(wal_layout, BlinkWalLayout::Legacy)
        && checkpoint_policy != BlinkCheckpointPolicy::Disabled
    {
        store.set_checkpoint_retention_limit(checkpoint_retention_bytes)?;
    }
    store.enable_planned_execution();
    Ok(store)
}

impl AsyncBlinkShard {
    fn start(start_config: AsyncBlinkShardStartConfig) -> Self {
        let AsyncBlinkShardStartConfig {
            mut store,
            coordinator_config: config,
            checkpoint_policy,
            physical_config,
            checkpoint_metrics,
            activity,
            tenant_id,
            #[cfg(feature = "blink-demand-paging")]
            foreground_resources,
            #[cfg(all(test, feature = "blink-demand-paging"))]
            commit_test_gate,
            #[cfg(feature = "blink-demand-paging")]
            metadata_reservation,
        } = start_config;
        let read_handle = BlinkShardReadHandle::Resident(store.versioned_read_handle());
        let (request_tx, mut request_rx) =
            tokio::sync::mpsc::channel::<QueuedBlinkTransaction>(config.queue_capacity.max(1));
        let failed_message = Arc::new(StdMutex::new(None));
        let coordinator_failed_message = Arc::clone(&failed_message);
        let coordinator_activity = Arc::clone(&activity);
        #[cfg(all(test, feature = "blink-demand-paging"))]
        let coordinator_commit_test_gate = commit_test_gate;
        let coordinator_task = tokio::spawn(async move {
            if let Some(physical_config) = physical_config {
                physical_checkpoint::PhysicalCoordinator::new(
                    store,
                    physical_checkpoint::CoordinatorSettings {
                        grouping: config,
                        policy: checkpoint_policy,
                        tenant_id,
                        physical: physical_config,
                        metrics: checkpoint_metrics,
                        activity: coordinator_activity,
                        failed_message: coordinator_failed_message,
                        #[cfg(all(test, feature = "blink-demand-paging"))]
                        commit_test_gate: coordinator_commit_test_gate.clone(),
                    },
                    request_rx,
                )
                .run()
                .await;
                return;
            }
            let mut pending = None;
            loop {
                let first = match pending.take() {
                    Some(request) => Some(request),
                    None => request_rx.recv().await,
                };
                let Some(first) = first else { break };
                let _activity_guard = coordinator_activity.enter();
                let mut queued = vec![first];
                let mut request_bytes = transaction_size(&queued[0].request);
                tokio::task::yield_now().await;
                while queued.len() < config.max_group_requests.max(1) {
                    let Ok(next) = request_rx.try_recv() else {
                        break;
                    };
                    let next_bytes = transaction_size(&next.request);
                    if request_bytes.saturating_add(next_bytes) > config.max_group_bytes.max(1) {
                        pending = Some(next);
                        break;
                    }
                    request_bytes = request_bytes.saturating_add(next_bytes);
                    queued.push(next);
                }
                if queued.len() > 1
                    && pending.is_none()
                    && config.max_collection_delay > std::time::Duration::ZERO
                {
                    let deadline = tokio::time::sleep(config.max_collection_delay);
                    tokio::pin!(deadline);
                    while queued.len() < config.max_group_requests.max(1) {
                        tokio::select! {
                            next = request_rx.recv() => {
                                let Some(next) = next else { break };
                                let next_bytes = transaction_size(&next.request);
                                if request_bytes.saturating_add(next_bytes) > config.max_group_bytes.max(1) {
                                    pending = Some(next);
                                    break;
                                }
                                request_bytes = request_bytes.saturating_add(next_bytes);
                                queued.push(next);
                            }
                            _ = &mut deadline => break,
                        }
                    }
                }
                let requests = queued
                    .iter()
                    .map(|entry| entry.request.clone())
                    .collect::<Vec<_>>();
                #[cfg(all(test, feature = "blink-demand-paging"))]
                let commit_test_gate = coordinator_commit_test_gate.clone();
                let (updated_store, wal_before, wal_after, results) =
                    match tokio::task::spawn_blocking(move || {
                        #[cfg(all(test, feature = "blink-demand-paging"))]
                        wait_for_one_shot_test_gate(commit_test_gate);
                        let wal_before = store.wal_metrics().ok().flatten();
                        let results = store.apply_transaction_group(&requests);
                        let wal_after = store.wal_metrics().ok().flatten();
                        (store, wal_before, wal_after, results)
                    })
                    .await
                    {
                        Ok(result) => result,
                        Err(error) => {
                            let message = format!("Blink coordinator worker failed: {error}");
                            for entry in queued {
                                let _ = entry
                                    .response_tx
                                    .send(Err(Error::durability(message.clone())));
                            }
                            if let Ok(mut failed) = coordinator_failed_message.lock() {
                                *failed = Some(message.clone());
                            }
                            request_rx.close();
                            break;
                        }
                    };
                store = updated_store;
                match results {
                    Ok(results) => {
                        let successful_mutations = results
                            .iter()
                            .filter(|result| {
                                result
                                    .as_ref()
                                    .is_ok_and(|transaction| transaction.commit_lsn.is_some())
                            })
                            .count();
                        let contains_mutations = successful_mutations > 0;
                        let group_size = successful_mutations;
                        for (entry, result) in queued.into_iter().zip(results) {
                            let _ = entry.response_tx.send(result);
                        }
                        if let Ok(mut metrics) = checkpoint_metrics.lock() {
                            if metrics.wal_bytes_at_measurement_start.is_none() {
                                metrics.wal_bytes_at_measurement_start =
                                    wal_before.as_ref().map(|wal| wal.wal_bytes);
                            }
                            let syncs_before = wal_before.as_ref().map_or(0, |wal| wal.wal_syncs);
                            let syncs_after =
                                wal_after.as_ref().map_or(syncs_before, |wal| wal.wal_syncs);
                            let sync_delta = if syncs_after >= syncs_before {
                                syncs_after - syncs_before
                            } else {
                                syncs_after
                            };
                            metrics.wal_syncs = metrics.wal_syncs.saturating_add(sync_delta);
                            metrics.commit_wal_syncs =
                                metrics.commit_wal_syncs.saturating_add(sync_delta);
                            metrics.observe_wal_bytes(
                                wal_after.as_ref().map_or(0, |wal| wal.wal_bytes),
                                current_unix_nanos(),
                            );
                            metrics.wal_bytes_at_measurement_end = Some(metrics.wal_bytes_current);
                            metrics.wal_bytes_max =
                                metrics.wal_bytes_max.max(metrics.wal_bytes_current);
                            metrics.successful_mutation_transactions = metrics
                                .successful_mutation_transactions
                                .saturating_add(successful_mutations as u64);
                            if contains_mutations {
                                metrics.mutation_commit_groups =
                                    metrics.mutation_commit_groups.saturating_add(1);
                                let histogram_bucket = group_size.min(64);
                                if metrics.transaction_group_histogram.len() <= histogram_bucket {
                                    metrics
                                        .transaction_group_histogram
                                        .resize(histogram_bucket + 1, 0);
                                }
                                metrics.transaction_group_histogram[histogram_bucket] = metrics
                                    .transaction_group_histogram[histogram_bucket]
                                    .saturating_add(1);
                                metrics.transaction_group_count =
                                    metrics.transaction_group_count.saturating_add(1);
                            }
                        }
                        let threshold_exceeded = match checkpoint_policy {
                            BlinkCheckpointPolicy::Disabled => false,
                            BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes } => {
                                contains_mutations
                                    && wal_after
                                        .as_ref()
                                        .is_some_and(|wal| wal.wal_bytes >= threshold_bytes)
                            }
                        };
                        if threshold_exceeded {
                            let wal_bytes_before =
                                wal_after.as_ref().map_or(0, |wal| wal.wal_bytes);
                            let started_unix_nanos = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map_or(0, |duration| duration.as_nanos());
                            if let Ok(mut metrics) = checkpoint_metrics.lock() {
                                metrics.checkpoints_started =
                                    metrics.checkpoints_started.saturating_add(1);
                                metrics.checkpoint_request_queued();
                                metrics.checkpoint_request_started(0);
                            }
                            let (updated_store, checkpoint_result, wal_bytes_after) =
                                match tokio::task::spawn_blocking(move || {
                                    let result = store.checkpoint();
                                    let wal_bytes_after = store
                                        .wal_metrics()
                                        .ok()
                                        .flatten()
                                        .map_or(0, |wal| wal.wal_bytes);
                                    (store, result, wal_bytes_after)
                                })
                                .await
                                {
                                    Ok(result) => result,
                                    Err(error) => {
                                        let message =
                                            format!("B-link checkpoint worker failed: {error}");
                                        let ended_unix_nanos = SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .map_or(0, |duration| duration.as_nanos());
                                        let duration_nanos = ended_unix_nanos
                                            .saturating_sub(started_unix_nanos)
                                            .min(u64::MAX as u128)
                                            as u64;
                                        if let Ok(mut metrics) = checkpoint_metrics.lock() {
                                            metrics.checkpoints_failed =
                                                metrics.checkpoints_failed.saturating_add(1);
                                            metrics.checkpoint_total_nanos = metrics
                                                .checkpoint_total_nanos
                                                .saturating_add(duration_nanos);
                                            metrics.checkpoint_max_nanos =
                                                metrics.checkpoint_max_nanos.max(duration_nanos);
                                            metrics.failed_message = Some(message.clone());
                                            metrics.record_event(BlinkCheckpointEvent {
                                                phase: "physical_checkpoint_worker".to_owned(),
                                                started_unix_nanos,
                                                ended_unix_nanos,
                                                wal_bytes_before,
                                                wal_bytes_after: 0,
                                                reclaimed_bytes: 0,
                                                duration_nanos,
                                                succeeded: false,
                                                ..BlinkCheckpointEvent::default()
                                            });
                                        }
                                        if let Ok(mut failed) = coordinator_failed_message.lock() {
                                            *failed = Some(message.clone());
                                        }
                                        request_rx.close();
                                        if let Some(entry) = pending.take() {
                                            let _ = entry
                                                .response_tx
                                                .send(Err(Error::durability(message.clone())));
                                        }
                                        while let Ok(entry) = request_rx.try_recv() {
                                            let _ = entry
                                                .response_tx
                                                .send(Err(Error::durability(message.clone())));
                                        }
                                        break;
                                    }
                                };
                            store = updated_store;
                            let ended_unix_nanos = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map_or(0, |duration| duration.as_nanos());
                            let checkpoint_duration_nanos = ended_unix_nanos
                                .saturating_sub(started_unix_nanos)
                                .min(u64::MAX as u128)
                                as u64;
                            let succeeded = checkpoint_result.is_ok();
                            if let Ok(mut metrics) = checkpoint_metrics.lock() {
                                if succeeded {
                                    metrics.checkpoints_completed =
                                        metrics.checkpoints_completed.saturating_add(1);
                                } else {
                                    metrics.checkpoints_failed =
                                        metrics.checkpoints_failed.saturating_add(1);
                                }
                                metrics.checkpoint_total_nanos = metrics
                                    .checkpoint_total_nanos
                                    .saturating_add(checkpoint_duration_nanos);
                                metrics.checkpoint_max_nanos =
                                    metrics.checkpoint_max_nanos.max(checkpoint_duration_nanos);
                                metrics.observe_wal_bytes(wal_bytes_after, ended_unix_nanos);
                                metrics.wal_bytes_at_measurement_end = Some(wal_bytes_after);
                                metrics.wal_bytes_max = metrics.wal_bytes_max.max(wal_bytes_after);
                                if let Some(threshold_bytes) = metrics.threshold_bytes {
                                    metrics.threshold_excess_max_bytes = metrics
                                        .threshold_excess_max_bytes
                                        .max(wal_bytes_before.saturating_sub(threshold_bytes));
                                }
                                metrics.record_event(BlinkCheckpointEvent {
                                    phase: "data_flush_sync_metadata_publish_sync_wal_reset_sync"
                                        .to_owned(),
                                    started_unix_nanos,
                                    ended_unix_nanos,
                                    wal_bytes_before,
                                    wal_bytes_after,
                                    reclaimed_bytes: wal_bytes_before
                                        .saturating_sub(wal_bytes_after),
                                    duration_nanos: checkpoint_duration_nanos,
                                    succeeded,
                                    ..BlinkCheckpointEvent::default()
                                });
                                let syncs_after = store
                                    .wal_metrics()
                                    .ok()
                                    .flatten()
                                    .map_or(0, |wal| wal.wal_syncs);
                                metrics.checkpoint_wal_syncs = metrics
                                    .checkpoint_wal_syncs
                                    .saturating_add(syncs_after.saturating_sub(
                                        wal_after.as_ref().map_or(syncs_after, |wal| wal.wal_syncs),
                                    ));
                                metrics.wal_syncs = metrics
                                    .commit_wal_syncs
                                    .saturating_add(metrics.checkpoint_wal_syncs);
                                if !succeeded {
                                    metrics.failed_message =
                                        checkpoint_result.as_ref().err().map(ToString::to_string);
                                }
                            }
                            if let Err(error) = checkpoint_result {
                                let message =
                                    format!("B-link automatic checkpoint failed: {error}");
                                if let Ok(mut failed) = coordinator_failed_message.lock() {
                                    *failed = Some(message.clone());
                                }
                                request_rx.close();
                                if let Some(entry) = pending.take() {
                                    let _ = entry
                                        .response_tx
                                        .send(Err(Error::durability(message.clone())));
                                }
                                while let Ok(entry) = request_rx.try_recv() {
                                    let _ = entry
                                        .response_tx
                                        .send(Err(Error::durability(message.clone())));
                                }
                                break;
                            }
                        }
                    }
                    Err(error) => {
                        let message = error.to_string();
                        for entry in queued {
                            let _ = entry
                                .response_tx
                                .send(Err(Error::durability(message.clone())));
                        }
                    }
                }
            }
        });
        Self {
            read_handle,
            request_tx: Mutex::new(Some(request_tx)),
            coordinator_task: Mutex::new(Some(coordinator_task)),
            failed_message,
            activity,
            #[cfg(feature = "blink-demand-paging")]
            tenant_id,
            #[cfg(feature = "blink-demand-paging")]
            foreground_resources,
            #[cfg(feature = "blink-demand-paging")]
            _metadata_reservation: metadata_reservation,
        }
    }

    #[cfg(feature = "blink-demand-paging")]
    fn start_paged(start_config: AsyncBlinkShardPagedStartConfig) -> Self {
        let AsyncBlinkShardPagedStartConfig {
            read_handle,
            writable,
            activity,
            blocking_workers,
            #[cfg(test)]
            read_test_gate,
            tenant_id,
            foreground_resources,
            metadata_reservation,
        } = start_config;
        let read_handle = writable.is_none().then_some(read_handle);
        Self {
            read_handle: BlinkShardReadHandle::Paged {
                handle: read_handle,
                writable,
                blocking_workers,
                activity: Arc::clone(&activity),
                #[cfg(test)]
                read_test_gate,
            },
            request_tx: Mutex::new(None),
            coordinator_task: Mutex::new(None),
            failed_message: Arc::new(StdMutex::new(None)),
            activity,
            tenant_id,
            foreground_resources,
            _metadata_reservation: metadata_reservation,
        }
    }

    async fn transact(
        &self,
        request: TransactionRequest,
        #[cfg(feature = "blink-demand-paging")] queued_reservation: Option<
            dodb_storage::blink::resource::Reservation,
        >,
    ) -> Result<TransactionResult, Error> {
        if self.read_handle.is_read_only() {
            return Err(Error::unsupported_format(
                "paged Blink service is configured read-only",
            ));
        }
        if let Ok(failed) = self.failed_message.lock()
            && let Some(message) = failed.as_ref()
        {
            return Err(Error::durability(message.clone()));
        }
        #[cfg(feature = "blink-demand-paging")]
        let preparation_reservation = match &self.foreground_resources {
            Some(resources) => {
                let preparation_bytes = transaction_request_allocation_bytes(
                    &request,
                    std::mem::size_of::<TransactionRequest>()
                        .checked_mul(2)
                        .ok_or_else(|| {
                            Error::overloaded("B-link transaction preparation size overflows")
                        })?,
                )?;
                Some(
                    resources
                        .reserve(
                            self.tenant_id,
                            dodb_storage::blink::resource::ResourceCategory::WritePreparation,
                            dodb_storage::blink::resource::ReservationClass::Foreground,
                            preparation_bytes,
                        )
                        .map_err(|_| {
                            Error::overloaded("B-link transaction preparation budget is exhausted")
                        })?,
                )
            }
            None => None,
        };
        #[cfg(feature = "blink-demand-paging")]
        if let BlinkShardReadHandle::Paged {
            writable: Some(writable),
            blocking_workers,
            activity,
            ..
        } = &self.read_handle
        {
            return Arc::clone(writable)
                .transact(
                    request,
                    queued_reservation,
                    preparation_reservation,
                    Arc::clone(blocking_workers),
                    Arc::clone(activity),
                )
                .await;
        }
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        let send_result = self
            .request_tx
            .lock()
            .await
            .as_ref()
            .ok_or_else(|| Error::invariant("Blink coordinator is closed"))?
            .try_send(QueuedBlinkTransaction {
                request,
                response_tx,
                #[cfg(feature = "blink-demand-paging")]
                _queued_reservation: queued_reservation,
                #[cfg(feature = "blink-demand-paging")]
                _preparation_reservation: preparation_reservation,
            });
        if let Err(error) = send_result {
            if matches!(error, tokio::sync::mpsc::error::TrySendError::Closed(_))
                && let Ok(failed) = self.failed_message.lock()
                && let Some(message) = failed.as_ref()
            {
                return Err(Error::durability(message.clone()));
            }
            return Err(match error {
                tokio::sync::mpsc::error::TrySendError::Full(_) => {
                    Error::overloaded("Blink coordinator queue is full")
                }
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    Error::invariant("Blink coordinator is closed")
                }
            });
        }
        response_rx
            .await
            .map_err(|_| Error::invariant("Blink coordinator dropped a response"))?
    }

    async fn shutdown(&self) -> Result<(), Error> {
        #[cfg(feature = "blink-demand-paging")]
        if let BlinkShardReadHandle::Paged {
            writable: Some(writable),
            ..
        } = &self.read_handle
        {
            writable.cancel_checkpoints();
        }
        self.request_tx.lock().await.take();
        if let Some(task) = self.coordinator_task.lock().await.take() {
            task.await.map_err(|error| {
                Error::invariant(format!("Blink coordinator task failed: {error}"))
            })?;
        }
        if let Ok(failed) = self.failed_message.lock()
            && let Some(message) = failed.as_ref()
        {
            return Err(Error::durability(message.clone()));
        }
        Ok(())
    }
}

struct AsyncBlinkShardStartConfig {
    store: dodb_storage::BlinkStore<ProductionFile, ProductionFile>,
    coordinator_config: CoordinatorConfig,
    checkpoint_policy: BlinkCheckpointPolicy,
    physical_config: Option<physical_checkpoint::PhysicalCheckpointConfig>,
    checkpoint_metrics: Arc<StdMutex<BlinkWalCheckpointMetrics>>,
    activity: Arc<CoordinatorActivity>,
    tenant_id: u64,
    #[cfg(feature = "blink-demand-paging")]
    foreground_resources: Option<dodb_storage::blink::resource::BlinkResourceManager>,
    #[cfg(all(test, feature = "blink-demand-paging"))]
    commit_test_gate: Option<Arc<OneShotTestGate>>,
    #[cfg(feature = "blink-demand-paging")]
    metadata_reservation: Option<dodb_storage::blink::resource::Reservation>,
}

#[cfg(feature = "blink-demand-paging")]
struct AsyncBlinkShardPagedStartConfig {
    read_handle: Arc<dodb_storage::blink::PagedReadHandle>,
    writable: Option<Arc<PagedWritableShard>>,
    activity: Arc<CoordinatorActivity>,
    blocking_workers: Arc<Semaphore>,
    #[cfg(test)]
    read_test_gate: Option<Arc<OneShotTestGate>>,
    tenant_id: u64,
    foreground_resources: Option<dodb_storage::blink::resource::BlinkResourceManager>,
    metadata_reservation: Option<dodb_storage::blink::resource::Reservation>,
}

fn transaction_size(request: &TransactionRequest) -> usize {
    request
        .conditions
        .iter()
        .map(|condition| {
            condition.key().pk.as_bytes().len() + condition.key().sk.as_bytes().len() + 32
        })
        .chain(request.mutations.iter().map(|mutation| match mutation {
            TransactionMutation::Put { key, value } => {
                key.pk.as_bytes().len() + key.sk.as_bytes().len() + value.len() + 32
            }
            TransactionMutation::Delete { key } => {
                key.pk.as_bytes().len() + key.sk.as_bytes().len() + 32
            }
        }))
        .fold(0usize, usize::saturating_add)
}

#[cfg(feature = "blink-demand-paging")]
fn transaction_request_allocation_bytes(
    request: &TransactionRequest,
    additional_bytes: usize,
) -> Result<u64, Error> {
    let condition_slots = request
        .conditions
        .capacity()
        .checked_mul(std::mem::size_of::<TransactionCondition>());
    let mutation_slots = request
        .mutations
        .capacity()
        .checked_mul(std::mem::size_of::<TransactionMutation>());
    let mut allocated_bytes = std::mem::size_of::<TransactionRequest>()
        .checked_add(additional_bytes)
        .and_then(|bytes| bytes.checked_add(condition_slots?))
        .and_then(|bytes| bytes.checked_add(mutation_slots?))
        .ok_or_else(|| Error::overloaded("B-link transaction allocation size overflows"))?;
    for condition in &request.conditions {
        allocated_bytes = allocated_bytes
            .checked_add(condition.key().allocated_capacity_bytes())
            .ok_or_else(|| Error::overloaded("B-link transaction allocation size overflows"))?;
    }
    for mutation in &request.mutations {
        allocated_bytes = allocated_bytes
            .checked_add(mutation.key().allocated_capacity_bytes())
            .ok_or_else(|| Error::overloaded("B-link transaction allocation size overflows"))?;
        if let TransactionMutation::Put { value, .. } = mutation {
            allocated_bytes = allocated_bytes
                .checked_add(value.capacity())
                .ok_or_else(|| Error::overloaded("B-link transaction allocation size overflows"))?;
        }
    }
    u64::try_from(allocated_bytes)
        .map_err(|_| Error::overloaded("B-link transaction allocation size exceeds u64"))
}

#[cfg(feature = "blink-demand-paging")]
fn request_owned_allocation_bytes(request: &Request) -> Result<u64, Error> {
    let mut allocated_bytes = std::mem::size_of::<Request>();
    let add_bytes = |total: &mut usize, bytes: usize| -> Result<(), Error> {
        *total = total
            .checked_add(bytes)
            .ok_or_else(|| Error::overloaded("B-link request allocation size overflows"))?;
        Ok(())
    };
    match request {
        Request::Get { key } => add_bytes(&mut allocated_bytes, key.allocated_capacity_bytes())?,
        Request::Put { key, value } => {
            add_bytes(&mut allocated_bytes, key.allocated_capacity_bytes())?;
            add_bytes(&mut allocated_bytes, value.capacity())?;
            add_bytes(
                &mut allocated_bytes,
                std::mem::size_of::<TransactionRequest>()
                    .checked_add(std::mem::size_of::<TransactionMutation>())
                    .ok_or_else(|| Error::overloaded("B-link request allocation size overflows"))?,
            )?;
        }
        Request::Delete { key } => {
            add_bytes(&mut allocated_bytes, key.allocated_capacity_bytes())?;
            add_bytes(
                &mut allocated_bytes,
                std::mem::size_of::<TransactionRequest>()
                    .checked_add(std::mem::size_of::<TransactionMutation>())
                    .ok_or_else(|| Error::overloaded("B-link request allocation size overflows"))?,
            )?;
        }
        Request::Query {
            pk,
            exclusive_after_sk,
            ..
        } => {
            add_bytes(&mut allocated_bytes, pk.allocated_capacity_bytes())?;
            if let Some(sort_key) = exclusive_after_sk {
                add_bytes(&mut allocated_bytes, sort_key.allocated_capacity_bytes())?;
            }
        }
        Request::Scan {
            exclusive_after_key,
            ..
        } => {
            if let Some(key) = exclusive_after_key {
                add_bytes(&mut allocated_bytes, key.allocated_capacity_bytes())?;
            }
        }
        Request::Transact { request } => {
            let transaction_bytes = transaction_request_allocation_bytes(request, 0)?;
            let transaction_bytes = usize::try_from(transaction_bytes).map_err(|_| {
                Error::overloaded("B-link transaction allocation size exceeds usize")
            })?;
            add_bytes(&mut allocated_bytes, transaction_bytes)?;
        }
    }
    u64::try_from(allocated_bytes)
        .map_err(|_| Error::overloaded("B-link request allocation size exceeds u64"))
}

#[cfg(feature = "blink-demand-paging")]
fn queued_request_allocation_bytes(request: &Request) -> Result<u64, Error> {
    request_owned_allocation_bytes(request)?
        .checked_add(
            u64::try_from(std::mem::size_of::<QueuedBlinkTransaction>())
                .map_err(|_| Error::overloaded("B-link queue entry size exceeds u64"))?,
        )
        .ok_or_else(|| Error::overloaded("B-link queued request size overflows"))
}

impl DodbService for BlinkTenantService {
    fn execute<'service>(
        &'service self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> ServiceFuture<'service> {
        Box::pin(async move { self.execute_request(tenant, request, budget).await })
    }

    fn shutdown<'service>(&'service self) -> ShutdownFuture<'service> {
        Box::pin(async move { self.shutdown().await })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StorageMetricsSnapshot {
    pub database_file_bytes: u64,
    pub wal_file_bytes: u64,
    pub persisted_shards: u64,
    pub open_shards: u64,
}

pub type FaultInjectorFactory =
    Arc<dyn Fn(TenantId, ShardId) -> Box<dyn FaultInjector + Send> + Send + Sync>;

impl LocalTenantService {
    pub fn new(config: LocalTenantServiceConfig) -> Result<Self, Error> {
        if config.max_open_shards == 0 {
            return Err(Error::invalid_request("max_open_shards must be nonzero"));
        }
        #[cfg(feature = "blink-demand-paging")]
        if config.blink_shared_resources.is_some() {
            return Err(Error::invalid_request(
                "B-link shared paging resources are unsupported by the B-tree service",
            ));
        }
        if config.blink_wal_layout != BlinkWalLayout::Legacy
            || config.blink_checkpoint_execution != BlinkCheckpointExecution::Sync
        {
            return Err(Error::invalid_request(
                "B-link WAL layout is unsupported by the B-tree service",
            ));
        }
        if config.blink_checkpoint_policy != BlinkCheckpointPolicy::Disabled {
            return Err(Error::invalid_request(
                "B-link WAL checkpoint policy is unsupported by the B-tree service",
            ));
        }
        std::fs::create_dir_all(&config.data_dir)?;
        Ok(Self {
            data_dir: config.data_dir,
            max_open_shards: config.max_open_shards,
            database_config: config.database_config,
            coordinator_config: config.coordinator_config,
            resolver: Arc::new(OneTenantOneShard),
            shards: Mutex::new(BTreeMap::new()),
            fault_injector_factory: Mutex::new(None),
        })
    }

    pub fn with_resolver(
        config: LocalTenantServiceConfig,
        resolver: Arc<dyn TenantShardResolver>,
    ) -> Result<Self, Error> {
        let service = Self::new(config)?;
        Ok(Self {
            resolver,
            ..service
        })
    }

    pub fn set_fault_injector_factory(&mut self, factory: FaultInjectorFactory) {
        if let Ok(mut current) = self.fault_injector_factory.try_lock() {
            *current = Some(factory);
        }
    }

    pub async fn shutdown(&self) {
        let shards = {
            let mut shards = self.shards.lock().await;
            let open_shards = shards.values().cloned().collect::<Vec<_>>();
            shards.clear();
            open_shards
        };
        for shard in shards {
            let _ = shard.shutdown().await;
        }
    }

    /// Checkpoints every currently open tenant shard. This is intentionally a
    /// small administrative hook for validation tooling; the client protocol
    /// does not expose checkpoints.
    pub async fn checkpoint_all(&self) -> Result<Vec<(TenantId, CheckpointReport)>, Error> {
        let shards = self.open_shard_handles().await;
        let mut reports = Vec::with_capacity(shards.len());
        for (tenant, shard) in shards {
            reports.push((tenant, shard.checkpoint().await?));
        }
        Ok(reports)
    }

    /// Runs the storage invariant checker for every currently open shard.
    /// Callers should use this after quiescing request traffic when they need a
    /// stable logical verification point.
    pub async fn check_invariants_all(&self) -> Result<Vec<(TenantId, InvariantReport)>, Error> {
        let shards = self.open_shard_handles().await;
        let mut reports = Vec::with_capacity(shards.len());
        for (tenant, shard) in shards {
            reports.push((tenant, shard.check_invariants().await?));
        }
        Ok(reports)
    }

    pub async fn storage_metrics_snapshot(&self) -> std::io::Result<StorageMetricsSnapshot> {
        let data_dir = self.data_dir.clone();
        let open_shards = self.shards.lock().await.len() as u64;
        tokio::task::spawn_blocking(move || collect_storage_metrics(&data_dir, open_shards))
            .await
            .map_err(std::io::Error::other)?
    }

    async fn open_shard_handles(
        &self,
    ) -> Vec<(TenantId, Arc<AsyncShard<ProductionFile, ProductionFile>>)> {
        self.shards
            .lock()
            .await
            .iter()
            .map(|(tenant, shard)| (*tenant, Arc::clone(shard)))
            .collect()
    }

    async fn execute_request(
        &self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> Result<Response, Error> {
        match request {
            Request::Get { key } => {
                let state = match self.read_shard(tenant).await? {
                    Some(shard) => match shard
                        .execute_with_response_budget(
                            BatchRequest::Get { key },
                            budget.max_response_bytes,
                        )
                        .await?
                    {
                        BatchResponse::Get(state) => state,
                        _ => return Err(Error::invariant("local get returned the wrong response")),
                    },
                    None => {
                        let response =
                            Response::Get(RevisionState::missing(dodb_core::Revision::ZERO));
                        ensure_synthesized_response_budget(&response, budget.max_response_bytes)?;
                        return Ok(response);
                    }
                };
                Ok(Response::Get(state))
            }
            Request::Put { key, value } => {
                let shard = self.open_shard(tenant).await?;
                let revision = match shard
                    .execute_with_response_budget(
                        BatchRequest::Put { key, value },
                        budget.max_response_bytes,
                    )
                    .await?
                {
                    BatchResponse::Put(revision) => revision,
                    _ => return Err(Error::invariant("local put returned the wrong response")),
                };
                Ok(Response::Put(revision))
            }
            Request::Delete { key } => {
                let shard = self.open_shard(tenant).await?;
                let revision = match shard
                    .execute_with_response_budget(
                        BatchRequest::Delete { key },
                        budget.max_response_bytes,
                    )
                    .await?
                {
                    BatchResponse::Delete(revision) => revision,
                    _ => return Err(Error::invariant("local delete returned the wrong response")),
                };
                Ok(Response::Delete(revision))
            }
            Request::Query {
                pk,
                exclusive_after_sk,
                limit,
            } => {
                let rows = match self.read_shard(tenant).await? {
                    Some(shard) => match shard
                        .execute_with_response_budget(
                            BatchRequest::Query {
                                pk,
                                exclusive_after_sk,
                                limit,
                            },
                            budget.max_response_bytes,
                        )
                        .await?
                    {
                        BatchResponse::Query(rows) => rows,
                        _ => {
                            return Err(Error::invariant(
                                "local query returned the wrong response",
                            ));
                        }
                    },
                    None => {
                        let response = Response::Query(Vec::new());
                        ensure_synthesized_response_budget(&response, budget.max_response_bytes)?;
                        return Ok(response);
                    }
                };
                Ok(Response::Query(
                    rows.into_iter().map(storage_document).collect(),
                ))
            }
            Request::Scan {
                exclusive_after_key,
                limit,
            } => {
                let rows = match self.read_shard(tenant).await? {
                    Some(shard) => match shard
                        .execute_with_response_budget(
                            BatchRequest::Scan {
                                exclusive_after_key,
                                limit,
                            },
                            budget.max_response_bytes,
                        )
                        .await?
                    {
                        BatchResponse::Scan(rows) => rows,
                        _ => {
                            return Err(Error::invariant("local scan returned the wrong response"));
                        }
                    },
                    None => {
                        let response = Response::Scan(Vec::new());
                        ensure_synthesized_response_budget(&response, budget.max_response_bytes)?;
                        return Ok(response);
                    }
                };
                Ok(Response::Scan(
                    rows.into_iter().map(storage_document).collect(),
                ))
            }
            Request::Transact { request } => {
                self.execute_transaction(tenant, request, budget).await
            }
        }
    }

    async fn execute_transaction(
        &self,
        tenant: TenantId,
        request: TransactionRequest,
        _budget: ExecutionBudget,
    ) -> Result<Response, Error> {
        request.validate()?;
        let commit_lsn =
            if request.mutations.is_empty() {
                match self.read_shard(tenant).await? {
                    Some(shard) => shard.execute_transaction(request).await?.commit_lsn,
                    None => {
                        let actual = ObservedState::missing(dodb_core::Revision::ZERO);
                        for condition in &request.conditions {
                            if !condition_matches(condition, &actual) {
                                return Err(Error::conflict(TransactionConflict {
                                    key: condition.key().clone(),
                                    expected: condition.expectation(),
                                    actual,
                                }));
                            }
                        }
                        None
                    }
                }
            } else {
                let result = self
                    .open_shard(tenant)
                    .await?
                    .execute_transaction(request)
                    .await?;
                Some(result.commit_lsn.ok_or_else(|| {
                    Error::invariant("mutation transaction returned no commit LSN")
                })?)
            };
        Ok(Response::Transact(match commit_lsn {
            Some(commit_lsn) => TransactionOutcome::committed(commit_lsn),
            None => TransactionOutcome::conditions_satisfied(),
        }))
    }

    async fn read_shard(
        &self,
        tenant: TenantId,
    ) -> Result<Option<Arc<AsyncShard<ProductionFile, ProductionFile>>>, Error> {
        let shard_id = self.resolver.resolve(tenant);
        let database_path = self.database_path(tenant, shard_id);
        let wal_path = database_path.with_extension("wal");
        if !database_path.exists() && !wal_path.exists() {
            return Ok(None);
        }
        Ok(Some(self.open_shard(tenant).await?))
    }

    async fn open_shard(
        &self,
        tenant: TenantId,
    ) -> Result<Arc<AsyncShard<ProductionFile, ProductionFile>>, Error> {
        let mut shards = self.shards.lock().await;
        if let Some(shard) = shards.get(&tenant) {
            return Ok(Arc::clone(shard));
        }
        if shards.len() >= self.max_open_shards {
            return Err(Error::overloaded("open shard cache is full"));
        }
        let shard_id = self.resolver.resolve(tenant);
        let database_path = self.database_path(tenant, shard_id);
        std::fs::create_dir_all(&self.data_dir)?;
        let mut database_config = self.database_config.clone();
        database_config.tenant_id = tenant;
        database_config.shard_id = shard_id;
        database_config.database_uuid = database_uuid(tenant, shard_id);
        let mut store = BTreeStore::<ProductionFile, ProductionFile>::open_path(
            &database_path,
            database_config,
        )?;
        if let Some(factory) = self.fault_injector_factory.lock().await.clone() {
            store.set_boxed_fault_injector(factory(tenant, shard_id));
        }
        let shard = Arc::new(AsyncShard::start_with_config(
            store,
            self.coordinator_config,
        ));
        shards.insert(tenant, Arc::clone(&shard));
        Ok(shard)
    }

    fn database_path(&self, tenant: TenantId, shard: ShardId) -> PathBuf {
        self.data_dir
            .join(format!("tenant-{}-shard-{}.db", tenant.get(), shard.get()))
    }
}

fn collect_storage_metrics(
    data_dir: &std::path::Path,
    open_shards: u64,
) -> std::io::Result<StorageMetricsSnapshot> {
    let mut metrics = StorageMetricsSnapshot {
        open_shards,
        ..StorageMetricsSnapshot::default()
    };
    let mut persisted_shards = HashSet::new();
    for entry in std::fs::read_dir(data_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let Some(extension) = path.extension().and_then(std::ffi::OsStr::to_str) else {
            continue;
        };
        let Some(file_name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
            continue;
        };
        if !file_name.starts_with("tenant-") {
            continue;
        }
        let file_bytes = entry.metadata()?.len();
        match extension {
            "db" => {
                metrics.database_file_bytes =
                    metrics.database_file_bytes.saturating_add(file_bytes);
                if let Some(stem) = path.file_stem() {
                    persisted_shards.insert(stem.to_os_string());
                }
            }
            "wal" => {
                metrics.wal_file_bytes = metrics.wal_file_bytes.saturating_add(file_bytes);
                if let Some(stem) = path.file_stem() {
                    persisted_shards.insert(stem.to_os_string());
                }
            }
            _ => {}
        }
    }
    metrics.persisted_shards = persisted_shards.len() as u64;
    Ok(metrics)
}

impl DodbService for LocalTenantService {
    fn execute<'service>(
        &'service self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> ServiceFuture<'service> {
        Box::pin(async move { self.execute_request(tenant, request, budget).await })
    }

    fn shutdown<'service>(&'service self) -> ShutdownFuture<'service> {
        Box::pin(async move { self.shutdown().await })
    }
}

fn storage_document(document: dodb_storage::Document) -> Document {
    Document {
        key: document.key,
        value: document.value,
        revision: document.revision,
    }
}

fn ensure_synthesized_response_budget(response: &Response, maximum: usize) -> Result<(), Error> {
    let limits = ProtocolLimits {
        max_response_frame_size: maximum,
        ..ProtocolLimits::default()
    };
    encode_response(&ResponseEnvelope::Success(response.clone()), limits)
        .map(|_| ())
        .map_err(|error| {
            Error::response_too_large(format!(
                "synthesized response exceeds the configured response limit: {error}"
            ))
        })
}

/// Returns the deterministic logical identity for a `(tenant, shard)` pair.
///
/// The storage UUID is only 128 bits, so it is a domain-separated digest rather
/// than a field concatenation. Both complete 64-bit identifiers participate;
/// this function deliberately does not identify a physical incarnation. A
/// future delete/recreate protocol can replace this with a persisted UUID
/// without changing the distinct tenant/shard fields in the storage identity.
fn database_uuid(tenant: TenantId, shard: ShardId) -> [u8; 16] {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut first = FNV_OFFSET;
    let mut second = FNV_OFFSET ^ 0x9e3779b97f4a7c15;
    let mut input = Vec::with_capacity(24);
    input.extend_from_slice(b"DODB logical database identity");
    input.extend_from_slice(&tenant.get().to_be_bytes());
    input.extend_from_slice(&shard.get().to_be_bytes());
    for (index, byte) in input.into_iter().enumerate() {
        first ^= u64::from(byte);
        first = first.wrapping_mul(FNV_PRIME);
        second ^= u64::from(byte).wrapping_add(index as u64);
        second = second.rotate_left(5).wrapping_mul(FNV_PRIME);
    }
    let mut uuid = [0u8; 16];
    uuid[..8].copy_from_slice(&first.to_be_bytes());
    uuid[8..].copy_from_slice(&second.to_be_bytes());
    uuid
}

fn condition_matches(condition: &TransactionCondition, actual: &ObservedState) -> bool {
    match condition {
        TransactionCondition::RevisionEquals {
            expected_revision, ..
        } => actual.revision() == *expected_revision,
        TransactionCondition::Exists { .. } => !actual.is_missing(),
        TransactionCondition::NotExists { .. } => actual.is_missing(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodb_core::DocumentKey;

    #[test]
    fn checkpoint_debt_metrics_track_excess_time_and_request_wait() {
        let mut metrics = BlinkWalCheckpointMetrics {
            threshold_bytes: Some(100),
            ..BlinkWalCheckpointMetrics::default()
        };

        metrics.observe_wal_bytes(100, 1_000);
        assert_eq!(metrics.checkpoint_debt_bytes_current, 0);
        assert_eq!(metrics.threshold_excess_since_unix_nanos, None);
        metrics.observe_wal_bytes(125, 2_000);
        assert_eq!(metrics.checkpoint_debt_bytes_current, 25);
        assert_eq!(metrics.checkpoint_debt_bytes_max, 25);
        assert_eq!(metrics.threshold_excess_since_unix_nanos, Some(2_000));
        assert_eq!(
            metrics.snapshot_at(6_000).threshold_excess_current_nanos,
            4_000
        );
        metrics.observe_wal_bytes(100, 7_000);
        assert_eq!(metrics.threshold_excess_total_nanos, 5_000);
        assert_eq!(metrics.threshold_excess_current_nanos, 5_000);
        assert_eq!(metrics.threshold_excess_since_unix_nanos, None);

        metrics.checkpoint_request_queued();
        assert_eq!(metrics.checkpoint_requests_pending, 1);
        metrics.checkpoint_request_started(3_000);
        assert_eq!(metrics.checkpoint_requests_queued, 1);
        assert_eq!(metrics.checkpoint_requests_pending, 0);
        assert_eq!(metrics.checkpoint_request_wait_nanos_total, 3_000);
        assert_eq!(metrics.checkpoint_request_wait_nanos_max, 3_000);
        metrics.checkpoint_request_queued();
        metrics.checkpoint_request_cancelled(4_000);
        assert_eq!(metrics.checkpoint_requests_cancelled, 1);
        assert_eq!(metrics.checkpoint_request_wait_nanos_total, 7_000);
    }

    #[test]
    fn tenant_checkpoint_metrics_aggregate_debt_wait_and_service() {
        let mut metrics = BlinkWalCheckpointMetrics {
            threshold_bytes: Some(100),
            ..BlinkWalCheckpointMetrics::default()
        };
        metrics.observe_tenant_wal_bytes(1, 150, 1_000);
        metrics.observe_tenant_wal_bytes(2, 125, 2_000);
        assert_eq!(metrics.checkpoint_debt_bytes_current, 75);
        assert_eq!(metrics.checkpoint_debt_bytes_max, 75);
        assert_eq!(metrics.per_tenant[&1].checkpoint_debt_bytes_current, 50);
        assert_eq!(metrics.per_tenant[&2].checkpoint_debt_bytes_current, 25);
        assert_eq!(
            metrics.snapshot_at(6_000).threshold_excess_current_nanos,
            9_000
        );

        metrics.tenant_checkpoint_request_queued(2);
        metrics.tenant_checkpoint_request_started(2, 800);
        metrics.tenant_checkpoint_slice_completed(2);
        assert_eq!(metrics.per_tenant[&2].checkpoint_requests_queued, 1);
        assert_eq!(metrics.per_tenant[&2].checkpoint_requests_pending, 0);
        assert_eq!(
            metrics.per_tenant[&2].checkpoint_request_wait_nanos_max,
            800
        );
        assert_eq!(metrics.per_tenant[&2].service_slices, 1);

        metrics.observe_tenant_wal_bytes(1, 100, 7_000);
        assert_eq!(metrics.per_tenant[&1].threshold_excess_total_nanos, 6_000);
        assert_eq!(metrics.checkpoint_debt_bytes_current, 25);
    }

    #[test]
    #[cfg(feature = "blink-demand-paging")]
    fn checkpoint_debt_admission_enforces_global_and_tenant_limits() {
        let ledger = CheckpointDebtLedger::new(100, 60);
        let reservation = ledger.reserve(1, 50).unwrap();
        assert!(ledger.reserve(1, 11).is_err());
        assert!(ledger.reserve(2, 51).is_err());
        reservation.commit(40).unwrap();
        assert!(ledger.reserve(1, 21).is_err());
        ledger.set_tenant_debt(1, 0).unwrap();
        let released_after_failure = ledger.reserve(2, 60).unwrap();
        drop(released_after_failure);
        let released_after_cancellation = ledger.reserve(1, 60).unwrap();
        drop(released_after_cancellation);
        ledger.set_tenant_debt(1, 0).unwrap();
        ledger.set_tenant_debt(2, 0).unwrap();
        assert!(ledger.reserve(2, 100).is_err());
    }

    #[test]
    #[cfg(feature = "blink-demand-paging")]
    fn abandoned_checkpoint_request_clears_tenant_pending_metrics() {
        let metrics = Arc::new(StdMutex::new(BlinkWalCheckpointMetrics::default()));
        metrics.lock().unwrap().tenant_checkpoint_request_queued(5);
        let request_guard = CheckpointRequestMetricsGuard {
            metrics: Arc::clone(&metrics),
            tenant_id: 5,
            queued_at: Instant::now(),
            pending: true,
        };
        drop(request_guard);

        let metrics = metrics.lock().unwrap();
        assert_eq!(metrics.per_tenant[&5].checkpoint_requests_pending, 0);
        assert_eq!(metrics.per_tenant[&5].checkpoint_requests_cancelled, 1);
    }

    fn blink_test_directory() -> tempfile::TempDir {
        match std::env::var_os("DODB_TEST_DATA_DIR") {
            Some(path) => tempfile::TempDir::new_in(path).unwrap(),
            None => tempfile::TempDir::new().unwrap(),
        }
    }

    #[cfg(feature = "blink-demand-paging")]
    fn constrained_blink_shared_resources() -> BlinkSharedResourceConfig {
        BlinkSharedResourceConfig {
            memory: dodb_storage::blink::resource::ResourceLimits {
                global_bytes: 16_384,
                per_tenant_bytes: 8_192,
                progress_pool_bytes: 1_024,
                max_tenants: 3,
                max_pending_waiters: 4,
                global_category_bytes: [16_384; 7],
                per_tenant_category_bytes: [8_192; 7],
            },
            disk: dodb_storage::blink::resource::DiskResourceLimits {
                global_bytes: 1_048_576,
                per_tenant_bytes: 524_288,
                max_tenants: 3,
                global_category_bytes: [1_048_576; 5],
                per_tenant_category_bytes: [524_288; 5],
            },
            open_tenant_metadata_bytes: 7_000,
            paged_read_only: false,
            paged_writer_enabled: false,
            maximum_working_pages: 4,
            page_loader: dodb_storage::blink::page_cache::PageLoaderConfig {
                maximum_cache_bytes: 8 * dodb_storage::PAGE_SIZE as u64,
                minimum_decoded_page_bytes: dodb_storage::PAGE_SIZE as u64,
                maximum_load_reconstruction_bytes: 16 * dodb_storage::PAGE_SIZE as u64,
                maximum_concurrent_loads: 4,
                maximum_followers: 32,
            },
            maximum_cached_index_nodes: 128,
            maximum_open_source_handles: 64,
            maximum_roots_per_tenant: 64,
            source_handle_charge_bytes: 256,
            source_gc_reservation_bytes: 65_536,
            checkpoint_capacity_wait_timeout: std::time::Duration::from_secs(30),
            maximum_checkpoint_debt_bytes_global: 64 * 1024 * 1024,
            maximum_checkpoint_debt_bytes_per_tenant: 32 * 1024 * 1024,
        }
    }

    #[cfg(feature = "blink-demand-paging")]
    fn foreground_blink_shared_resources() -> BlinkSharedResourceConfig {
        let mut resources = constrained_blink_shared_resources();
        resources.memory.global_bytes = 131_072;
        resources.memory.per_tenant_bytes = 65_536;
        resources.memory.progress_pool_bytes = 4_096;
        resources.memory.max_tenants = 2;
        resources.memory.max_pending_waiters = 4;
        resources.memory.global_category_bytes = [131_072; 7];
        resources.memory.per_tenant_category_bytes = [65_536; 7];
        resources.open_tenant_metadata_bytes = 512;
        resources.paged_read_only = false;
        resources
    }

    #[test]
    #[cfg(feature = "blink-demand-paging")]
    fn shared_blink_admission_rejects_zero_queue_and_group_limits() {
        let directory = blink_test_directory();
        for invalid_limit in 0..3 {
            let mut config = LocalTenantServiceConfig {
                data_dir: directory.path().join(format!("invalid-{invalid_limit}")),
                max_open_shards: 1,
                blink_open_waiter_limit: 4,
                blink_shared_resources: Some(foreground_blink_shared_resources()),
                ..LocalTenantServiceConfig::default()
            };
            match invalid_limit {
                0 => config.coordinator_config.queue_capacity = 0,
                1 => config.coordinator_config.max_group_requests = 0,
                _ => config.coordinator_config.max_group_bytes = 0,
            }
            let data_dir = config.data_dir.clone();
            assert!(BlinkTenantService::new(config).is_err());
            assert!(!data_dir.exists());
        }
    }

    #[test]
    #[cfg(feature = "blink-demand-paging")]
    fn paged_open_rejects_checkpoint_debt_cap_below_one_worst_case_transaction() {
        let directory = blink_test_directory();
        let data_dir = directory.path().join("undersized-checkpoint-debt-cap");
        let mut resources = writable_paged_blink_shared_resources();
        let maximum_transaction_bytes =
            dodb_storage::blink::maximum_paged_transaction_wal_append_bytes(
                resources.maximum_working_pages,
            )
            .unwrap();
        resources.maximum_checkpoint_debt_bytes_per_tenant =
            maximum_transaction_bytes.saturating_sub(1);
        resources.maximum_checkpoint_debt_bytes_global = maximum_transaction_bytes;
        assert!(
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: data_dir.clone(),
                blink_shared_resources: Some(resources),
                ..LocalTenantServiceConfig::default()
            })
            .is_err()
        );
        assert!(!data_dir.exists());
    }

    #[cfg(feature = "blink-demand-paging")]
    fn paged_blink_shared_resources() -> BlinkSharedResourceConfig {
        BlinkSharedResourceConfig {
            memory: dodb_storage::blink::resource::ResourceLimits {
                global_bytes: 32 * 1024 * 1024,
                per_tenant_bytes: 16 * 1024 * 1024,
                progress_pool_bytes: 1024 * 1024,
                max_tenants: 2,
                max_pending_waiters: 4,
                global_category_bytes: [32 * 1024 * 1024; 7],
                per_tenant_category_bytes: [16 * 1024 * 1024; 7],
            },
            disk: dodb_storage::blink::resource::DiskResourceLimits {
                global_bytes: 256 * 1024 * 1024,
                per_tenant_bytes: 128 * 1024 * 1024,
                max_tenants: 2,
                global_category_bytes: [256 * 1024 * 1024; 5],
                per_tenant_category_bytes: [128 * 1024 * 1024; 5],
            },
            open_tenant_metadata_bytes: 4096,
            paged_read_only: true,
            paged_writer_enabled: false,
            maximum_working_pages: 8,
            page_loader: dodb_storage::blink::page_cache::PageLoaderConfig {
                maximum_cache_bytes: 2 * 1024 * 1024,
                minimum_decoded_page_bytes: dodb_storage::PAGE_SIZE as u64,
                maximum_load_reconstruction_bytes: 2 * 1024 * 1024,
                maximum_concurrent_loads: 4,
                maximum_followers: 32,
            },
            maximum_cached_index_nodes: 256,
            maximum_open_source_handles: 64,
            maximum_roots_per_tenant: 64,
            source_handle_charge_bytes: 256,
            source_gc_reservation_bytes: 65_536,
            checkpoint_capacity_wait_timeout: std::time::Duration::from_secs(30),
            maximum_checkpoint_debt_bytes_global: 64 * 1024 * 1024,
            maximum_checkpoint_debt_bytes_per_tenant: 32 * 1024 * 1024,
        }
    }

    #[cfg(feature = "blink-demand-paging")]
    fn writable_paged_blink_shared_resources() -> BlinkSharedResourceConfig {
        let mut resources = paged_blink_shared_resources();
        resources.paged_read_only = false;
        resources.paged_writer_enabled = true;
        resources
    }

    #[cfg(feature = "blink-demand-paging")]
    fn create_empty_paged_service_fixture(database_path: &std::path::Path, tenant: TenantId) {
        use dodb_storage::blink::paged_manifest::{PagedManifest, PagedManifestStore};
        use dodb_storage::blink::paged_recovery::build_checkpoint_index;
        use dodb_storage::blink::source_manager::{SourceManager, SourceManagerConfig};
        use dodb_storage::blink::version_index::{
            SourceFileKind, StoreIdentity, VersionDescriptor, VersionIndex, VersionLocator,
            VersionToken, WalFrameLocator,
        };

        let shard_id = ShardId::new(tenant.get());
        let database_config = DatabaseConfig {
            tenant_id: tenant,
            shard_id,
            database_uuid: database_uuid(tenant, shard_id),
            ..DatabaseConfig::default()
        };
        let store_identity = StoreIdentity {
            database_uuid: database_config.database_uuid,
            wal_epoch: database_config.shard_epoch.get(),
        };
        let wal_identity = dodb_storage::WalIdentity::new(
            database_config.database_uuid,
            tenant,
            shard_id,
            database_config.shard_epoch,
        );
        PagedManifestStore::prepare_format(database_path, store_identity).unwrap();
        let sidecar_directory = PagedManifestStore::sidecar_directory(database_path);
        let pack_directory = sidecar_directory.join("packs");
        let wal_directory = sidecar_directory.join("segments");
        let scratch_directory = sidecar_directory.join("scratch");
        std::fs::create_dir_all(&pack_directory).unwrap();
        std::fs::create_dir_all(&wal_directory).unwrap();
        let resource_config = paged_blink_shared_resources();
        let resources = dodb_storage::blink::resource::BlinkResourceManager::new_with_disk_limits(
            resource_config.memory,
            resource_config.disk,
        )
        .unwrap();
        let index = Arc::new(
            VersionIndex::open(
                PagedManifestStore::version_index_directory(database_path).as_path(),
                store_identity,
                resources.clone(),
                tenant.get(),
                resource_config.maximum_cached_index_nodes,
            )
            .unwrap(),
        );
        let sources = SourceManager::new(
            SourceManagerConfig {
                store_identity,
                checkpoint_directory: pack_directory.clone(),
                wal_directory: wal_directory.clone(),
                scratch_directory,
                tenant_id: tenant.get(),
                maximum_open_handles: resource_config.maximum_open_source_handles,
                maximum_roots: resource_config.maximum_roots_per_tenant,
                handle_charge_bytes: resource_config.source_handle_charge_bytes,
                gc_reservation_bytes: resource_config.source_gc_reservation_bytes,
            },
            resources.clone(),
        )
        .unwrap();
        let recovery_wal_path = wal_directory.join("segment-00000000000000000001.wal");
        let mut leaf_body = vec![0; dodb_storage::PAGE_SIZE - dodb_storage::PAGE_HEADER_SIZE];
        leaf_body[0..4].copy_from_slice(b"BLKL");
        leaf_body[4..6].copy_from_slice(&2u16.to_le_bytes());
        leaf_body[10..12].copy_from_slice(&48u16.to_le_bytes());
        leaf_body[12..14].copy_from_slice(&48u16.to_le_bytes());
        let leaf_body_length = leaf_body.len() as u16;
        leaf_body[18..20].copy_from_slice(&leaf_body_length.to_le_bytes());
        leaf_body[20..28].copy_from_slice(&u64::MAX.to_le_bytes());
        let root_page_id = dodb_core::PageId::new(2);
        let page_image = dodb_storage::encode_page(
            dodb_storage::PageHeader::new(
                dodb_storage::PageType::Leaf,
                root_page_id,
                dodb_core::Lsn::new(2),
            ),
            &leaf_body,
        )
        .unwrap();
        let mut wal = dodb_storage::WalLog::<ProductionFile>::open_with_page_image_format(
            ProductionFile::open(&recovery_wal_path).unwrap(),
            wal_identity.clone(),
            dodb_storage::WalPageImageFormat::ExperimentalBlink,
        )
        .unwrap();
        wal.append_commit(
            1,
            dodb_core::Lsn::new(2),
            &[dodb_storage::WalPageImage {
                page_id: root_page_id,
                image: page_image,
            }],
            None,
        )
        .unwrap();
        drop(wal);
        let recovery_source =
            PagedManifestStore::recovery_source_identity(&recovery_wal_path, 1).unwrap();
        let mut wal_file = ProductionFile::open_existing(&recovery_wal_path).unwrap();
        let wal_scan = dodb_storage::WalLog::scan_committed_page_locations(
            &mut wal_file,
            &wal_identity,
            dodb_storage::WalPageImageFormat::ExperimentalBlink,
            None,
            std::collections::BTreeMap::new(),
        )
        .unwrap();
        let recovered_location = wal_scan.locations.first().unwrap();
        let image_lsn = recovered_location.image_lsn.unwrap();
        let image_crc32c = recovered_location.image_crc32c.unwrap();
        let mut root = index.empty_root();
        root = index
            .insert(
                root,
                root_page_id,
                VersionDescriptor {
                    page_id: root_page_id,
                    token: VersionToken::Wal {
                        source_file_number: 1,
                        record_lsn: recovered_location.record_lsn,
                    },
                    image_lsn,
                    image_crc32c,
                    locator: VersionLocator::WalImage {
                        frame: WalFrameLocator {
                            source: recovery_source,
                            frame_offset: recovered_location.frame_offset,
                            frame_length: recovered_location.frame_length,
                            header_crc32c: recovered_location.header_crc32c,
                            payload_offset: recovered_location.payload_offset,
                            payload_length: recovered_location.payload_length,
                            payload_crc32c: recovered_location.payload_crc32c,
                            committed_prefix_length: wal_scan.committed_prefix_length,
                            commit_frame_offset: recovered_location.commit_frame_offset,
                            commit_lsn: recovered_location.commit_lsn,
                        },
                    },
                },
            )
            .unwrap();
        let initial_handle = dodb_storage::blink::PagedReadHandle::new(
            dodb_storage::blink::PagedReadConfig {
                tenant_id: tenant.get(),
                root,
                root_page_id,
                free_list_head: None,
                high_water_page_id: root_page_id,
                loader: resource_config.page_loader,
            },
            Arc::clone(&index),
            resources.clone(),
            sources,
            |source| {
                Ok(if source.kind == SourceFileKind::WalSegment {
                    dodb_storage::wal::INIT_FRAME_SIZE
                } else {
                    dodb_storage::PAGE_SIZE
                })
            },
        )
        .unwrap();
        let checkpoint_pack = initial_handle
            .write_checkpoint_pack(
                dodb_storage::blink::PagedCheckpointPackRequest {
                    directory: pack_directory.clone(),
                    file_number: 1,
                    identity: wal_identity.clone(),
                    checkpoint_lsn: dodb_core::Lsn::new(2),
                    previous_checkpoint_lsn: None,
                    coalesce: false,
                },
                None,
            )
            .unwrap();
        let checkpoint_pack_path = pack_directory.join("base-00000000000000000001.pages");
        let (durable_root, checkpoint_source) = build_checkpoint_index(
            &index,
            root,
            &checkpoint_pack_path,
            &checkpoint_pack.entry,
            true,
            &wal_identity,
            root_page_id,
        )
        .unwrap();
        assert_eq!(checkpoint_source.kind, SourceFileKind::CheckpointPack);
        let active_wal_path = wal_directory.join("segment-00000000000000000002.wal");
        let active_wal = dodb_storage::WalLog::<ProductionFile>::open_with_page_image_format_and_fault_injector_and_start_lsn(
            ProductionFile::open(&active_wal_path).unwrap(),
            wal_identity,
            dodb_storage::WalPageImageFormat::ExperimentalBlink,
            dodb_core::Lsn::new(2),
            None,
        )
        .unwrap();
        drop(active_wal);
        PagedManifestStore::create_with_index(
            database_path,
            PagedManifest {
                identity: store_identity,
                generation: 1,
                tree_root: root_page_id,
                free_list_root: None,
                high_water_page_id: root_page_id,
                durable_index_root: durable_root.node_reference().unwrap(),
                source_catalog_root: None,
                recovery_wal_file_number: 2,
                recovery_offset: std::fs::metadata(active_wal_path).unwrap().len(),
                committed_prefix_lsn: dodb_core::Lsn::new(2),
                previous_generation: 0,
            },
            &index,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn cached_blink_reads_do_not_depend_on_repeated_path_checks() {
        for wal_layout in [
            BlinkWalLayout::Legacy,
            BlinkWalLayout::Segmented {
                max_segment_bytes: 1_048_576,
            },
            BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
        ] {
            let directory = blink_test_directory();
            let service = BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                blink_wal_layout: wal_layout,
                ..LocalTenantServiceConfig::default()
            })
            .unwrap();
            let tenant = TenantId::new(7);
            let key = DocumentKey::new("cached", "row");
            let budget = ExecutionBudget::new(4096);
            service
                .execute_request(
                    tenant,
                    Request::Put {
                        key: key.clone(),
                        value: b"cached-value".to_vec(),
                    },
                    budget,
                )
                .await
                .unwrap();
            let original_path = directory.path().join("blink");
            let moved_path = directory.path().join("temporarily-moved");
            std::fs::rename(&original_path, &moved_path).unwrap();
            for request in [
                Request::Get { key: key.clone() },
                Request::Query {
                    pk: key.pk.clone(),
                    exclusive_after_sk: None,
                    limit: 16,
                },
                Request::Scan {
                    exclusive_after_key: None,
                    limit: 16,
                },
                Request::Transact {
                    request: TransactionRequest::new(
                        vec![TransactionCondition::Exists { key: key.clone() }],
                        Vec::new(),
                    ),
                },
            ] {
                match service
                    .execute_request(tenant, request, budget)
                    .await
                    .unwrap()
                {
                    Response::Get(state) => {
                        assert_eq!(state.value(), Some(b"cached-value".as_slice()))
                    }
                    Response::Query(rows) | Response::Scan(rows) => {
                        assert_eq!(rows.len(), 1);
                        assert_eq!(rows[0].value, b"cached-value");
                    }
                    Response::Transact(outcome) => assert!(outcome.commit_lsn.is_none()),
                    response => panic!("unexpected response: {response:?}"),
                }
            }
            std::fs::rename(&moved_path, &original_path).unwrap();
            service.shutdown_checked().await.unwrap();
            match service
                .execute_request(tenant, Request::Get { key }, budget)
                .await
                .unwrap()
            {
                Response::Get(state) => assert_eq!(state.value(), Some(b"cached-value".as_slice())),
                response => panic!("unexpected response: {response:?}"),
            }
            service.shutdown_checked().await.unwrap();
        }
    }

    #[tokio::test]
    async fn absent_blink_reads_preserve_capacity_and_do_not_cache_absence() {
        let directory = blink_test_directory();
        let service = BlinkTenantService::new(LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 1,
            ..LocalTenantServiceConfig::default()
        })
        .unwrap();
        let tenant = TenantId::new(1);
        assert!(service.read_shard(tenant).await.unwrap().is_none());
        assert_eq!(
            std::fs::read_dir(directory.path().join("blink"))
                .unwrap()
                .count(),
            0
        );
        let opened = service.open_shard(tenant).await.unwrap();
        assert!(Arc::ptr_eq(
            &opened,
            &service.read_shard(tenant).await.unwrap().unwrap()
        ));
        assert!(
            service
                .read_shard(TenantId::new(2))
                .await
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            service.open_shard(TenantId::new(2)).await,
            Err(Error::Overloaded(_))
        ));
        service.shutdown_checked().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn racing_blink_read_and_open_share_one_shard() {
        let directory = blink_test_directory();
        let service = Arc::new(
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let tenant = TenantId::new(3);
        let barrier = Arc::new(tokio::sync::Barrier::new(9));
        let mut readers = Vec::new();
        for _reader_number in 0..8 {
            let reader_service = Arc::clone(&service);
            let reader_barrier = Arc::clone(&barrier);
            readers.push(tokio::spawn(async move {
                reader_barrier.wait().await;
                reader_service.read_shard(tenant).await.unwrap()
            }));
        }
        barrier.wait().await;
        let opened = service.open_shard(tenant).await.unwrap();
        for reader in readers {
            if let Some(shard) = reader.await.unwrap() {
                assert!(Arc::ptr_eq(&opened, &shard));
            }
        }
        assert_eq!(service.shards.lock().unwrap().shards.len(), 1);
        service.shutdown_checked().await.unwrap();
    }

    #[test]
    fn blink_open_limits_must_be_nonzero() {
        for config in [
            LocalTenantServiceConfig {
                blink_open_worker_count: 0,
                ..LocalTenantServiceConfig::default()
            },
            LocalTenantServiceConfig {
                blink_open_waiter_limit: 0,
                ..LocalTenantServiceConfig::default()
            },
            LocalTenantServiceConfig {
                blink_open_shutdown_timeout: std::time::Duration::ZERO,
                ..LocalTenantServiceConfig::default()
            },
        ] {
            assert!(BlinkTenantService::new(config).is_err());
        }
    }

    #[test]
    #[cfg(feature = "blink-demand-paging")]
    fn shared_paging_resources_reject_incompatible_layout_before_filesystem_access() {
        let directory = blink_test_directory();
        for (tenant_path, wal_layout) in [
            ("legacy", BlinkWalLayout::Legacy),
            (
                "segmented-without-bases",
                BlinkWalLayout::Segmented {
                    max_segment_bytes: 1_048_576,
                },
            ),
        ] {
            let data_path = directory.path().join(tenant_path);
            assert!(
                BlinkTenantService::new(LocalTenantServiceConfig {
                    data_dir: data_path.clone(),
                    blink_wal_layout: wal_layout,
                    blink_shared_resources: Some(constrained_blink_shared_resources()),
                    ..LocalTenantServiceConfig::default()
                })
                .is_err()
            );
            assert!(!data_path.exists());
        }
        let btree_path = directory.path().join("btree");
        assert!(
            LocalTenantService::new(LocalTenantServiceConfig {
                data_dir: btree_path.clone(),
                blink_shared_resources: Some(constrained_blink_shared_resources()),
                ..LocalTenantServiceConfig::default()
            })
            .is_err()
        );
        assert!(!btree_path.exists());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn shared_blink_metadata_budget_precedes_open_and_releases_on_shutdown() {
        let directory = blink_test_directory();
        let service = BlinkTenantService::new(LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 3,
            blink_open_waiter_limit: 4,
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_shared_resources: Some(constrained_blink_shared_resources()),
            ..LocalTenantServiceConfig::default()
        })
        .unwrap();
        service.open_shard(TenantId::new(301)).await.unwrap();
        service.open_shard(TenantId::new(302)).await.unwrap();
        let third_tenant_path = service.database_path(TenantId::new(303));
        assert!(service.open_shard(TenantId::new(303)).await.is_err());
        assert!(!third_tenant_path.exists());
        let manager = service.shared_resources.as_ref().unwrap();
        assert_eq!(
            manager.snapshot().unwrap().current_by_category
                [dodb_storage::blink::resource::ResourceCategory::Metadata as usize],
            14_000
        );
        service.shutdown_checked().await.unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 0);
        service.open_shard(TenantId::new(303)).await.unwrap();
        assert!(third_tenant_path.exists());
        service.shutdown_checked().await.unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn foreground_preparation_limit_rejects_before_queue_and_releases_charge() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(713);
        let mut shared_resources = foreground_blink_shared_resources();
        let write_preparation =
            dodb_storage::blink::resource::ResourceCategory::WritePreparation as usize;
        shared_resources.memory.global_category_bytes[write_preparation] = 1_024;
        shared_resources.memory.per_tenant_category_bytes[write_preparation] = 1_024;
        let load_reconstruction =
            dodb_storage::blink::resource::ResourceCategory::LoadReconstruction as usize;
        shared_resources.memory.global_category_bytes[load_reconstruction] = 2_048;
        shared_resources.memory.per_tenant_category_bytes[load_reconstruction] = 2_048;
        let service = BlinkTenantService::new(LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 1,
            blink_open_waiter_limit: 4,
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_shared_resources: Some(shared_resources),
            ..LocalTenantServiceConfig::default()
        })
        .unwrap();
        let mut oversized_pk = Vec::with_capacity(1_024);
        oversized_pk.push(b'p');
        let mut oversized_value = Vec::with_capacity(2_048);
        oversized_value.push(7);
        let rejected = service
            .execute_request(
                tenant,
                Request::Put {
                    key: dodb_core::DocumentKey::new(oversized_pk, b"sk".to_vec()),
                    value: oversized_value,
                },
                ExecutionBudget::new(1024),
            )
            .await;
        assert!(rejected.is_err());
        let resources = service.shared_resources.as_ref().unwrap();
        let snapshot = resources.snapshot().unwrap();
        assert_eq!(
            snapshot.current_by_category
                [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize],
            0
        );
        assert_eq!(snapshot.current_by_category[write_preparation], 0);
        assert!(
            service
                .execute_request(
                    tenant,
                    Request::Scan {
                        exclusive_after_key: None,
                        limit: 8,
                    },
                    ExecutionBudget::new(4096),
                )
                .await
                .is_err()
        );
        assert_eq!(
            resources.snapshot().unwrap().current_by_category
                [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize],
            0
        );
        assert_eq!(
            resources.snapshot().unwrap().current_by_category[load_reconstruction],
            0
        );
        assert_eq!(
            snapshot.current_by_category
                [dodb_storage::blink::resource::ResourceCategory::Metadata as usize],
            512
        );
        match service
            .execute_request(
                tenant,
                Request::Get {
                    key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"sk".to_vec()),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap()
        {
            Response::Get(state) => assert!(state.value().is_none()),
            response => panic!("unexpected response: {response:?}"),
        }
        service.shutdown_checked().await.unwrap();
        assert_eq!(resources.snapshot().unwrap().current_total, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn cancelled_transaction_keeps_foreground_charges_until_commit_finishes() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(714);
        let write_request = Request::Put {
            key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"sk".to_vec()),
            value: b"committed-after-cancel".to_vec(),
        };
        let queued_limit = queued_request_allocation_bytes(&write_request).unwrap();
        let mut shared_resources = foreground_blink_shared_resources();
        let queued_foreground =
            dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize;
        shared_resources.memory.global_category_bytes[queued_foreground] = queued_limit;
        shared_resources.memory.per_tenant_category_bytes[queued_foreground] = queued_limit;
        let service = Arc::new(
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                max_open_shards: 1,
                blink_open_waiter_limit: 4,
                blink_wal_layout: BlinkWalLayout::SegmentedBases {
                    max_segment_bytes: 1_048_576,
                    coalesce: true,
                },
                blink_shared_resources: Some(shared_resources),
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let commit_gate = Arc::new(OneShotTestGate {
            barrier: std::sync::Barrier::new(2),
            blocks_next_worker: std::sync::atomic::AtomicBool::new(true),
        });
        service.set_commit_test_gate(Arc::clone(&commit_gate));
        let write_service = Arc::clone(&service);
        let write_task = tokio::spawn(async move {
            write_service
                .execute_request(tenant, write_request, ExecutionBudget::new(1024))
                .await
        });
        let resources = service.shared_resources.as_ref().unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let snapshot = resources.snapshot().unwrap();
                if service.coordinator_activity.active.load(Ordering::SeqCst) > 0
                    && snapshot.current_by_category
                        [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize]
                        > 0
                    && snapshot.current_by_category
                        [dodb_storage::blink::resource::ResourceCategory::WritePreparation as usize]
                        > 0
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let rejected_tenant = TenantId::new(715);
        let rejected_path = service.database_path(rejected_tenant);
        assert!(
            service
                .execute_request(
                    rejected_tenant,
                    Request::Put {
                        key: dodb_core::DocumentKey::new(b"other".to_vec(), b"key".to_vec()),
                        value: b"global-limit".to_vec(),
                    },
                    ExecutionBudget::new(1024),
                )
                .await
                .is_err()
        );
        assert!(!rejected_path.exists());
        write_task.abort();
        assert!(write_task.await.unwrap_err().is_cancelled());
        let held_snapshot = resources.snapshot().unwrap();
        assert!(
            held_snapshot.current_by_category
                [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize]
                > 0
        );
        assert!(
            held_snapshot.current_by_category
                [dodb_storage::blink::resource::ResourceCategory::WritePreparation as usize]
                > 0
        );
        tokio::task::spawn_blocking({
            let commit_gate = Arc::clone(&commit_gate);
            move || commit_gate.barrier.wait()
        })
        .await
        .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let snapshot = resources.snapshot().unwrap();
                if snapshot.current_by_category
                    [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize]
                    == 0
                    && snapshot.current_by_category
                        [dodb_storage::blink::resource::ResourceCategory::WritePreparation as usize]
                        == 0
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        match service
            .execute_request(
                tenant,
                Request::Get {
                    key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"sk".to_vec()),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap()
        {
            Response::Get(state) => {
                assert_eq!(state.value(), Some(b"committed-after-cancel".as_slice()))
            }
            response => panic!("unexpected response: {response:?}"),
        }
        service.shutdown_checked().await.unwrap();
        assert_eq!(resources.snapshot().unwrap().current_total, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn closed_blink_coordinator_releases_foreground_charges() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(720);
        let service = BlinkTenantService::new(LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 1,
            blink_open_waiter_limit: 4,
            coordinator_config: CoordinatorConfig {
                queue_capacity: 1,
                max_group_requests: 1,
                ..CoordinatorConfig::default()
            },
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_shared_resources: Some(foreground_blink_shared_resources()),
            ..LocalTenantServiceConfig::default()
        })
        .unwrap();
        let shard = service.open_shard(tenant).await.unwrap();
        *shard.request_tx.lock().await = None;
        let rejected = service
            .execute_request(
                tenant,
                Request::Put {
                    key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"closed".to_vec()),
                    value: b"not-committed".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap_err();
        assert!(rejected.to_string().contains("coordinator is closed"));
        let resources = service.shared_resources.as_ref().unwrap();
        let snapshot = resources.snapshot().unwrap();
        assert_eq!(
            snapshot.current_by_category
                [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize],
            0
        );
        assert_eq!(
            snapshot.current_by_category
                [dodb_storage::blink::resource::ResourceCategory::WritePreparation as usize],
            0
        );
        drop(shard);
        service.shutdown_checked().await.unwrap();
        assert_eq!(resources.snapshot().unwrap().current_total, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn foreground_queue_count_limit_rejects_excess_without_leaking_charges() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(716);
        let first_request = Request::Put {
            key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"first".to_vec()),
            value: b"first-value".to_vec(),
        };
        let second_request = Request::Put {
            key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"second".to_vec()),
            value: b"second-value".to_vec(),
        };
        let first_charge = queued_request_allocation_bytes(&first_request).unwrap();
        let second_charge = queued_request_allocation_bytes(&second_request).unwrap();
        let service = Arc::new(
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                max_open_shards: 1,
                blink_open_waiter_limit: 4,
                coordinator_config: CoordinatorConfig {
                    queue_capacity: 1,
                    max_group_requests: 1,
                    ..CoordinatorConfig::default()
                },
                blink_wal_layout: BlinkWalLayout::SegmentedBases {
                    max_segment_bytes: 1_048_576,
                    coalesce: true,
                },
                blink_shared_resources: Some(foreground_blink_shared_resources()),
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let commit_gate = Arc::new(OneShotTestGate {
            barrier: std::sync::Barrier::new(2),
            blocks_next_worker: std::sync::atomic::AtomicBool::new(true),
        });
        service.set_commit_test_gate(Arc::clone(&commit_gate));
        let first_service = Arc::clone(&service);
        let first_task = tokio::spawn(async move {
            first_service
                .execute_request(tenant, first_request, ExecutionBudget::new(1024))
                .await
        });
        let resources = service.shared_resources.as_ref().unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if service.coordinator_activity.active.load(Ordering::SeqCst) > 0 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let second_service = Arc::clone(&service);
        let second_task = tokio::spawn(async move {
            second_service
                .execute_request(tenant, second_request, ExecutionBudget::new(1024))
                .await
        });
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let snapshot = resources.snapshot().unwrap();
                let queued_bytes = snapshot.current_by_category
                    [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize];
                if queued_bytes >= first_charge + second_charge {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let rejected = service
            .execute_request(
                tenant,
                Request::Put {
                    key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"third".to_vec()),
                    value: b"third-value".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap_err();
        assert!(rejected.to_string().contains("coordinator queue is full"));
        let held_snapshot = resources.snapshot().unwrap();
        assert!(
            held_snapshot.current_by_category
                [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize]
                >= first_charge + second_charge
        );
        tokio::task::spawn_blocking({
            let commit_gate = Arc::clone(&commit_gate);
            move || commit_gate.barrier.wait()
        })
        .await
        .unwrap();
        first_task.await.unwrap().unwrap();
        second_task.await.unwrap().unwrap();
        service.shutdown_checked().await.unwrap();
        assert_eq!(resources.snapshot().unwrap().current_total, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_route_rejects_incomplete_store_before_creating_open_sidecars() {
        let directory = blink_test_directory();
        let service = BlinkTenantService::new(LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 1,
            blink_open_worker_count: 1,
            blink_open_waiter_limit: 4,
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_shared_resources: Some(paged_blink_shared_resources()),
            ..LocalTenantServiceConfig::default()
        })
        .unwrap();
        let tenant = TenantId::new(711);
        let database_path = service.database_path(tenant);
        let shard_id = ShardId::new(tenant.get());
        dodb_storage::blink::paged_manifest::PagedManifestStore::prepare_format(
            &database_path,
            dodb_storage::blink::version_index::StoreIdentity {
                database_uuid: database_uuid(tenant, shard_id),
                wal_epoch: service.database_config.shard_epoch.get(),
            },
        )
        .unwrap();
        let sidecar_directory =
            dodb_storage::blink::paged_manifest::PagedManifestStore::sidecar_directory(
                &database_path,
            );
        assert!(service.open_shard(tenant).await.is_err());
        assert!(!sidecar_directory.join("index").exists());
        assert!(!sidecar_directory.join("packs").exists());
        assert!(!sidecar_directory.join("segments").exists());
        assert!(!sidecar_directory.join("scratch").exists());
        assert_eq!(
            service
                .shared_resources
                .as_ref()
                .unwrap()
                .snapshot()
                .unwrap()
                .current_total,
            0
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_service_opens_recovered_store_for_get_query_and_scan() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(712);
        let service = Arc::new(
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                max_open_shards: 1,
                blink_open_worker_count: 1,
                blink_open_waiter_limit: 4,
                blink_open_shutdown_timeout: std::time::Duration::from_millis(10),
                blink_wal_layout: BlinkWalLayout::SegmentedBases {
                    max_segment_bytes: 1_048_576,
                    coalesce: true,
                },
                blink_shared_resources: Some(paged_blink_shared_resources()),
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let database_path = service.database_path(tenant);
        create_empty_paged_service_fixture(&database_path, tenant);
        let page_read_gate = Arc::new(OneShotTestGate {
            barrier: std::sync::Barrier::new(2),
            blocks_next_worker: std::sync::atomic::AtomicBool::new(true),
        });
        service.set_page_read_test_gate(Arc::clone(&page_read_gate));
        let get_service = Arc::clone(&service);
        let get_task = tokio::spawn(async move {
            get_service
                .execute_request(
                    tenant,
                    Request::Get {
                        key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"sk".to_vec()),
                    },
                    ExecutionBudget::new(1024),
                )
                .await
        });
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if service.coordinator_activity.active.load(Ordering::SeqCst) == 1
                    && service.open_workers.available_permits() == 0
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(
            service
                .execute_request(
                    tenant,
                    Request::Query {
                        pk: dodb_core::PrimaryKey::new(b"pk".to_vec()),
                        exclusive_after_sk: None,
                        limit: 8,
                    },
                    ExecutionBudget::new(1024),
                )
                .await
                .is_err()
        );
        assert!(service.shutdown_checked().await.is_err());
        get_task.abort();
        assert!(get_task.await.unwrap_err().is_cancelled());
        tokio::task::spawn_blocking({
            let page_read_gate = Arc::clone(&page_read_gate);
            move || page_read_gate.barrier.wait()
        })
        .await
        .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if service.open_workers.available_permits() == 1
                    && service.coordinator_activity.active.load(Ordering::SeqCst) == 0
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let get_response = service
            .execute_request(
                tenant,
                Request::Get {
                    key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"sk".to_vec()),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();
        assert!(matches!(
            get_response,
            Response::Get(RevisionState::Missing { .. })
        ));
        let query_response = service
            .execute_request(
                tenant,
                Request::Query {
                    pk: dodb_core::PrimaryKey::new(b"pk".to_vec()),
                    exclusive_after_sk: None,
                    limit: 8,
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();
        assert_eq!(query_response, Response::Query(Vec::new()));
        let scan_response = service
            .execute_request(
                tenant,
                Request::Scan {
                    exclusive_after_key: None,
                    limit: 8,
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();
        assert_eq!(scan_response, Response::Scan(Vec::new()));
        assert_eq!(service.open_workers.available_permits(), 1);
        assert!(
            service
                .execute_request(
                    tenant,
                    Request::Put {
                        key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"next".to_vec()),
                        value: b"unsupported".to_vec(),
                    },
                    ExecutionBudget::new(1024),
                )
                .await
                .is_err()
        );
        let manager = service.shared_resources.as_ref().unwrap();
        service.shutdown_checked().await.unwrap();
        assert_eq!(manager.snapshot().unwrap().current_total, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn cancelled_transaction_before_enqueue_releases_foreground_charges() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(724);
        let service = Arc::new(
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                max_open_shards: 1,
                blink_open_waiter_limit: 4,
                blink_wal_layout: BlinkWalLayout::SegmentedBases {
                    max_segment_bytes: 1_048_576,
                    coalesce: true,
                },
                blink_shared_resources: Some(foreground_blink_shared_resources()),
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let shard = service.open_shard(tenant).await.unwrap();
        let request_sender_guard = shard.request_tx.lock().await;
        let write_service = Arc::clone(&service);
        let write_task = tokio::spawn(async move {
            write_service
                .execute_request(
                    tenant,
                    Request::Put {
                        key: dodb_core::DocumentKey::new(
                            b"pk".to_vec(),
                            b"before-enqueue".to_vec(),
                        ),
                        value: b"cancel-before-enqueue".to_vec(),
                    },
                    ExecutionBudget::new(1024),
                )
                .await
        });
        let resources = service.shared_resources.as_ref().unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let snapshot = resources.snapshot().unwrap();
                if snapshot.current_by_category
                    [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize]
                    > 0
                    && snapshot.current_by_category
                        [dodb_storage::blink::resource::ResourceCategory::WritePreparation as usize]
                        > 0
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        write_task.abort();
        assert!(write_task.await.unwrap_err().is_cancelled());
        assert_eq!(
            resources.snapshot().unwrap().current_by_category
                [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize],
            0
        );
        assert_eq!(
            resources.snapshot().unwrap().current_by_category
                [dodb_storage::blink::resource::ResourceCategory::WritePreparation as usize],
            0
        );
        drop(request_sender_guard);
        assert!(matches!(
            service
                .execute_request(
                    tenant,
                    Request::Put {
                        key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"after-cancel".to_vec()),
                        value: b"coordinator-alive".to_vec(),
                    },
                    ExecutionBudget::new(1024),
                )
                .await
                .unwrap(),
            Response::Put(_)
        ));
        drop(shard);
        service.shutdown_checked().await.unwrap();
        assert_eq!(resources.snapshot().unwrap().current_total, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn cancelled_paged_write_keeps_charges_until_shutdown_drain() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(719);
        let service = Arc::new(
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                max_open_shards: 1,
                blink_open_waiter_limit: 4,
                blink_wal_layout: BlinkWalLayout::SegmentedBases {
                    max_segment_bytes: 1_048_576,
                    coalesce: true,
                },
                blink_shared_resources: Some(writable_paged_blink_shared_resources()),
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        create_empty_paged_service_fixture(&service.database_path(tenant), tenant);
        let commit_gate = Arc::new(OneShotTestGate {
            barrier: std::sync::Barrier::new(2),
            blocks_next_worker: std::sync::atomic::AtomicBool::new(true),
        });
        service.set_commit_test_gate(Arc::clone(&commit_gate));
        let write_service = Arc::clone(&service);
        let write_task = tokio::spawn(async move {
            write_service
                .execute_request(
                    tenant,
                    Request::Put {
                        key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"shutdown".to_vec()),
                        value: b"durable-on-drain".to_vec(),
                    },
                    ExecutionBudget::new(1024),
                )
                .await
        });
        let resources = service.shared_resources.as_ref().unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let snapshot = resources.snapshot().unwrap();
                if service.coordinator_activity.active.load(Ordering::SeqCst) > 0
                    && snapshot.current_by_category
                        [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize]
                        > 0
                    && snapshot.current_by_category
                        [dodb_storage::blink::resource::ResourceCategory::WritePreparation as usize]
                        > 0
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let gate_release = tokio::task::spawn_blocking({
            let commit_gate = Arc::clone(&commit_gate);
            move || commit_gate.barrier.wait()
        });
        write_task.abort();
        assert!(write_task.await.unwrap_err().is_cancelled());
        let held_snapshot = resources.snapshot().unwrap();
        assert!(
            held_snapshot.current_by_category
                [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize]
                > 0
        );
        assert!(
            held_snapshot.current_by_category
                [dodb_storage::blink::resource::ResourceCategory::WritePreparation as usize]
                > 0
        );
        let shutdown_service = Arc::clone(&service);
        let shutdown_task = tokio::spawn(async move { shutdown_service.shutdown_checked().await });
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if service.shards.lock().unwrap().shutting_down {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        gate_release.await.unwrap();
        shutdown_task.await.unwrap().unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if resources.snapshot().unwrap().current_total == 0 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let drained_resources = resources.snapshot().unwrap();
        assert_eq!(
            drained_resources.current_total, 0,
            "remaining resources after cancelled paged write shutdown: {drained_resources:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_writer_admission_releases_charges_before_dispatch() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(718);
        let service = BlinkTenantService::new(LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 1,
            blink_open_waiter_limit: 4,
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_shared_resources: Some(writable_paged_blink_shared_resources()),
            ..LocalTenantServiceConfig::default()
        })
        .unwrap();
        let database_path = service.database_path(tenant);
        create_empty_paged_service_fixture(&database_path, tenant);
        let shard = service.open_shard(tenant).await.unwrap();
        let writable = match &shard.read_handle {
            BlinkShardReadHandle::Paged {
                writable: Some(writable),
                ..
            } => Arc::clone(writable),
            _ => panic!("paged writer shard was not opened"),
        };
        let transaction_permit = writable
            .transaction_worker
            .clone()
            .try_acquire_owned()
            .unwrap();
        let rejected = service
            .execute_request(
                tenant,
                Request::Put {
                    key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"blocked".to_vec()),
                    value: b"not-enqueued".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap_err();
        assert!(
            rejected
                .to_string()
                .contains("paged transaction capacity is full")
        );
        let resources = service.shared_resources.as_ref().unwrap();
        let snapshot = resources.snapshot().unwrap();
        assert_eq!(
            snapshot.current_by_category
                [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize],
            0
        );
        assert_eq!(
            snapshot.current_by_category
                [dodb_storage::blink::resource::ResourceCategory::WritePreparation as usize],
            0
        );
        drop(transaction_permit);
        assert!(matches!(
            service
                .execute_request(
                    tenant,
                    Request::Put {
                        key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"ready".to_vec()),
                        value: b"committed".to_vec(),
                    },
                    ExecutionBudget::new(1024),
                )
                .await
                .unwrap(),
            Response::Put(_)
        ));
        drop(writable);
        drop(shard);
        service.shutdown_checked().await.unwrap();
        assert_eq!(resources.snapshot().unwrap().current_total, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_writer_service_commits_and_recovers_service_requests() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(717);
        let config = LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 1,
            blink_open_waiter_limit: 4,
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold {
                threshold_bytes: 1,
            },
            blink_checkpoint_execution: BlinkCheckpointExecution::Background,
            blink_shared_resources: Some(writable_paged_blink_shared_resources()),
            ..LocalTenantServiceConfig::default()
        };
        let service = BlinkTenantService::new(config.clone()).unwrap();
        let database_path = service.database_path(tenant);
        create_empty_paged_service_fixture(&database_path, tenant);
        let key = dodb_core::DocumentKey::new(b"pk".to_vec(), b"sk".to_vec());
        let put_response = service
            .execute_request(
                tenant,
                Request::Put {
                    key: key.clone(),
                    value: b"durable-paged-value".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();
        assert!(matches!(put_response, Response::Put(_)));
        service
            .checkpoint_metrics_handle()
            .wait_for_coordinators_idle()
            .await;
        let checkpoint_metrics = service.checkpoint_metrics_snapshot();
        assert_eq!(checkpoint_metrics.checkpoints_completed, 1);
        let tenant_checkpoint_metrics = &checkpoint_metrics.per_tenant[&tenant.get()];
        assert_eq!(tenant_checkpoint_metrics.checkpoint_requests_queued, 1);
        assert_eq!(tenant_checkpoint_metrics.checkpoint_requests_pending, 0);
        assert!(tenant_checkpoint_metrics.service_slices >= 3);
        assert_eq!(tenant_checkpoint_metrics.checkpoint_debt_bytes_current, 0);
        match service
            .execute_request(
                tenant,
                Request::Get { key: key.clone() },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap()
        {
            Response::Get(state) => {
                assert_eq!(state.value(), Some(b"durable-paged-value".as_slice()))
            }
            response => panic!("unexpected response: {response:?}"),
        }
        service.shutdown_checked().await.unwrap();
        assert_eq!(
            service
                .shared_resources
                .as_ref()
                .unwrap()
                .snapshot()
                .unwrap()
                .current_total,
            0
        );
        let reopened_service = BlinkTenantService::new(config).unwrap();
        match reopened_service
            .execute_request(tenant, Request::Get { key }, ExecutionBudget::new(1024))
            .await
            .unwrap()
        {
            Response::Get(state) => {
                assert_eq!(state.value(), Some(b"durable-paged-value".as_slice()))
            }
            response => panic!("unexpected response: {response:?}"),
        }
        reopened_service.shutdown_checked().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_checkpoint_resumes_after_disk_capacity_release() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(721);
        let mut shared_resources = writable_paged_blink_shared_resources();
        let output_category = dodb_storage::blink::resource::DiskCategory::CheckpointOutput;
        let output_limit = 1024 * 1024;
        shared_resources.disk.global_category_bytes[output_category as usize] = output_limit;
        shared_resources.disk.per_tenant_category_bytes[output_category as usize] = output_limit;
        let config = LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 1,
            blink_open_waiter_limit: 4,
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold {
                threshold_bytes: u64::MAX,
            },
            blink_checkpoint_execution: BlinkCheckpointExecution::Background,
            blink_shared_resources: Some(shared_resources),
            ..LocalTenantServiceConfig::default()
        };
        let service = BlinkTenantService::new(config.clone()).unwrap();
        let database_path = service.database_path(tenant);
        create_empty_paged_service_fixture(&database_path, tenant);
        service
            .execute_request(
                tenant,
                Request::Put {
                    key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"sk".to_vec()),
                    value: b"capacity-release-checkpoint".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();
        let shard = service
            .shards
            .lock()
            .unwrap()
            .shards
            .get(&tenant)
            .cloned()
            .unwrap();
        let writable = match &shard.read_handle {
            BlinkShardReadHandle::Paged {
                writable: Some(writable),
                ..
            } => Arc::clone(writable),
            BlinkShardReadHandle::Resident(_) | BlinkShardReadHandle::Paged { .. } => {
                panic!("expected the writable paged service route")
            }
        };
        let resources = service.shared_resources.as_ref().unwrap().clone();
        let disk = resources.disk();
        let tenant_disk = disk.tenant_snapshot(tenant.get()).unwrap().unwrap();
        let disk_headroom = 256;
        let blocker_bytes = output_limit
            .saturating_sub(tenant_disk.current_by_category[output_category as usize])
            .saturating_sub(disk_headroom);
        assert!(blocker_bytes > 0);
        let blocker = disk
            .reserve(tenant.get(), output_category, blocker_bytes)
            .unwrap();
        let checkpoint = {
            let writable = Arc::clone(&writable);
            tokio::spawn(async move { writable.run_threshold_checkpoint(1).await })
        };
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if disk.snapshot().unwrap().pending_waiters == 1 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap_or_else(|_| {
            panic!(
                "checkpoint capacity wait did not start: finished={}, metrics={:?}",
                checkpoint.is_finished(),
                service.checkpoint_metrics_snapshot()
            )
        });
        drop(blocker);
        checkpoint.await.unwrap().unwrap();
        let disk_snapshot = disk.snapshot().unwrap();
        assert_eq!(disk_snapshot.pending_waiters, 0);
        let metrics = service.checkpoint_metrics_snapshot();
        assert_eq!(metrics.checkpoints_completed, 1);
        assert_eq!(metrics.checkpoints_failed, 0);
        service.shutdown_checked().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_checkpoint_reclaims_old_reader_source_before_publication() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(726);
        let mut shared_resources = writable_paged_blink_shared_resources();
        let disk_capacity = 64 * 1024 * 1024;
        shared_resources.disk.global_bytes = disk_capacity;
        shared_resources.disk.per_tenant_bytes = disk_capacity;
        shared_resources.disk.global_category_bytes = [disk_capacity; 5];
        shared_resources.disk.per_tenant_category_bytes = [disk_capacity; 5];
        shared_resources.maximum_checkpoint_debt_bytes_global = 1024 * 1024;
        shared_resources.maximum_checkpoint_debt_bytes_per_tenant = 1024 * 1024;
        let config = LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 1,
            blink_open_waiter_limit: 4,
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold {
                threshold_bytes: u64::MAX,
            },
            blink_checkpoint_execution: BlinkCheckpointExecution::Background,
            blink_shared_resources: Some(shared_resources),
            ..LocalTenantServiceConfig::default()
        };
        let service = BlinkTenantService::new(config.clone()).unwrap();
        let database_path = service.database_path(tenant);
        create_empty_paged_service_fixture(&database_path, tenant);
        let wal_directory =
            dodb_storage::blink::paged_manifest::PagedManifestStore::sidecar_directory(
                &database_path,
            )
            .join("segments");
        let wal_directory_bytes = || {
            std::fs::read_dir(&wal_directory)
                .unwrap()
                .flatten()
                .map(|entry| entry.metadata().unwrap().len())
                .sum::<u64>()
        };
        let disk = service.shared_resources.as_ref().unwrap().disk();
        let keys = (0u32..50)
            .map(|key_number| {
                dodb_core::DocumentKey::new(
                    b"pinned-reader".to_vec(),
                    key_number.to_le_bytes().to_vec(),
                )
            })
            .collect::<Vec<_>>();
        let key = keys[0].clone();
        let old_value = vec![17; 4096];
        let new_value = vec![29; 4096];
        for write_key in &keys {
            service
                .execute_request(
                    tenant,
                    Request::Put {
                        key: write_key.clone(),
                        value: old_value.clone(),
                    },
                    ExecutionBudget::new(16 * 1024),
                )
                .await
                .unwrap();
        }
        let disk_usage_after_seed = disk.tenant_snapshot(tenant.get()).unwrap().unwrap();
        assert_eq!(
            disk_usage_after_seed.current_by_category
                [dodb_storage::blink::resource::DiskCategory::Wal as usize],
            wal_directory_bytes(),
            "service transaction groups must account their complete WAL directory"
        );
        let shard = service
            .shards
            .lock()
            .unwrap()
            .shards
            .get(&tenant)
            .cloned()
            .unwrap();
        let writable = match &shard.read_handle {
            BlinkShardReadHandle::Paged {
                writable: Some(writable),
                ..
            } => Arc::clone(writable),
            BlinkShardReadHandle::Resident(_) | BlinkShardReadHandle::Paged { .. } => {
                panic!("expected the writable paged service route")
            }
        };
        let old_reader = writable.snapshot().unwrap();
        let old_reader_value = old_reader.get(&key).unwrap();
        assert_eq!(old_reader_value.value(), Some(old_value.as_slice()));
        drop(old_reader_value);
        let wal_paths_before_checkpoint = std::fs::read_dir(&wal_directory)
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "wal"))
            .collect::<Vec<_>>();
        writable.run_threshold_checkpoint(1).await.unwrap();
        let retained_old_wal_paths = wal_paths_before_checkpoint
            .into_iter()
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        assert!(
            !retained_old_wal_paths.is_empty(),
            "old reader should retain at least one WAL source after checkpoint publication"
        );
        assert_eq!(
            old_reader.get(&key).unwrap().value(),
            Some(old_value.as_slice())
        );

        for write_key in &keys {
            service
                .execute_request(
                    tenant,
                    Request::Put {
                        key: write_key.clone(),
                        value: new_value.clone(),
                    },
                    ExecutionBudget::new(16 * 1024),
                )
                .await
                .unwrap();
        }
        let disk_usage_after_overwrites = disk.tenant_snapshot(tenant.get()).unwrap().unwrap();
        assert_eq!(
            disk_usage_after_overwrites.current_by_category
                [dodb_storage::blink::resource::DiskCategory::Wal as usize],
            wal_directory_bytes(),
            "post-checkpoint transaction groups must account their new WAL segment"
        );
        let disk_usage_before_blocker = disk.snapshot().unwrap().current_total;
        let headroom = 700 * 1024;
        let blocker_bytes = disk_capacity
            .saturating_sub(disk_usage_before_blocker)
            .saturating_sub(headroom);
        assert!(blocker_bytes > 0);
        let blocker = disk
            .reserve(
                tenant.get(),
                dodb_storage::blink::resource::DiskCategory::Scratch,
                blocker_bytes,
            )
            .unwrap();
        assert!(retained_old_wal_paths.iter().any(|path| path.is_file()));
        assert_eq!(
            old_reader.get(&key).unwrap().value(),
            Some(old_value.as_slice())
        );
        let checkpoint = {
            let writable = Arc::clone(&writable);
            tokio::spawn(async move { writable.run_threshold_checkpoint(1).await })
        };
        tokio::time::timeout(std::time::Duration::from_secs(15), async {
            loop {
                if disk.snapshot().unwrap().pending_waiters == 1 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap_or_else(|_| {
            panic!(
                "checkpoint did not wait for old-reader source capacity: finished={}, metrics={:?}",
                checkpoint.is_finished(),
                service.checkpoint_metrics_snapshot()
            )
        });
        drop(old_reader);
        checkpoint.await.unwrap().unwrap();
        drop(blocker);

        assert_eq!(disk.snapshot().unwrap().pending_waiters, 0);
        let metrics = service.checkpoint_metrics_snapshot();
        assert_eq!(metrics.checkpoints_completed, 2);
        assert_eq!(metrics.checkpoints_failed, 0);
        service.shutdown_checked().await.unwrap();
        let reopened_service = BlinkTenantService::new(config).unwrap();
        match reopened_service
            .execute_request(
                tenant,
                Request::Get { key },
                ExecutionBudget::new(16 * 1024),
            )
            .await
            .unwrap()
        {
            Response::Get(state) => assert_eq!(state.value(), Some(new_value.as_slice())),
            response => panic!("unexpected response: {response:?}"),
        }
        reopened_service.shutdown_checked().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_service_shutdown_cancels_checkpoint_waiting_for_disk_capacity() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(724);
        let mut shared_resources = writable_paged_blink_shared_resources();
        let output_category = dodb_storage::blink::resource::DiskCategory::CheckpointOutput;
        let output_limit = 1024 * 1024;
        shared_resources.disk.global_category_bytes[output_category as usize] = output_limit;
        shared_resources.disk.per_tenant_category_bytes[output_category as usize] = output_limit;
        let config = LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 1,
            blink_open_waiter_limit: 4,
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold {
                threshold_bytes: u64::MAX,
            },
            blink_checkpoint_execution: BlinkCheckpointExecution::Background,
            blink_shared_resources: Some(shared_resources),
            ..LocalTenantServiceConfig::default()
        };
        let service = BlinkTenantService::new(config).unwrap();
        let database_path = service.database_path(tenant);
        create_empty_paged_service_fixture(&database_path, tenant);
        service
            .execute_request(
                tenant,
                Request::Put {
                    key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"shutdown-wait".to_vec()),
                    value: b"shutdown-capacity-wait-checkpoint".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();
        let shard = service
            .shards
            .lock()
            .unwrap()
            .shards
            .get(&tenant)
            .cloned()
            .unwrap();
        let writable = match &shard.read_handle {
            BlinkShardReadHandle::Paged {
                writable: Some(writable),
                ..
            } => Arc::clone(writable),
            BlinkShardReadHandle::Resident(_) | BlinkShardReadHandle::Paged { .. } => {
                panic!("expected the writable paged service route")
            }
        };
        let resources = service.shared_resources.as_ref().unwrap().clone();
        let disk = resources.disk();
        let tenant_disk = disk.tenant_snapshot(tenant.get()).unwrap().unwrap();
        let disk_headroom = 256;
        let blocker_bytes = output_limit
            .saturating_sub(tenant_disk.current_by_category[output_category as usize])
            .saturating_sub(disk_headroom);
        assert!(blocker_bytes > 0);
        let blocker = disk
            .reserve(tenant.get(), output_category, blocker_bytes)
            .unwrap();
        let checkpoint = {
            let writable = Arc::clone(&writable);
            tokio::spawn(async move { writable.run_threshold_checkpoint(1).await })
        };
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if disk.snapshot().unwrap().pending_waiters == 1 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        service.shutdown_checked().await.unwrap();
        checkpoint.await.unwrap().unwrap();
        assert_eq!(disk.snapshot().unwrap().pending_waiters, 0);
        assert_eq!(service.checkpoint_metrics_snapshot().checkpoints_failed, 0);
        drop(blocker);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_checkpoint_progresses_while_foreground_queue_budget_is_full() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(725);
        let mut shared_resources = writable_paged_blink_shared_resources();
        shared_resources.memory.global_bytes = 64 * 1024 * 1024;
        shared_resources.memory.per_tenant_bytes = 32 * 1024 * 1024;
        let config = LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 1,
            blink_open_waiter_limit: 4,
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold {
                threshold_bytes: u64::MAX,
            },
            blink_checkpoint_execution: BlinkCheckpointExecution::Background,
            blink_shared_resources: Some(shared_resources),
            ..LocalTenantServiceConfig::default()
        };
        let service = BlinkTenantService::new(config).unwrap();
        let database_path = service.database_path(tenant);
        create_empty_paged_service_fixture(&database_path, tenant);
        service
            .execute_request(
                tenant,
                Request::Put {
                    key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"foreground-full".to_vec()),
                    value: b"checkpoint-with-full-foreground-queue".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();
        let shard = service
            .shards
            .lock()
            .unwrap()
            .shards
            .get(&tenant)
            .cloned()
            .unwrap();
        let writable = match &shard.read_handle {
            BlinkShardReadHandle::Paged {
                writable: Some(writable),
                ..
            } => Arc::clone(writable),
            BlinkShardReadHandle::Resident(_) | BlinkShardReadHandle::Paged { .. } => {
                panic!("expected the writable paged service route")
            }
        };
        let resources = service.shared_resources.as_ref().unwrap().clone();
        let category = dodb_storage::blink::resource::ResourceCategory::QueuedForeground;
        let category_limit = shared_resources.memory.per_tenant_category_bytes[category as usize];
        let category_current = resources.snapshot().unwrap().current_by_category[category as usize];
        let blocker_bytes = category_limit.saturating_sub(category_current);
        assert!(blocker_bytes > 0);
        let foreground_blocker = resources
            .reserve(
                tenant.get(),
                category,
                dodb_storage::blink::resource::ReservationClass::Foreground,
                blocker_bytes,
            )
            .unwrap();
        assert_eq!(
            resources.snapshot().unwrap().current_by_category[category as usize],
            category_limit
        );

        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            writable.run_threshold_checkpoint(1),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(
            service.checkpoint_metrics_snapshot().checkpoints_completed,
            1
        );
        drop(foreground_blocker);
        service.shutdown_checked().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_service_backpressures_writes_at_checkpoint_debt_limit() {
        let directory = blink_test_directory();
        let tenant = TenantId::new(722);
        let mut resources = writable_paged_blink_shared_resources();
        let debt_limit = dodb_storage::blink::maximum_paged_transaction_wal_append_bytes(
            resources.maximum_working_pages,
        )
        .unwrap();
        resources.maximum_checkpoint_debt_bytes_per_tenant = debt_limit;
        resources.maximum_checkpoint_debt_bytes_global = debt_limit.saturating_mul(2);
        let service = BlinkTenantService::new(LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 1,
            blink_open_waiter_limit: 4,
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold {
                threshold_bytes: 1,
            },
            blink_checkpoint_execution: BlinkCheckpointExecution::Background,
            blink_shared_resources: Some(resources),
            ..LocalTenantServiceConfig::default()
        })
        .unwrap();
        let database_path = service.database_path(tenant);
        create_empty_paged_service_fixture(&database_path, tenant);
        service.open_shard(tenant).await.unwrap();
        let progress_started = Arc::new(AtomicBool::new(false));
        let progress_barrier = Arc::new(std::sync::Barrier::new(2));
        let progress_started_by_worker = Arc::clone(&progress_started);
        let progress_barrier_for_worker = Arc::clone(&progress_barrier);
        let blocked_progress = physical_checkpoint::submit_checkpoint_progress(move || {
            progress_started_by_worker.store(true, Ordering::Release);
            progress_barrier_for_worker.wait();
        })
        .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while !progress_started.load(Ordering::Acquire) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        service
            .execute_request(
                tenant,
                Request::Put {
                    key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"first".to_vec()),
                    value: b"first-debt-bearing-value".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if service
                    .checkpoint_metrics_snapshot()
                    .per_tenant
                    .get(&tenant.get())
                    .is_some_and(|metrics| metrics.checkpoint_requests_queued == 1)
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(
            service.checkpoint_metrics_snapshot().per_tenant[&tenant.get()]
                .checkpoint_debt_bytes_current
                <= debt_limit
        );
        let rejected = service
            .execute_request(
                tenant,
                Request::Put {
                    key: dodb_core::DocumentKey::new(b"pk".to_vec(), b"second".to_vec()),
                    value: b"must-wait-for-checkpoint-debt".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap_err();
        assert!(
            rejected
                .to_string()
                .contains("checkpoint debt admission limit")
        );
        tokio::task::spawn_blocking({
            let progress_barrier = Arc::clone(&progress_barrier);
            move || progress_barrier.wait()
        })
        .await
        .unwrap();
        blocked_progress.await.unwrap();
        service
            .checkpoint_metrics_handle()
            .wait_for_coordinators_idle()
            .await;
        let metrics = service.checkpoint_metrics_snapshot();
        assert_eq!(metrics.checkpoints_completed, 1);
        assert!(metrics.per_tenant[&tenant.get()].checkpoint_debt_bytes_current == 0);
        service.shutdown_checked().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_writer_checkpoint_releases_the_global_worker_between_slices() {
        let directory = blink_test_directory();
        let hot_tenant = TenantId::new(719);
        let small_tenant = TenantId::new(720);
        let config = LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            max_open_shards: 2,
            blink_open_waiter_limit: 4,
            blink_wal_layout: BlinkWalLayout::SegmentedBases {
                max_segment_bytes: 1_048_576,
                coalesce: true,
            },
            blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold {
                threshold_bytes: 524_288,
            },
            blink_shared_resources: Some(writable_paged_blink_shared_resources()),
            ..LocalTenantServiceConfig::default()
        };
        let service = BlinkTenantService::new(config).unwrap();
        create_empty_paged_service_fixture(&service.database_path(hot_tenant), hot_tenant);
        create_empty_paged_service_fixture(&service.database_path(small_tenant), small_tenant);
        let (hot_result, small_result) = tokio::join!(
            async {
                for key_number in 0..80u32 {
                    let key = dodb_core::DocumentKey::new(
                        b"hot".to_vec(),
                        key_number.to_be_bytes().to_vec(),
                    );
                    let response = service
                        .execute_request(
                            hot_tenant,
                            Request::Put {
                                key,
                                value: vec![key_number as u8; 8_192],
                            },
                            ExecutionBudget::new(1024),
                        )
                        .await?;
                    assert!(matches!(response, Response::Put(_)));
                }
                Result::<(), Error>::Ok(())
            },
            async {
                for key_number in 0..32u32 {
                    let key = dodb_core::DocumentKey::new(
                        b"small".to_vec(),
                        key_number.to_be_bytes().to_vec(),
                    );
                    let response = service
                        .execute_request(
                            small_tenant,
                            Request::Put {
                                key,
                                value: vec![key_number as u8; 16_384],
                            },
                            ExecutionBudget::new(1024),
                        )
                        .await?;
                    assert!(matches!(response, Response::Put(_)));
                }
                Result::<(), Error>::Ok(())
            }
        );
        hot_result.unwrap();
        small_result.unwrap();
        let checkpoint_metrics = service.checkpoint_metrics_snapshot();
        assert!(checkpoint_metrics.checkpoints_completed > 0);
        let hot_metrics = &checkpoint_metrics.per_tenant[&hot_tenant.get()];
        let small_metrics = &checkpoint_metrics.per_tenant[&small_tenant.get()];
        assert!(hot_metrics.checkpoint_requests_queued > 0);
        assert!(small_metrics.checkpoint_requests_queued > 0);
        assert_eq!(hot_metrics.checkpoint_requests_pending, 0);
        assert_eq!(small_metrics.checkpoint_requests_pending, 0);
        assert!(hot_metrics.service_slices >= 4);
        assert!(small_metrics.service_slices > 0);
        service.shutdown_checked().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 5)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_writer_fairness_serves_four_active_tenants_under_synchronized_and_staggered_pressure()
     {
        for (scenario_number, staggered) in [(0u64, false), (1, true)] {
            let directory = blink_test_directory();
            let tenants = (0..4u64)
                .map(|tenant_offset| TenantId::new(7_300 + scenario_number * 10 + tenant_offset))
                .collect::<Vec<_>>();
            let mut shared_resources = writable_paged_blink_shared_resources();
            shared_resources.memory.max_tenants = tenants.len();
            shared_resources.memory.max_pending_waiters = 8;
            shared_resources.disk.max_tenants = tenants.len();
            let service = Arc::new(
                BlinkTenantService::new(LocalTenantServiceConfig {
                    data_dir: directory.path().to_path_buf(),
                    max_open_shards: tenants.len(),
                    blink_open_waiter_limit: 8,
                    blink_wal_layout: BlinkWalLayout::SegmentedBases {
                        max_segment_bytes: 1_048_576,
                        coalesce: true,
                    },
                    blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold {
                        threshold_bytes: 64 * 1024,
                    },
                    blink_shared_resources: Some(shared_resources),
                    ..LocalTenantServiceConfig::default()
                })
                .unwrap(),
            );
            for tenant in &tenants {
                create_empty_paged_service_fixture(&service.database_path(*tenant), *tenant);
                service.open_shard(*tenant).await.unwrap();
            }

            let start_barrier = Arc::new(tokio::sync::Barrier::new(tenants.len() + 1));
            let mut writers = Vec::with_capacity(tenants.len());
            for (tenant_index, tenant) in tenants.iter().copied().enumerate() {
                let service = Arc::clone(&service);
                let start_barrier = Arc::clone(&start_barrier);
                let initial_delay = if staggered {
                    std::time::Duration::from_millis(15 * tenant_index as u64)
                } else {
                    std::time::Duration::ZERO
                };
                writers.push(tokio::spawn(async move {
                    start_barrier.wait().await;
                    tokio::time::sleep(initial_delay).await;
                    let write_count = if tenant_index == 0 { 64 } else { 20 };
                    let value_size = if tenant_index == 0 { 8_192 } else { 4_096 };
                    for write_number in 0..write_count {
                        let key = dodb_core::DocumentKey::new(
                            b"fairness".to_vec(),
                            (write_number as u32).to_be_bytes().to_vec(),
                        );
                        service
                            .execute_request(
                                tenant,
                                Request::Put {
                                    key,
                                    value: vec![tenant_index as u8; value_size],
                                },
                                ExecutionBudget::new(1024),
                            )
                            .await?;
                    }
                    Result::<(), Error>::Ok(())
                }));
            }
            start_barrier.wait().await;
            for writer in writers {
                writer.await.unwrap().unwrap();
            }
            service
                .checkpoint_metrics_handle()
                .wait_for_coordinators_idle()
                .await;

            let metrics = service.checkpoint_metrics_snapshot();
            assert!(metrics.checkpoints_completed > 0);
            let mut maximum_wait_nanos = 0;
            for tenant in tenants {
                let tenant_metrics = &metrics.per_tenant[&tenant.get()];
                assert!(tenant_metrics.checkpoint_requests_queued > 0);
                assert_eq!(tenant_metrics.checkpoint_requests_pending, 0);
                assert!(tenant_metrics.service_slices > 0);
                maximum_wait_nanos =
                    maximum_wait_nanos.max(tenant_metrics.checkpoint_request_wait_nanos_max);
            }
            assert!(
                maximum_wait_nanos <= 10_000_000_000,
                "four-tenant maximum checkpoint service wait exceeded 10 seconds in staggered={staggered}: {maximum_wait_nanos} ns"
            );
            println!(
                "four-tenant pressure staggered={staggered} maximum_wait_nanos={maximum_wait_nanos} per_tenant={:?}",
                metrics.per_tenant
            );
            service.shutdown_checked().await.unwrap();
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[cfg(feature = "blink-demand-paging")]
    async fn paged_writer_service_failures_release_admission_and_recover_atomic_state() {
        for (tenant_id, fault_point, publication_fault, expected_value) in [
            (721, "before_wal_append", false, Some(None)),
            (722, "during_wal_sync", false, None),
            (
                723,
                "before_paged_generation_publication",
                true,
                Some(Some(b"failure-boundary".as_slice())),
            ),
        ] {
            let directory = blink_test_directory();
            let tenant = TenantId::new(tenant_id);
            let config = LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                max_open_shards: 1,
                blink_open_waiter_limit: 4,
                blink_wal_layout: BlinkWalLayout::SegmentedBases {
                    max_segment_bytes: 1_048_576,
                    coalesce: true,
                },
                blink_shared_resources: Some(writable_paged_blink_shared_resources()),
                ..LocalTenantServiceConfig::default()
            };
            let service = BlinkTenantService::new(config.clone()).unwrap();
            create_empty_paged_service_fixture(&service.database_path(tenant), tenant);
            let shard = service.open_shard(tenant).await.unwrap();
            let writable = match &shard.read_handle {
                BlinkShardReadHandle::Paged {
                    writable: Some(writable),
                    ..
                } => Arc::clone(writable),
                _ => panic!("paged writer shard was not opened"),
            };
            {
                let mut writer = writable.writer.lock().unwrap();
                let fault = Box::new(FailAtOnce {
                    point: fault_point,
                    fired: false,
                });
                if publication_fault {
                    writer.set_boxed_publication_fault_injector(fault);
                } else {
                    writer.set_boxed_wal_fault_injector(fault);
                }
            }
            let key = dodb_core::DocumentKey::new(b"pk".to_vec(), b"fault".to_vec());
            assert!(
                service
                    .execute_request(
                        tenant,
                        Request::Put {
                            key: key.clone(),
                            value: b"failure-boundary".to_vec(),
                        },
                        ExecutionBudget::new(1024),
                    )
                    .await
                    .is_err()
            );
            let resources = service.shared_resources.as_ref().unwrap();
            let snapshot = resources.snapshot().unwrap();
            assert_eq!(
                snapshot.current_by_category
                    [dodb_storage::blink::resource::ResourceCategory::QueuedForeground as usize],
                0
            );
            assert_eq!(
                snapshot.current_by_category
                    [dodb_storage::blink::resource::ResourceCategory::WritePreparation as usize],
                0
            );
            drop(writable);
            drop(shard);
            service.shutdown_checked().await.unwrap();

            let reopened_service = BlinkTenantService::new(config).unwrap();
            let actual_value = match reopened_service
                .execute_request(tenant, Request::Get { key }, ExecutionBudget::new(1024))
                .await
                .unwrap()
            {
                Response::Get(state) => state.value().map(<[u8]>::to_vec),
                response => panic!("unexpected response: {response:?}"),
            };
            match expected_value {
                Some(expected_value) => {
                    assert_eq!(actual_value.as_deref(), expected_value);
                }
                None => assert!(
                    actual_value.is_none()
                        || actual_value.as_deref() == Some(b"failure-boundary".as_slice())
                ),
            }
            reopened_service.shutdown_checked().await.unwrap();
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn blocked_blink_open_does_not_hold_registry_and_same_tenant_open_coalesces() {
        let directory = blink_test_directory();
        let service = Arc::new(
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                blink_open_worker_count: 2,
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let gated_tenant = TenantId::new(101);
        service.set_open_test_gate(gated_tenant, Arc::new(Semaphore::new(0)));
        let leader_service = Arc::clone(&service);
        let leader = tokio::spawn(async move { leader_service.open_shard(gated_tenant).await });
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if service
                    .shards
                    .lock()
                    .unwrap()
                    .opening
                    .contains(&gated_tenant)
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let mut coalesced = Vec::new();
        for _waiter_number in 0..6 {
            let waiter_service = Arc::clone(&service);
            coalesced.push(tokio::spawn(async move {
                waiter_service.open_shard(gated_tenant).await
            }));
        }
        let reader_service = Arc::clone(&service);
        let reader = tokio::spawn(async move { reader_service.read_shard(gated_tenant).await });
        let unrelated = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            service.open_shard(TenantId::new(102)),
        )
        .await
        .unwrap()
        .unwrap();
        let gate = service
            .open_test_gate
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .1
            .clone();
        gate.add_permits(1);
        let opened = leader.await.unwrap().unwrap();
        assert!(!Arc::ptr_eq(&opened, &unrelated));
        for waiter in coalesced {
            assert!(Arc::ptr_eq(&opened, &waiter.await.unwrap().unwrap()));
        }
        assert!(Arc::ptr_eq(
            &opened,
            &reader.await.unwrap().unwrap().unwrap()
        ));
        assert_eq!(service.shards.lock().unwrap().opening.len(), 0);
        service.shutdown_checked().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn blink_open_waiters_and_inflight_tenants_obey_configured_limits() {
        let directory = blink_test_directory();
        let service = Arc::new(
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                max_open_shards: 1,
                blink_open_worker_count: 1,
                blink_open_waiter_limit: 1,
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let tenant = TenantId::new(107);
        service.set_open_test_gate(tenant, Arc::new(Semaphore::new(0)));
        let leader_service = Arc::clone(&service);
        let leader = tokio::spawn(async move { leader_service.open_shard(tenant).await });
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if service.shards.lock().unwrap().opening.contains(&tenant) {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let waiter_service = Arc::clone(&service);
        let waiter = tokio::spawn(async move { waiter_service.open_shard(tenant).await });
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while service.open_waiters.available_permits() != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(matches!(
            service.open_shard(tenant).await,
            Err(Error::Overloaded(_))
        ));
        assert!(matches!(
            service.open_shard(TenantId::new(108)).await,
            Err(Error::Overloaded(_))
        ));
        let gate = service
            .open_test_gate
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .1
            .clone();
        gate.add_permits(1);
        let opened = leader.await.unwrap().unwrap();
        assert!(Arc::ptr_eq(&opened, &waiter.await.unwrap().unwrap()));
        assert_eq!(service.open_waiters.available_permits(), 1);
        assert!(service.shards.lock().unwrap().opening.is_empty());
        service.shutdown_checked().await.unwrap();
    }

    #[tokio::test]
    async fn failed_blink_open_releases_tenant_reservation() {
        let directory = blink_test_directory();
        let service = BlinkTenantService::new(LocalTenantServiceConfig {
            data_dir: directory.path().to_path_buf(),
            ..LocalTenantServiceConfig::default()
        })
        .unwrap();
        let tenant = TenantId::new(103);
        let database_path = service.database_path(tenant);
        std::fs::create_dir_all(database_path.parent().unwrap()).unwrap();
        std::fs::write(&database_path, b"corrupt database").unwrap();
        assert!(service.open_shard(tenant).await.is_err());
        assert!(service.shards.lock().unwrap().opening.is_empty());
        service.shutdown_checked().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancelled_blink_open_releases_reservation_and_worker_capacity() {
        let directory = blink_test_directory();
        let service = Arc::new(
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                blink_open_worker_count: 1,
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let tenant = TenantId::new(106);
        service.set_open_test_gate(tenant, Arc::new(Semaphore::new(0)));
        let open_service = Arc::clone(&service);
        let opening = tokio::spawn(async move { open_service.open_shard(tenant).await });
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if service.shards.lock().unwrap().opening.contains(&tenant) {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        opening.abort();
        let join_error = match opening.await {
            Ok(_) => panic!("cancelled B-link open task completed"),
            Err(error) => error,
        };
        assert!(join_error.is_cancelled());
        assert!(service.shards.lock().unwrap().opening.is_empty());
        service.set_open_test_gate(tenant, Arc::new(Semaphore::new(1)));
        let shard = service.open_shard(tenant).await.unwrap();
        assert!(Arc::ptr_eq(
            &shard,
            &service.read_shard(tenant).await.unwrap().unwrap()
        ));
        assert!(service.shards.lock().unwrap().opening.is_empty());
        service.shutdown_checked().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn blink_shutdown_bounds_wait_for_inflight_open_and_drains_it() {
        let directory = blink_test_directory();
        let service = Arc::new(
            BlinkTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_path_buf(),
                blink_open_shutdown_timeout: std::time::Duration::from_millis(20),
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        service.open_shard(TenantId::new(105)).await.unwrap();
        let tenant = TenantId::new(104);
        service.set_open_test_gate(tenant, Arc::new(Semaphore::new(0)));
        let open_service = Arc::clone(&service);
        let opening = tokio::spawn(async move { open_service.open_shard(tenant).await });
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if service.shards.lock().unwrap().opening.contains(&tenant) {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(matches!(
            service.shutdown_checked().await,
            Err(Error::Overloaded(_))
        ));
        assert!(
            service
                .shards
                .lock()
                .unwrap()
                .shards
                .contains_key(&TenantId::new(105))
        );
        service
            .open_test_gate
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .1
            .add_permits(1);
        assert!(opening.await.unwrap().is_err());
        service.shutdown_checked().await.unwrap();
        assert!(service.shards.lock().unwrap().opening.is_empty());
    }

    #[test]
    fn database_uuid_includes_all_shard_bits() {
        let tenant = TenantId::new(9);
        assert_ne!(
            database_uuid(tenant, ShardId::new(1)),
            database_uuid(tenant, ShardId::new(0x1_0000_0001))
        );
    }

    #[test]
    fn logical_database_uuid_is_stable_for_reopen() {
        let first = database_uuid(TenantId::new(41), ShardId::new(41));
        let reopened = database_uuid(TenantId::new(41), ShardId::new(41));
        assert_eq!(first, reopened);
        assert_ne!(first, database_uuid(TenantId::new(42), ShardId::new(41)));
    }

    #[test]
    fn request_latency_histogram_records_cumulative_operation_buckets() {
        let metrics = ServerMetrics::default();
        metrics.record_request_latency(1, 40_000);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.request_latency_buckets[1][0], 0);
        assert_eq!(snapshot.request_latency_buckets[1][1], 1);
        assert_eq!(
            snapshot.request_latency_buckets[1][REQUEST_LATENCY_BUCKETS_SECONDS.len()],
            1
        );
        assert_eq!(snapshot.request_latency_buckets[0][1], 0);
    }

    #[test]
    fn transport_classification_separates_disconnects_from_actionable_failures() {
        let application_closed = classify_connection_error(
            TransportStage::AcceptStream,
            &ConnectionError::ApplicationClosed(quinn::ApplicationClose {
                error_code: VarInt::from_u32(0),
                reason: bytes::Bytes::new(),
            }),
        );
        assert_eq!(
            application_closed.reason,
            TransportReason::ApplicationClosed
        );
        assert_eq!(application_closed.outcome, TransportOutcome::Benign);

        let locally_closed =
            classify_connection_error(TransportStage::Handshake, &ConnectionError::LocallyClosed);
        assert_eq!(locally_closed.reason, TransportReason::LocallyClosed);
        assert_eq!(locally_closed.outcome, TransportOutcome::Benign);

        let transport_failure = classify_connection_failure(
            TransportStage::Handshake,
            ConnectionFailureKind::TransportError,
        );
        assert_eq!(transport_failure.reason, TransportReason::TransportError);
        assert_eq!(transport_failure.outcome, TransportOutcome::Error);

        let idle_timeout =
            classify_connection_error(TransportStage::AcceptStream, &ConnectionError::TimedOut);
        assert_eq!(idle_timeout.reason, TransportReason::TimedOut);
        assert_eq!(idle_timeout.outcome, TransportOutcome::Benign);

        let active_request_timeout =
            classify_connection_error(TransportStage::ReadHeader, &ConnectionError::TimedOut);
        assert_eq!(active_request_timeout.outcome, TransportOutcome::Error);
    }

    #[test]
    fn peer_stream_reset_and_stop_are_benign_diagnostic_events() {
        let read_reset = classify_read_error(
            TransportStage::ReadHeader,
            &ReadExactError::ReadError(ReadError::Reset(VarInt::from_u32(7))),
        );
        assert_eq!(read_reset.reason, TransportReason::PeerReset);
        assert_eq!(read_reset.outcome, TransportOutcome::Benign);

        let write_stopped = classify_write_error(
            TransportStage::WriteResponse,
            &WriteError::Stopped(VarInt::from_u32(8)),
        );
        assert_eq!(write_stopped.reason, TransportReason::PeerStopped);
        assert_eq!(write_stopped.outcome, TransportOutcome::Benign);
    }

    #[test]
    fn transport_metric_labels_are_bounded_and_only_actionable_events_raise_errors() {
        for stage in TransportStage::ALL {
            assert!(!stage.as_str().is_empty());
        }
        for reason in TransportReason::ALL {
            assert!(!reason.as_str().is_empty());
        }

        let metrics = ServerMetrics::default();
        let benign = TransportClassification {
            reason: TransportReason::PeerReset,
            outcome: TransportOutcome::Benign,
        };
        metrics.record_transport_event(TransportStage::ReadHeader, benign, "peer reset");
        assert_eq!(metrics.snapshot().transport_errors, 0);

        let actionable = TransportClassification {
            reason: TransportReason::TransportError,
            outcome: TransportOutcome::Error,
        };
        metrics.record_transport_event(
            TransportStage::Handshake,
            actionable,
            "bounded test detail",
        );
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.transport_errors, 1);
        assert_eq!(snapshot.transport_events.len(), 2);
        assert!(snapshot.transport_events.iter().any(|event| {
            event.stage == "read_header"
                && event.reason == "peer_reset"
                && event.outcome == "benign"
                && event.value == 1
        }));
        assert!(snapshot.transport_events.iter().any(|event| {
            event.stage == "handshake"
                && event.reason == "transport_error"
                && event.outcome == "error"
                && event.value == 1
        }));
    }

    #[test]
    fn storage_metrics_count_database_wal_and_persisted_files_separately_from_open_shards() {
        let data_dir = tempfile::tempdir().unwrap();
        std::fs::write(data_dir.path().join("tenant-1-shard-1.db"), [1_u8; 19]).unwrap();
        std::fs::write(data_dir.path().join("tenant-1-shard-1.wal"), [2_u8; 5]).unwrap();
        std::fs::write(data_dir.path().join("tenant-2-shard-2.wal"), [3_u8; 7]).unwrap();
        std::fs::write(data_dir.path().join("unrelated.db"), [4_u8; 101]).unwrap();

        let snapshot = collect_storage_metrics(data_dir.path(), 1).unwrap();
        assert_eq!(snapshot.database_file_bytes, 19);
        assert_eq!(snapshot.wal_file_bytes, 12);
        assert_eq!(snapshot.persisted_shards, 2);
        assert_eq!(snapshot.open_shards, 1);
    }
}
