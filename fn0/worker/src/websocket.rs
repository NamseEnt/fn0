use crate::websocket_directory::{
    ConnectionDirectory, ConnectionOwner, WorkerIdentity, directory_from_env,
    worker_identity_from_env,
};
use crate::websocket_quic::QuicTransport;
use crate::worker_pool::{self, DispatchError, RequestEnvelope};
use base64::Engine;
use bytes::Bytes;
use dashmap::DashMap;
use fastwebsockets::upgrade::UpgradeFut;
use fastwebsockets::{Frame, OpCode, Payload, WebSocketError, WebSocketRead, WebSocketWrite};
use fn0::{
    Body, WebSocketCommandDispatcher, WebSocketCommandError, WebSocketCommandErrorKind,
    WebSocketCommandFuture, WebSocketDeliveryState, WebSocketMessageKind,
};
use http_body_util::{BodyExt, Empty, Full};
use hyper_util::rt::TokioIo;
use rand::RngCore;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::{mpsc, oneshot, watch};

const PROJECT_CONNECTION_LIMIT: usize = 1_000;
const WORKER_CONNECTION_LIMIT: usize = 10_000;
const OUTBOUND_COMMAND_CAPACITY: usize = 4;
const INBOUND_PENDING_LIMIT: usize = 4;
const CALLBACK_DEADLINE: Duration = Duration::from_secs(15);
const CLOSE_HANDSHAKE_DEADLINE: Duration = Duration::from_secs(10);
const PING_INTERVAL: Duration = Duration::from_secs(30);
const PONG_DEADLINE: Duration = Duration::from_secs(15);

type UpgradedIo = TokioIo<hyper::upgrade::Upgraded>;
type SocketReader = WebSocketRead<ReadHalf<UpgradedIo>>;
type SocketWriter = WebSocketWrite<WriteHalf<UpgradedIo>>;

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
    project_counts: DashMap<String, Arc<AtomicUsize>>,
    project_generations: DashMap<String, Arc<std::sync::atomic::AtomicU64>>,
    worker_count: Arc<AtomicUsize>,
    draining: AtomicBool,
    directory: Arc<dyn ConnectionDirectory>,
    identity: WorkerIdentity,
    quic: OnceLock<Arc<QuicTransport>>,
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
            project_counts: DashMap::new(),
            project_generations: DashMap::new(),
            worker_count: Arc::new(AtomicUsize::new(0)),
            draining: AtomicBool::new(false),
            directory,
            identity,
            quic: OnceLock::new(),
        });
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
            let (command_sender, command_receiver) = mpsc::channel(OUTBOUND_COMMAND_CAPACITY);
            let (control_sender, control_receiver) = mpsc::unbounded_channel();
            let (closed_sender, closed_receiver) = watch::channel(false);
            let entry = Arc::new(ConnectionEntry {
                project_id: project_id.clone(),
                command_sender,
                closing: AtomicBool::new(false),
                closed_receiver,
                control_sender: control_sender.clone(),
            });
            service
                .connections
                .insert(connection_id.clone(), entry.clone());
            if service.draining.load(Ordering::Acquire)
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
            service
                .run_connection(
                    project_id,
                    connection_id,
                    route_uri,
                    reader,
                    writer,
                    command_receiver,
                    control_sender,
                    control_receiver,
                    closed_sender,
                    capacity_guard,
                )
                .await;
        });
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
    ) {
        let disconnect_info = Arc::new(Mutex::new(None));
        let reader_handle = tokio::spawn(read_loop(
            self.clone(),
            project_id.clone(),
            connection_id.clone(),
            route_uri.clone(),
            reader,
            control_sender,
            disconnect_info.clone(),
        ));
        writer_loop(
            &mut writer,
            command_receiver,
            control_receiver,
            disconnect_info.clone(),
        )
        .await;
        reader_handle.abort();
        self.connections.remove(&connection_id);
        self.unpublish_connection(&connection_id).await;
        let _ = closed_sender.send(true);
        drop(capacity_guard);
        let final_info = disconnect_info
            .lock()
            .expect("disconnect info lock")
            .clone()
            .unwrap_or_else(DisconnectInfo::transport_error);
        self.invoke_disconnect(&project_id, &connection_id, &route_uri, final_info);
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
}

impl WebSocketCommandDispatcher for WebSocketService {
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

async fn read_loop(
    service: Arc<WebSocketService>,
    project_id: String,
    connection_id: String,
    route_uri: hyper::Uri,
    mut reader: SocketReader,
    control_sender: mpsc::UnboundedSender<WriterControl>,
    disconnect_info: Arc<Mutex<Option<DisconnectInfo>>>,
) {
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
}
