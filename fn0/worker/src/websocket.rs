use crate::websocket_directory::{
    ConnectionDirectory, ConnectionOwner, WorkerIdentity, directory_from_env,
    worker_identity_from_env,
};
use crate::websocket_quic::QuicTransport;
use crate::worker_pool::{self, DispatchError, RequestEnvelope};
use base64::Engine;
use bytes::Bytes;
use dashmap::DashMap;
use fastwebsockets::handshake;
use fastwebsockets::upgrade::UpgradeFut;
use fastwebsockets::{Frame, OpCode, Payload, WebSocketError, WebSocketRead, WebSocketWrite};
use fn0::{
    Body, WebSocketCommandDispatcher, WebSocketCommandError, WebSocketCommandErrorKind,
    WebSocketCommandFuture, WebSocketConnectFuture, WebSocketDeliveryState, WebSocketMessageKind,
};
use http_body_util::{BodyExt, Empty, Full};
use hyper_util::rt::TokioIo;
use rand::RngCore;
use rustls::pki_types::ServerName;
use sha1::{Digest, Sha1};
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, watch};
use tokio_rustls::TlsConnector;
use url::Url;

const PROJECT_CONNECTION_LIMIT: usize = fn0_shared_schema::MAX_WEBSOCKET_CONNECTIONS_PER_PROJECT;
const WORKER_CONNECTION_LIMIT: usize = 10_000;
const OUTBOUND_COMMAND_CAPACITY: usize = 4;
const INBOUND_PENDING_LIMIT: usize = 4;
const CALLBACK_DEADLINE: Duration = Duration::from_secs(15);
const CLOSE_HANDSHAKE_DEADLINE: Duration = Duration::from_secs(10);
const PING_INTERVAL: Duration = Duration::from_secs(30);
const PONG_DEADLINE: Duration = Duration::from_secs(15);
const OUTBOUND_DIAL_TIMEOUT: Duration = Duration::from_secs(10);
const SINGLETON_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
const SINGLETON_SAFETY_DEADLINE: Duration = Duration::from_secs(30);

type UpgradedIo = TokioIo<hyper::upgrade::Upgraded>;
type SocketReader = WebSocketRead<ReadHalf<UpgradedIo>>;
type SocketWriter = WebSocketWrite<WriteHalf<UpgradedIo>>;
type SingletonKey = (String, String, String);
type SingletonConnectSlot =
    tokio::sync::OnceCell<Result<Arc<PreparedSingleton>, WebSocketCommandError>>;
type OutboundHandshakeRequest = (String, String, u16, hyper::Request<Empty<Bytes>>, String);

struct PreparedSingleton {
    connection_id: String,
    project_id: String,
    route_uri: hyper::Uri,
    response_headers: hyper::HeaderMap,
    lease_activation_sender: Mutex<Option<oneshot::Sender<()>>>,
    message_ready_sender: Mutex<Option<oneshot::Sender<()>>>,
    activated: Arc<AtomicBool>,
    activation: tokio::sync::OnceCell<watch::Receiver<Option<Result<(), WebSocketCommandError>>>>,
}

enum OutboundConnectResult {
    Active(String),
    Prepared(Arc<PreparedSingleton>),
}

#[derive(Clone)]
struct SingletonBinding {
    key: SingletonKey,
    slot: Arc<SingletonConnectSlot>,
    singleton_id: String,
    claim_token: String,
    initial_lease_deadline: i64,
    activated: Arc<AtomicBool>,
}

struct OutboundExecutor;

impl<Fut> hyper::rt::Executor<Fut> for OutboundExecutor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, future: Fut) {
        tokio::spawn(future);
    }
}

fn outbound_tls_connector() -> anyhow::Result<TlsConnector> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(TlsConnector::from(Arc::new(config)))
}

#[derive(Clone, Debug)]
pub struct DisconnectInfo {
    close_code: Option<u16>,
    reason: Option<String>,
    cause: &'static str,
}

impl DisconnectInfo {
    fn application() -> Self {
        Self {
            close_code: Some(1000),
            reason: None,
            cause: "application",
        }
    }

    fn deployment() -> Self {
        Self {
            close_code: Some(1012),
            reason: None,
            cause: "deployment",
        }
    }

    fn heartbeat_timeout() -> Self {
        Self {
            close_code: None,
            reason: None,
            cause: "heartbeat-timeout",
        }
    }

    fn transport_error() -> Self {
        Self {
            close_code: None,
            reason: None,
            cause: "transport-error",
        }
    }

    fn internal_error() -> Self {
        Self {
            close_code: Some(1011),
            reason: None,
            cause: "internal-error",
        }
    }

    fn protocol_error(code: u16) -> Self {
        Self {
            close_code: Some(code),
            reason: None,
            cause: "protocol-error",
        }
    }
}

pub enum CapacityError {
    Project,
    Worker,
}

pub(crate) struct CapacityGuard {
    project_count: Arc<AtomicUsize>,
    worker_count: Arc<AtomicUsize>,
    project_generation: Arc<std::sync::atomic::AtomicU64>,
    reserved_generation: u64,
}

impl Drop for CapacityGuard {
    fn drop(&mut self) {
        self.project_count.fetch_sub(1, Ordering::AcqRel);
        self.worker_count.fetch_sub(1, Ordering::AcqRel);
    }
}

struct ConnectionEntry {
    project_id: String,
    command_sender: mpsc::Sender<SocketCommand>,
    closing: AtomicBool,
    closed_receiver: watch::Receiver<bool>,
    control_sender: mpsc::UnboundedSender<WriterControl>,
}

struct RegisteredConnection {
    command_receiver: mpsc::Receiver<SocketCommand>,
    control_sender: mpsc::UnboundedSender<WriterControl>,
    control_receiver: mpsc::UnboundedReceiver<WriterControl>,
    closed_sender: watch::Sender<bool>,
}

enum SocketCommand {
    Send {
        message_kind: WebSocketMessageKind,
        body: Body,
        ready_sender: oneshot::Sender<()>,
        response_sender: oneshot::Sender<Result<(), WebSocketCommandError>>,
        deadline: tokio::time::Instant,
    },
    Close {
        code: u16,
        info: DisconnectInfo,
        response_sender: Option<oneshot::Sender<Result<(), WebSocketCommandError>>>,
    },
}

pub(crate) struct AdmittedSend {
    pub ready_receiver: oneshot::Receiver<()>,
    pub response_receiver: oneshot::Receiver<Result<(), WebSocketCommandError>>,
}

enum WriterControl {
    Ping(Bytes),
    Pong,
    PeerClose(Bytes, DisconnectInfo),
    Close(u16, DisconnectInfo),
    TransportLost(DisconnectInfo),
}

pub struct WebSocketService {
    worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
    connections: DashMap<String, Arc<ConnectionEntry>>,
    singleton_connections: DashMap<SingletonKey, Arc<SingletonConnectSlot>>,
    project_counts: DashMap<String, Arc<AtomicUsize>>,
    project_generations: DashMap<String, Arc<std::sync::atomic::AtomicU64>>,
    worker_count: Arc<AtomicUsize>,
    draining: AtomicBool,
    directory: Arc<dyn ConnectionDirectory>,
    identity: WorkerIdentity,
    quic: OnceLock<Arc<QuicTransport>>,
    self_reference: OnceLock<Weak<WebSocketService>>,
}

impl WebSocketService {
    pub async fn new(
        worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
    ) -> anyhow::Result<Arc<Self>> {
        let identity = worker_identity_from_env();
        let directory = directory_from_env(&identity)?;
        let service = Arc::new(Self {
            worker_senders,
            connections: DashMap::new(),
            singleton_connections: DashMap::new(),
            project_counts: DashMap::new(),
            project_generations: DashMap::new(),
            worker_count: Arc::new(AtomicUsize::new(0)),
            draining: AtomicBool::new(false),
            directory,
            identity,
            quic: OnceLock::new(),
            self_reference: OnceLock::new(),
        });
        service
            .self_reference
            .set(Arc::downgrade(&service))
            .map_err(|_| anyhow::anyhow!("websocket service self reference already initialized"))?;
        if let Some(quic) = QuicTransport::from_env(Arc::downgrade(&service))? {
            service
                .quic
                .set(quic.clone())
                .map_err(|_| anyhow::anyhow!("QUIC transport already initialized"))?;
            quic.spawn_server();
        }
        Ok(service)
    }

    pub fn connection_id() -> String {
        let mut random_bytes = [0_u8; 32];
        rand::thread_rng().fill_bytes(&mut random_bytes);
        format!(
            "v1.{}",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes)
        )
    }

    pub fn reserve_capacity(&self, project_id: &str) -> Result<CapacityGuard, CapacityError> {
        if self.draining.load(Ordering::Acquire) {
            return Err(CapacityError::Worker);
        }
        reserve_counter(&self.worker_count, WORKER_CONNECTION_LIMIT)
            .map_err(|_| CapacityError::Worker)?;
        let project_count = self
            .project_counts
            .entry(project_id.to_string())
            .or_insert_with(|| Arc::new(AtomicUsize::new(0)))
            .clone();
        if reserve_counter(&project_count, PROJECT_CONNECTION_LIMIT).is_err() {
            self.worker_count.fetch_sub(1, Ordering::AcqRel);
            return Err(CapacityError::Project);
        }
        let project_generation = self
            .project_generations
            .entry(project_id.to_string())
            .or_insert_with(|| Arc::new(std::sync::atomic::AtomicU64::new(0)))
            .clone();
        let reserved_generation = project_generation.load(Ordering::Acquire);
        Ok(CapacityGuard {
            project_count,
            worker_count: self.worker_count.clone(),
            project_generation,
            reserved_generation,
        })
    }

    pub async fn invoke_connect(
        &self,
        project_id: &str,
        connection_id: &str,
        uri: &hyper::Uri,
        request_headers: &hyper::HeaderMap,
        client_address: Option<std::net::SocketAddr>,
    ) -> anyhow::Result<fn0::Response> {
        let body = Empty::<Bytes>::new()
            .map_err(|never: std::convert::Infallible| match never {})
            .boxed_unsync();
        let mut request = synthetic_request(uri, request_headers, body)?;
        request
            .headers_mut()
            .insert("x-fn0-internal-websocket-event", "connect".parse()?);
        request.headers_mut().insert(
            "x-fn0-internal-websocket-connection-id",
            connection_id.parse()?,
        );
        if let Some(client_address) = client_address {
            request.headers_mut().insert(
                "x-fn0-internal-websocket-client-address",
                client_address.to_string().parse()?,
            );
        }
        self.invoke(project_id, request).await
    }

    pub async fn publish_connection(
        &self,
        project_id: &str,
        connection_id: &str,
    ) -> anyhow::Result<()> {
        self.directory
            .put_connection(
                connection_id,
                &ConnectionOwner {
                    project_id: project_id.to_string(),
                    worker_id: self.identity.worker_id.clone(),
                    endpoint: self.identity.endpoint.clone(),
                },
            )
            .await
    }

    pub async fn unpublish_connection(&self, connection_id: &str) {
        if let Err(error) = self
            .directory
            .delete_connection(connection_id, &self.identity.worker_id)
            .await
        {
            tracing::warn!(%connection_id, %error, "websocket directory delete failed");
        }
    }

    pub(crate) fn worker_id(&self) -> &str {
        &self.identity.worker_id
    }

    pub(crate) fn has_connection(&self, connection_id: &str) -> bool {
        self.connections.contains_key(connection_id)
    }

    pub(crate) fn admit_local_send(
        &self,
        caller_project_id: &str,
        connection_id: &str,
        message_kind: WebSocketMessageKind,
        body: Body,
        deadline: tokio::time::Instant,
    ) -> Result<AdmittedSend, WebSocketCommandError> {
        let Some(entry) = self
            .connections
            .get(connection_id)
            .map(|entry| entry.clone())
        else {
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::ConnectionNotFound,
            ));
        };
        if entry.project_id != caller_project_id || entry.closing.load(Ordering::Acquire) {
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::ConnectionNotFound,
            ));
        }
        let (ready_sender, ready_receiver) = oneshot::channel();
        let (response_sender, response_receiver) = oneshot::channel();
        let send_result = entry.command_sender.try_send(SocketCommand::Send {
            message_kind,
            body,
            ready_sender,
            response_sender,
            deadline,
        });
        if let Err(send_error) = send_result {
            return match send_error {
                mpsc::error::TrySendError::Full(_) => {
                    entry.closing.store(true, Ordering::Release);
                    let info = DisconnectInfo::protocol_error(1013);
                    let _ = entry.control_sender.send(WriterControl::Close(1013, info));
                    Err(WebSocketCommandError::not_sent(
                        WebSocketCommandErrorKind::Backpressure,
                    ))
                }
                mpsc::error::TrySendError::Closed(_) => Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::ConnectionNotFound,
                )),
            };
        }
        Ok(AdmittedSend {
            ready_receiver,
            response_receiver,
        })
    }

    pub(crate) fn disconnect_local(
        &self,
        caller_project_id: &str,
        connection_id: &str,
    ) -> WebSocketCommandFuture {
        let entry = self
            .connections
            .get(connection_id)
            .map(|entry| entry.clone());
        disconnect_entry(entry, caller_project_id)
    }

    pub fn spawn_connection(
        self: &Arc<Self>,
        project_id: String,
        connection_id: String,
        route_uri: hyper::Uri,
        upgrade: UpgradeFut,
        capacity_guard: CapacityGuard,
    ) {
        let service = self.clone();
        tokio::spawn(async move {
            let websocket = match upgrade.await {
                Ok(websocket) => websocket,
                Err(error) => {
                    tracing::warn!(%project_id, %connection_id, %error, "websocket upgrade failed");
                    service.unpublish_connection(&connection_id).await;
                    drop(capacity_guard);
                    return;
                }
            };
            let (mut reader, writer) = websocket.split(tokio::io::split);
            reader.set_auto_close(false);
            reader.set_auto_pong(false);
            reader.set_max_message_size(usize::MAX);
            let registered =
                service.register_connection(&project_id, &connection_id, &capacity_guard);
            service
                .run_connection(
                    project_id,
                    connection_id,
                    route_uri,
                    reader,
                    writer,
                    registered.command_receiver,
                    registered.control_sender,
                    registered.control_receiver,
                    registered.closed_sender,
                    capacity_guard,
                    None,
                    None,
                    None,
                )
                .await;
        });
    }

    #[allow(clippy::too_many_arguments)]
    async fn connect_outbound(
        self: &Arc<Self>,
        project_id: String,
        url: String,
        receive_path: String,
        remaining: Duration,
        singleton_binding: Option<SingletonBinding>,
        handshake_headers: Vec<(String, String)>,
        protocols: Vec<String>,
    ) -> Result<OutboundConnectResult, WebSocketCommandError> {
        let deadline = tokio::time::Instant::now() + remaining;
        let capacity_guard = self.reserve_capacity(&project_id).map_err(|_| {
            WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Backpressure)
        })?;
        let connection_id = Self::connection_id();
        let route_uri = format!("https://fn0-websocket.internal{receive_path}")
            .parse::<hyper::Uri>()
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal))?;
        let (scheme, host, port, request, expected_accept) =
            build_outbound_handshake_request(&url, handshake_headers, &protocols)?;
        let stream = tokio::time::timeout_at(
            deadline,
            tokio::time::timeout(
                OUTBOUND_DIAL_TIMEOUT,
                TcpStream::connect((host.as_str(), port)),
            ),
        )
        .await
        .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::DeadlineExceeded))?
        .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?
        .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?;
        let result = if scheme == "ws" {
            tokio::time::timeout_at(
                deadline,
                self.finish_outbound_handshake(
                    project_id,
                    connection_id.clone(),
                    route_uri,
                    request,
                    stream,
                    capacity_guard,
                    singleton_binding,
                    expected_accept,
                    protocols,
                ),
            )
            .await
            .map_err(|_| {
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::DeadlineExceeded)
            })??
        } else {
            let tls_connector = outbound_tls_connector().map_err(|_| {
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal)
            })?;
            let server_name = ServerName::try_from(host).map_err(|_| {
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal)
            })?;
            let tls_stream = tokio::time::timeout_at(
                deadline,
                tokio::time::timeout(
                    OUTBOUND_DIAL_TIMEOUT,
                    tls_connector.connect(server_name, stream),
                ),
            )
            .await
            .map_err(|_| {
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::DeadlineExceeded)
            })?
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?;
            tokio::time::timeout_at(
                deadline,
                self.finish_outbound_handshake(
                    project_id,
                    connection_id.clone(),
                    route_uri,
                    request,
                    tls_stream,
                    capacity_guard,
                    singleton_binding,
                    expected_accept,
                    protocols,
                ),
            )
            .await
            .map_err(|_| {
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::DeadlineExceeded)
            })??
        };
        Ok(result)
    }

    #[allow(clippy::too_many_arguments)]
    async fn connect_singleton_outbound(
        self: &Arc<Self>,
        project_id: String,
        singleton_id: String,
        url: String,
        route_path: String,
        headers: Vec<(String, String)>,
        protocols: Vec<String>,
        claim_token: String,
        initial_lease_deadline: i64,
        remaining: Duration,
    ) -> Result<String, WebSocketCommandError> {
        let singleton_key = (
            project_id.clone(),
            singleton_id.clone(),
            claim_token.clone(),
        );
        let slot = self
            .singleton_connections
            .entry(singleton_key.clone())
            .or_insert_with(|| Arc::new(SingletonConnectSlot::new()))
            .clone();
        let service = self.clone();
        let key_for_connect = singleton_key.clone();
        let slot_for_connect = slot.clone();
        let result = slot
            .get_or_init(move || async move {
                let activated = Arc::new(AtomicBool::new(false));
                let result = service
                    .connect_outbound(
                        project_id,
                        url,
                        route_path,
                        remaining,
                        Some(SingletonBinding {
                            key: key_for_connect,
                            slot: slot_for_connect,
                            singleton_id,
                            claim_token,
                            initial_lease_deadline,
                            activated,
                        }),
                        headers,
                        protocols,
                    )
                    .await?;
                match result {
                    OutboundConnectResult::Prepared(prepared) => Ok(prepared),
                    OutboundConnectResult::Active(_) => Err(WebSocketCommandError::not_sent(
                        WebSocketCommandErrorKind::Internal,
                    )),
                }
            })
            .await
            .clone();
        match result {
            Ok(prepared)
                if self
                    .connections
                    .get(&prepared.connection_id)
                    .is_some_and(|entry| !entry.closing.load(Ordering::Acquire)) =>
            {
                Ok(prepared.connection_id.clone())
            }
            Ok(_) => {
                self.singleton_connections
                    .remove_if(&singleton_key, |_, current| Arc::ptr_eq(current, &slot));
                Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::ConnectionNotFound,
                ))
            }
            Err(error) => {
                self.singleton_connections
                    .remove_if(&singleton_key, |_, current| Arc::ptr_eq(current, &slot));
                Err(error)
            }
        }
    }

    async fn activate_singleton_outbound(
        self: &Arc<Self>,
        singleton_key: SingletonKey,
        connection_id: &str,
    ) -> Result<(), WebSocketCommandError> {
        let slot = self
            .singleton_connections
            .get(&singleton_key)
            .map(|entry| entry.clone())
            .ok_or_else(|| {
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::ConnectionNotFound)
            })?;
        let prepared = match slot.get() {
            Some(Ok(prepared)) if prepared.connection_id == connection_id => prepared.clone(),
            _ => {
                return Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::ConnectionNotFound,
                ));
            }
        };
        let service = self.clone();
        let prepared_for_activation = prepared.clone();
        let mut activation_receiver = prepared
            .activation
            .get_or_init(|| async move {
                let (activation_sender, activation_receiver) = watch::channel(None);
                tokio::spawn(async move {
                    let result = service
                        .activate_prepared_singleton(prepared_for_activation)
                        .await;
                    let _ = activation_sender.send(Some(result));
                });
                activation_receiver
            })
            .await
            .clone();
        loop {
            if let Some(result) = *activation_receiver.borrow() {
                return result;
            }
            activation_receiver.changed().await.map_err(|_| {
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal)
            })?;
        }
    }

    async fn activate_prepared_singleton(
        self: &Arc<Self>,
        prepared: Arc<PreparedSingleton>,
    ) -> Result<(), WebSocketCommandError> {
        if self
            .connections
            .get(&prepared.connection_id)
            .is_none_or(|entry| entry.closing.load(Ordering::Acquire))
        {
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::ConnectionNotFound,
            ));
        }
        prepared.activated.store(true, Ordering::Release);
        let lease_activation_sender = prepared
            .lease_activation_sender
            .lock()
            .expect("singleton lease activation lock")
            .take();
        let Some(lease_activation_sender) = lease_activation_sender else {
            close_connection(
                self,
                &prepared.project_id,
                &prepared.connection_id,
                1011,
                DisconnectInfo::internal_error(),
            );
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Internal,
            ));
        };
        if lease_activation_sender.send(()).is_err() {
            close_connection(
                self,
                &prepared.project_id,
                &prepared.connection_id,
                1011,
                DisconnectInfo::internal_error(),
            );
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::ConnectionNotFound,
            ));
        }
        let callback_response = self
            .invoke_connect(
                &prepared.project_id,
                &prepared.connection_id,
                &prepared.route_uri,
                &prepared.response_headers,
                None,
            )
            .await;
        if !matches!(callback_response, Ok(response) if response.status().is_success()) {
            close_connection(
                self,
                &prepared.project_id,
                &prepared.connection_id,
                1011,
                DisconnectInfo::internal_error(),
            );
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Internal,
            ));
        }
        if self
            .connections
            .get(&prepared.connection_id)
            .is_none_or(|entry| entry.closing.load(Ordering::Acquire))
        {
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Transport,
            ));
        }
        if self
            .publish_connection(&prepared.project_id, &prepared.connection_id)
            .await
            .is_err()
        {
            close_connection(
                self,
                &prepared.project_id,
                &prepared.connection_id,
                1011,
                DisconnectInfo::transport_error(),
            );
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Transport,
            ));
        }
        let message_ready_sender = prepared
            .message_ready_sender
            .lock()
            .expect("singleton message activation lock")
            .take();
        let Some(message_ready_sender) = message_ready_sender else {
            close_connection(
                self,
                &prepared.project_id,
                &prepared.connection_id,
                1011,
                DisconnectInfo::internal_error(),
            );
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Internal,
            ));
        };
        if message_ready_sender.send(()).is_err() {
            close_connection(
                self,
                &prepared.project_id,
                &prepared.connection_id,
                1011,
                DisconnectInfo::transport_error(),
            );
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::ConnectionNotFound,
            ));
        }
        Ok(())
    }

    fn abort_singleton_outbound(
        &self,
        singleton_key: &SingletonKey,
        connection_id: &str,
    ) -> Result<(), WebSocketCommandError> {
        let Some(slot) = self
            .singleton_connections
            .get(singleton_key)
            .map(|entry| entry.clone())
        else {
            return Ok(());
        };
        let Some(Ok(prepared)) = slot.get() else {
            return Ok(());
        };
        if prepared.connection_id != connection_id {
            return Ok(());
        }
        close_connection(
            self,
            &prepared.project_id,
            &prepared.connection_id,
            1011,
            DisconnectInfo::internal_error(),
        );
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn finish_outbound_handshake<Stream>(
        self: &Arc<Self>,
        project_id: String,
        connection_id: String,
        route_uri: hyper::Uri,
        request: hyper::Request<Empty<Bytes>>,
        stream: Stream,
        capacity_guard: CapacityGuard,
        singleton_binding: Option<SingletonBinding>,
        expected_accept: String,
        requested_protocols: Vec<String>,
    ) -> Result<OutboundConnectResult, WebSocketCommandError>
    where
        Stream: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let (websocket, response) = handshake::client(&OutboundExecutor, request, stream)
            .await
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?;
        validate_outbound_handshake(response.headers(), &expected_accept, &requested_protocols)?;
        let (mut reader, writer) = websocket.split(tokio::io::split);
        reader.set_auto_close(false);
        reader.set_auto_pong(false);
        reader.set_max_message_size(usize::MAX);
        if self.draining.load(Ordering::Acquire)
            || self
                .project_generations
                .get(&project_id)
                .is_some_and(|generation| {
                    generation.load(Ordering::Acquire) != capacity_guard.reserved_generation
                })
        {
            drop(reader);
            drop(writer);
            drop(capacity_guard);
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Transport,
            ));
        }
        let registered = self.register_connection(&project_id, &connection_id, &capacity_guard);
        let (message_ready_sender, message_ready_receiver) = oneshot::channel();
        let is_singleton = singleton_binding.is_some();
        let singleton_activated = singleton_binding
            .as_ref()
            .map(|binding| binding.activated.clone());
        let (lease_activation_sender, lease_activation_receiver) = if is_singleton {
            let (sender, receiver) = oneshot::channel();
            (Some(sender), Some(receiver))
        } else {
            (None, None)
        };
        let service = self.clone();
        let spawned_project_id = project_id.clone();
        let spawned_connection_id = connection_id.clone();
        let spawned_route_uri = route_uri.clone();
        tokio::spawn(async move {
            service
                .run_connection(
                    spawned_project_id,
                    spawned_connection_id,
                    spawned_route_uri,
                    reader,
                    writer,
                    registered.command_receiver,
                    registered.control_sender,
                    registered.control_receiver,
                    registered.closed_sender,
                    capacity_guard,
                    singleton_binding,
                    Some(message_ready_receiver),
                    lease_activation_receiver,
                )
                .await;
        });
        if is_singleton {
            return Ok(OutboundConnectResult::Prepared(Arc::new(
                PreparedSingleton {
                    connection_id,
                    project_id,
                    route_uri,
                    response_headers: response.headers().clone(),
                    lease_activation_sender: Mutex::new(lease_activation_sender),
                    message_ready_sender: Mutex::new(Some(message_ready_sender)),
                    activated: singleton_activated.expect("singleton activation state"),
                    activation: tokio::sync::OnceCell::new(),
                },
            )));
        }
        if self
            .connections
            .get(&connection_id)
            .is_none_or(|entry| entry.closing.load(Ordering::Acquire))
        {
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Transport,
            ));
        }
        if self
            .publish_connection(&project_id, &connection_id)
            .await
            .is_err()
        {
            close_connection(
                self,
                &project_id,
                &connection_id,
                1011,
                DisconnectInfo::transport_error(),
            );
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Transport,
            ));
        }
        let _ = message_ready_sender.send(());
        Ok(OutboundConnectResult::Active(connection_id))
    }

    fn register_connection(
        &self,
        project_id: &str,
        connection_id: &str,
        capacity_guard: &CapacityGuard,
    ) -> RegisteredConnection {
        let (command_sender, command_receiver) = mpsc::channel(OUTBOUND_COMMAND_CAPACITY);
        let (control_sender, control_receiver) = mpsc::unbounded_channel();
        let (closed_sender, closed_receiver) = watch::channel(false);
        let entry = Arc::new(ConnectionEntry {
            project_id: project_id.to_string(),
            command_sender,
            closing: AtomicBool::new(false),
            closed_receiver,
            control_sender: control_sender.clone(),
        });
        self.connections
            .insert(connection_id.to_string(), entry.clone());
        if self.draining.load(Ordering::Acquire)
            || capacity_guard.project_generation.load(Ordering::Acquire)
                != capacity_guard.reserved_generation
        {
            entry.closing.store(true, Ordering::Release);
            let _ = entry.command_sender.try_send(SocketCommand::Close {
                code: 1012,
                info: DisconnectInfo::deployment(),
                response_sender: None,
            });
        }
        RegisteredConnection {
            command_receiver,
            control_sender,
            control_receiver,
            closed_sender,
        }
    }

    pub async fn close_project(&self, project_id: &str) {
        self.project_generations
            .entry(project_id.to_string())
            .or_insert_with(|| Arc::new(std::sync::atomic::AtomicU64::new(0)))
            .fetch_add(1, Ordering::AcqRel);
        let targets: Vec<Arc<ConnectionEntry>> = self
            .connections
            .iter()
            .filter(|entry| entry.value().project_id == project_id)
            .map(|entry| entry.value().clone())
            .collect();
        for entry in targets {
            if entry
                .closing
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                let command_sender = entry.command_sender.clone();
                tokio::spawn(async move {
                    let _ = command_sender
                        .send(SocketCommand::Close {
                            code: 1012,
                            info: DisconnectInfo::deployment(),
                            response_sender: None,
                        })
                        .await;
                });
            }
        }
    }

    pub async fn close_all(&self) {
        self.draining.store(true, Ordering::Release);
        let project_ids: std::collections::HashSet<String> = self
            .connections
            .iter()
            .map(|entry| entry.value().project_id.clone())
            .collect();
        for project_id in project_ids {
            self.close_project(&project_id).await;
        }
    }

    pub fn connection_count(&self) -> usize {
        self.worker_count.load(Ordering::Acquire)
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_connection(
        self: &Arc<Self>,
        project_id: String,
        connection_id: String,
        route_uri: hyper::Uri,
        reader: SocketReader,
        mut writer: SocketWriter,
        command_receiver: mpsc::Receiver<SocketCommand>,
        control_sender: mpsc::UnboundedSender<WriterControl>,
        control_receiver: mpsc::UnboundedReceiver<WriterControl>,
        closed_sender: watch::Sender<bool>,
        capacity_guard: CapacityGuard,
        singleton_binding: Option<SingletonBinding>,
        message_ready: Option<oneshot::Receiver<()>>,
        lease_activation: Option<oneshot::Receiver<()>>,
    ) {
        let disconnect_info = Arc::new(Mutex::new(None));
        let singleton_id = singleton_binding
            .as_ref()
            .map(|binding| binding.singleton_id.clone());
        let singleton_claim_token = singleton_binding
            .as_ref()
            .map(|binding| binding.claim_token.clone());
        let singleton_activated = singleton_binding
            .as_ref()
            .map(|binding| binding.activated.clone());
        let lease_handle = singleton_binding.as_ref().map(|binding| {
            tokio::spawn(singleton_lease_loop(
                self.clone(),
                project_id.clone(),
                connection_id.clone(),
                binding.clone(),
                lease_activation.expect("singleton lease activation receiver"),
            ))
        });
        let reader_handle = tokio::spawn(read_loop(
            self.clone(),
            project_id.clone(),
            connection_id.clone(),
            route_uri.clone(),
            reader,
            control_sender,
            disconnect_info.clone(),
            message_ready,
        ));
        writer_loop(
            &mut writer,
            command_receiver,
            control_receiver,
            disconnect_info.clone(),
        )
        .await;
        reader_handle.abort();
        if let Some(lease_handle) = lease_handle {
            lease_handle.abort();
        }
        self.connections.remove(&connection_id);
        if let Some(singleton_binding) = singleton_binding {
            self.singleton_connections
                .remove_if(&singleton_binding.key, |_, current| {
                    Arc::ptr_eq(current, &singleton_binding.slot)
                });
        }
        self.unpublish_connection(&connection_id).await;
        let _ = closed_sender.send(true);
        drop(capacity_guard);
        let final_info = disconnect_info
            .lock()
            .expect("disconnect info lock")
            .clone()
            .unwrap_or_else(DisconnectInfo::transport_error);
        let lifecycle_started = singleton_activated
            .as_ref()
            .is_none_or(|activated| activated.load(Ordering::Acquire));
        if lifecycle_started {
            self.invoke_disconnect(&project_id, &connection_id, &route_uri, final_info);
        }
        if lifecycle_started
            && let (Some(singleton_id), Some(claim_token)) = (singleton_id, singleton_claim_token)
        {
            let service = self.clone();
            tokio::spawn(async move {
                let _ = service
                    .notify_singleton_status(
                        &project_id,
                        &singleton_id,
                        &claim_token,
                        &connection_id,
                        "disconnected",
                    )
                    .await;
            });
        }
    }

    fn invoke_disconnect(
        self: &Arc<Self>,
        project_id: &str,
        connection_id: &str,
        route_uri: &hyper::Uri,
        info: DisconnectInfo,
    ) {
        let service = self.clone();
        let project_id = project_id.to_string();
        let connection_id = connection_id.to_string();
        let route_uri = route_uri.clone();
        tokio::spawn(async move {
            let body = Empty::<Bytes>::new()
                .map_err(|never: std::convert::Infallible| match never {})
                .boxed_unsync();
            let Ok(mut request) = synthetic_request(&route_uri, &hyper::HeaderMap::new(), body)
            else {
                return;
            };
            request.headers_mut().insert(
                "x-fn0-internal-websocket-event",
                "disconnect".parse().expect("static header"),
            );
            request.headers_mut().insert(
                "x-fn0-internal-websocket-connection-id",
                connection_id.parse().expect("connection id header"),
            );
            request.headers_mut().insert(
                "x-fn0-internal-websocket-disconnect-cause",
                info.cause.parse().expect("static cause header"),
            );
            if let Some(close_code) = info.close_code {
                request.headers_mut().insert(
                    "x-fn0-internal-websocket-close-code",
                    close_code.to_string().parse().expect("close code header"),
                );
            }
            if let Some(reason) = info.reason
                && let Ok(reason_header) = reason.parse()
            {
                request
                    .headers_mut()
                    .insert("x-fn0-internal-websocket-close-reason", reason_header);
            }
            if let Err(error) = service.invoke(&project_id, request).await {
                tracing::warn!(%project_id, %connection_id, %error, "websocket on_disconnect failed");
            }
        });
    }

    async fn invoke(
        &self,
        project_id: &str,
        request: fn0::Request,
    ) -> anyhow::Result<fn0::Response> {
        let (response_sender, response_receiver) = oneshot::channel();
        let (envelope, started_receiver) =
            RequestEnvelope::new(project_id.to_string(), request, response_sender)
                .with_start_signal();
        worker_pool::dispatch(&self.worker_senders, envelope).map_err(|error| match error {
            DispatchError::Full => anyhow::anyhow!("worker queue full"),
            DispatchError::Closed => anyhow::anyhow!("worker queue closed"),
        })?;
        tokio::time::timeout(CALLBACK_DEADLINE, started_receiver)
            .await
            .map_err(|_| anyhow::anyhow!("websocket callback admission deadline exceeded"))?
            .map_err(|_| anyhow::anyhow!("websocket callback admission failed"))?;
        tokio::time::timeout(CALLBACK_DEADLINE, response_receiver)
            .await
            .map_err(|_| anyhow::anyhow!("websocket callback deadline exceeded"))?
            .map_err(|_| anyhow::anyhow!("websocket callback response dropped"))?
    }

    async fn notify_singleton_status(
        &self,
        project_id: &str,
        singleton_id: &str,
        claim_token: &str,
        connection_id: &str,
        status: &str,
    ) -> anyhow::Result<bool> {
        let body = serde_json::to_vec(&serde_json::json!({
            "project_id": project_id,
            "singleton_id": singleton_id,
            "claim_token": claim_token,
            "connection_id": connection_id,
            "status": status,
        }))?;
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri("https://fn0-control.internal/__forte_action/websocket_singleton_status")
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .header("x-fn0-internal-websocket-status", "true")
            .body(
                Full::new(Bytes::from(body))
                    .map_err(|never: std::convert::Infallible| match never {})
                    .boxed_unsync(),
            )?;
        let response = self.invoke("fn0-control", request).await?;
        if !response.status().is_success() {
            return Ok(false);
        }
        let body = response.into_body().collect().await?.to_bytes();
        Ok(body.as_ref() == b"\"Ok\"")
    }
}

fn singleton_system_header(header_name: &str) -> bool {
    let header_name = header_name.to_ascii_lowercase();
    matches!(
        header_name.as_str(),
        "host"
            | "upgrade"
            | "connection"
            | "content-length"
            | "transfer-encoding"
            | "sec-websocket-key"
            | "sec-websocket-version"
            | "sec-websocket-extensions"
            | "sec-websocket-accept"
            | "sec-websocket-protocol"
    ) || header_name.starts_with("x-fn0-")
}

fn build_outbound_handshake_request(
    url: &str,
    handshake_headers: Vec<(String, String)>,
    protocols: &[String],
) -> Result<OutboundHandshakeRequest, WebSocketCommandError> {
    let parsed_url = Url::parse(url)
        .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal))?;
    let scheme = parsed_url.scheme().to_string();
    if scheme != "ws" && scheme != "wss" {
        return Err(WebSocketCommandError::not_sent(
            WebSocketCommandErrorKind::Internal,
        ));
    }
    let host = parsed_url
        .host_str()
        .ok_or_else(|| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal))?
        .to_string();
    let port = parsed_url
        .port_or_known_default()
        .ok_or_else(|| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal))?;
    let authority = match parsed_url.host() {
        Some(url::Host::Ipv6(address)) => format!("[{address}]:{port}"),
        Some(url::Host::Ipv4(address)) => format!("{address}:{port}"),
        Some(url::Host::Domain(domain)) => format!("{domain}:{port}"),
        None => {
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Internal,
            ));
        }
    };
    let request_path = if parsed_url.path().is_empty() {
        "/"
    } else {
        parsed_url.path()
    };
    let request_path = match parsed_url.query() {
        Some(query) => format!("{request_path}?{query}"),
        None => request_path.to_string(),
    };
    if protocols
        .iter()
        .any(|protocol| !valid_websocket_protocol(protocol))
    {
        return Err(WebSocketCommandError::not_sent(
            WebSocketCommandErrorKind::Internal,
        ));
    }
    let websocket_key = fastwebsockets::handshake::generate_key();
    let expected_accept = websocket_accept(&websocket_key);
    let mut request_builder = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(format!("http://{authority}{request_path}"))
        .header(hyper::header::HOST, &authority)
        .header(hyper::header::UPGRADE, "websocket")
        .header(hyper::header::CONNECTION, "Upgrade")
        .header("Sec-WebSocket-Key", websocket_key)
        .header("Sec-WebSocket-Version", "13");
    for (header_name, header_value) in handshake_headers {
        if singleton_system_header(&header_name) {
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::Internal,
            ));
        }
        let header_name = hyper::header::HeaderName::from_bytes(header_name.as_bytes())
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal))?;
        let header_value = hyper::header::HeaderValue::from_str(&header_value)
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal))?;
        request_builder = request_builder.header(header_name, header_value);
    }
    if !protocols.is_empty() {
        request_builder = request_builder.header("Sec-WebSocket-Protocol", protocols.join(", "));
    }
    let request = request_builder
        .body(Empty::<Bytes>::new())
        .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal))?;
    Ok((scheme, host, port, request, expected_accept))
}

fn websocket_accept(websocket_key: &str) -> String {
    let mut digest = Sha1::new();
    digest.update(websocket_key.as_bytes());
    digest.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    base64::engine::general_purpose::STANDARD.encode(digest.finalize())
}

fn validate_outbound_handshake(
    headers: &hyper::HeaderMap,
    expected_accept: &str,
    requested_protocols: &[String],
) -> Result<(), WebSocketCommandError> {
    let accept_values = headers
        .get_all("sec-websocket-accept")
        .iter()
        .map(|value| {
            value
                .to_str()
                .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let valid_accept = accept_values.as_slice() == [expected_accept];
    if !valid_accept || headers.contains_key("sec-websocket-extensions") {
        return Err(WebSocketCommandError::not_sent(
            WebSocketCommandErrorKind::Transport,
        ));
    }
    let selected_protocols = headers
        .get_all("sec-websocket-protocol")
        .iter()
        .map(|value| {
            value
                .to_str()
                .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if selected_protocols.len() > 1 {
        return Err(WebSocketCommandError::not_sent(
            WebSocketCommandErrorKind::Transport,
        ));
    }
    if let Some(selected_protocol) = selected_protocols.first()
        && (!valid_websocket_protocol(selected_protocol)
            || !requested_protocols
                .iter()
                .any(|requested_protocol| requested_protocol == selected_protocol))
    {
        return Err(WebSocketCommandError::not_sent(
            WebSocketCommandErrorKind::Transport,
        ));
    }
    Ok(())
}

fn valid_websocket_protocol(protocol: &str) -> bool {
    !protocol.is_empty()
        && protocol.bytes().all(|byte| {
            matches!(byte, b'!' | b'#'..=b'\'' | b'*' | b'+' | b'-' | b'.' | b'0'..=b'9' | b'A'..=b'Z' | b'^'..=b'z' | b'|' | b'~')
        })
}

async fn singleton_lease_loop(
    service: Arc<WebSocketService>,
    project_id: String,
    connection_id: String,
    binding: SingletonBinding,
    lease_activation: oneshot::Receiver<()>,
) {
    let current_epoch_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(i64::MAX);
    let initial_remaining_millis = binding
        .initial_lease_deadline
        .saturating_sub(current_epoch_millis)
        .max(0) as u64;
    let pending_deadline = tokio::time::Instant::now()
        + SINGLETON_SAFETY_DEADLINE.min(Duration::from_millis(initial_remaining_millis));
    tokio::select! {
        activation = lease_activation => {
            if activation.is_err() {
                fence_singleton(&service, &project_id, &connection_id);
                return;
            }
        }
        _ = tokio::time::sleep_until(pending_deadline) => {
            fence_singleton(&service, &project_id, &connection_id);
            return;
        }
    }
    let mut safety_deadline = tokio::time::Instant::now() + SINGLETON_SAFETY_DEADLINE;
    loop {
        tokio::select! {
            _ = tokio::time::sleep(SINGLETON_HEARTBEAT_INTERVAL) => {}
            _ = tokio::time::sleep_until(safety_deadline) => {
                fence_singleton(&service, &project_id, &connection_id);
                return;
            }
        }
        let heartbeat = tokio::time::timeout_at(
            safety_deadline,
            service.notify_singleton_status(
                &project_id,
                &binding.singleton_id,
                &binding.claim_token,
                &connection_id,
                "heartbeat",
            ),
        )
        .await;
        if !matches!(heartbeat, Ok(Ok(true))) {
            fence_singleton(&service, &project_id, &connection_id);
            return;
        }
        safety_deadline = tokio::time::Instant::now() + SINGLETON_SAFETY_DEADLINE;
    }
}

fn fence_singleton(service: &WebSocketService, project_id: &str, connection_id: &str) {
    close_connection(
        service,
        project_id,
        connection_id,
        1011,
        DisconnectInfo::heartbeat_timeout(),
    );
}

fn close_connection(
    service: &WebSocketService,
    project_id: &str,
    connection_id: &str,
    close_code: u16,
    disconnect_info: DisconnectInfo,
) {
    let entry = service
        .connections
        .get(connection_id)
        .map(|entry| entry.clone());
    let Some(entry) = entry else {
        return;
    };
    if entry.project_id != project_id
        || entry
            .closing
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
    {
        return;
    }
    let _ = entry
        .control_sender
        .send(WriterControl::Close(close_code, disconnect_info));
}

impl WebSocketCommandDispatcher for WebSocketService {
    fn connect(
        &self,
        caller_project_id: String,
        url: String,
        receive_path: String,
        remaining: Duration,
    ) -> WebSocketConnectFuture {
        let Some(service) = self.self_reference.get().and_then(Weak::upgrade) else {
            return Box::pin(async {
                Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::Internal,
                ))
            });
        };
        Box::pin(async move {
            match service
                .connect_outbound(
                    caller_project_id,
                    url,
                    receive_path,
                    remaining,
                    None,
                    Vec::new(),
                    Vec::new(),
                )
                .await?
            {
                OutboundConnectResult::Active(connection_id) => Ok(connection_id),
                OutboundConnectResult::Prepared(_) => Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::Internal,
                )),
            }
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn connect_singleton(
        &self,
        project_id: String,
        singleton_id: String,
        url: String,
        route_path: String,
        headers: Vec<(String, String)>,
        protocols: Vec<String>,
        claim_token: String,
        initial_lease_deadline: i64,
        remaining: Duration,
    ) -> WebSocketConnectFuture {
        let Some(service) = self.self_reference.get().and_then(Weak::upgrade) else {
            return Box::pin(async {
                Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::Internal,
                ))
            });
        };
        Box::pin(async move {
            service
                .connect_singleton_outbound(
                    project_id,
                    singleton_id,
                    url,
                    route_path,
                    headers,
                    protocols,
                    claim_token,
                    initial_lease_deadline,
                    remaining,
                )
                .await
        })
    }

    fn activate_singleton(
        &self,
        project_id: String,
        singleton_id: String,
        claim_token: String,
        connection_id: String,
        remaining: Duration,
    ) -> WebSocketCommandFuture {
        let Some(service) = self.self_reference.get().and_then(Weak::upgrade) else {
            return Box::pin(async {
                Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::Internal,
                ))
            });
        };
        Box::pin(async move {
            let deadline = tokio::time::Instant::now() + remaining;
            tokio::time::timeout_at(
                deadline,
                service.activate_singleton_outbound(
                    (project_id, singleton_id, claim_token),
                    &connection_id,
                ),
            )
            .await
            .unwrap_or_else(|_| {
                Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::DeadlineExceeded,
                ))
            })
        })
    }

    fn abort_singleton(
        &self,
        project_id: String,
        singleton_id: String,
        claim_token: String,
        connection_id: String,
        _remaining: Duration,
    ) -> WebSocketCommandFuture {
        let Some(service) = self.self_reference.get().and_then(Weak::upgrade) else {
            return Box::pin(async {
                Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::Internal,
                ))
            });
        };
        Box::pin(async move {
            service
                .abort_singleton_outbound(&(project_id, singleton_id, claim_token), &connection_id)
        })
    }

    fn send(
        &self,
        caller_project_id: String,
        connection_id: String,
        message_kind: WebSocketMessageKind,
        body: Body,
        remaining: Duration,
    ) -> WebSocketCommandFuture {
        let deadline = tokio::time::Instant::now() + remaining;
        if self.connections.contains_key(&connection_id) {
            let admitted = self.admit_local_send(
                &caller_project_id,
                &connection_id,
                message_kind,
                body,
                deadline,
            );
            return Box::pin(async move {
                let admitted = admitted?;
                await_send_response(admitted.response_receiver, deadline).await
            });
        }
        let directory = self.directory.clone();
        let identity = self.identity.clone();
        let quic = self.quic.get().cloned();
        Box::pin(async move {
            let owner = directory
                .lookup_connection(&connection_id)
                .await
                .map_err(|_| {
                    WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport)
                })?;
            let Some(owner) = owner else {
                return Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::ConnectionNotFound,
                ));
            };
            if owner.project_id != caller_project_id {
                return Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::ConnectionNotFound,
                ));
            }
            if owner.worker_id == identity.worker_id {
                let _ = directory
                    .delete_connection(&connection_id, &owner.worker_id)
                    .await;
                return Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::ConnectionNotFound,
                ));
            }
            if owner.endpoint.is_empty() {
                return Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::ConnectionNotFound,
                ));
            }
            let Some(quic) = quic else {
                return Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::Transport,
                ));
            };
            let transport_remaining =
                deadline.saturating_duration_since(tokio::time::Instant::now());
            let result = tokio::time::timeout_at(
                deadline,
                quic.send(
                    &owner.endpoint,
                    caller_project_id,
                    connection_id.clone(),
                    owner.worker_id.clone(),
                    message_kind,
                    body,
                    transport_remaining,
                ),
            )
            .await
            .unwrap_or_else(|_| {
                Err(WebSocketCommandError::unknown(
                    WebSocketCommandErrorKind::DeadlineExceeded,
                ))
            });
            if result
                .as_ref()
                .is_err_and(|error| error.kind == WebSocketCommandErrorKind::ConnectionNotFound)
            {
                let _ = directory
                    .delete_connection(&connection_id, &owner.worker_id)
                    .await;
            }
            result
        })
    }

    fn disconnect(
        &self,
        caller_project_id: String,
        connection_id: String,
        remaining: Duration,
    ) -> WebSocketCommandFuture {
        let deadline = tokio::time::Instant::now() + remaining;
        let entry = self
            .connections
            .get(&connection_id)
            .map(|entry| entry.clone());
        if entry.is_some() {
            let disconnect_future = disconnect_entry(entry, &caller_project_id);
            return Box::pin(async move {
                tokio::time::timeout_at(deadline, disconnect_future)
                    .await
                    .unwrap_or_else(|_| {
                        Err(WebSocketCommandError::unknown(
                            WebSocketCommandErrorKind::DeadlineExceeded,
                        ))
                    })
            });
        }
        let directory = self.directory.clone();
        let identity = self.identity.clone();
        let quic = self.quic.get().cloned();
        Box::pin(async move {
            let owner = directory
                .lookup_connection(&connection_id)
                .await
                .map_err(|_| {
                    WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport)
                })?;
            let Some(owner) = owner else {
                return Ok(());
            };
            if owner.project_id != caller_project_id {
                return Ok(());
            }
            if owner.worker_id == identity.worker_id {
                let _ = directory
                    .delete_connection(&connection_id, &owner.worker_id)
                    .await;
                return Ok(());
            }
            if owner.endpoint.is_empty() {
                return Ok(());
            }
            let Some(quic) = quic else {
                return Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::Transport,
                ));
            };
            let transport_remaining =
                deadline.saturating_duration_since(tokio::time::Instant::now());
            tokio::time::timeout_at(
                deadline,
                quic.disconnect(
                    &owner.endpoint,
                    caller_project_id,
                    connection_id,
                    owner.worker_id,
                    transport_remaining,
                ),
            )
            .await
            .unwrap_or_else(|_| {
                Err(WebSocketCommandError::unknown(
                    WebSocketCommandErrorKind::DeadlineExceeded,
                ))
            })
        })
    }
}

async fn await_send_response(
    response_receiver: oneshot::Receiver<Result<(), WebSocketCommandError>>,
    deadline: tokio::time::Instant,
) -> Result<(), WebSocketCommandError> {
    match tokio::time::timeout_at(deadline, response_receiver).await {
        Ok(Ok(result)) => result,
        Ok(Err(_)) => Err(WebSocketCommandError::unknown(
            WebSocketCommandErrorKind::Transport,
        )),
        Err(_) => Err(WebSocketCommandError::unknown(
            WebSocketCommandErrorKind::DeadlineExceeded,
        )),
    }
}

fn disconnect_entry(
    entry: Option<Arc<ConnectionEntry>>,
    caller_project_id: &str,
) -> WebSocketCommandFuture {
    let Some(entry) = entry else {
        return Box::pin(async { Ok(()) });
    };
    if entry.project_id != caller_project_id {
        return Box::pin(async { Ok(()) });
    }
    let first_close = entry
        .closing
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok();
    if first_close {
        let (response_sender, response_receiver) = oneshot::channel();
        let command_sender = entry.command_sender.clone();
        tokio::spawn(async move {
            let _ = command_sender
                .send(SocketCommand::Close {
                    code: 1000,
                    info: DisconnectInfo::application(),
                    response_sender: Some(response_sender),
                })
                .await;
        });
        return Box::pin(async move { response_receiver.await.unwrap_or(Ok(())) });
    }
    Box::pin(async move {
        let mut closed_receiver = entry.closed_receiver.clone();
        if *closed_receiver.borrow() {
            return Ok(());
        }
        let _ = closed_receiver.changed().await;
        Ok(())
    })
}

fn reserve_counter(counter: &AtomicUsize, limit: usize) -> Result<(), ()> {
    counter
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
            (current < limit).then_some(current + 1)
        })
        .map(|_| ())
        .map_err(|_| ())
}

fn synthetic_request(
    uri: &hyper::Uri,
    request_headers: &hyper::HeaderMap,
    body: Body,
) -> anyhow::Result<fn0::Request> {
    let mut headers = request_headers.clone();
    let internal_headers: Vec<hyper::header::HeaderName> = headers
        .keys()
        .filter(|header_name| header_name.as_str().starts_with("x-fn0-internal-"))
        .cloned()
        .collect();
    for header_name in internal_headers {
        headers.remove(header_name);
    }
    let absolute_uri = if uri.authority().is_some() {
        uri.clone()
    } else {
        let host = headers
            .get(hyper::header::HOST)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| anyhow::anyhow!("websocket request missing host"))?;
        format!("https://{host}{uri}").parse()?
    };
    let mut request = hyper::Request::builder()
        .method(hyper::Method::POST)
        .uri(absolute_uri)
        .body(body)?;
    *request.headers_mut() = headers;
    Ok(request)
}

#[allow(clippy::too_many_arguments)]
async fn read_loop(
    service: Arc<WebSocketService>,
    project_id: String,
    connection_id: String,
    route_uri: hyper::Uri,
    mut reader: SocketReader,
    control_sender: mpsc::UnboundedSender<WriterControl>,
    disconnect_info: Arc<Mutex<Option<DisconnectInfo>>>,
    message_ready: Option<oneshot::Receiver<()>>,
) {
    if let Some(message_ready) = message_ready
        && message_ready.await.is_err()
    {
        return;
    }
    let pending_messages = Arc::new(AtomicUsize::new(0));
    let mut assembly: Option<(WebSocketMessageKind, Vec<u8>)> = None;
    loop {
        let mut obligated_sender = |frame: Frame<'static>| {
            let control_sender = control_sender.clone();
            async move {
                match frame.opcode {
                    OpCode::Pong => control_sender
                        .send(WriterControl::Pong)
                        .map_err(|_| anyhow::anyhow!("writer closed")),
                    _ => Ok(()),
                }
            }
        };
        let frame = match reader.read_frame(&mut obligated_sender).await {
            Ok(frame) => frame,
            Err(error) => {
                let info = match error {
                    WebSocketError::FrameTooLarge => DisconnectInfo::protocol_error(1009),
                    _ => DisconnectInfo::transport_error(),
                };
                store_disconnect_info(&disconnect_info, info.clone());
                let _ = control_sender.send(WriterControl::TransportLost(info));
                return;
            }
        };
        if matches!(
            frame.opcode,
            OpCode::Text | OpCode::Binary | OpCode::Continuation
        ) && service
            .connections
            .get(&connection_id)
            .is_none_or(|entry| entry.closing.load(Ordering::Acquire))
        {
            return;
        }
        match frame.opcode {
            OpCode::Ping => {
                let _ = control_sender
                    .send(WriterControl::Ping(Bytes::copy_from_slice(&frame.payload)));
            }
            OpCode::Pong => {
                let _ = control_sender.send(WriterControl::Pong);
            }
            OpCode::Close => {
                let info = peer_close_info(&frame.payload);
                store_disconnect_info(&disconnect_info, info.clone());
                let _ = control_sender.send(WriterControl::PeerClose(
                    Bytes::copy_from_slice(&frame.payload),
                    info,
                ));
                return;
            }
            OpCode::Text | OpCode::Binary => {
                if assembly.is_some() {
                    close_reader(
                        &control_sender,
                        &disconnect_info,
                        1002,
                        DisconnectInfo::protocol_error(1002),
                    );
                    return;
                }
                let message_kind = if frame.opcode == OpCode::Text {
                    WebSocketMessageKind::Text
                } else {
                    WebSocketMessageKind::Binary
                };
                let mut message_bytes = frame.payload.to_vec();
                if frame.fin {
                    if let Err(close_code) = dispatch_inbound(
                        &service,
                        &project_id,
                        &connection_id,
                        &route_uri,
                        message_kind,
                        std::mem::take(&mut message_bytes),
                        &pending_messages,
                        &control_sender,
                    ) {
                        close_reader(
                            &control_sender,
                            &disconnect_info,
                            close_code,
                            DisconnectInfo::protocol_error(close_code),
                        );
                        return;
                    }
                } else {
                    assembly = Some((message_kind, message_bytes));
                }
            }
            OpCode::Continuation => {
                let Some((_, message_bytes)) = assembly.as_mut() else {
                    close_reader(
                        &control_sender,
                        &disconnect_info,
                        1002,
                        DisconnectInfo::protocol_error(1002),
                    );
                    return;
                };
                message_bytes.extend_from_slice(&frame.payload);
                if frame.fin {
                    let (message_kind, message_bytes) = assembly.take().expect("assembly exists");
                    if let Err(close_code) = dispatch_inbound(
                        &service,
                        &project_id,
                        &connection_id,
                        &route_uri,
                        message_kind,
                        message_bytes,
                        &pending_messages,
                        &control_sender,
                    ) {
                        close_reader(
                            &control_sender,
                            &disconnect_info,
                            close_code,
                            DisconnectInfo::protocol_error(close_code),
                        );
                        return;
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_inbound(
    service: &Arc<WebSocketService>,
    project_id: &str,
    connection_id: &str,
    route_uri: &hyper::Uri,
    message_kind: WebSocketMessageKind,
    message_bytes: Vec<u8>,
    pending_messages: &Arc<AtomicUsize>,
    control_sender: &mpsc::UnboundedSender<WriterControl>,
) -> Result<(), u16> {
    if message_kind == WebSocketMessageKind::Text && std::str::from_utf8(&message_bytes).is_err() {
        return Err(1007);
    }
    if reserve_counter(pending_messages, INBOUND_PENDING_LIMIT).is_err() {
        return Err(1013);
    }
    let body = Full::new(Bytes::from(message_bytes))
        .map_err(|never: std::convert::Infallible| match never {})
        .boxed_unsync();
    let Ok(mut request) = synthetic_request(route_uri, &hyper::HeaderMap::new(), body) else {
        pending_messages.fetch_sub(1, Ordering::AcqRel);
        return Err(1011);
    };
    request.headers_mut().insert(
        "x-fn0-internal-websocket-event",
        "message".parse().expect("static header"),
    );
    request.headers_mut().insert(
        "x-fn0-internal-websocket-connection-id",
        connection_id.parse().expect("connection id header"),
    );
    request.headers_mut().insert(
        "x-fn0-internal-websocket-message-kind",
        match message_kind {
            WebSocketMessageKind::Text => "text".parse().expect("static header"),
            WebSocketMessageKind::Binary => "binary".parse().expect("static header"),
        },
    );
    let (response_sender, response_receiver) = oneshot::channel();
    let (envelope, started_receiver) =
        RequestEnvelope::new(project_id.to_string(), request, response_sender).with_start_signal();
    if worker_pool::dispatch(&service.worker_senders, envelope).is_err() {
        pending_messages.fetch_sub(1, Ordering::AcqRel);
        return Err(1013);
    }
    let project_id = project_id.to_string();
    let pending_messages = pending_messages.clone();
    let control_sender = control_sender.clone();
    tokio::spawn(async move {
        let started = started_receiver.await.is_ok();
        pending_messages.fetch_sub(1, Ordering::AcqRel);
        let response = tokio::time::timeout(CALLBACK_DEADLINE, response_receiver).await;
        match response {
            Ok(Ok(Ok(_))) if started => {}
            Ok(Ok(Ok(_))) => {
                let info = DisconnectInfo::protocol_error(1013);
                let _ = control_sender.send(WriterControl::Close(1013, info));
            }
            Ok(Ok(Err(error))) => {
                tracing::warn!(%project_id, %error, "websocket on_message platform failed");
                let info = DisconnectInfo::protocol_error(1013);
                let _ = control_sender.send(WriterControl::Close(1013, info));
            }
            Ok(Err(_)) | Err(_) => {
                tracing::warn!(%project_id, "websocket on_message callback failed");
                let info = DisconnectInfo::protocol_error(1013);
                let _ = control_sender.send(WriterControl::Close(1013, info));
            }
        }
    });
    Ok(())
}

async fn writer_loop(
    writer: &mut SocketWriter,
    mut command_receiver: mpsc::Receiver<SocketCommand>,
    mut control_receiver: mpsc::UnboundedReceiver<WriterControl>,
    disconnect_info: Arc<Mutex<Option<DisconnectInfo>>>,
) {
    let mut ping_interval = tokio::time::interval(PING_INTERVAL);
    ping_interval.tick().await;
    let pong_timeout = tokio::time::sleep(Duration::from_secs(86_400));
    tokio::pin!(pong_timeout);
    let mut awaiting_pong = false;
    let mut close_sent = false;
    let mut close_response: Option<oneshot::Sender<Result<(), WebSocketCommandError>>> = None;
    let close_timeout = tokio::time::sleep(Duration::from_secs(86_400));
    tokio::pin!(close_timeout);

    loop {
        tokio::select! {
            biased;
            Some(control) = control_receiver.recv() => {
                match control {
                    WriterControl::Ping(payload) => {
                        if writer.write_frame(Frame::pong(Payload::Bytes(payload.into()))).await.is_err() {
                            store_disconnect_info(&disconnect_info, DisconnectInfo::transport_error());
                            finish_close_response(close_response.take());
                            return;
                        }
                    }
                    WriterControl::Pong => {
                        awaiting_pong = false;
                        pong_timeout.as_mut().reset(tokio::time::Instant::now() + Duration::from_secs(86_400));
                    }
                    WriterControl::PeerClose(payload, info) => {
                        store_disconnect_info(&disconnect_info, info);
                        if !close_sent {
                            let _ = writer.write_frame(Frame::close_raw(Payload::Bytes(payload.into()))).await;
                            let _ = writer.flush().await;
                        }
                        finish_close_response(close_response.take());
                        return;
                    }
                    WriterControl::Close(code, info) => {
                        store_disconnect_info(&disconnect_info, info);
                        if writer.write_frame(Frame::close(code, &[])).await.is_err() {
                            finish_close_response(close_response.take());
                            return;
                        }
                        let _ = writer.flush().await;
                        close_sent = true;
                        close_timeout.as_mut().reset(tokio::time::Instant::now() + CLOSE_HANDSHAKE_DEADLINE);
                    }
                    WriterControl::TransportLost(info) => {
                        store_disconnect_info(&disconnect_info, info);
                        finish_close_response(close_response.take());
                        return;
                    }
                }
            }
            Some(command) = command_receiver.recv(), if !close_sent => {
                match command {
                    SocketCommand::Send {
                        message_kind,
                        body,
                        ready_sender,
                        response_sender,
                        deadline,
                    } => {
                        if tokio::time::Instant::now() >= deadline {
                            let _ = response_sender.send(Err(WebSocketCommandError::not_sent(
                                WebSocketCommandErrorKind::DeadlineExceeded,
                            )));
                            store_disconnect_info(&disconnect_info, DisconnectInfo::protocol_error(1013));
                            let _ = writer.write_frame(Frame::close(1013, &[])).await;
                            let _ = writer.flush().await;
                            return;
                        }
                        let _ = ready_sender.send(());
                        let wrote_frame = Arc::new(AtomicBool::new(false));
                        let result = tokio::time::timeout_at(
                            deadline,
                            send_message(writer, message_kind, body, wrote_frame.clone()),
                        )
                        .await;
                        let result = match result {
                            Ok(result) => result,
                            Err(_) => {
                                let delivery = if wrote_frame.load(Ordering::Acquire) {
                                    WebSocketDeliveryState::Unknown
                                } else {
                                    WebSocketDeliveryState::NotSent
                                };
                                let _ = response_sender.send(Err(WebSocketCommandError {
                                    kind: WebSocketCommandErrorKind::DeadlineExceeded,
                                    delivery,
                                }));
                                store_disconnect_info(&disconnect_info, DisconnectInfo::transport_error());
                                return;
                            }
                        };
                        let must_close = result.as_ref().is_err_and(|error| {
                            error.kind != WebSocketCommandErrorKind::InvalidText
                                || error.delivery == WebSocketDeliveryState::Unknown
                        });
                        let close_code = if result.as_ref().is_err_and(|error| error.kind == WebSocketCommandErrorKind::InvalidText) {
                            1007
                        } else {
                            1011
                        };
                        let _ = response_sender.send(result);
                        if must_close {
                            store_disconnect_info(&disconnect_info, DisconnectInfo::protocol_error(close_code));
                            let _ = writer.write_frame(Frame::close(close_code, &[])).await;
                            let _ = writer.flush().await;
                            return;
                        }
                    }
                    SocketCommand::Close { code, info, response_sender } => {
                        store_disconnect_info(&disconnect_info, info);
                        close_response = response_sender;
                        if writer.write_frame(Frame::close(code, &[])).await.is_err() {
                            finish_close_response(close_response.take());
                            return;
                        }
                        let _ = writer.flush().await;
                        close_sent = true;
                        close_timeout.as_mut().reset(tokio::time::Instant::now() + CLOSE_HANDSHAKE_DEADLINE);
                    }
                }
            }
            _ = ping_interval.tick(), if !close_sent && !awaiting_pong => {
                let payload = Bytes::copy_from_slice(&unix_millis().to_be_bytes());
                if writer
                    .write_frame(Frame::new(true, OpCode::Ping, None, Payload::Bytes(payload.into())))
                    .await
                    .is_err()
                {
                    store_disconnect_info(&disconnect_info, DisconnectInfo::transport_error());
                    return;
                }
                let _ = writer.flush().await;
                awaiting_pong = true;
                pong_timeout.as_mut().reset(tokio::time::Instant::now() + PONG_DEADLINE);
            }
            _ = &mut pong_timeout, if awaiting_pong && !close_sent => {
                store_disconnect_info(&disconnect_info, DisconnectInfo::heartbeat_timeout());
                return;
            }
            _ = &mut close_timeout, if close_sent => {
                finish_close_response(close_response.take());
                return;
            }
            else => {
                finish_close_response(close_response.take());
                return;
            }
        }
    }
}

async fn send_message(
    writer: &mut SocketWriter,
    message_kind: WebSocketMessageKind,
    mut body: Body,
    wrote_frame: Arc<AtomicBool>,
) -> Result<(), WebSocketCommandError> {
    let mut validator = Utf8Validator::default();
    let mut wrote_any_frame = false;
    let mut first_frame = true;
    while let Some(frame_result) = body.frame().await {
        let frame = frame_result
            .map_err(|_| delivery_error(WebSocketCommandErrorKind::Internal, wrote_any_frame))?;
        let Ok(data) = frame.into_data() else {
            continue;
        };
        if message_kind == WebSocketMessageKind::Text && validator.push(&data).is_err() {
            return Err(delivery_error(
                WebSocketCommandErrorKind::InvalidText,
                wrote_any_frame,
            ));
        }
        let opcode = if first_frame {
            match message_kind {
                WebSocketMessageKind::Text => OpCode::Text,
                WebSocketMessageKind::Binary => OpCode::Binary,
            }
        } else {
            OpCode::Continuation
        };
        wrote_frame.store(true, Ordering::Release);
        writer
            .write_frame(Frame::new(false, opcode, None, Payload::Bytes(data.into())))
            .await
            .map_err(|_| WebSocketCommandError::unknown(WebSocketCommandErrorKind::Transport))?;
        wrote_any_frame = true;
        first_frame = false;
    }
    if message_kind == WebSocketMessageKind::Text && validator.finish().is_err() {
        return Err(delivery_error(
            WebSocketCommandErrorKind::InvalidText,
            wrote_any_frame,
        ));
    }
    let final_opcode = if first_frame {
        match message_kind {
            WebSocketMessageKind::Text => OpCode::Text,
            WebSocketMessageKind::Binary => OpCode::Binary,
        }
    } else {
        OpCode::Continuation
    };
    wrote_frame.store(true, Ordering::Release);
    writer
        .write_frame(Frame::new(
            true,
            final_opcode,
            None,
            Payload::Bytes(Bytes::new().into()),
        ))
        .await
        .map_err(|_| WebSocketCommandError::unknown(WebSocketCommandErrorKind::Transport))?;
    writer
        .flush()
        .await
        .map_err(|_| WebSocketCommandError::unknown(WebSocketCommandErrorKind::Transport))?;
    Ok(())
}

fn delivery_error(kind: WebSocketCommandErrorKind, wrote_frame: bool) -> WebSocketCommandError {
    if wrote_frame {
        WebSocketCommandError::unknown(kind)
    } else {
        WebSocketCommandError::not_sent(kind)
    }
}

#[derive(Default)]
struct Utf8Validator {
    pending: Vec<u8>,
}

impl Utf8Validator {
    fn push(&mut self, bytes: &[u8]) -> Result<(), ()> {
        if self.pending.is_empty() {
            return validate_utf8_part(bytes, &mut self.pending);
        }
        let mut combined = Vec::with_capacity(self.pending.len() + bytes.len());
        combined.extend_from_slice(&self.pending);
        combined.extend_from_slice(bytes);
        self.pending.clear();
        validate_utf8_part(&combined, &mut self.pending)
    }

    fn finish(self) -> Result<(), ()> {
        self.pending.is_empty().then_some(()).ok_or(())
    }
}

fn validate_utf8_part(bytes: &[u8], pending: &mut Vec<u8>) -> Result<(), ()> {
    match std::str::from_utf8(bytes) {
        Ok(_) => Ok(()),
        Err(error) if error.error_len().is_none() => {
            pending.extend_from_slice(&bytes[error.valid_up_to()..]);
            Ok(())
        }
        Err(_) => Err(()),
    }
}

fn close_reader(
    control_sender: &mpsc::UnboundedSender<WriterControl>,
    disconnect_info: &Arc<Mutex<Option<DisconnectInfo>>>,
    code: u16,
    info: DisconnectInfo,
) {
    store_disconnect_info(disconnect_info, info.clone());
    let _ = control_sender.send(WriterControl::Close(code, info));
}

fn peer_close_info(payload: &[u8]) -> DisconnectInfo {
    let close_code = (payload.len() >= 2).then(|| u16::from_be_bytes([payload[0], payload[1]]));
    let reason = (payload.len() > 2).then(|| String::from_utf8_lossy(&payload[2..]).to_string());
    DisconnectInfo {
        close_code,
        reason,
        cause: "peer",
    }
}

fn store_disconnect_info(destination: &Arc<Mutex<Option<DisconnectInfo>>>, info: DisconnectInfo) {
    let mut destination = destination.lock().expect("disconnect info lock");
    if destination.is_none() {
        *destination = Some(info);
    }
}

fn finish_close_response(
    response_sender: Option<oneshot::Sender<Result<(), WebSocketCommandError>>>,
) {
    if let Some(response_sender) = response_sender {
        let _ = response_sender.send(Ok(()));
    }
}

fn unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::websocket_directory::MemoryDirectory;
    use fn0::WebSocketCommandDispatcher;
    use std::convert::Infallible;
    use std::net::{SocketAddr, UdpSocket};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::time::timeout;

    #[test]
    fn utf8_validator_accepts_split_code_point() {
        let mut validator = Utf8Validator::default();
        validator.push(&[0xF0, 0x9F]).expect("valid prefix");
        validator.push(&[0x98, 0x80]).expect("valid suffix");
        validator.finish().expect("complete text");
    }

    #[test]
    fn utf8_validator_rejects_invalid_and_incomplete_text() {
        let mut invalid = Utf8Validator::default();
        assert!(invalid.push(&[0xFF]).is_err());
        let mut incomplete = Utf8Validator::default();
        incomplete.push(&[0xE2, 0x82]).expect("valid prefix");
        assert!(incomplete.finish().is_err());
    }

    #[test]
    fn connection_ids_are_opaque_and_unique() {
        let first = WebSocketService::connection_id();
        let second = WebSocketService::connection_id();
        assert!(first.starts_with("v1."));
        assert_ne!(first, second);
    }

    #[test]
    fn singleton_system_headers_cannot_be_overridden() {
        assert!(singleton_system_header("Host"));
        assert!(singleton_system_header("Sec-WebSocket-Key"));
        assert!(singleton_system_header("Sec-WebSocket-Protocol"));
        assert!(singleton_system_header("x-fn0-private"));
        assert!(!singleton_system_header("authorization"));
    }

    #[test]
    fn singleton_fencing_margin_covers_active_send_and_close_handshake() {
        let control_lease = Duration::from_secs(60);
        assert!(
            control_lease - SINGLETON_SAFETY_DEADLINE
                >= CALLBACK_DEADLINE + CLOSE_HANDSHAKE_DEADLINE
        );
    }

    #[test]
    fn singleton_handshake_preserves_query_headers_and_protocols() {
        let (scheme, host, port, request, expected_accept) = build_outbound_handshake_request(
            "wss://example.com/stream?token=secret&mode=full",
            vec![
                ("authorization".to_string(), "Bearer credential".to_string()),
                ("x-market".to_string(), "seoul".to_string()),
            ],
            &["market.v1".to_string(), "market.v2".to_string()],
        )
        .unwrap();
        assert_eq!(scheme, "wss");
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);
        assert_eq!(
            request.headers()["sec-websocket-key"]
                .to_str()
                .map(websocket_accept)
                .unwrap(),
            expected_accept
        );
        assert_eq!(
            request.uri().path_and_query().unwrap().as_str(),
            "/stream?token=secret&mode=full"
        );
        assert_eq!(request.headers()["authorization"], "Bearer credential");
        assert_eq!(request.headers()["x-market"], "seoul");
        assert_eq!(
            request.headers()["sec-websocket-protocol"],
            "market.v1, market.v2"
        );
    }

    #[test]
    fn outbound_handshake_requires_matching_accept() {
        assert_eq!(
            websocket_accept("dGhlIHNhbXBsZSBub25jZQ=="),
            "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        );
        let expected_accept = websocket_accept("test-key");
        let mut headers = hyper::HeaderMap::new();
        headers.insert(
            "sec-websocket-accept",
            expected_accept.parse().expect("accept header"),
        );
        assert!(validate_outbound_handshake(&headers, &expected_accept, &[]).is_ok());

        headers.insert(
            "sec-websocket-accept",
            "invalid".parse().expect("invalid accept header"),
        );
        assert!(validate_outbound_handshake(&headers, &expected_accept, &[]).is_err());

        headers.append(
            "sec-websocket-accept",
            expected_accept.parse().expect("duplicate accept header"),
        );
        assert!(validate_outbound_handshake(&headers, &expected_accept, &[]).is_err());

        headers.remove("sec-websocket-accept");
        assert!(validate_outbound_handshake(&headers, &expected_accept, &[]).is_err());
    }

    #[test]
    fn outbound_handshake_rejects_unrequested_protocol_and_extensions() {
        let expected_accept = websocket_accept("test-key");
        let mut headers = hyper::HeaderMap::new();
        headers.insert(
            "sec-websocket-accept",
            expected_accept.parse().expect("accept header"),
        );
        headers.insert(
            "sec-websocket-protocol",
            "market.v1".parse().expect("protocol header"),
        );
        assert!(validate_outbound_handshake(&headers, &expected_accept, &[]).is_err());
        assert!(
            validate_outbound_handshake(&headers, &expected_accept, &["market.v1".to_string()])
                .is_ok()
        );

        headers.append(
            "sec-websocket-protocol",
            "market.v1".parse().expect("duplicate protocol header"),
        );
        assert!(
            validate_outbound_handshake(&headers, &expected_accept, &["market.v1".to_string()])
                .is_err()
        );

        headers.remove("sec-websocket-protocol");
        headers.insert(
            "sec-websocket-extensions",
            "permessage-deflate".parse().expect("extension header"),
        );
        assert!(validate_outbound_handshake(&headers, &expected_accept, &[]).is_err());
    }

    #[test]
    fn outbound_protocol_names_follow_rfc_token_rules() {
        assert!(valid_websocket_protocol("graphql-transport-ws"));
        assert!(valid_websocket_protocol("market.v1+json"));
        assert!(!valid_websocket_protocol(""));
        assert!(!valid_websocket_protocol("market v1"));
        assert!(!valid_websocket_protocol("market,v1"));
        assert!(!valid_websocket_protocol("market/v1"));
    }

    #[tokio::test]
    async fn duplicate_singleton_initialization_runs_once() {
        let slot = Arc::new(tokio::sync::OnceCell::<Result<String, WebSocketCommandError>>::new());
        let initialization_count = Arc::new(AtomicUsize::new(0));
        let mut tasks = Vec::new();
        for task_number in 0..8 {
            let slot = slot.clone();
            let initialization_count = initialization_count.clone();
            tasks.push(tokio::spawn(async move {
                slot.get_or_init(move || async move {
                    initialization_count.fetch_add(1, Ordering::AcqRel);
                    tokio::task::yield_now().await;
                    Ok(format!("connection-{task_number}"))
                })
                .await
                .clone()
            }));
        }
        let mut connection_ids = Vec::new();
        for task in tasks {
            connection_ids.push(task.await.unwrap().unwrap());
        }
        assert_eq!(initialization_count.load(Ordering::Acquire), 1);
        assert!(
            connection_ids
                .iter()
                .all(|connection_id| connection_id == &connection_ids[0])
        );
    }

    #[tokio::test]
    async fn singleton_on_connect_completes_before_first_message_callback() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_address = listener.local_addr().unwrap();
        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let bytes_read = stream.read(&mut buffer).await.unwrap();
                if bytes_read == 0 {
                    return;
                }
                request.extend_from_slice(&buffer[..bytes_read]);
            }
            let request_text = String::from_utf8(request).unwrap();
            let websocket_key = request_text
                .lines()
                .find_map(|line| {
                    line.strip_prefix("sec-websocket-key: ")
                        .or_else(|| line.strip_prefix("Sec-WebSocket-Key: "))
                })
                .unwrap();
            let websocket_accept = websocket_accept(websocket_key);
            let handshake_response = format!(
                "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {websocket_accept}\r\nSec-WebSocket-Protocol: market.v1\r\n\r\n"
            );
            stream
                .write_all(handshake_response.as_bytes())
                .await
                .unwrap();
            stream.write_all(b"\x81\x05hello").await.unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
        });

        let (worker_sender, mut worker_receiver) = mpsc::channel::<RequestEnvelope>(16);
        let callback_order = Arc::new(Mutex::new(Vec::new()));
        let callback_order_for_worker = callback_order.clone();
        let worker_task = tokio::spawn(async move {
            while let Some(mut envelope) = worker_receiver.recv().await {
                envelope.signal_started();
                if let Some(event_name) = envelope
                    .req
                    .headers()
                    .get("x-fn0-internal-websocket-event")
                    .and_then(|value| value.to_str().ok())
                {
                    callback_order_for_worker
                        .lock()
                        .unwrap()
                        .push(event_name.to_string());
                    if event_name == "connect" {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                }
                let body = Full::new(Bytes::from_static(b"\"Ok\""))
                    .map_err(|never: Infallible| match never {})
                    .boxed_unsync();
                let response = hyper::Response::builder()
                    .status(hyper::StatusCode::NO_CONTENT)
                    .body(body)
                    .unwrap();
                let _ = envelope.resp_tx.send(Ok(response));
            }
        });

        let directory = Arc::new(MemoryDirectory::default());
        let service = Arc::new(WebSocketService {
            worker_senders: Arc::new(vec![worker_sender]),
            connections: DashMap::new(),
            singleton_connections: DashMap::new(),
            project_counts: DashMap::new(),
            project_generations: DashMap::new(),
            worker_count: Arc::new(AtomicUsize::new(0)),
            draining: AtomicBool::new(false),
            directory: directory.clone(),
            identity: WorkerIdentity {
                worker_id: "worker".to_string(),
                endpoint: String::new(),
            },
            quic: OnceLock::new(),
            self_reference: OnceLock::new(),
        });
        service
            .self_reference
            .set(Arc::downgrade(&service))
            .unwrap();
        let initial_lease_deadline = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
            + 60_000;
        let connection_id = service
            .connect_singleton_outbound(
                "project".to_string(),
                "feed".to_string(),
                format!("ws://{server_address}/stream?market=seoul"),
                "/ws_singleton/feed".to_string(),
                vec![("authorization".to_string(), "Bearer secret".to_string())],
                vec!["market.v1".to_string()],
                "claim-token".to_string(),
                initial_lease_deadline,
                Duration::from_secs(5),
            )
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(callback_order.lock().unwrap().is_empty());
        assert!(
            directory
                .lookup_connection(&connection_id)
                .await
                .unwrap()
                .is_none()
        );
        service
            .activate_singleton_outbound(
                (
                    "project".to_string(),
                    "feed".to_string(),
                    "claim-token".to_string(),
                ),
                &connection_id,
            )
            .await
            .unwrap();
        timeout(Duration::from_secs(1), async {
            loop {
                if callback_order.lock().unwrap().len() >= 2 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let callback_order = callback_order.lock().unwrap().clone();
        assert_eq!(&callback_order[..2], &["connect", "message"]);
        server_task.abort();
        worker_task.abort();
    }

    #[tokio::test]
    async fn deployment_closes_existing_connection_with_service_restart() {
        let directory = Arc::new(MemoryDirectory::default());
        let service = Arc::new(WebSocketService {
            worker_senders: Arc::new(Vec::new()),
            connections: DashMap::new(),
            singleton_connections: DashMap::new(),
            project_counts: DashMap::new(),
            project_generations: DashMap::new(),
            worker_count: Arc::new(AtomicUsize::new(0)),
            draining: AtomicBool::new(false),
            directory,
            identity: WorkerIdentity {
                worker_id: "worker".to_string(),
                endpoint: String::new(),
            },
            quic: OnceLock::new(),
            self_reference: OnceLock::new(),
        });
        service
            .self_reference
            .set(Arc::downgrade(&service))
            .unwrap();
        let (command_sender, mut command_receiver) = mpsc::channel(OUTBOUND_COMMAND_CAPACITY);
        let (_closed_sender, closed_receiver) = watch::channel(false);
        let (control_sender, _control_receiver) = mpsc::unbounded_channel();
        service.connections.insert(
            "connection".to_string(),
            Arc::new(ConnectionEntry {
                project_id: "project".to_string(),
                command_sender,
                closing: AtomicBool::new(false),
                closed_receiver,
                control_sender,
            }),
        );
        service.close_project("project").await;
        let command = timeout(Duration::from_secs(1), command_receiver.recv())
            .await
            .unwrap()
            .unwrap();
        let SocketCommand::Close { code, info, .. } = command else {
            panic!("expected close command");
        };
        assert_eq!(code, 1012);
        assert_eq!(info.cause, "deployment");
    }

    #[tokio::test]
    async fn singleton_fencing_blocks_sends_and_closes_connection() {
        let directory = Arc::new(MemoryDirectory::default());
        let service = Arc::new(WebSocketService {
            worker_senders: Arc::new(Vec::new()),
            connections: DashMap::new(),
            singleton_connections: DashMap::new(),
            project_counts: DashMap::new(),
            project_generations: DashMap::new(),
            worker_count: Arc::new(AtomicUsize::new(0)),
            draining: AtomicBool::new(false),
            directory,
            identity: WorkerIdentity {
                worker_id: "worker".to_string(),
                endpoint: String::new(),
            },
            quic: OnceLock::new(),
            self_reference: OnceLock::new(),
        });
        service
            .self_reference
            .set(Arc::downgrade(&service))
            .unwrap();
        let (command_sender, _command_receiver) = mpsc::channel(OUTBOUND_COMMAND_CAPACITY);
        for queue_position in 0..OUTBOUND_COMMAND_CAPACITY {
            let (ready_sender, _ready_receiver) = oneshot::channel();
            let (response_sender, _response_receiver) = oneshot::channel();
            command_sender
                .try_send(SocketCommand::Send {
                    message_kind: WebSocketMessageKind::Text,
                    body: Full::new(Bytes::from(format!("queued-{queue_position}")))
                        .map_err(|never: std::convert::Infallible| match never {})
                        .boxed_unsync(),
                    ready_sender,
                    response_sender,
                    deadline: tokio::time::Instant::now() + Duration::from_secs(60),
                })
                .unwrap();
        }
        let (_closed_sender, closed_receiver) = watch::channel(false);
        let (control_sender, mut control_receiver) = mpsc::unbounded_channel();
        service.connections.insert(
            "connection".to_string(),
            Arc::new(ConnectionEntry {
                project_id: "project".to_string(),
                command_sender,
                closing: AtomicBool::new(false),
                closed_receiver,
                control_sender,
            }),
        );
        fence_singleton(&service, "project", "connection");
        assert!(
            service
                .connections
                .get("connection")
                .unwrap()
                .closing
                .load(Ordering::Acquire)
        );
        let control = control_receiver.recv().await.unwrap();
        let WriterControl::Close(code, info) = control else {
            panic!("expected close control");
        };
        assert_eq!(code, 1011);
        assert_eq!(info.cause, "heartbeat-timeout");
    }

    #[tokio::test]
    async fn expired_initial_singleton_lease_fences_immediately() {
        let directory = Arc::new(MemoryDirectory::default());
        let service = Arc::new(WebSocketService {
            worker_senders: Arc::new(Vec::new()),
            connections: DashMap::new(),
            singleton_connections: DashMap::new(),
            project_counts: DashMap::new(),
            project_generations: DashMap::new(),
            worker_count: Arc::new(AtomicUsize::new(0)),
            draining: AtomicBool::new(false),
            directory,
            identity: WorkerIdentity {
                worker_id: "worker".to_string(),
                endpoint: String::new(),
            },
            quic: OnceLock::new(),
            self_reference: OnceLock::new(),
        });
        service
            .self_reference
            .set(Arc::downgrade(&service))
            .unwrap();
        let (command_sender, _command_receiver) = mpsc::channel(OUTBOUND_COMMAND_CAPACITY);
        let (_closed_sender, closed_receiver) = watch::channel(false);
        let (control_sender, mut control_receiver) = mpsc::unbounded_channel();
        service.connections.insert(
            "connection".to_string(),
            Arc::new(ConnectionEntry {
                project_id: "project".to_string(),
                command_sender,
                closing: AtomicBool::new(false),
                closed_receiver,
                control_sender,
            }),
        );
        let singleton_key = (
            "project".to_string(),
            "feed".to_string(),
            "claim".to_string(),
        );
        let slot = Arc::new(SingletonConnectSlot::new());
        let (_lease_activation_sender, lease_activation_receiver) = oneshot::channel();
        singleton_lease_loop(
            service,
            "project".to_string(),
            "connection".to_string(),
            SingletonBinding {
                key: singleton_key,
                slot,
                singleton_id: "feed".to_string(),
                claim_token: "claim".to_string(),
                initial_lease_deadline: 0,
                activated: Arc::new(AtomicBool::new(false)),
            },
            lease_activation_receiver,
        )
        .await;
        let control = control_receiver.recv().await.unwrap();
        let WriterControl::Close(code, info) = control else {
            panic!("expected close control");
        };
        assert_eq!(code, 1011);
        assert_eq!(info.cause, "heartbeat-timeout");
    }

    #[tokio::test]
    async fn distributed_send_reaches_connection_on_second_worker() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let certificate =
            rcgen::generate_simple_self_signed(vec!["fn0-worker.internal".to_string()])
                .expect("generate test certificate");
        let certificate_pem = certificate.cert.pem();
        let key_pem = certificate.signing_key.serialize_pem();
        let bearer = "test-websocket-bearer".to_string();
        let server_name = "fn0-worker.internal".to_string();
        let source_endpoint = free_udp_address();
        let target_endpoint = free_udp_address();
        let shared_directory = Arc::new(MemoryDirectory::default());

        let source_service = new_test_service(
            shared_directory.clone(),
            source_endpoint,
            certificate_pem.clone(),
            key_pem.clone(),
            bearer.clone(),
            server_name.clone(),
        )
        .await;
        let target_service = new_test_service(
            shared_directory,
            target_endpoint,
            certificate_pem,
            key_pem,
            bearer,
            server_name,
        )
        .await;

        let project_id = "distributed-websocket-test";
        let connection_id = WebSocketService::connection_id();
        let (command_sender, mut command_receiver) = mpsc::channel(OUTBOUND_COMMAND_CAPACITY);
        let (_closed_sender, closed_receiver) = watch::channel(false);
        let (control_sender, _control_receiver) = mpsc::unbounded_channel();
        target_service.connections.insert(
            connection_id.clone(),
            Arc::new(ConnectionEntry {
                project_id: project_id.to_string(),
                command_sender,
                closing: AtomicBool::new(false),
                closed_receiver,
                control_sender,
            }),
        );
        target_service
            .publish_connection(project_id, &connection_id)
            .await
            .expect("publish target connection");

        let message = Bytes::from_static(b"cross-worker-delivery");
        let body = Full::new(message.clone())
            .map_err(|never: Infallible| match never {})
            .boxed_unsync();
        let send_task = tokio::spawn(source_service.send(
            project_id.to_string(),
            connection_id.clone(),
            WebSocketMessageKind::Text,
            body,
            Duration::from_secs(5),
        ));

        let command = timeout(Duration::from_secs(5), command_receiver.recv())
            .await
            .expect("target worker did not receive command")
            .expect("target worker command channel closed");
        let SocketCommand::Send {
            body,
            ready_sender,
            response_sender,
            ..
        } = command
        else {
            panic!("target worker received a non-send command");
        };
        ready_sender.send(()).expect("send ready signal");
        let received_body = body
            .collect()
            .await
            .expect("collect target worker body")
            .to_bytes();
        assert_eq!(received_body, message);
        response_sender
            .send(Ok(()))
            .expect("send command completion");
        send_task
            .await
            .expect("source worker send task panicked")
            .expect("distributed send failed");

        target_service.unpublish_connection(&connection_id).await;
    }

    async fn new_test_service(
        directory: Arc<MemoryDirectory>,
        endpoint: SocketAddr,
        certificate_pem: String,
        key_pem: String,
        bearer: String,
        server_name: String,
    ) -> Arc<WebSocketService> {
        let service = Arc::new(WebSocketService {
            worker_senders: Arc::new(Vec::new()),
            connections: DashMap::new(),
            singleton_connections: DashMap::new(),
            project_counts: DashMap::new(),
            project_generations: DashMap::new(),
            worker_count: Arc::new(AtomicUsize::new(0)),
            draining: AtomicBool::new(false),
            directory,
            identity: WorkerIdentity {
                worker_id: format!("test-worker-{endpoint}"),
                endpoint: endpoint.to_string(),
            },
            quic: OnceLock::new(),
            self_reference: OnceLock::new(),
        });
        service
            .self_reference
            .set(Arc::downgrade(&service))
            .expect("set test websocket service self reference");
        let quic = QuicTransport::from_test_config(
            Arc::downgrade(&service),
            endpoint,
            certificate_pem,
            key_pem,
            bearer,
            server_name,
        )
        .expect("create test QUIC transport");
        assert!(service.quic.set(quic.clone()).is_ok());
        quic.spawn_server();
        tokio::task::yield_now().await;
        service
    }

    fn free_udp_address() -> SocketAddr {
        UdpSocket::bind("127.0.0.1:0")
            .expect("allocate UDP port")
            .local_addr()
            .expect("read UDP address")
    }
}
