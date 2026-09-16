use crate::websocket_directory::{
    ConnectionDirectory, ConnectionOwner, WorkerIdentity, directory_from_env,
    worker_identity_from_env,
};
use crate::websocket_quic::{QuicSendRequest, QuicTransport};
use crate::worker_pool::{self, RequestEnvelope, StartGate};
use base64::Engine;
use bytes::Bytes;
use dashmap::DashMap;
use fastwebsockets::handshake;
use fastwebsockets::upgrade::UpgradeFut;
use fastwebsockets::{Frame, OpCode, Payload, WebSocketError, WebSocketRead, WebSocketWrite};
use fn0::{
    Body, EgressBudget, EgressDenied, OutboundDialError, OutboundDialer,
    WebSocketCommandDispatcher, WebSocketCommandError, WebSocketCommandErrorKind,
    WebSocketCommandFuture, WebSocketConnectFuture, WebSocketDeliveryState, WebSocketMessageKind,
    WebSocketSingletonConnectRequest,
};
use http_body_util::{BodyExt, Empty, Full};
use hyper_util::rt::TokioIo;
use rand::RngCore;
use rustls::pki_types::ServerName;
use sha1::{Digest, Sha1};
use std::future::Future;
use std::pin::Pin;
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
const SINGLETON_RETRY_INITIAL_DELAY: Duration = Duration::from_millis(250);
const SINGLETON_RETRY_MAX_DELAY: Duration = Duration::from_secs(5);
const CONTROL_FRAME_WRITE_DEADLINE: Duration = Duration::from_secs(5);

type UpgradedIo = TokioIo<hyper::upgrade::Upgraded>;
type SocketReader = WebSocketRead<ReadHalf<UpgradedIo>>;
type SocketWriter = WebSocketWrite<WriteHalf<UpgradedIo>>;
type SingletonKey = (String, String, String);
type SingletonResolveKey = (String, String);
type SingletonConnectSlot =
    tokio::sync::OnceCell<Result<Arc<PreparedSingleton>, WebSocketCommandError>>;
type OutboundHandshakeRequest = (String, String, u16, hyper::Request<Empty<Bytes>>, String);

struct OutboundConnectOptions {
    project_id: String,
    url: String,
    receive_path: String,
    remaining: Duration,
    singleton_binding: Option<SingletonBinding>,
    handshake_headers: Vec<(String, String)>,
    protocols: Vec<String>,
}

struct OutboundHandshakeOptions<Stream> {
    project_id: String,
    connection_id: String,
    route_uri: hyper::Uri,
    request: hyper::Request<Empty<Bytes>>,
    stream: Stream,
    capacity_guard: CapacityGuard,
    singleton_binding: Option<SingletonBinding>,
    expected_accept: String,
    requested_protocols: Vec<String>,
}

struct RunConnectionOptions {
    project_id: String,
    connection_id: String,
    route_uri: hyper::Uri,
    reader: SocketReader,
    writer: SocketWriter,
    command_receiver: mpsc::Receiver<SocketCommand>,
    control_sender: mpsc::UnboundedSender<WriterControl>,
    control_receiver: mpsc::UnboundedReceiver<WriterControl>,
    closed_sender: watch::Sender<bool>,
    force_close_receiver: watch::Receiver<bool>,
    capacity_guard: CapacityGuard,
    singleton_binding: Option<SingletonBinding>,
    message_ready: Option<oneshot::Receiver<()>>,
    lease_activation: Option<oneshot::Receiver<()>>,
}

struct ReadLoopOptions {
    service: Arc<WebSocketService>,
    project_id: String,
    connection_id: String,
    route_uri: hyper::Uri,
    reader: SocketReader,
    control_sender: mpsc::UnboundedSender<WriterControl>,
    disconnect_info: Arc<Mutex<Option<DisconnectInfo>>>,
    message_ready: Option<oneshot::Receiver<()>>,
    lease_guard: Option<Arc<SingletonLeaseGuard>>,
}

struct DispatchInboundOptions<'a> {
    service: &'a Arc<WebSocketService>,
    project_id: &'a str,
    connection_id: &'a str,
    route_uri: &'a hyper::Uri,
    message_kind: WebSocketMessageKind,
    message_bytes: Vec<u8>,
    pending_messages: &'a Arc<AtomicUsize>,
    control_sender: &'a mpsc::UnboundedSender<WriterControl>,
    lease_guard: Option<&'a Arc<SingletonLeaseGuard>>,
}

struct PreparedSingleton {
    connection_id: String,
    project_id: String,
    route_uri: hyper::Uri,
    response_headers: hyper::HeaderMap,
    lease_guard: Arc<SingletonLeaseGuard>,
    lease_activation_sender: Mutex<Option<oneshot::Sender<()>>>,
    message_ready_sender: Mutex<Option<oneshot::Sender<()>>>,
    activation_lifecycle: Arc<SingletonActivationLifecycle>,
    activation: tokio::sync::OnceCell<watch::Receiver<Option<Result<(), WebSocketCommandError>>>>,
}

#[derive(Clone)]
pub(crate) struct SingletonConnectionResolution {
    pub connection_id: String,
    pub lease_expires_at_millis: i64,
    pub resolved_at_millis: i64,
}

#[derive(Clone)]
enum SingletonResolveResult {
    Connected(CachedSingletonConnection),
    Unavailable,
    Failed(String),
}

#[derive(Clone)]
struct CachedSingletonConnection {
    connection_id: String,
    valid_until: tokio::time::Instant,
}

struct SingletonResolveEntry {
    request_started_at: tokio::time::Instant,
    started: tokio::sync::OnceCell<()>,
    result: tokio::sync::OnceCell<SingletonResolveResult>,
    result_ready: tokio::sync::Notify,
}

impl SingletonResolveEntry {
    fn new(request_started_at: tokio::time::Instant) -> Self {
        Self {
            request_started_at,
            started: tokio::sync::OnceCell::new(),
            result: tokio::sync::OnceCell::new(),
            result_ready: tokio::sync::Notify::new(),
        }
    }

    async fn start(
        self: &Arc<Self>,
        resolver: Arc<dyn SingletonConnectionResolver>,
        project_id: String,
        singleton_id: String,
    ) {
        let entry = self.clone();
        let _ = self
            .started
            .get_or_init(|| async move {
                tokio::spawn(async move {
                    let result = match resolver.resolve(&project_id, &singleton_id).await {
                        Ok(Some(resolution)) => {
                            SingletonResolveResult::Connected(CachedSingletonConnection {
                                connection_id: resolution.connection_id,
                                valid_until: singleton_cache_valid_until(
                                    entry.request_started_at,
                                    resolution.lease_expires_at_millis,
                                    resolution.resolved_at_millis,
                                ),
                            })
                        }
                        Ok(None) => SingletonResolveResult::Unavailable,
                        Err(error) => SingletonResolveResult::Failed(error.to_string()),
                    };
                    let _ = entry.result.set(result);
                    entry.result_ready.notify_waiters();
                });
            })
            .await;
    }

    async fn wait(&self) -> SingletonResolveResult {
        loop {
            if let Some(result) = self.result.get() {
                return result.clone();
            }
            let notified = self.result_ready.notified();
            if let Some(result) = self.result.get() {
                return result.clone();
            }
            notified.await;
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SingletonActivationState {
    Prepared,
    Activating,
    CallbackRunning,
    Active,
    AbortRequested,
    Aborted,
    Failed,
}

struct SingletonActivationLifecycle {
    state: Mutex<SingletonActivationState>,
    activation_started: AtomicBool,
    callback_started: AtomicBool,
    terminal_sender: watch::Sender<bool>,
}

impl SingletonActivationLifecycle {
    fn new() -> Self {
        let (terminal_sender, _) = watch::channel(false);
        Self {
            state: Mutex::new(SingletonActivationState::Prepared),
            activation_started: AtomicBool::new(false),
            callback_started: AtomicBool::new(false),
            terminal_sender,
        }
    }

    fn begin(&self) -> bool {
        let mut state = self.state.lock().expect("singleton activation state lock");
        if *state != SingletonActivationState::Prepared {
            return false;
        }
        *state = SingletonActivationState::Activating;
        self.activation_started.store(true, Ordering::Release);
        true
    }

    fn begin_callback(&self) -> bool {
        let mut state = self.state.lock().expect("singleton activation state lock");
        if *state != SingletonActivationState::Activating {
            return false;
        }
        *state = SingletonActivationState::CallbackRunning;
        self.callback_started.store(true, Ordering::Release);
        true
    }

    fn commit_active(&self) -> bool {
        let mut state = self.state.lock().expect("singleton activation state lock");
        if *state != SingletonActivationState::CallbackRunning {
            return false;
        }
        *state = SingletonActivationState::Active;
        true
    }

    fn request_abort(&self) -> bool {
        let terminal = {
            let mut state = self.state.lock().expect("singleton activation state lock");
            match *state {
                SingletonActivationState::Prepared => {
                    *state = SingletonActivationState::Aborted;
                    true
                }
                SingletonActivationState::Activating
                | SingletonActivationState::CallbackRunning
                | SingletonActivationState::Active => {
                    *state = SingletonActivationState::AbortRequested;
                    false
                }
                SingletonActivationState::AbortRequested => false,
                SingletonActivationState::Aborted | SingletonActivationState::Failed => true,
            }
        };
        if terminal {
            let _ = self.terminal_sender.send(true);
        }
        self.activation_started()
    }

    fn finish(&self, succeeded: bool) -> bool {
        let active = {
            let mut state = self.state.lock().expect("singleton activation state lock");
            *state = match *state {
                SingletonActivationState::AbortRequested | SingletonActivationState::Aborted => {
                    SingletonActivationState::Aborted
                }
                SingletonActivationState::Active if succeeded => SingletonActivationState::Active,
                _ => SingletonActivationState::Failed,
            };
            *state == SingletonActivationState::Active
        };
        let _ = self.terminal_sender.send(true);
        active
    }

    fn activation_started(&self) -> bool {
        self.activation_started.load(Ordering::Acquire)
    }

    fn callback_started(&self) -> bool {
        self.callback_started.load(Ordering::Acquire)
    }

    fn is_active(&self) -> bool {
        *self.state.lock().expect("singleton activation state lock")
            == SingletonActivationState::Active
    }

    fn terminal_receiver(&self) -> watch::Receiver<bool> {
        self.terminal_sender.subscribe()
    }
}

enum OutboundConnectResult {
    Active(String),
    Prepared(Arc<PreparedSingleton>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SingletonStatusResponse {
    Accepted,
    Rejected,
    Retryable,
}

#[derive(Clone, Copy)]
struct SingletonLeaseTiming {
    heartbeat_interval: Duration,
    safety_deadline: Duration,
    retry_initial_delay: Duration,
    retry_max_delay: Duration,
}

const SINGLETON_LEASE_TIMING: SingletonLeaseTiming = SingletonLeaseTiming {
    heartbeat_interval: SINGLETON_HEARTBEAT_INTERVAL,
    safety_deadline: SINGLETON_SAFETY_DEADLINE,
    retry_initial_delay: SINGLETON_RETRY_INITIAL_DELAY,
    retry_max_delay: SINGLETON_RETRY_MAX_DELAY,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SingletonFenceReason {
    OwnershipRejected,
    SafetyDeadlineExpired,
}

/// The local answer to "may this singleton connection still carry application traffic?".
///
/// The lease task extends and revokes it, but every send admission, frame write, and callback
/// start reads the clock itself. A process that resumes after a pause past `valid_until` is
/// refused before the lease task gets a chance to run and close the socket.
struct SingletonLeaseGuard {
    valid_until: Mutex<tokio::time::Instant>,
    revoked: AtomicBool,
}

impl SingletonLeaseGuard {
    fn new(valid_until: tokio::time::Instant) -> Self {
        Self {
            valid_until: Mutex::new(valid_until),
            revoked: AtomicBool::new(false),
        }
    }

    fn permits_traffic(&self) -> bool {
        !self.revoked.load(Ordering::Acquire) && tokio::time::Instant::now() < self.valid_until()
    }

    fn valid_until(&self) -> tokio::time::Instant {
        *self.valid_until.lock().expect("singleton lease guard lock")
    }

    fn extend_to(&self, valid_until: tokio::time::Instant) {
        let mut current_valid_until = self.valid_until.lock().expect("singleton lease guard lock");
        if valid_until > *current_valid_until {
            *current_valid_until = valid_until;
        }
    }

    fn revoke(&self) {
        self.revoked.store(true, Ordering::Release);
    }

    fn start_gate(self: &Arc<Self>) -> StartGate {
        let lease_guard = self.clone();
        Arc::new(move || lease_guard.permits_traffic())
    }
}

#[derive(Debug, Eq, PartialEq)]
enum SingletonLeaseStep {
    AttemptAt(tokio::time::Instant),
    Fence(SingletonFenceReason),
}

struct SingletonLeaseSchedule {
    timing: SingletonLeaseTiming,
    safety_deadline: tokio::time::Instant,
    next_attempt: tokio::time::Instant,
    retry_backoff: Duration,
}

impl SingletonLeaseSchedule {
    fn new(
        timing: SingletonLeaseTiming,
        safety_deadline: tokio::time::Instant,
        now: tokio::time::Instant,
    ) -> Self {
        Self {
            timing,
            safety_deadline,
            next_attempt: now + timing.heartbeat_interval,
            retry_backoff: timing.retry_initial_delay,
        }
    }

    fn next_step(&self, now: tokio::time::Instant) -> SingletonLeaseStep {
        if now >= self.safety_deadline {
            SingletonLeaseStep::Fence(SingletonFenceReason::SafetyDeadlineExpired)
        } else {
            SingletonLeaseStep::AttemptAt(self.next_attempt.min(self.safety_deadline))
        }
    }

    /// `None` is a renewal whose answer never arrived. Control may have extended the lease, but
    /// the worker cannot know that, so it is treated like a retryable failure. An accepted
    /// renewal extends the deadline from when the request was sent, not when the answer arrived,
    /// because control stamps its lease no earlier than that moment.
    fn record_renewal(
        &mut self,
        response: Option<SingletonStatusResponse>,
        request_started_at: tokio::time::Instant,
        now: tokio::time::Instant,
        jittered_retry_delay: Duration,
    ) -> Result<(), SingletonFenceReason> {
        match response {
            Some(SingletonStatusResponse::Accepted) => {
                self.safety_deadline = self
                    .safety_deadline
                    .max(request_started_at + self.timing.safety_deadline);
                self.next_attempt = now + self.timing.heartbeat_interval;
                self.retry_backoff = self.timing.retry_initial_delay;
                Ok(())
            }
            Some(SingletonStatusResponse::Rejected) => Err(SingletonFenceReason::OwnershipRejected),
            Some(SingletonStatusResponse::Retryable) | None => {
                let remaining = self.safety_deadline.saturating_duration_since(now);
                self.next_attempt = now + jittered_retry_delay.min(remaining);
                self.retry_backoff = self
                    .retry_backoff
                    .saturating_mul(2)
                    .min(self.timing.retry_max_delay);
                Ok(())
            }
        }
    }
}

pub(crate) type SingletonConnectionResolveFuture = Pin<
    Box<
        dyn Future<Output = anyhow::Result<Option<SingletonConnectionResolution>>> + Send + 'static,
    >,
>;

pub(crate) trait SingletonConnectionResolver: Send + Sync {
    fn resolve(&self, project_id: &str, singleton_id: &str) -> SingletonConnectionResolveFuture;
}

struct ControlSingletonConnectionResolver {
    worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
}

#[derive(serde::Deserialize)]
enum SingletonResolveResponse {
    Connected {
        connection_id: String,
        lease_expires_at_millis: i64,
        resolved_at_millis: i64,
    },
    Unavailable,
    Unauthorized,
    Error,
}

impl SingletonConnectionResolver for ControlSingletonConnectionResolver {
    fn resolve(&self, project_id: &str, singleton_id: &str) -> SingletonConnectionResolveFuture {
        let worker_senders = self.worker_senders.clone();
        let project_id = project_id.to_string();
        let singleton_id = singleton_id.to_string();
        Box::pin(async move {
            let body = serde_json::to_vec(&serde_json::json!({
                "project_id": project_id,
                "singleton_id": singleton_id,
            }))?;
            let request = hyper::Request::builder()
                .method(hyper::Method::POST)
                .uri("https://fn0-control.internal/__forte_action/websocket_singleton_resolve")
                .header(hyper::header::CONTENT_TYPE, "application/json")
                .header("x-fn0-internal-websocket-singleton-resolve", "true")
                .body(
                    Full::new(Bytes::from(body))
                        .map_err(|never: std::convert::Infallible| match never {})
                        .boxed_unsync(),
                )?;
            let response = worker_pool::invoke_and_wait(
                &worker_senders,
                |response_sender| {
                    RequestEnvelope::new("fn0-control".to_string(), request, response_sender)
                },
                CALLBACK_DEADLINE,
                CALLBACK_DEADLINE,
            )
            .await?;
            if !response.status().is_success() {
                anyhow::bail!("singleton resolve returned status {}", response.status());
            }
            let body = response.into_body().collect().await?.to_bytes();
            match serde_json::from_slice(&body)? {
                SingletonResolveResponse::Connected {
                    connection_id,
                    lease_expires_at_millis,
                    resolved_at_millis,
                } => Ok(Some(SingletonConnectionResolution {
                    connection_id,
                    lease_expires_at_millis,
                    resolved_at_millis,
                })),
                SingletonResolveResponse::Unavailable => Ok(None),
                SingletonResolveResponse::Unauthorized | SingletonResolveResponse::Error => {
                    anyhow::bail!("singleton resolve failed in control")
                }
            }
        })
    }
}

#[derive(Clone)]
struct SingletonBinding {
    key: SingletonKey,
    slot: Arc<SingletonConnectSlot>,
    singleton_id: String,
    claim_token: String,
    lease_guard: Arc<SingletonLeaseGuard>,
    activation_lifecycle: Arc<SingletonActivationLifecycle>,
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

    fn egress_quota_exceeded() -> Self {
        Self {
            close_code: Some(1008),
            reason: None,
            cause: "egress-quota-exceeded",
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
    force_close_sender: watch::Sender<bool>,
    lease_guard: Option<Arc<SingletonLeaseGuard>>,
}

impl ConnectionEntry {
    fn lease_permits_traffic(&self) -> bool {
        self.lease_guard
            .as_ref()
            .is_none_or(|lease_guard| lease_guard.permits_traffic())
    }

    fn accepts_application_traffic(&self) -> bool {
        !self.closing.load(Ordering::Acquire) && self.lease_permits_traffic()
    }
}

struct RegisteredConnection {
    command_receiver: mpsc::Receiver<SocketCommand>,
    control_sender: mpsc::UnboundedSender<WriterControl>,
    control_receiver: mpsc::UnboundedReceiver<WriterControl>,
    closed_sender: watch::Sender<bool>,
    force_close_receiver: watch::Receiver<bool>,
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
    singleton_resolve_cache: DashMap<SingletonResolveKey, Arc<SingletonResolveEntry>>,
    project_counts: DashMap<String, Arc<AtomicUsize>>,
    project_generations: DashMap<String, Arc<std::sync::atomic::AtomicU64>>,
    worker_count: Arc<AtomicUsize>,
    draining: AtomicBool,
    directory: Arc<dyn ConnectionDirectory>,
    identity: WorkerIdentity,
    quic: OnceLock<Arc<QuicTransport>>,
    self_reference: OnceLock<Weak<WebSocketService>>,
    outbound_dialer: OutboundDialer,
    egress_budget: Arc<dyn EgressBudget>,
    singleton_resolver: Arc<dyn SingletonConnectionResolver>,
}

impl WebSocketService {
    pub async fn new(
        worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
        outbound_dialer: OutboundDialer,
        egress_budget: Arc<dyn EgressBudget>,
    ) -> anyhow::Result<Arc<Self>> {
        let identity = worker_identity_from_env();
        let directory = directory_from_env(&identity)?;
        let singleton_resolver = Arc::new(ControlSingletonConnectionResolver {
            worker_senders: worker_senders.clone(),
        });
        let service = Arc::new(Self {
            worker_senders,
            connections: DashMap::new(),
            singleton_connections: DashMap::new(),
            singleton_resolve_cache: DashMap::new(),
            project_counts: DashMap::new(),
            project_generations: DashMap::new(),
            worker_count: Arc::new(AtomicUsize::new(0)),
            draining: AtomicBool::new(false),
            directory,
            identity,
            quic: OnceLock::new(),
            self_reference: OnceLock::new(),
            outbound_dialer,
            egress_budget,
            singleton_resolver,
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
        self.invoke_connect_with_gate(
            project_id,
            connection_id,
            uri,
            request_headers,
            client_address,
            None,
        )
        .await
    }

    async fn invoke_connect_with_gate(
        &self,
        project_id: &str,
        connection_id: &str,
        uri: &hyper::Uri,
        request_headers: &hyper::HeaderMap,
        client_address: Option<std::net::SocketAddr>,
        start_gate: Option<StartGate>,
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
        self.invoke_gated(project_id, request, start_gate).await
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
        if !entry.lease_permits_traffic() {
            fence_singleton(self, caller_project_id, connection_id);
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::ConnectionNotFound,
            ));
        }
        if self.egress_budget.known_exhausted(caller_project_id) {
            close_connection(
                self,
                caller_project_id,
                connection_id,
                1008,
                DisconnectInfo::egress_quota_exceeded(),
            );
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::EgressQuotaExceeded,
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
                service.register_connection(&project_id, &connection_id, &capacity_guard, None);
            service
                .run_connection(RunConnectionOptions {
                    project_id,
                    connection_id,
                    route_uri,
                    reader,
                    writer,
                    command_receiver: registered.command_receiver,
                    control_sender: registered.control_sender,
                    control_receiver: registered.control_receiver,
                    closed_sender: registered.closed_sender,
                    force_close_receiver: registered.force_close_receiver,
                    capacity_guard,
                    singleton_binding: None,
                    message_ready: None,
                    lease_activation: None,
                })
                .await;
        });
    }

    async fn connect_outbound(
        self: &Arc<Self>,
        options: OutboundConnectOptions,
    ) -> Result<OutboundConnectResult, WebSocketCommandError> {
        let OutboundConnectOptions {
            project_id,
            url,
            receive_path,
            remaining,
            singleton_binding,
            handshake_headers,
            protocols,
        } = options;
        let deadline = tokio::time::Instant::now() + remaining;
        if self.egress_budget.known_exhausted(&project_id) {
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::EgressQuotaExceeded,
            ));
        }
        let capacity_guard = self.reserve_capacity(&project_id).map_err(|_| {
            WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Backpressure)
        })?;
        let connection_id = Self::connection_id();
        let route_uri = format!("https://fn0-websocket.internal{receive_path}")
            .parse::<hyper::Uri>()
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal))?;
        let (scheme, host, port, request, expected_accept) =
            build_outbound_handshake_request(&url, handshake_headers, &protocols)?;
        let stream: TcpStream = tokio::time::timeout_at(
            deadline,
            self.outbound_dialer
                .connect(&host, port, OUTBOUND_DIAL_TIMEOUT),
        )
        .await
        .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::DeadlineExceeded))?
        .map_err(|error| match error {
            OutboundDialError::DestinationForbidden => {
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::DestinationForbidden)
            }
            OutboundDialError::NameResolution(_)
            | OutboundDialError::NoAddresses
            | OutboundDialError::Connect(_)
            | OutboundDialError::Timeout => {
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport)
            }
        })?;
        let result = if scheme == "ws" {
            tokio::time::timeout_at(
                deadline,
                self.finish_outbound_handshake(OutboundHandshakeOptions {
                    project_id,
                    connection_id: connection_id.clone(),
                    route_uri,
                    request,
                    stream,
                    capacity_guard,
                    singleton_binding,
                    expected_accept,
                    requested_protocols: protocols,
                }),
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
                self.finish_outbound_handshake(OutboundHandshakeOptions {
                    project_id,
                    connection_id: connection_id.clone(),
                    route_uri,
                    request,
                    stream: tls_stream,
                    capacity_guard,
                    singleton_binding,
                    expected_accept,
                    requested_protocols: protocols,
                }),
            )
            .await
            .map_err(|_| {
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::DeadlineExceeded)
            })??
        };
        Ok(result)
    }

    async fn connect_singleton_outbound(
        self: &Arc<Self>,
        request: WebSocketSingletonConnectRequest,
    ) -> Result<String, WebSocketCommandError> {
        let WebSocketSingletonConnectRequest {
            project_id,
            singleton_id,
            url,
            route_path,
            headers,
            protocols,
            claim_token,
            initial_lease_deadline,
            remaining,
        } = request;
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
                let activation_lifecycle = Arc::new(SingletonActivationLifecycle::new());
                let lease_guard = Arc::new(SingletonLeaseGuard::new(initial_lease_valid_until(
                    initial_lease_deadline,
                )));
                let result = service
                    .connect_outbound(OutboundConnectOptions {
                        project_id,
                        url,
                        receive_path: route_path,
                        remaining,
                        singleton_binding: Some(SingletonBinding {
                            key: key_for_connect,
                            slot: slot_for_connect,
                            singleton_id,
                            claim_token,
                            lease_guard,
                            activation_lifecycle,
                        }),
                        handshake_headers: headers,
                        protocols,
                    })
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
                if result.is_ok() && !prepared.activation_lifecycle.is_active() {
                    return Err(WebSocketCommandError::not_sent(
                        WebSocketCommandErrorKind::ConnectionNotFound,
                    ));
                }
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
        if !prepared.activation_lifecycle.begin() {
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::ConnectionNotFound,
            ));
        }
        let result = self
            .activate_prepared_singleton_started(prepared.clone())
            .await;
        let active = prepared.activation_lifecycle.finish(result.is_ok());
        if result.is_ok() && !active {
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::ConnectionNotFound,
            ));
        }
        result
    }

    async fn activate_prepared_singleton_started(
        self: &Arc<Self>,
        prepared: Arc<PreparedSingleton>,
    ) -> Result<(), WebSocketCommandError> {
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
        if !prepared.activation_lifecycle.begin_callback() {
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
        if !prepared.lease_guard.permits_traffic() {
            fence_singleton(self, &prepared.project_id, &prepared.connection_id);
            return Err(WebSocketCommandError::not_sent(
                WebSocketCommandErrorKind::ConnectionNotFound,
            ));
        }
        let callback_response = self
            .invoke_connect_with_gate(
                &prepared.project_id,
                &prepared.connection_id,
                &prepared.route_uri,
                &prepared.response_headers,
                None,
                Some(prepared.lease_guard.start_gate()),
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
        if !prepared.activation_lifecycle.commit_active() {
            self.unpublish_connection(&prepared.connection_id).await;
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

    async fn abort_singleton_outbound(
        &self,
        singleton_key: &SingletonKey,
        connection_id: &str,
        deadline: tokio::time::Instant,
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
        let mut activation_terminal_receiver = prepared.activation_lifecycle.terminal_receiver();
        prepared.activation_lifecycle.request_abort();
        let closed_receiver = self
            .connections
            .get(connection_id)
            .map(|entry| entry.closed_receiver.clone());
        if closed_receiver.is_some() {
            close_connection(
                self,
                &prepared.project_id,
                &prepared.connection_id,
                1011,
                DisconnectInfo::internal_error(),
            );
        }
        let close_wait = async {
            if let Some(mut closed_receiver) = closed_receiver {
                wait_for_connection_close(&mut closed_receiver, deadline).await
            } else {
                Ok(())
            }
        };
        let activation_wait =
            wait_for_activation_terminal(&mut activation_terminal_receiver, deadline);
        tokio::try_join!(close_wait, activation_wait)?;
        Ok(())
    }

    async fn finish_outbound_handshake<Stream>(
        self: &Arc<Self>,
        options: OutboundHandshakeOptions<Stream>,
    ) -> Result<OutboundConnectResult, WebSocketCommandError>
    where
        Stream: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let OutboundHandshakeOptions {
            project_id,
            connection_id,
            route_uri,
            request,
            stream,
            capacity_guard,
            singleton_binding,
            expected_accept,
            requested_protocols,
        } = options;
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
        let lease_guard = singleton_binding
            .as_ref()
            .map(|binding| binding.lease_guard.clone());
        let registered = self.register_connection(
            &project_id,
            &connection_id,
            &capacity_guard,
            lease_guard.clone(),
        );
        let (message_ready_sender, message_ready_receiver) = oneshot::channel();
        let is_singleton = singleton_binding.is_some();
        let singleton_activation_lifecycle = singleton_binding
            .as_ref()
            .map(|binding| binding.activation_lifecycle.clone());
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
                .run_connection(RunConnectionOptions {
                    project_id: spawned_project_id,
                    connection_id: spawned_connection_id,
                    route_uri: spawned_route_uri,
                    reader,
                    writer,
                    command_receiver: registered.command_receiver,
                    control_sender: registered.control_sender,
                    control_receiver: registered.control_receiver,
                    closed_sender: registered.closed_sender,
                    force_close_receiver: registered.force_close_receiver,
                    capacity_guard,
                    singleton_binding,
                    message_ready: Some(message_ready_receiver),
                    lease_activation: lease_activation_receiver,
                })
                .await;
        });
        if is_singleton {
            return Ok(OutboundConnectResult::Prepared(Arc::new(
                PreparedSingleton {
                    connection_id,
                    project_id,
                    route_uri,
                    response_headers: response.headers().clone(),
                    lease_guard: lease_guard.expect("singleton lease guard"),
                    lease_activation_sender: Mutex::new(lease_activation_sender),
                    message_ready_sender: Mutex::new(Some(message_ready_sender)),
                    activation_lifecycle: singleton_activation_lifecycle
                        .expect("singleton activation lifecycle"),
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
        lease_guard: Option<Arc<SingletonLeaseGuard>>,
    ) -> RegisteredConnection {
        let (command_sender, command_receiver) = mpsc::channel(OUTBOUND_COMMAND_CAPACITY);
        let (control_sender, control_receiver) = mpsc::unbounded_channel();
        let (closed_sender, closed_receiver) = watch::channel(false);
        let (force_close_sender, force_close_receiver) = watch::channel(false);
        let entry = Arc::new(ConnectionEntry {
            project_id: project_id.to_string(),
            command_sender,
            closing: AtomicBool::new(false),
            closed_receiver,
            control_sender: control_sender.clone(),
            force_close_sender,
            lease_guard,
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
            force_close_receiver,
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

    /// Resolves the singleton's current physical connection once and sends to it once. When the
    /// owner is replaced between the lookup and the write, the send fails with
    /// `ConnectionNotFound` instead of being retried on the replacement, because the caller
    /// cannot tell whether the replacement has finished its own protocol setup.
    async fn send_to_singleton(
        &self,
        caller_project_id: String,
        singleton_id: String,
        message_kind: WebSocketMessageKind,
        body: Body,
        remaining: Duration,
    ) -> Result<(), WebSocketCommandError> {
        let deadline = tokio::time::Instant::now() + remaining;
        let cache_key = (caller_project_id.clone(), singleton_id.clone());
        let (cache_entry, cached_connection) = self
            .resolve_singleton_cached(&caller_project_id, &singleton_id, deadline)
            .await?;
        let result = self
            .send(
                caller_project_id,
                cached_connection.connection_id,
                message_kind,
                body,
                deadline.saturating_duration_since(tokio::time::Instant::now()),
            )
            .await;
        if result
            .as_ref()
            .is_err_and(|error| error.kind == WebSocketCommandErrorKind::ConnectionNotFound)
        {
            self.singleton_resolve_cache
                .remove_if(&cache_key, |_, current| Arc::ptr_eq(current, &cache_entry));
        }
        result
    }

    async fn resolve_singleton_cached(
        &self,
        project_id: &str,
        singleton_id: &str,
        deadline: tokio::time::Instant,
    ) -> Result<(Arc<SingletonResolveEntry>, CachedSingletonConnection), WebSocketCommandError>
    {
        let cache_key = (project_id.to_string(), singleton_id.to_string());
        let mut expired_cache_seen = false;
        loop {
            let request_started_at = tokio::time::Instant::now();
            let cache_entry = self
                .singleton_resolve_cache
                .entry(cache_key.clone())
                .or_insert_with(|| Arc::new(SingletonResolveEntry::new(request_started_at)))
                .clone();
            cache_entry
                .start(
                    self.singleton_resolver.clone(),
                    project_id.to_string(),
                    singleton_id.to_string(),
                )
                .await;
            let result = match tokio::time::timeout_at(deadline, cache_entry.wait()).await {
                Ok(result) => result,
                Err(_) => {
                    return Err(WebSocketCommandError::not_sent(
                        WebSocketCommandErrorKind::DeadlineExceeded,
                    ));
                }
            };
            match result {
                SingletonResolveResult::Connected(cached_connection)
                    if tokio::time::Instant::now() < cached_connection.valid_until =>
                {
                    return Ok((cache_entry, cached_connection));
                }
                SingletonResolveResult::Connected(_) => {
                    self.singleton_resolve_cache
                        .remove_if(&cache_key, |_, current| Arc::ptr_eq(current, &cache_entry));
                    if expired_cache_seen {
                        return Err(WebSocketCommandError::not_sent(
                            WebSocketCommandErrorKind::ConnectionNotFound,
                        ));
                    }
                    expired_cache_seen = true;
                }
                SingletonResolveResult::Unavailable => {
                    self.singleton_resolve_cache
                        .remove_if(&cache_key, |_, current| Arc::ptr_eq(current, &cache_entry));
                    return Err(WebSocketCommandError::not_sent(
                        WebSocketCommandErrorKind::ConnectionNotFound,
                    ));
                }
                SingletonResolveResult::Failed(error) => {
                    self.singleton_resolve_cache
                        .remove_if(&cache_key, |_, current| Arc::ptr_eq(current, &cache_entry));
                    tracing::warn!(%project_id, %singleton_id, %error, "websocket singleton resolve failed");
                    return Err(WebSocketCommandError::not_sent(
                        WebSocketCommandErrorKind::Transport,
                    ));
                }
            }
        }
    }

    pub fn connection_count(&self) -> usize {
        self.worker_count.load(Ordering::Acquire)
    }

    async fn run_connection(self: &Arc<Self>, options: RunConnectionOptions) {
        let RunConnectionOptions {
            project_id,
            connection_id,
            route_uri,
            reader,
            mut writer,
            command_receiver,
            control_sender,
            control_receiver,
            closed_sender,
            mut force_close_receiver,
            capacity_guard,
            singleton_binding,
            message_ready,
            lease_activation,
        } = options;
        let disconnect_info = Arc::new(Mutex::new(None));
        let singleton_id = singleton_binding
            .as_ref()
            .map(|binding| binding.singleton_id.clone());
        let singleton_claim_token = singleton_binding
            .as_ref()
            .map(|binding| binding.claim_token.clone());
        let singleton_activation_lifecycle = singleton_binding
            .as_ref()
            .map(|binding| binding.activation_lifecycle.clone());
        let lease_guard = singleton_binding
            .as_ref()
            .map(|binding| binding.lease_guard.clone());
        let lease_handle = singleton_binding.as_ref().map(|binding| {
            tokio::spawn(singleton_lease_loop(
                self.clone(),
                project_id.clone(),
                connection_id.clone(),
                binding.clone(),
                lease_activation.expect("singleton lease activation receiver"),
            ))
        });
        let reader_handle = tokio::spawn(read_loop(ReadLoopOptions {
            service: self.clone(),
            project_id: project_id.clone(),
            connection_id: connection_id.clone(),
            route_uri: route_uri.clone(),
            reader,
            control_sender,
            disconnect_info: disconnect_info.clone(),
            message_ready,
            lease_guard: lease_guard.clone(),
        }));
        let outbound_frame_admission = OutboundFrameAdmission {
            project_id: project_id.clone(),
            lease_guard,
            egress_budget: self.egress_budget.clone(),
        };
        tokio::select! {
            _ = writer_loop(
                &mut writer,
                command_receiver,
                control_receiver,
                disconnect_info.clone(),
                &outbound_frame_admission,
            ) => {}
            _ = wait_for_force_close(&mut force_close_receiver) => {}
        }
        drop(writer);
        reader_handle.abort();
        let _ = reader_handle.await;
        if let Some(lease_handle) = lease_handle {
            lease_handle.abort();
            let _ = lease_handle.await;
        }
        self.connections.remove(&connection_id);
        self.unpublish_connection(&connection_id).await;
        let _ = closed_sender.send(true);
        drop(capacity_guard);
        let activation_started = singleton_activation_lifecycle
            .as_ref()
            .is_some_and(|activation_lifecycle| activation_lifecycle.request_abort());
        if activation_started
            && let Some(activation_lifecycle) = singleton_activation_lifecycle.as_ref()
        {
            let mut activation_terminal_receiver = activation_lifecycle.terminal_receiver();
            wait_for_activation_terminal_unbounded(&mut activation_terminal_receiver).await;
        }
        let callback_started = singleton_activation_lifecycle
            .as_ref()
            .is_some_and(|activation_lifecycle| activation_lifecycle.callback_started());
        if let Some(singleton_binding) = singleton_binding {
            self.singleton_connections
                .remove_if(&singleton_binding.key, |_, current| {
                    Arc::ptr_eq(current, &singleton_binding.slot)
                });
        }
        let final_info = disconnect_info
            .lock()
            .expect("disconnect info lock")
            .clone()
            .unwrap_or_else(DisconnectInfo::transport_error);
        let invoke_lifecycle = singleton_activation_lifecycle.is_none() || callback_started;
        if invoke_lifecycle {
            self.invoke_disconnect(&project_id, &connection_id, &route_uri, final_info);
        }
        if callback_started
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
        self.invoke_gated(project_id, request, None).await
    }

    async fn invoke_gated(
        &self,
        project_id: &str,
        request: fn0::Request,
        start_gate: Option<StartGate>,
    ) -> anyhow::Result<fn0::Response> {
        worker_pool::invoke_and_wait(
            &self.worker_senders,
            |response_sender| {
                let envelope =
                    RequestEnvelope::new(project_id.to_string(), request, response_sender);
                match start_gate {
                    Some(start_gate) => envelope.with_start_gate(start_gate),
                    None => envelope,
                }
            },
            CALLBACK_DEADLINE,
            CALLBACK_DEADLINE,
        )
        .await
    }

    async fn notify_singleton_status(
        &self,
        project_id: &str,
        singleton_id: &str,
        claim_token: &str,
        connection_id: &str,
        status: &str,
    ) -> anyhow::Result<SingletonStatusResponse> {
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
            return Ok(SingletonStatusResponse::Retryable);
        }
        let body = response.into_body().collect().await?.to_bytes();
        Ok(classify_singleton_status_response(&body))
    }
}

fn classify_singleton_status_response(body: &[u8]) -> SingletonStatusResponse {
    match body {
        b"\"Ok\"" => SingletonStatusResponse::Accepted,
        b"\"Ignored\"" | b"\"Unauthorized\"" => SingletonStatusResponse::Rejected,
        _ => SingletonStatusResponse::Retryable,
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

fn initial_lease_valid_until(initial_lease_deadline_millis: i64) -> tokio::time::Instant {
    let current_epoch_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(i64::MAX);
    let initial_remaining_millis = initial_lease_deadline_millis
        .saturating_sub(current_epoch_millis)
        .max(0) as u64;
    tokio::time::Instant::now()
        + SINGLETON_SAFETY_DEADLINE.min(Duration::from_millis(initial_remaining_millis))
}

fn singleton_cache_valid_until(
    request_started_at: tokio::time::Instant,
    lease_expires_at_millis: i64,
    resolved_at_millis: i64,
) -> tokio::time::Instant {
    let relative_millis = lease_expires_at_millis
        .saturating_sub(resolved_at_millis)
        .max(0) as u64;
    let safety_millis = SINGLETON_SAFETY_DEADLINE.as_millis() as u64;
    request_started_at + Duration::from_millis(relative_millis.saturating_sub(safety_millis))
}

async fn singleton_lease_loop(
    service: Arc<WebSocketService>,
    project_id: String,
    connection_id: String,
    binding: SingletonBinding,
    lease_activation: oneshot::Receiver<()>,
) {
    let lease_guard = binding.lease_guard.clone();
    tokio::select! {
        activation = lease_activation => {
            if activation.is_err() {
                fence_singleton(&service, &project_id, &connection_id);
                return;
            }
        }
        _ = tokio::time::sleep_until(lease_guard.valid_until()) => {
            fence_singleton(&service, &project_id, &connection_id);
            return;
        }
    }
    if !lease_guard.permits_traffic() {
        fence_singleton(&service, &project_id, &connection_id);
        return;
    }
    lease_guard.extend_to(tokio::time::Instant::now() + SINGLETON_SAFETY_DEADLINE);
    let fence_reason = maintain_singleton_lease(
        SINGLETON_LEASE_TIMING,
        &lease_guard,
        || {
            service.notify_singleton_status(
                &project_id,
                &binding.singleton_id,
                &binding.claim_token,
                &connection_id,
                "heartbeat",
            )
        },
        singleton_retry_delay,
    )
    .await;
    match fence_reason {
        SingletonFenceReason::OwnershipRejected => {
            tracing::warn!(%project_id, %connection_id, "websocket singleton ownership rejected");
        }
        SingletonFenceReason::SafetyDeadlineExpired => {
            tracing::warn!(%project_id, %connection_id, "websocket singleton lease safety deadline expired");
        }
    }
    fence_singleton(&service, &project_id, &connection_id);
}

async fn maintain_singleton_lease<Renew, RenewalFuture, RetryDelay>(
    timing: SingletonLeaseTiming,
    lease_guard: &SingletonLeaseGuard,
    mut renew: Renew,
    mut retry_delay: RetryDelay,
) -> SingletonFenceReason
where
    Renew: FnMut() -> RenewalFuture,
    RenewalFuture: Future<Output = anyhow::Result<SingletonStatusResponse>>,
    RetryDelay: FnMut(Duration) -> Duration,
{
    let mut schedule = SingletonLeaseSchedule::new(
        timing,
        lease_guard.valid_until(),
        tokio::time::Instant::now(),
    );
    loop {
        let attempt_at = match schedule.next_step(tokio::time::Instant::now()) {
            SingletonLeaseStep::Fence(fence_reason) => {
                lease_guard.revoke();
                return fence_reason;
            }
            SingletonLeaseStep::AttemptAt(attempt_at) => attempt_at,
        };
        tokio::time::sleep_until(attempt_at).await;
        let request_started_at = tokio::time::Instant::now();
        if request_started_at >= schedule.safety_deadline {
            continue;
        }
        let response = tokio::time::timeout_at(schedule.safety_deadline, renew())
            .await
            .ok()
            .and_then(Result::ok);
        let jittered_retry_delay = retry_delay(schedule.retry_backoff);
        match schedule.record_renewal(
            response,
            request_started_at,
            tokio::time::Instant::now(),
            jittered_retry_delay,
        ) {
            Ok(()) => lease_guard.extend_to(schedule.safety_deadline),
            Err(fence_reason) => {
                lease_guard.revoke();
                return fence_reason;
            }
        }
    }
}

fn singleton_retry_delay(backoff: Duration) -> Duration {
    let minimum = backoff / 2;
    let jitter_range_millis = u64::try_from((backoff - minimum).as_millis()).unwrap_or(u64::MAX);
    let jitter_millis = if jitter_range_millis == 0 {
        0
    } else {
        rand::thread_rng().next_u64() % (jitter_range_millis + 1)
    };
    minimum + Duration::from_millis(jitter_millis)
}

fn fence_singleton(service: &WebSocketService, project_id: &str, connection_id: &str) {
    if let Some(lease_guard) = service
        .connections
        .get(connection_id)
        .and_then(|entry| entry.lease_guard.clone())
    {
        lease_guard.revoke();
    }
    close_connection(
        service,
        project_id,
        connection_id,
        1011,
        DisconnectInfo::heartbeat_timeout(),
    );
}

async fn wait_for_connection_close(
    closed_receiver: &mut watch::Receiver<bool>,
    deadline: tokio::time::Instant,
) -> Result<(), WebSocketCommandError> {
    tokio::time::timeout_at(deadline, wait_for_true(closed_receiver))
        .await
        .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::DeadlineExceeded))
}

async fn wait_for_activation_terminal(
    activation_terminal_receiver: &mut watch::Receiver<bool>,
    deadline: tokio::time::Instant,
) -> Result<(), WebSocketCommandError> {
    tokio::time::timeout_at(deadline, wait_for_true(activation_terminal_receiver))
        .await
        .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::DeadlineExceeded))
}

async fn wait_for_activation_terminal_unbounded(
    activation_terminal_receiver: &mut watch::Receiver<bool>,
) {
    wait_for_true(activation_terminal_receiver).await;
}

async fn wait_for_true(receiver: &mut watch::Receiver<bool>) {
    while !*receiver.borrow() {
        if receiver.changed().await.is_err() {
            return;
        }
    }
}

async fn wait_for_force_close(force_close_receiver: &mut watch::Receiver<bool>) {
    while !*force_close_receiver.borrow() {
        if force_close_receiver.changed().await.is_err() {
            return;
        }
    }
}

fn schedule_force_close(force_close_sender: watch::Sender<bool>, delay: Duration) {
    tokio::spawn(async move {
        tokio::time::sleep(delay).await;
        let _ = force_close_sender.send(true);
    });
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
    let force_close_sender = entry.force_close_sender.clone();
    schedule_force_close(force_close_sender, CLOSE_HANDSHAKE_DEADLINE);
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
                .connect_outbound(OutboundConnectOptions {
                    project_id: caller_project_id,
                    url,
                    receive_path,
                    remaining,
                    singleton_binding: None,
                    handshake_headers: Vec::new(),
                    protocols: Vec::new(),
                })
                .await?
            {
                OutboundConnectResult::Active(connection_id) => Ok(connection_id),
                OutboundConnectResult::Prepared(_) => Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::Internal,
                )),
            }
        })
    }

    fn connect_singleton(
        &self,
        request: WebSocketSingletonConnectRequest,
    ) -> WebSocketConnectFuture {
        let Some(service) = self.self_reference.get().and_then(Weak::upgrade) else {
            return Box::pin(async {
                Err(WebSocketCommandError::not_sent(
                    WebSocketCommandErrorKind::Internal,
                ))
            });
        };
        Box::pin(async move { service.connect_singleton_outbound(request).await })
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
            service
                .abort_singleton_outbound(
                    &(project_id, singleton_id, claim_token),
                    &connection_id,
                    deadline,
                )
                .await
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
                quic.send(QuicSendRequest {
                    endpoint: owner.endpoint.clone(),
                    caller_project_id,
                    connection_id: connection_id.clone(),
                    target_worker_id: owner.worker_id.clone(),
                    message_kind,
                    body,
                    remaining: transport_remaining,
                }),
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

    fn send_singleton(
        &self,
        caller_project_id: String,
        singleton_id: String,
        message_kind: WebSocketMessageKind,
        body: Body,
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
            service
                .send_to_singleton(
                    caller_project_id,
                    singleton_id,
                    message_kind,
                    body,
                    remaining,
                )
                .await
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

async fn read_loop(options: ReadLoopOptions) {
    let ReadLoopOptions {
        service,
        project_id,
        connection_id,
        route_uri,
        mut reader,
        control_sender,
        disconnect_info,
        message_ready,
        lease_guard,
    } = options;
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
            .is_none_or(|entry| !entry.accepts_application_traffic())
        {
            if lease_guard
                .as_ref()
                .is_some_and(|lease_guard| !lease_guard.permits_traffic())
            {
                fence_singleton(&service, &project_id, &connection_id);
            }
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
                    if let Err(close_code) = dispatch_inbound(DispatchInboundOptions {
                        service: &service,
                        project_id: &project_id,
                        connection_id: &connection_id,
                        route_uri: &route_uri,
                        message_kind,
                        message_bytes: std::mem::take(&mut message_bytes),
                        pending_messages: &pending_messages,
                        control_sender: &control_sender,
                        lease_guard: lease_guard.as_ref(),
                    }) {
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
                    if let Err(close_code) = dispatch_inbound(DispatchInboundOptions {
                        service: &service,
                        project_id: &project_id,
                        connection_id: &connection_id,
                        route_uri: &route_uri,
                        message_kind,
                        message_bytes,
                        pending_messages: &pending_messages,
                        control_sender: &control_sender,
                        lease_guard: lease_guard.as_ref(),
                    }) {
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

fn dispatch_inbound(options: DispatchInboundOptions<'_>) -> Result<(), u16> {
    let DispatchInboundOptions {
        service,
        project_id,
        connection_id,
        route_uri,
        message_kind,
        message_bytes,
        pending_messages,
        control_sender,
        lease_guard,
    } = options;
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
    if lease_guard.is_some_and(|lease_guard| !lease_guard.permits_traffic()) {
        pending_messages.fetch_sub(1, Ordering::AcqRel);
        return Err(1011);
    }
    let (response_sender, response_receiver) = oneshot::channel();
    let envelope = RequestEnvelope::new(project_id.to_string(), request, response_sender);
    let envelope = match lease_guard {
        Some(lease_guard) => envelope.with_start_gate(lease_guard.start_gate()),
        None => envelope,
    };
    let (envelope, started_receiver) = envelope.with_start_signal();
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

struct OutboundFrameAdmission {
    project_id: String,
    lease_guard: Option<Arc<SingletonLeaseGuard>>,
    egress_budget: Arc<dyn EgressBudget>,
}

impl OutboundFrameAdmission {
    fn lease_permits_traffic(&self) -> bool {
        self.lease_guard
            .as_ref()
            .is_none_or(|lease_guard| lease_guard.permits_traffic())
    }

    async fn admit_frame(
        &self,
        payload_bytes: usize,
        wrote_any_frame: bool,
    ) -> Result<(), WebSocketCommandError> {
        if !self.lease_permits_traffic() {
            return Err(delivery_error(
                WebSocketCommandErrorKind::ConnectionNotFound,
                wrote_any_frame,
            ));
        }
        match self
            .egress_budget
            .charge(&self.project_id, payload_bytes as u64)
            .await
        {
            Ok(()) => {}
            Err(EgressDenied::QuotaExhausted | EgressDenied::QuotaNotConfigured) => {
                return Err(delivery_error(
                    WebSocketCommandErrorKind::EgressQuotaExceeded,
                    wrote_any_frame,
                ));
            }
            Err(EgressDenied::BudgetUnavailable) => {
                return Err(delivery_error(
                    WebSocketCommandErrorKind::Transport,
                    wrote_any_frame,
                ));
            }
        }
        if !self.lease_permits_traffic() {
            return Err(delivery_error(
                WebSocketCommandErrorKind::ConnectionNotFound,
                wrote_any_frame,
            ));
        }
        Ok(())
    }
}

fn send_failure_close(error: &WebSocketCommandError) -> Option<(u16, DisconnectInfo)> {
    match error.kind {
        WebSocketCommandErrorKind::InvalidText
            if error.delivery == WebSocketDeliveryState::NotSent =>
        {
            None
        }
        WebSocketCommandErrorKind::InvalidText => {
            Some((1007, DisconnectInfo::protocol_error(1007)))
        }
        WebSocketCommandErrorKind::EgressQuotaExceeded => {
            Some((1008, DisconnectInfo::egress_quota_exceeded()))
        }
        WebSocketCommandErrorKind::ConnectionNotFound => {
            Some((1011, DisconnectInfo::heartbeat_timeout()))
        }
        _ => Some((1011, DisconnectInfo::protocol_error(1011))),
    }
}

async fn writer_loop<Writer>(
    writer: &mut WebSocketWrite<Writer>,
    mut command_receiver: mpsc::Receiver<SocketCommand>,
    mut control_receiver: mpsc::UnboundedReceiver<WriterControl>,
    disconnect_info: Arc<Mutex<Option<DisconnectInfo>>>,
    outbound_frame_admission: &OutboundFrameAdmission,
) where
    Writer: AsyncWrite + Unpin,
{
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
                        let write_deadline =
                            tokio::time::Instant::now() + CONTROL_FRAME_WRITE_DEADLINE;
                        if !write_frame_until(
                            writer,
                            Frame::pong(Payload::Bytes(payload.into())),
                            write_deadline,
                        )
                        .await
                        {
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
                            let close_deadline = tokio::time::Instant::now() + CLOSE_HANDSHAKE_DEADLINE;
                            let _ = write_frame_until(
                                writer,
                                Frame::close_raw(Payload::Bytes(payload.into())),
                                close_deadline,
                            )
                            .await;
                        }
                        finish_close_response(close_response.take());
                        return;
                    }
                    WriterControl::Close(code, info) => {
                        store_disconnect_info(&disconnect_info, info);
                        if close_sent {
                            continue;
                        }
                        let close_deadline = tokio::time::Instant::now() + CLOSE_HANDSHAKE_DEADLINE;
                        if !write_frame_until(
                            writer,
                            Frame::close(code, &[]),
                            close_deadline,
                        )
                        .await
                        {
                            finish_close_response(close_response.take());
                            return;
                        }
                        close_sent = true;
                        close_timeout.as_mut().reset(close_deadline);
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
                            let close_deadline = tokio::time::Instant::now() + CLOSE_HANDSHAKE_DEADLINE;
                            let _ = write_frame_until(
                                writer,
                                Frame::close(1013, &[]),
                                close_deadline,
                            )
                            .await;
                            return;
                        }
                        if !outbound_frame_admission.lease_permits_traffic() {
                            let _ = response_sender.send(Err(WebSocketCommandError::not_sent(
                                WebSocketCommandErrorKind::ConnectionNotFound,
                            )));
                            store_disconnect_info(&disconnect_info, DisconnectInfo::heartbeat_timeout());
                            let close_deadline = tokio::time::Instant::now() + CLOSE_HANDSHAKE_DEADLINE;
                            let _ = write_frame_until(
                                writer,
                                Frame::close(1011, &[]),
                                close_deadline,
                            )
                            .await;
                            return;
                        }
                        let _ = ready_sender.send(());
                        let wrote_frame = Arc::new(AtomicBool::new(false));
                        let result = tokio::time::timeout_at(
                            deadline,
                            send_message(
                                writer,
                                message_kind,
                                body,
                                wrote_frame.clone(),
                                outbound_frame_admission,
                            ),
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
                        let failure_close = result.as_ref().err().and_then(send_failure_close);
                        let _ = response_sender.send(result);
                        if let Some((close_code, close_info)) = failure_close {
                            store_disconnect_info(&disconnect_info, close_info);
                            let close_deadline = tokio::time::Instant::now() + CLOSE_HANDSHAKE_DEADLINE;
                            let _ = write_frame_until(
                                writer,
                                Frame::close(close_code, &[]),
                                close_deadline,
                            )
                            .await;
                            return;
                        }
                    }
                    SocketCommand::Close { code, info, response_sender } => {
                        store_disconnect_info(&disconnect_info, info);
                        close_response = response_sender;
                        let close_deadline = tokio::time::Instant::now() + CLOSE_HANDSHAKE_DEADLINE;
                        if !write_frame_until(
                            writer,
                            Frame::close(code, &[]),
                            close_deadline,
                        )
                        .await
                        {
                            finish_close_response(close_response.take());
                            return;
                        }
                        close_sent = true;
                        close_timeout.as_mut().reset(close_deadline);
                    }
                }
            }
            _ = ping_interval.tick(), if !close_sent && !awaiting_pong => {
                let payload = Bytes::copy_from_slice(&unix_millis().to_be_bytes());
                let write_deadline =
                    tokio::time::Instant::now() + CONTROL_FRAME_WRITE_DEADLINE;
                if !write_frame_until(
                    writer,
                    Frame::new(true, OpCode::Ping, None, Payload::Bytes(payload.into())),
                    write_deadline,
                )
                .await
                {
                    store_disconnect_info(&disconnect_info, DisconnectInfo::transport_error());
                    return;
                }
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

async fn write_frame_until<Writer>(
    writer: &mut WebSocketWrite<Writer>,
    frame: Frame<'static>,
    deadline: tokio::time::Instant,
) -> bool
where
    Writer: AsyncWrite + Unpin,
{
    tokio::time::timeout_at(deadline, async {
        writer.write_frame(frame).await?;
        writer.flush().await
    })
    .await
    .is_ok_and(|result| result.is_ok())
}

async fn send_message<Writer>(
    writer: &mut WebSocketWrite<Writer>,
    message_kind: WebSocketMessageKind,
    mut body: Body,
    wrote_frame: Arc<AtomicBool>,
    outbound_frame_admission: &OutboundFrameAdmission,
) -> Result<(), WebSocketCommandError>
where
    Writer: AsyncWrite + Unpin,
{
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
        outbound_frame_admission
            .admit_frame(data.len(), wrote_any_frame)
            .await?;
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
    outbound_frame_admission
        .admit_frame(0, wrote_any_frame)
        .await?;
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
    use fn0::{EgressChargeFuture, PrivateDestinationAccess, WebSocketCommandDispatcher};
    use std::collections::VecDeque;
    use std::convert::Infallible;
    use std::net::{SocketAddr, UdpSocket};
    use std::task::{Context, Poll};
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
    fn singleton_status_response_distinguishes_rejection_from_retryable_failure() {
        assert_eq!(
            classify_singleton_status_response(b"\"Ok\""),
            SingletonStatusResponse::Accepted
        );
        assert_eq!(
            classify_singleton_status_response(b"\"Ignored\""),
            SingletonStatusResponse::Rejected
        );
        assert_eq!(
            classify_singleton_status_response(b"\"Unauthorized\""),
            SingletonStatusResponse::Rejected
        );
        assert_eq!(
            classify_singleton_status_response(b"\"Error\""),
            SingletonStatusResponse::Retryable
        );
    }

    fn test_lease_timing() -> SingletonLeaseTiming {
        SINGLETON_LEASE_TIMING
    }

    fn fixed_retry_delay(_backoff: Duration) -> Duration {
        Duration::from_secs(1)
    }

    #[tokio::test(start_paused = true)]
    async fn partition_recovered_before_safety_deadline_keeps_connection() {
        let started_at = tokio::time::Instant::now();
        let lease_guard = Arc::new(SingletonLeaseGuard::new(
            started_at + SINGLETON_SAFETY_DEADLINE,
        ));
        let attempts = Arc::new(Mutex::new(Vec::new()));
        let attempts_for_renewal = attempts.clone();
        let guard_for_renewal = lease_guard.clone();
        let accepted = Arc::new(AtomicBool::new(false));
        let fence_reason = maintain_singleton_lease(
            test_lease_timing(),
            &lease_guard,
            move || {
                let elapsed = started_at.elapsed();
                attempts_for_renewal
                    .lock()
                    .unwrap()
                    .push((elapsed, guard_for_renewal.permits_traffic()));
                let response = if elapsed < Duration::from_secs(25) {
                    SingletonStatusResponse::Retryable
                } else if !accepted.swap(true, Ordering::AcqRel) {
                    SingletonStatusResponse::Accepted
                } else {
                    SingletonStatusResponse::Rejected
                };
                async move { Ok(response) }
            },
            fixed_retry_delay,
        )
        .await;
        assert_eq!(fence_reason, SingletonFenceReason::OwnershipRejected);
        let attempts = attempts.lock().unwrap();
        assert!(attempts.iter().all(|(_, permits_traffic)| *permits_traffic));
        let (rejected_at, _) = *attempts.last().unwrap();
        assert!(rejected_at > SINGLETON_SAFETY_DEADLINE);
        assert!(!lease_guard.permits_traffic());
    }

    #[tokio::test(start_paused = true)]
    async fn partition_longer_than_safety_deadline_fences_exactly_at_deadline() {
        let started_at = tokio::time::Instant::now();
        let lease_guard = SingletonLeaseGuard::new(started_at + SINGLETON_SAFETY_DEADLINE);
        let fence_reason = maintain_singleton_lease(
            test_lease_timing(),
            &lease_guard,
            || async { Err(anyhow::anyhow!("control unreachable")) },
            fixed_retry_delay,
        )
        .await;
        assert_eq!(fence_reason, SingletonFenceReason::SafetyDeadlineExpired);
        assert_eq!(started_at.elapsed(), SINGLETON_SAFETY_DEADLINE);
        assert!(!lease_guard.permits_traffic());
    }

    #[tokio::test(start_paused = true)]
    async fn lost_heartbeat_response_fences_from_last_confirmed_request() {
        let started_at = tokio::time::Instant::now();
        let lease_guard = SingletonLeaseGuard::new(started_at + SINGLETON_SAFETY_DEADLINE);
        let attempt_count = Arc::new(AtomicUsize::new(0));
        let attempt_count_for_renewal = attempt_count.clone();
        let fence_reason = maintain_singleton_lease(
            test_lease_timing(),
            &lease_guard,
            move || {
                let attempt_number = attempt_count_for_renewal.fetch_add(1, Ordering::AcqRel);
                async move {
                    if attempt_number == 0 {
                        tokio::time::sleep(Duration::from_secs(4)).await;
                        Ok(SingletonStatusResponse::Accepted)
                    } else {
                        std::future::pending().await
                    }
                }
            },
            fixed_retry_delay,
        )
        .await;
        assert_eq!(fence_reason, SingletonFenceReason::SafetyDeadlineExpired);
        assert_eq!(
            started_at.elapsed(),
            SINGLETON_HEARTBEAT_INTERVAL + SINGLETON_SAFETY_DEADLINE
        );
    }

    #[tokio::test(start_paused = true)]
    async fn stale_claim_rejection_fences_without_waiting_for_deadline() {
        let started_at = tokio::time::Instant::now();
        let lease_guard = SingletonLeaseGuard::new(started_at + SINGLETON_SAFETY_DEADLINE);
        let fence_reason = maintain_singleton_lease(
            test_lease_timing(),
            &lease_guard,
            || async { Ok(SingletonStatusResponse::Rejected) },
            fixed_retry_delay,
        )
        .await;
        assert_eq!(fence_reason, SingletonFenceReason::OwnershipRejected);
        assert_eq!(started_at.elapsed(), SINGLETON_HEARTBEAT_INTERVAL);
        assert!(tokio::time::Instant::now() < lease_guard.valid_until());
        assert!(!lease_guard.permits_traffic());
    }

    #[test]
    fn lease_schedule_never_moves_deadline_backwards() {
        let now = tokio::time::Instant::now();
        let mut schedule =
            SingletonLeaseSchedule::new(test_lease_timing(), now + Duration::from_secs(30), now);
        schedule
            .record_renewal(
                Some(SingletonStatusResponse::Accepted),
                now - Duration::from_secs(5),
                now,
                Duration::from_secs(1),
            )
            .unwrap();
        assert_eq!(schedule.safety_deadline, now + Duration::from_secs(30));
        assert_eq!(
            schedule.next_step(now + Duration::from_secs(30)),
            SingletonLeaseStep::Fence(SingletonFenceReason::SafetyDeadlineExpired)
        );
    }

    struct UnlimitedEgressBudget;

    impl EgressBudget for UnlimitedEgressBudget {
        fn charge(&self, _project_id: &str, _byte_count: u64) -> EgressChargeFuture {
            Box::pin(async { Ok(()) })
        }

        fn known_exhausted(&self, _project_id: &str) -> bool {
            false
        }
    }

    struct LimitedEgressBudget {
        remaining_bytes: Mutex<u64>,
        exhausted: bool,
    }

    impl EgressBudget for LimitedEgressBudget {
        fn charge(&self, _project_id: &str, byte_count: u64) -> EgressChargeFuture {
            let mut remaining_bytes = self.remaining_bytes.lock().unwrap();
            let result = if *remaining_bytes >= byte_count {
                *remaining_bytes -= byte_count;
                Ok(())
            } else {
                Err(EgressDenied::QuotaExhausted)
            };
            Box::pin(async move { result })
        }

        fn known_exhausted(&self, _project_id: &str) -> bool {
            self.exhausted
        }
    }

    struct ScriptedSingletonResolver {
        answers: Mutex<VecDeque<Option<String>>>,
        lookups: Mutex<Vec<(String, String)>>,
    }

    impl ScriptedSingletonResolver {
        fn new(answers: Vec<Option<String>>) -> Arc<Self> {
            Arc::new(Self {
                answers: Mutex::new(answers.into()),
                lookups: Mutex::new(Vec::new()),
            })
        }
    }

    impl SingletonConnectionResolver for ScriptedSingletonResolver {
        fn resolve(
            &self,
            project_id: &str,
            singleton_id: &str,
        ) -> SingletonConnectionResolveFuture {
            self.lookups
                .lock()
                .unwrap()
                .push((project_id.to_string(), singleton_id.to_string()));
            let answer = self.answers.lock().unwrap().pop_front().flatten();
            let resolved_at_millis = unix_millis() as i64;
            let answer = answer.map(|connection_id| SingletonConnectionResolution {
                connection_id,
                lease_expires_at_millis: resolved_at_millis + 60_000,
                resolved_at_millis,
            });
            Box::pin(async move { Ok(answer) })
        }
    }

    struct BlockingSingletonResolver {
        started_count: Arc<AtomicUsize>,
        release: Arc<tokio::sync::Notify>,
    }

    impl BlockingSingletonResolver {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                started_count: Arc::new(AtomicUsize::new(0)),
                release: Arc::new(tokio::sync::Notify::new()),
            })
        }
    }

    impl SingletonConnectionResolver for BlockingSingletonResolver {
        fn resolve(
            &self,
            _project_id: &str,
            _singleton_id: &str,
        ) -> SingletonConnectionResolveFuture {
            let started_count = self.started_count.clone();
            let release = self.release.clone();
            Box::pin(async move {
                let release_wait = release.notified();
                started_count.fetch_add(1, Ordering::AcqRel);
                release_wait.await;
                let resolved_at_millis = unix_millis() as i64;
                Ok(Some(SingletonConnectionResolution {
                    connection_id: "connection".to_string(),
                    lease_expires_at_millis: resolved_at_millis + 60_000,
                    resolved_at_millis,
                }))
            })
        }
    }

    struct FailingSingletonResolver {
        lookup_count: Arc<AtomicUsize>,
    }

    impl FailingSingletonResolver {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                lookup_count: Arc::new(AtomicUsize::new(0)),
            })
        }
    }

    impl SingletonConnectionResolver for FailingSingletonResolver {
        fn resolve(
            &self,
            _project_id: &str,
            _singleton_id: &str,
        ) -> SingletonConnectionResolveFuture {
            self.lookup_count.fetch_add(1, Ordering::AcqRel);
            Box::pin(async { Err(anyhow::anyhow!("resolution response lost")) })
        }
    }

    fn test_service(
        worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
        directory: Arc<MemoryDirectory>,
        identity: WorkerIdentity,
    ) -> Arc<WebSocketService> {
        test_service_with(
            worker_senders,
            directory,
            identity,
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            Arc::new(UnlimitedEgressBudget),
            ScriptedSingletonResolver::new(Vec::new()),
        )
    }

    fn test_service_with(
        worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
        directory: Arc<MemoryDirectory>,
        identity: WorkerIdentity,
        outbound_dialer: OutboundDialer,
        egress_budget: Arc<dyn EgressBudget>,
        singleton_resolver: Arc<dyn SingletonConnectionResolver>,
    ) -> Arc<WebSocketService> {
        let service = Arc::new(WebSocketService {
            worker_senders,
            connections: DashMap::new(),
            singleton_connections: DashMap::new(),
            singleton_resolve_cache: DashMap::new(),
            project_counts: DashMap::new(),
            project_generations: DashMap::new(),
            worker_count: Arc::new(AtomicUsize::new(0)),
            draining: AtomicBool::new(false),
            directory,
            identity,
            quic: OnceLock::new(),
            self_reference: OnceLock::new(),
            outbound_dialer,
            egress_budget,
            singleton_resolver,
        });
        service
            .self_reference
            .set(Arc::downgrade(&service))
            .expect("set test websocket service self reference");
        service
    }

    fn local_worker_identity() -> WorkerIdentity {
        WorkerIdentity {
            worker_id: "worker".to_string(),
            endpoint: String::new(),
        }
    }

    struct TestConnection {
        command_receiver: mpsc::Receiver<SocketCommand>,
        control_receiver: mpsc::UnboundedReceiver<WriterControl>,
        _closed_sender: watch::Sender<bool>,
    }

    fn insert_test_connection(
        service: &WebSocketService,
        project_id: &str,
        connection_id: &str,
        lease_guard: Option<Arc<SingletonLeaseGuard>>,
    ) -> TestConnection {
        let (command_sender, command_receiver) = mpsc::channel(OUTBOUND_COMMAND_CAPACITY);
        let (closed_sender, closed_receiver) = watch::channel(false);
        let (force_close_sender, _force_close_receiver) = watch::channel(false);
        let (control_sender, control_receiver) = mpsc::unbounded_channel();
        service.connections.insert(
            connection_id.to_string(),
            Arc::new(ConnectionEntry {
                project_id: project_id.to_string(),
                command_sender,
                closing: AtomicBool::new(false),
                closed_receiver,
                control_sender,
                force_close_sender,
                lease_guard,
            }),
        );
        TestConnection {
            command_receiver,
            control_receiver,
            _closed_sender: closed_sender,
        }
    }

    fn text_body(text: &'static str) -> Body {
        Full::new(Bytes::from_static(text.as_bytes()))
            .map_err(|never: Infallible| match never {})
            .boxed_unsync()
    }

    fn connection_id_with(fill_byte: u8) -> String {
        format!(
            "v1.{}",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([fill_byte; 32])
        )
    }

    #[tokio::test(start_paused = true)]
    async fn resumed_process_refuses_send_before_lease_task_runs() {
        let service = test_service(
            Arc::new(Vec::new()),
            Arc::new(MemoryDirectory::default()),
            local_worker_identity(),
        );
        let lease_guard = Arc::new(SingletonLeaseGuard::new(
            tokio::time::Instant::now() + SINGLETON_SAFETY_DEADLINE,
        ));
        let mut connection =
            insert_test_connection(&service, "project", "connection", Some(lease_guard));
        tokio::time::advance(SINGLETON_SAFETY_DEADLINE + Duration::from_secs(1)).await;
        let admitted = service.admit_local_send(
            "project",
            "connection",
            WebSocketMessageKind::Text,
            text_body("late"),
            tokio::time::Instant::now() + Duration::from_secs(5),
        );
        let Err(error) = admitted else {
            panic!("expired lease admitted a send");
        };
        assert_eq!(
            error,
            WebSocketCommandError::not_sent(WebSocketCommandErrorKind::ConnectionNotFound)
        );
        assert!(connection.command_receiver.try_recv().is_err());
        let Ok(WriterControl::Close(close_code, close_info)) =
            connection.control_receiver.try_recv()
        else {
            panic!("expired lease did not close the connection");
        };
        assert_eq!(close_code, 1011);
        assert_eq!(close_info.cause, "heartbeat-timeout");
    }

    #[tokio::test(start_paused = true)]
    async fn expired_lease_refuses_inbound_callback_and_worker_start() {
        let service = test_service(
            Arc::new(Vec::new()),
            Arc::new(MemoryDirectory::default()),
            local_worker_identity(),
        );
        let lease_guard = Arc::new(SingletonLeaseGuard::new(
            tokio::time::Instant::now() + SINGLETON_SAFETY_DEADLINE,
        ));
        let start_gate = lease_guard.start_gate();
        assert!(start_gate());
        tokio::time::advance(SINGLETON_SAFETY_DEADLINE).await;
        assert!(!start_gate());
        let (control_sender, _control_receiver) = mpsc::unbounded_channel();
        let pending_messages = Arc::new(AtomicUsize::new(0));
        let route_uri = "https://fn0-websocket.internal/ws_singleton/feed"
            .parse()
            .unwrap();
        let dispatch_result = dispatch_inbound(DispatchInboundOptions {
            service: &service,
            project_id: "project",
            connection_id: "connection",
            route_uri: &route_uri,
            message_kind: WebSocketMessageKind::Text,
            message_bytes: b"late".to_vec(),
            pending_messages: &pending_messages,
            control_sender: &control_sender,
            lease_guard: Some(&lease_guard),
        });
        assert_eq!(dispatch_result, Err(1011));
        assert_eq!(pending_messages.load(Ordering::Acquire), 0);
    }

    async fn channel_body(chunk_receiver: mpsc::Receiver<Bytes>) -> Body {
        let stream = futures::stream::unfold(chunk_receiver, |mut chunk_receiver| async move {
            chunk_receiver.recv().await.map(|chunk| {
                (
                    Ok::<_, anyhow::Error>(http_body::Frame::data(chunk)),
                    chunk_receiver,
                )
            })
        });
        http_body_util::StreamBody::new(stream).boxed_unsync()
    }

    async fn read_written_bytes(reader: &mut tokio::io::DuplexStream) -> Vec<u8> {
        let mut written = Vec::new();
        let mut buffer = [0_u8; 1024];
        while let Ok(Ok(read_count)) =
            timeout(Duration::from_millis(1), reader.read(&mut buffer)).await
        {
            if read_count == 0 {
                break;
            }
            written.extend_from_slice(&buffer[..read_count]);
        }
        written
    }

    #[tokio::test(start_paused = true)]
    async fn streaming_send_stops_before_next_frame_after_lease_expiry() {
        let (mut peer, local) = tokio::io::duplex(64 * 1024);
        let (local_reader, local_writer) = tokio::io::split(local);
        let (_, mut writer) = fastwebsockets::after_handshake_split(
            local_reader,
            local_writer,
            fastwebsockets::Role::Server,
        );
        let lease_guard = Arc::new(SingletonLeaseGuard::new(
            tokio::time::Instant::now() + SINGLETON_SAFETY_DEADLINE,
        ));
        let admission = OutboundFrameAdmission {
            project_id: "project".to_string(),
            lease_guard: Some(lease_guard),
            egress_budget: Arc::new(UnlimitedEgressBudget),
        };
        let (chunk_sender, chunk_receiver) = mpsc::channel(1);
        let wrote_frame = Arc::new(AtomicBool::new(false));
        chunk_sender
            .send(Bytes::from_static(b"first-chunk"))
            .await
            .unwrap();
        let body = channel_body(chunk_receiver).await;
        let send = send_message(
            &mut writer,
            WebSocketMessageKind::Binary,
            body,
            wrote_frame.clone(),
            &admission,
        );
        tokio::pin!(send);
        assert!(
            timeout(Duration::from_millis(1), send.as_mut())
                .await
                .is_err()
        );
        assert!(wrote_frame.load(Ordering::Acquire));
        tokio::time::advance(SINGLETON_SAFETY_DEADLINE).await;
        chunk_sender
            .send(Bytes::from_static(b"second-chunk"))
            .await
            .unwrap();
        let error = send.await.unwrap_err();
        assert_eq!(
            error,
            WebSocketCommandError::unknown(WebSocketCommandErrorKind::ConnectionNotFound)
        );
        let written = read_written_bytes(&mut peer).await;
        assert!(written.windows(11).any(|window| window == b"first-chunk"));
        assert!(!written.windows(12).any(|window| window == b"second-chunk"));
        assert_eq!(
            send_failure_close(&error)
                .map(|(close_code, close_info)| (close_code, close_info.cause)),
            Some((1011, "heartbeat-timeout"))
        );
    }

    #[tokio::test]
    async fn egress_refusal_mid_stream_stops_the_message_and_closes_with_policy_violation() {
        let (mut peer, local) = tokio::io::duplex(64 * 1024);
        let (local_reader, local_writer) = tokio::io::split(local);
        let (_, mut writer) = fastwebsockets::after_handshake_split(
            local_reader,
            local_writer,
            fastwebsockets::Role::Server,
        );
        let admission = OutboundFrameAdmission {
            project_id: "project".to_string(),
            lease_guard: None,
            egress_budget: Arc::new(LimitedEgressBudget {
                remaining_bytes: Mutex::new(11),
                exhausted: false,
            }),
        };
        let (chunk_sender, chunk_receiver) = mpsc::channel(2);
        chunk_sender
            .send(Bytes::from_static(b"first-chunk"))
            .await
            .unwrap();
        chunk_sender
            .send(Bytes::from_static(b"second-chunk"))
            .await
            .unwrap();
        drop(chunk_sender);
        let error = send_message(
            &mut writer,
            WebSocketMessageKind::Binary,
            channel_body(chunk_receiver).await,
            Arc::new(AtomicBool::new(false)),
            &admission,
        )
        .await
        .unwrap_err();
        assert_eq!(
            error,
            WebSocketCommandError::unknown(WebSocketCommandErrorKind::EgressQuotaExceeded)
        );
        drop(writer);
        let written = read_written_bytes(&mut peer).await;
        assert!(!written.windows(12).any(|window| window == b"second-chunk"));
        assert_eq!(
            send_failure_close(&error)
                .map(|(close_code, close_info)| (close_code, close_info.cause)),
            Some((1008, "egress-quota-exceeded"))
        );
    }

    #[tokio::test]
    async fn egress_refusal_before_first_frame_is_not_sent() {
        let (_peer, local) = tokio::io::duplex(1024);
        let (local_reader, local_writer) = tokio::io::split(local);
        let (_, mut writer) = fastwebsockets::after_handshake_split(
            local_reader,
            local_writer,
            fastwebsockets::Role::Server,
        );
        let admission = OutboundFrameAdmission {
            project_id: "project".to_string(),
            lease_guard: None,
            egress_budget: Arc::new(LimitedEgressBudget {
                remaining_bytes: Mutex::new(0),
                exhausted: false,
            }),
        };
        let wrote_frame = Arc::new(AtomicBool::new(false));
        let error = send_message(
            &mut writer,
            WebSocketMessageKind::Text,
            text_body("hello"),
            wrote_frame.clone(),
            &admission,
        )
        .await
        .unwrap_err();
        assert_eq!(
            error,
            WebSocketCommandError::not_sent(WebSocketCommandErrorKind::EgressQuotaExceeded)
        );
        assert!(!wrote_frame.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn known_exhausted_project_send_is_refused_and_connection_closed() {
        let service = test_service_with(
            Arc::new(Vec::new()),
            Arc::new(MemoryDirectory::default()),
            local_worker_identity(),
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            Arc::new(LimitedEgressBudget {
                remaining_bytes: Mutex::new(0),
                exhausted: true,
            }),
            ScriptedSingletonResolver::new(Vec::new()),
        );
        let mut connection = insert_test_connection(&service, "project", "connection", None);
        let Err(error) = service.admit_local_send(
            "project",
            "connection",
            WebSocketMessageKind::Text,
            text_body("hello"),
            tokio::time::Instant::now() + Duration::from_secs(5),
        ) else {
            panic!("exhausted project admitted a send");
        };
        assert_eq!(
            error,
            WebSocketCommandError::not_sent(WebSocketCommandErrorKind::EgressQuotaExceeded)
        );
        assert!(connection.command_receiver.try_recv().is_err());
        let Ok(WriterControl::Close(close_code, _)) = connection.control_receiver.try_recv() else {
            panic!("exhausted project connection was not closed");
        };
        assert_eq!(close_code, 1008);
    }

    #[tokio::test]
    async fn named_send_reaches_the_resolved_current_connection() {
        let current_connection_id = connection_id_with(1);
        let resolver = ScriptedSingletonResolver::new(vec![Some(current_connection_id.clone())]);
        let service = test_service_with(
            Arc::new(Vec::new()),
            Arc::new(MemoryDirectory::default()),
            local_worker_identity(),
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            Arc::new(UnlimitedEgressBudget),
            resolver.clone(),
        );
        let mut connection =
            insert_test_connection(&service, "project", &current_connection_id, None);
        let send_task = tokio::spawn({
            let service = service.clone();
            async move {
                service
                    .send_singleton(
                        "project".to_string(),
                        "market_feed".to_string(),
                        WebSocketMessageKind::Text,
                        text_body("subscribe"),
                        Duration::from_secs(5),
                    )
                    .await
            }
        });
        let Some(SocketCommand::Send {
            body,
            response_sender,
            ..
        }) = connection.command_receiver.recv().await
        else {
            panic!("named send did not reach the connection");
        };
        assert_eq!(
            body.collect().await.unwrap().to_bytes(),
            Bytes::from_static(b"subscribe")
        );
        response_sender.send(Ok(())).unwrap();
        send_task.await.unwrap().unwrap();
        assert_eq!(
            resolver.lookups.lock().unwrap().as_slice(),
            [("project".to_string(), "market_feed".to_string())]
        );
    }

    #[tokio::test]
    async fn concurrent_cache_misses_share_one_resolution_after_one_waiter_is_cancelled() {
        let resolver = BlockingSingletonResolver::new();
        let service = test_service_with(
            Arc::new(Vec::new()),
            Arc::new(MemoryDirectory::default()),
            local_worker_identity(),
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            Arc::new(UnlimitedEgressBudget),
            resolver.clone(),
        );
        let first_service = service.clone();
        let first = tokio::spawn(async move {
            first_service
                .resolve_singleton_cached(
                    "project",
                    "feed",
                    tokio::time::Instant::now() + Duration::from_secs(5),
                )
                .await
        });
        while resolver.started_count.load(Ordering::Acquire) == 0 {
            tokio::task::yield_now().await;
        }
        let second_service = service.clone();
        let second = tokio::spawn(async move {
            second_service
                .resolve_singleton_cached(
                    "project",
                    "feed",
                    tokio::time::Instant::now() + Duration::from_secs(5),
                )
                .await
        });
        tokio::task::yield_now().await;
        assert_eq!(resolver.started_count.load(Ordering::Acquire), 1);
        first.abort();
        resolver.release.notify_waiters();
        let (_, cached_connection) = second.await.unwrap().unwrap();
        assert_eq!(cached_connection.connection_id, "connection");
        assert_eq!(resolver.started_count.load(Ordering::Acquire), 1);
    }

    #[test]
    fn singleton_cache_expires_at_request_start_minus_safety_margin() {
        let request_started_at = tokio::time::Instant::now();
        let valid_until = singleton_cache_valid_until(request_started_at, 61_000, 1_000);
        assert_eq!(
            valid_until.duration_since(request_started_at),
            Duration::from_secs(30)
        );
    }

    #[tokio::test(start_paused = true)]
    async fn cache_expiry_triggers_a_new_control_lookup() {
        let resolver = ScriptedSingletonResolver::new(vec![
            Some("first-connection".to_string()),
            Some("second-connection".to_string()),
        ]);
        let service = test_service_with(
            Arc::new(Vec::new()),
            Arc::new(MemoryDirectory::default()),
            local_worker_identity(),
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            Arc::new(UnlimitedEgressBudget),
            resolver.clone(),
        );
        let (_, first_connection) = service
            .resolve_singleton_cached(
                "project",
                "feed",
                tokio::time::Instant::now() + Duration::from_secs(5),
            )
            .await
            .unwrap();
        assert_eq!(first_connection.connection_id, "first-connection");
        tokio::time::advance(Duration::from_secs(30)).await;
        let (_, second_connection) = service
            .resolve_singleton_cached(
                "project",
                "feed",
                tokio::time::Instant::now() + Duration::from_secs(5),
            )
            .await
            .unwrap();
        assert_eq!(second_connection.connection_id, "second-connection");
        assert_eq!(resolver.lookups.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn failed_resolution_is_not_cached() {
        let resolver = FailingSingletonResolver::new();
        let service = test_service_with(
            Arc::new(Vec::new()),
            Arc::new(MemoryDirectory::default()),
            local_worker_identity(),
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            Arc::new(UnlimitedEgressBudget),
            resolver.clone(),
        );
        for _ in 0..2 {
            let result = service
                .resolve_singleton_cached(
                    "project",
                    "feed",
                    tokio::time::Instant::now() + Duration::from_secs(5),
                )
                .await;
            let Err(error) = result else {
                panic!("failed singleton resolution unexpectedly succeeded");
            };
            assert_eq!(error.kind, WebSocketCommandErrorKind::Transport);
        }
        assert_eq!(resolver.lookup_count.load(Ordering::Acquire), 2);
    }

    #[tokio::test]
    async fn named_send_to_replaced_owner_fails_without_retrying_on_replacement() {
        let stale_connection_id = connection_id_with(2);
        let replacement_connection_id = connection_id_with(3);
        let resolver = ScriptedSingletonResolver::new(vec![
            Some(stale_connection_id),
            Some(replacement_connection_id.clone()),
        ]);
        let service = test_service_with(
            Arc::new(Vec::new()),
            Arc::new(MemoryDirectory::default()),
            local_worker_identity(),
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            Arc::new(UnlimitedEgressBudget),
            resolver.clone(),
        );
        let mut replacement =
            insert_test_connection(&service, "project", &replacement_connection_id, None);
        let error = service
            .send_singleton(
                "project".to_string(),
                "market_feed".to_string(),
                WebSocketMessageKind::Text,
                text_body("order"),
                Duration::from_secs(5),
            )
            .await
            .unwrap_err();
        assert_eq!(
            error,
            WebSocketCommandError::not_sent(WebSocketCommandErrorKind::ConnectionNotFound)
        );
        assert_eq!(resolver.lookups.lock().unwrap().len(), 1);
        assert!(replacement.command_receiver.try_recv().is_err());
        let second_send = tokio::spawn({
            let service = service.clone();
            async move {
                service
                    .send_singleton(
                        "project".to_string(),
                        "market_feed".to_string(),
                        WebSocketMessageKind::Text,
                        text_body("replacement-message"),
                        Duration::from_secs(5),
                    )
                    .await
            }
        });
        let Some(SocketCommand::Send {
            response_sender, ..
        }) = replacement.command_receiver.recv().await
        else {
            panic!("replacement send did not reach the replacement connection");
        };
        response_sender.send(Ok(())).unwrap();
        second_send.await.unwrap().unwrap();
        assert_eq!(resolver.lookups.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn named_send_resolves_in_the_calling_project_only() {
        let other_project_connection_id = connection_id_with(4);
        let resolver =
            ScriptedSingletonResolver::new(vec![Some(other_project_connection_id.clone())]);
        let service = test_service_with(
            Arc::new(Vec::new()),
            Arc::new(MemoryDirectory::default()),
            local_worker_identity(),
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            Arc::new(UnlimitedEgressBudget),
            resolver.clone(),
        );
        let mut other_project_connection = insert_test_connection(
            &service,
            "other-project",
            &other_project_connection_id,
            None,
        );
        let error = service
            .send_singleton(
                "calling-project".to_string(),
                "market_feed".to_string(),
                WebSocketMessageKind::Text,
                text_body("order"),
                Duration::from_secs(5),
            )
            .await
            .unwrap_err();
        assert_eq!(error.kind, WebSocketCommandErrorKind::ConnectionNotFound);
        assert_eq!(
            resolver.lookups.lock().unwrap()[0].0,
            "calling-project".to_string()
        );
        assert!(
            other_project_connection
                .command_receiver
                .try_recv()
                .is_err()
        );
    }

    #[tokio::test]
    async fn named_send_without_current_connection_is_not_sent() {
        let service = test_service_with(
            Arc::new(Vec::new()),
            Arc::new(MemoryDirectory::default()),
            local_worker_identity(),
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            Arc::new(UnlimitedEgressBudget),
            ScriptedSingletonResolver::new(vec![None]),
        );
        let error = service
            .send_singleton(
                "project".to_string(),
                "market_feed".to_string(),
                WebSocketMessageKind::Text,
                text_body("order"),
                Duration::from_secs(5),
            )
            .await
            .unwrap_err();
        assert_eq!(
            error,
            WebSocketCommandError::not_sent(WebSocketCommandErrorKind::ConnectionNotFound)
        );
    }

    #[tokio::test]
    async fn blocked_destination_policy_refuses_private_websocket_before_dialing() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let listener_address = listener.local_addr().unwrap();
        let service = test_service_with(
            Arc::new(Vec::new()),
            Arc::new(MemoryDirectory::default()),
            local_worker_identity(),
            OutboundDialer::system(PrivateDestinationAccess::Blocked),
            Arc::new(UnlimitedEgressBudget),
            ScriptedSingletonResolver::new(Vec::new()),
        );
        let error = service
            .connect(
                "project".to_string(),
                format!("ws://{listener_address}/socket"),
                "/ws_out/feed".to_string(),
                Duration::from_secs(5),
            )
            .await
            .unwrap_err();
        assert_eq!(
            error,
            WebSocketCommandError::not_sent(WebSocketCommandErrorKind::DestinationForbidden)
        );
        assert!(
            timeout(Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
        assert_eq!(service.connection_count(), 0);
    }

    struct PendingWriter;

    impl AsyncWrite for PendingWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
            _buffer: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            Poll::Pending
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Pending
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Pending
        }
    }

    #[tokio::test]
    async fn close_frame_write_is_bounded_by_close_deadline() {
        let (_, mut writer) = fastwebsockets::after_handshake_split(
            tokio::io::empty(),
            PendingWriter,
            fastwebsockets::Role::Server,
        );
        let result = write_frame_until(
            &mut writer,
            Frame::close(1011, &[]),
            tokio::time::Instant::now() + Duration::from_millis(20),
        )
        .await;
        assert!(!result);
    }

    #[tokio::test]
    async fn pong_frame_write_is_bounded_by_control_deadline() {
        let (_, mut writer) = fastwebsockets::after_handshake_split(
            tokio::io::empty(),
            PendingWriter,
            fastwebsockets::Role::Server,
        );
        let result = write_frame_until(
            &mut writer,
            Frame::pong(Payload::Borrowed(b"ping")),
            tokio::time::Instant::now() + Duration::from_millis(20),
        )
        .await;
        assert!(!result);
    }

    #[tokio::test]
    async fn force_close_signal_is_delivered_after_deadline() {
        let (force_close_sender, mut force_close_receiver) = watch::channel(false);
        schedule_force_close(force_close_sender, Duration::from_millis(10));
        timeout(
            Duration::from_millis(100),
            wait_for_force_close(&mut force_close_receiver),
        )
        .await
        .unwrap();
        assert!(*force_close_receiver.borrow());
    }

    #[test]
    fn singleton_abort_fences_callback_start_and_active_commit() {
        let activation_before_callback = SingletonActivationLifecycle::new();
        assert!(activation_before_callback.begin());
        assert!(activation_before_callback.request_abort());
        assert!(!activation_before_callback.begin_callback());
        assert!(!activation_before_callback.finish(false));

        let activation_during_callback = SingletonActivationLifecycle::new();
        assert!(activation_during_callback.begin());
        assert!(activation_during_callback.begin_callback());
        assert!(activation_during_callback.request_abort());
        assert!(!activation_during_callback.commit_active());
        assert!(!activation_during_callback.finish(false));
    }

    #[tokio::test]
    async fn singleton_abort_waits_for_connection_and_activation_completion() {
        let directory = Arc::new(MemoryDirectory::default());
        let service = test_service(
            Arc::new(Vec::new()),
            directory,
            WorkerIdentity {
                worker_id: "worker".to_string(),
                endpoint: String::new(),
            },
        );
        let singleton_key = (
            "project".to_string(),
            "feed".to_string(),
            "claim".to_string(),
        );
        let slot = Arc::new(SingletonConnectSlot::new());
        let activation_lifecycle = Arc::new(SingletonActivationLifecycle::new());
        assert!(activation_lifecycle.begin());
        let prepared = Arc::new(PreparedSingleton {
            connection_id: "connection".to_string(),
            project_id: "project".to_string(),
            route_uri: "https://fn0-websocket.internal/ws_singleton/feed"
                .parse()
                .unwrap(),
            response_headers: hyper::HeaderMap::new(),
            lease_guard: Arc::new(SingletonLeaseGuard::new(
                tokio::time::Instant::now() + SINGLETON_SAFETY_DEADLINE,
            )),
            lease_activation_sender: Mutex::new(None),
            message_ready_sender: Mutex::new(None),
            activation_lifecycle: activation_lifecycle.clone(),
            activation: tokio::sync::OnceCell::new(),
        });
        assert!(slot.set(Ok(prepared)).is_ok());
        service
            .singleton_connections
            .insert(singleton_key.clone(), slot);
        let (command_sender, _command_receiver) = mpsc::channel(OUTBOUND_COMMAND_CAPACITY);
        let (closed_sender, closed_receiver) = watch::channel(false);
        let (control_sender, mut control_receiver) = mpsc::unbounded_channel();
        let (force_close_sender, _force_close_receiver) = watch::channel(false);
        service.connections.insert(
            "connection".to_string(),
            Arc::new(ConnectionEntry {
                project_id: "project".to_string(),
                command_sender,
                closing: AtomicBool::new(false),
                closed_receiver,
                control_sender,
                force_close_sender,
                lease_guard: None,
            }),
        );
        let service_for_abort = service.clone();
        let abort_task = tokio::spawn(async move {
            service_for_abort
                .abort_singleton_outbound(
                    &singleton_key,
                    "connection",
                    tokio::time::Instant::now() + Duration::from_secs(1),
                )
                .await
        });
        let close_control = control_receiver.recv().await.unwrap();
        assert!(matches!(close_control, WriterControl::Close(1011, _)));
        assert!(!abort_task.is_finished());
        closed_sender.send(true).unwrap();
        tokio::task::yield_now().await;
        assert!(!abort_task.is_finished());
        activation_lifecycle.finish(false);
        abort_task.await.unwrap().unwrap();
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
        let service = test_service(
            Arc::new(vec![worker_sender]),
            directory.clone(),
            WorkerIdentity {
                worker_id: "worker".to_string(),
                endpoint: String::new(),
            },
        );
        let initial_lease_deadline = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
            + 60_000;
        let connection_id = service
            .connect_singleton_outbound(WebSocketSingletonConnectRequest {
                project_id: "project".to_string(),
                singleton_id: "feed".to_string(),
                url: format!("ws://{server_address}/stream?market=seoul"),
                route_path: "/ws_singleton/feed".to_string(),
                headers: vec![("authorization".to_string(), "Bearer secret".to_string())],
                protocols: vec!["market.v1".to_string()],
                claim_token: "claim-token".to_string(),
                initial_lease_deadline,
                remaining: Duration::from_secs(5),
            })
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
        let service = test_service(
            Arc::new(Vec::new()),
            directory,
            WorkerIdentity {
                worker_id: "worker".to_string(),
                endpoint: String::new(),
            },
        );
        let (command_sender, mut command_receiver) = mpsc::channel(OUTBOUND_COMMAND_CAPACITY);
        let (_closed_sender, closed_receiver) = watch::channel(false);
        let (force_close_sender, _force_close_receiver) = watch::channel(false);
        let (control_sender, _control_receiver) = mpsc::unbounded_channel();
        service.connections.insert(
            "connection".to_string(),
            Arc::new(ConnectionEntry {
                project_id: "project".to_string(),
                command_sender,
                closing: AtomicBool::new(false),
                closed_receiver,
                control_sender,
                force_close_sender,
                lease_guard: None,
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
        let service = test_service(
            Arc::new(Vec::new()),
            directory,
            WorkerIdentity {
                worker_id: "worker".to_string(),
                endpoint: String::new(),
            },
        );
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
        let (force_close_sender, _force_close_receiver) = watch::channel(false);
        let (control_sender, mut control_receiver) = mpsc::unbounded_channel();
        service.connections.insert(
            "connection".to_string(),
            Arc::new(ConnectionEntry {
                project_id: "project".to_string(),
                command_sender,
                closing: AtomicBool::new(false),
                closed_receiver,
                control_sender,
                force_close_sender,
                lease_guard: None,
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
        let service = test_service(
            Arc::new(Vec::new()),
            directory,
            WorkerIdentity {
                worker_id: "worker".to_string(),
                endpoint: String::new(),
            },
        );
        let (command_sender, _command_receiver) = mpsc::channel(OUTBOUND_COMMAND_CAPACITY);
        let (_closed_sender, closed_receiver) = watch::channel(false);
        let (force_close_sender, _force_close_receiver) = watch::channel(false);
        let (control_sender, mut control_receiver) = mpsc::unbounded_channel();
        service.connections.insert(
            "connection".to_string(),
            Arc::new(ConnectionEntry {
                project_id: "project".to_string(),
                command_sender,
                closing: AtomicBool::new(false),
                closed_receiver,
                control_sender,
                force_close_sender,
                lease_guard: None,
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
                lease_guard: Arc::new(SingletonLeaseGuard::new(initial_lease_valid_until(0))),
                activation_lifecycle: Arc::new(SingletonActivationLifecycle::new()),
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
        let (force_close_sender, _force_close_receiver) = watch::channel(false);
        let (control_sender, _control_receiver) = mpsc::unbounded_channel();
        target_service.connections.insert(
            connection_id.clone(),
            Arc::new(ConnectionEntry {
                project_id: project_id.to_string(),
                command_sender,
                closing: AtomicBool::new(false),
                closed_receiver,
                control_sender,
                force_close_sender,
                lease_guard: None,
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
        let service = test_service(
            Arc::new(Vec::new()),
            directory,
            WorkerIdentity {
                worker_id: format!("test-worker-{endpoint}"),
                endpoint: endpoint.to_string(),
            },
        );
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
