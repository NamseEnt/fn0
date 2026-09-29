use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fmt;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

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
    TransactionConflict, TransactionRequest,
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
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

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
    pub database_config: DatabaseConfig,
    pub coordinator_config: CoordinatorConfig,
}

impl Default for LocalTenantServiceConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("."),
            max_open_shards: 1_024,
            database_config: DatabaseConfig::default(),
            coordinator_config: CoordinatorConfig::default(),
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
