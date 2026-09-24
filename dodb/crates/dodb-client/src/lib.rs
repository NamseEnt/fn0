use std::fmt;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dodb_core::{DocumentKey, Revision, RevisionState, TenantId, TransactionRequest};
use dodb_protocol::{
    ApplicationError, MutationOutcome, ProtocolError, ProtocolLimits, ResponseEnvelope,
    decode_header, decode_response_parts, encode_request,
};
use dodb_service::{Document, Response, TransactionOutcome};
use quinn::rustls::pki_types::CertificateDer;
use quinn::{ClientConfig as QuinnClientConfig, Endpoint, VarInt};
use tokio::sync::Notify;

const RECONNECT_BACKOFF_BASE: Duration = Duration::from_millis(50);
const RECONNECT_BACKOFF_MAX: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub enum ClientError {
    InvalidInput(String),
    Tls(String),
    Endpoint(String),
    Transport(String),
    Protocol(ProtocolError),
    Application(Box<ApplicationError>),
    UnknownMutationOutcome {
        detail: String,
        cause: Option<Box<ApplicationError>>,
    },
}

impl fmt::Display for ClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(detail) => write!(formatter, "invalid client input: {detail}"),
            Self::Tls(detail) => write!(formatter, "TLS configuration error: {detail}"),
            Self::Endpoint(detail) => write!(formatter, "QUIC endpoint error: {detail}"),
            Self::Transport(detail) => write!(formatter, "transport error: {detail}"),
            Self::Protocol(error) => write!(formatter, "protocol error: {error}"),
            Self::Application(error) => {
                write!(
                    formatter,
                    "application error {:?}: {}",
                    error.kind, error.detail
                )
            }
            Self::UnknownMutationOutcome { detail, .. } => {
                write!(formatter, "mutation outcome is unknown: {detail}")
            }
        }
    }
}

impl std::error::Error for ClientError {}

impl ClientError {
    fn invalidates_connection(&self) -> bool {
        matches!(
            self,
            Self::Endpoint(_)
                | Self::Transport(_)
                | Self::Protocol(_)
                | Self::UnknownMutationOutcome { cause: None, .. }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientTlsConfig {
    root_certificates: Vec<Vec<u8>>,
}

impl ClientTlsConfig {
    pub fn from_der(root_certificates: Vec<Vec<u8>>) -> Result<Self, ClientError> {
        if root_certificates.is_empty() {
            return Err(ClientError::Tls(
                "at least one trusted root certificate is required".to_owned(),
            ));
        }
        Ok(Self { root_certificates })
    }

    pub fn from_pem(root_certificates_pem: &[u8]) -> Result<Self, ClientError> {
        let mut reader = Cursor::new(root_certificates_pem);
        let certificates = rustls_pemfile::certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| ClientError::Tls(error.to_string()))?
            .into_iter()
            .map(|certificate| certificate.to_vec())
            .collect::<Vec<_>>();
        Self::from_der(certificates)
    }

    fn to_quinn_config(&self) -> Result<QuinnClientConfig, ClientError> {
        let mut roots = quinn::rustls::RootCertStore::empty();
        for certificate in &self.root_certificates {
            roots
                .add(CertificateDer::from(certificate.clone()))
                .map_err(|error| ClientError::Tls(error.to_string()))?;
        }
        QuinnClientConfig::with_root_certificates(Arc::new(roots))
            .map_err(|error| ClientError::Tls(error.to_string()))
    }
}

#[derive(Clone)]
pub struct DodbConnection {
    inner: Arc<DodbConnectionInner>,
}

struct DodbConnectionInner {
    endpoint: Endpoint,
    server_addr: SocketAddr,
    server_name: String,
    limits: ProtocolLimits,
    state: Mutex<ConnectionState>,
    reconnect_notify: Notify,
}

struct ConnectionState {
    current: Option<InstalledConnection>,
    generation: u64,
    reconnecting: bool,
    failures: u32,
    next_attempt_at: Option<Instant>,
    closed: bool,
}

#[derive(Clone)]
struct InstalledConnection {
    connection: quinn::Connection,
    generation: u64,
}

impl DodbConnection {
    /// Opens one authenticated QUIC connection that can serve any number of
    /// tenant-scoped handles.
    pub async fn connect(
        bind_addr: SocketAddr,
        server_addr: SocketAddr,
        server_name: &str,
        tls: ClientTlsConfig,
        limits: ProtocolLimits,
    ) -> Result<Self, ClientError> {
        let mut endpoint = Endpoint::client(bind_addr)
            .map_err(|error| ClientError::Endpoint(error.to_string()))?;
        endpoint.set_default_client_config(tls.to_quinn_config()?);
        let connection = Self::from_endpoint(endpoint, server_addr, server_name, limits)?;
        connection.ensure_connection().await?;
        Ok(connection)
    }

    pub async fn connect_with_endpoint(
        mut endpoint: Endpoint,
        server_addr: SocketAddr,
        server_name: &str,
        tls: ClientTlsConfig,
        limits: ProtocolLimits,
    ) -> Result<Self, ClientError> {
        endpoint.set_default_client_config(tls.to_quinn_config()?);
        let connection = Self::from_endpoint(endpoint, server_addr, server_name, limits)?;
        connection.ensure_connection().await?;
        Ok(connection)
    }

    /// Creates a shared connection manager without dialing until its first request.
    pub fn connect_lazy(
        bind_addr: SocketAddr,
        server_addr: SocketAddr,
        server_name: &str,
        tls: ClientTlsConfig,
        limits: ProtocolLimits,
    ) -> Result<Self, ClientError> {
        let mut endpoint = Endpoint::client(bind_addr)
            .map_err(|error| ClientError::Endpoint(error.to_string()))?;
        endpoint.set_default_client_config(tls.to_quinn_config()?);
        Self::from_endpoint(endpoint, server_addr, server_name, limits)
    }

    /// Creates a shared connection manager around an existing endpoint without
    /// dialing until its first request.
    pub fn connect_lazy_with_endpoint(
        mut endpoint: Endpoint,
        server_addr: SocketAddr,
        server_name: &str,
        tls: ClientTlsConfig,
        limits: ProtocolLimits,
    ) -> Result<Self, ClientError> {
        endpoint.set_default_client_config(tls.to_quinn_config()?);
        Self::from_endpoint(endpoint, server_addr, server_name, limits)
    }

    fn from_endpoint(
        endpoint: Endpoint,
        server_addr: SocketAddr,
        server_name: &str,
        limits: ProtocolLimits,
    ) -> Result<Self, ClientError> {
        limits.validate().map_err(ClientError::Protocol)?;
        Ok(Self {
            inner: Arc::new(DodbConnectionInner {
                endpoint,
                server_addr,
                server_name: server_name.to_owned(),
                limits,
                state: Mutex::new(ConnectionState {
                    current: None,
                    generation: 0,
                    reconnecting: false,
                    failures: 0,
                    next_attempt_at: None,
                    closed: false,
                }),
                reconnect_notify: Notify::new(),
            }),
        })
    }

    pub fn for_tenant(&self, tenant: TenantId) -> DodbClient {
        DodbClient {
            connection: self.clone(),
            tenant,
        }
    }

    pub fn remote_addr(&self) -> SocketAddr {
        self.inner.server_addr
    }

    /// Explicitly shuts down the shared endpoint and connection.
    pub fn close(&self) {
        let connection = {
            let mut state = self.inner.state.lock().expect("connection state poisoned");
            state.closed = true;
            state.reconnecting = false;
            state.next_attempt_at = None;
            state.current.take()
        };
        if let Some(connection) = connection {
            connection
                .connection
                .close(VarInt::from_u32(0), b"client shutdown");
        }
        self.inner
            .endpoint
            .close(VarInt::from_u32(0), b"client shutdown");
        self.inner.reconnect_notify.notify_waiters();
    }

    async fn ensure_connection(&self) -> Result<InstalledConnection, ClientError> {
        loop {
            let notified = self.inner.reconnect_notify.notified();
            let action = {
                let mut state = self.inner.state.lock().expect("connection state poisoned");
                if state.closed {
                    return Err(ClientError::Endpoint(
                        "client connection is explicitly closed".to_owned(),
                    ));
                }
                if let Some(current) = &state.current
                    && current.connection.close_reason().is_none()
                {
                    return Ok(current.clone());
                }
                state.current.take();
                if state.reconnecting {
                    ReconnectAction::Wait
                } else if let Some(next_attempt_at) = state.next_attempt_at {
                    if next_attempt_at > Instant::now() {
                        ReconnectAction::Sleep(
                            next_attempt_at.saturating_duration_since(Instant::now()),
                        )
                    } else {
                        state.next_attempt_at = None;
                        state.reconnecting = true;
                        ReconnectAction::Dial
                    }
                } else {
                    state.reconnecting = true;
                    ReconnectAction::Dial
                }
            };
            match action {
                ReconnectAction::Wait => notified.await,
                ReconnectAction::Sleep(duration) => tokio::time::sleep(duration).await,
                ReconnectAction::Dial => return self.dial().await,
            }
        }
    }

    async fn dial(&self) -> Result<InstalledConnection, ClientError> {
        let result = match self
            .inner
            .endpoint
            .connect(self.inner.server_addr, &self.inner.server_name)
        {
            Ok(connecting) => connecting
                .await
                .map_err(|error| ClientError::Transport(error.to_string())),
            Err(error) => Err(ClientError::Endpoint(error.to_string())),
        };
        let result = {
            let mut state = self.inner.state.lock().expect("connection state poisoned");
            state.reconnecting = false;
            match result {
                Ok(connection) if state.closed => {
                    connection.close(VarInt::from_u32(0), b"client shutdown");
                    Err(ClientError::Endpoint(
                        "client connection is explicitly closed".to_owned(),
                    ))
                }
                Ok(connection) => {
                    state.generation = state.generation.wrapping_add(1);
                    state.failures = 0;
                    state.next_attempt_at = None;
                    let installed = InstalledConnection {
                        connection,
                        generation: state.generation,
                    };
                    state.current = Some(installed.clone());
                    Ok(installed)
                }
                Err(error) => {
                    if !state.closed {
                        state.failures = state.failures.saturating_add(1);
                        state.next_attempt_at =
                            Some(Instant::now() + reconnect_delay(state.failures));
                    }
                    Err(error)
                }
            }
        };
        self.inner.reconnect_notify.notify_waiters();
        result
    }

    fn invalidate(&self, generation: u64) {
        let connection = {
            let mut state = self.inner.state.lock().expect("connection state poisoned");
            if state.closed
                || !generation_is_current(
                    state.current.as_ref().map(|current| current.generation),
                    generation,
                )
            {
                None
            } else {
                state.current.take()
            }
        };
        if let Some(connection) = connection {
            connection
                .connection
                .close(VarInt::from_u32(0), b"connection invalidated");
        }
    }
}

enum ReconnectAction {
    Wait,
    Sleep(Duration),
    Dial,
}

fn generation_is_current(current_generation: Option<u64>, failed_generation: u64) -> bool {
    current_generation == Some(failed_generation)
}

fn reconnect_delay(failures: u32) -> Duration {
    let shift = failures.saturating_sub(1).min(5);
    RECONNECT_BACKOFF_BASE
        .checked_mul(1u32 << shift)
        .unwrap_or(RECONNECT_BACKOFF_MAX)
        .min(RECONNECT_BACKOFF_MAX)
}

#[derive(Clone)]
pub struct DodbClient {
    connection: DodbConnection,
    tenant: TenantId,
}

impl DodbClient {
    /// Opens a connection and returns a tenant-scoped handle.
    ///
    /// New code serving multiple tenants should use [`DodbConnection::connect`]
    /// once and call [`DodbConnection::for_tenant`] for each tenant.
    pub async fn connect(
        bind_addr: std::net::SocketAddr,
        server_addr: std::net::SocketAddr,
        server_name: &str,
        tenant: TenantId,
        tls: ClientTlsConfig,
        limits: ProtocolLimits,
    ) -> Result<Self, ClientError> {
        Ok(
            DodbConnection::connect(bind_addr, server_addr, server_name, tls, limits)
                .await?
                .for_tenant(tenant),
        )
    }

    pub async fn connect_with_endpoint(
        endpoint: Endpoint,
        server_addr: std::net::SocketAddr,
        server_name: &str,
        tenant: TenantId,
        tls: ClientTlsConfig,
        limits: ProtocolLimits,
    ) -> Result<Self, ClientError> {
        Ok(
            DodbConnection::connect_with_endpoint(endpoint, server_addr, server_name, tls, limits)
                .await?
                .for_tenant(tenant),
        )
    }

    pub fn connection(&self) -> &DodbConnection {
        &self.connection
    }

    pub fn tenant(&self) -> TenantId {
        self.tenant
    }

    pub fn remote_addr(&self) -> std::net::SocketAddr {
        self.connection.remote_addr()
    }

    /// Releases this tenant handle without shutting down other handles.
    ///
    /// Call [`DodbConnection::close`] on the shared connection when the
    /// owner is ready to shut down the transport.
    pub fn close(&self) {
        // Kept as a source-compatible convenience for the original
        // tenant-bound client API. The shared connection has explicit
        // ownership and is closed through DodbConnection.
    }

    pub async fn get(&self, key: DocumentKey) -> Result<RevisionState, ClientError> {
        match self.execute(dodb_service::Request::Get { key }).await? {
            Response::Get(state) => Ok(state),
            _ => Err(unexpected_response(1, 0)),
        }
    }

    pub async fn put(&self, key: DocumentKey, value: Vec<u8>) -> Result<Revision, ClientError> {
        match self
            .execute(dodb_service::Request::Put { key, value })
            .await?
        {
            Response::Put(revision) => Ok(revision),
            _ => Err(unexpected_response(2, 0)),
        }
    }

    pub async fn delete(&self, key: DocumentKey) -> Result<Revision, ClientError> {
        match self.execute(dodb_service::Request::Delete { key }).await? {
            Response::Delete(revision) => Ok(revision),
            _ => Err(unexpected_response(3, 0)),
        }
    }

    pub async fn query(
        &self,
        pk: dodb_core::PrimaryKey,
        exclusive_after_sk: Option<dodb_core::SortKey>,
        limit: usize,
    ) -> Result<Vec<Document>, ClientError> {
        match self
            .execute(dodb_service::Request::Query {
                pk,
                exclusive_after_sk,
                limit,
            })
            .await?
        {
            Response::Query(rows) => Ok(rows),
            _ => Err(unexpected_response(4, 0)),
        }
    }

    pub async fn scan(
        &self,
        exclusive_after_key: Option<DocumentKey>,
        limit: usize,
    ) -> Result<Vec<Document>, ClientError> {
        match self
            .execute(dodb_service::Request::Scan {
                exclusive_after_key,
                limit,
            })
            .await?
        {
            Response::Scan(rows) => Ok(rows),
            _ => Err(unexpected_response(5, 0)),
        }
    }

    pub async fn transact(
        &self,
        request: TransactionRequest,
    ) -> Result<TransactionOutcome, ClientError> {
        match self
            .execute(dodb_service::Request::Transact { request })
            .await?
        {
            Response::Transact(outcome) => Ok(outcome),
            _ => Err(unexpected_response(8, 0)),
        }
    }

    async fn execute(&self, request: dodb_service::Request) -> Result<Response, ClientError> {
        let mutation = request.is_mutation();
        let expected_response_type = response_opcode_for_request(&request);
        let encoded_request = encode_request(self.tenant, &request, self.connection.inner.limits)
            .map_err(ClientError::Protocol)?;
        let installed = self.connection.ensure_connection().await?;
        let result = self
            .execute_on_connection(
                &installed.connection,
                mutation,
                expected_response_type,
                &encoded_request,
            )
            .await;
        if let Err(error) = &result
            && error.invalidates_connection()
        {
            self.connection.invalidate(installed.generation);
        }
        result
    }

    async fn execute_on_connection(
        &self,
        connection: &quinn::Connection,
        mutation: bool,
        expected_response_type: u8,
        encoded_request: &[u8],
    ) -> Result<Response, ClientError> {
        let (mut send, mut receive) = connection
            .open_bi()
            .await
            .map_err(|error| uncertain_mutation(mutation, error.to_string()))?;
        if let Err(error) = send.write_all(encoded_request).await {
            return Err(uncertain_mutation(
                mutation,
                format!("request write failed: {error}"),
            ));
        }
        if let Err(error) = send.finish() {
            return Err(uncertain_mutation(
                mutation,
                format!("request finish failed: {error}"),
            ));
        }
        let mut header_bytes = [0u8; dodb_protocol::HEADER_SIZE];
        if let Err(error) = receive.read_exact(&mut header_bytes).await {
            return Err(uncertain_mutation(
                mutation,
                format!("response header failed: {error}"),
            ));
        }
        let header =
            decode_header(&header_bytes).map_err(|error| uncertain_protocol(mutation, error))?;
        let frame_length = dodb_protocol::HEADER_SIZE.saturating_add(header.payload_length);
        if frame_length > self.connection.inner.limits.max_response_frame_size {
            return Err(uncertain_protocol(
                mutation,
                ProtocolError::PayloadTooLarge {
                    length: frame_length,
                    maximum: self.connection.inner.limits.max_response_frame_size,
                },
            ));
        }
        let mut payload = vec![0u8; header.payload_length];
        if let Err(error) = receive.read_exact(&mut payload).await {
            return Err(uncertain_mutation(
                mutation,
                format!("response payload failed: {error}"),
            ));
        }
        let trailing = receive
            .read_to_end(1)
            .await
            .map_err(|_| uncertain_protocol(mutation, ProtocolError::TrailingBytes))?;
        if !trailing.is_empty() {
            return Err(uncertain_protocol(mutation, ProtocolError::TrailingBytes));
        }
        match decode_response_parts(
            header,
            &payload,
            Some(expected_response_type),
            self.connection.inner.limits,
        )
        .map_err(|error| uncertain_protocol(mutation, error))?
        {
            ResponseEnvelope::Success(response) => Ok(response),
            ResponseEnvelope::Error(error) => {
                if mutation && error.mutation_outcome == MutationOutcome::Unknown {
                    return Err(ClientError::UnknownMutationOutcome {
                        detail: error.detail.clone(),
                        cause: Some(Box::new(error)),
                    });
                }
                Err(ClientError::Application(Box::new(error)))
            }
        }
    }
}

fn response_opcode_for_request(request: &dodb_service::Request) -> u8 {
    dodb_protocol::request_opcode(request)
}

fn unexpected_response(expected: u8, actual: u8) -> ClientError {
    ClientError::Protocol(ProtocolError::InvalidResponseType { expected, actual })
}

fn uncertain_mutation(mutation: bool, detail: String) -> ClientError {
    if mutation {
        ClientError::UnknownMutationOutcome {
            detail,
            cause: None,
        }
    } else {
        ClientError::Transport(detail)
    }
}

fn uncertain_protocol(mutation: bool, error: ProtocolError) -> ClientError {
    if mutation {
        ClientError::UnknownMutationOutcome {
            detail: error.to_string(),
            cause: None,
        }
    } else {
        ClientError::Protocol(error)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{generation_is_current, reconnect_delay};

    #[test]
    fn late_failure_cannot_invalidate_a_newer_generation() {
        assert!(!generation_is_current(Some(11), 10));
        assert!(generation_is_current(Some(10), 10));
        assert!(!generation_is_current(None, 10));
    }

    #[test]
    fn reconnect_backoff_is_bounded() {
        assert_eq!(reconnect_delay(1), Duration::from_millis(50));
        assert_eq!(reconnect_delay(2), Duration::from_millis(100));
        assert_eq!(reconnect_delay(5), Duration::from_millis(800));
        assert_eq!(reconnect_delay(6), Duration::from_secs(1));
        assert_eq!(reconnect_delay(32), Duration::from_secs(1));
    }
}
