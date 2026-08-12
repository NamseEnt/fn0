use crate::websocket::WebSocketService;
use dashmap::DashMap;
use fn0::{
    Body, WebSocketCommandError, WebSocketCommandErrorKind, WebSocketDeliveryState,
    WebSocketMessageKind,
};
use futures::TryStreamExt;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Weak};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_util::io::ReaderStream;

const PROTOCOL_VERSION: u8 = 1;
const HEADER_LIMIT: usize = 16 * 1024;
const ALPN: &[u8] = b"fn0-websocket/1";
const DIAL_TIMEOUT: Duration = Duration::from_secs(2);
const RECONCILE_CONNECTION_LIMIT: usize = 256;
const RECONCILE_BODY_LIMIT: usize = 1024 * 1024;
const RECONCILE_PATH: &str = "/__fn0_websocket/reconcile";

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum CommandKind {
    Send,
    Disconnect,
}

#[derive(Deserialize, Serialize)]
struct CommandHeader {
    version: u8,
    bearer: String,
    caller_project_id: String,
    connection_id: String,
    target_worker_id: String,
    command: CommandKind,
    message_kind: Option<MessageKind>,
    deadline_unix_millis: u64,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum MessageKind {
    Text,
    Binary,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ResponseStage {
    Ready,
    Complete,
    Rejected,
}

#[derive(Deserialize, Serialize)]
struct CommandResponse {
    stage: ResponseStage,
    error: Option<WireError>,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum WireErrorKind {
    ConnectionNotFound,
    Backpressure,
    DeadlineExceeded,
    Transport,
    InvalidText,
    Internal,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum WireDeliveryState {
    NotSent,
    Unknown,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
struct WireError {
    kind: WireErrorKind,
    delivery: WireDeliveryState,
}

#[derive(Deserialize)]
struct ReconcileRequest {
    worker_id: String,
    connection_ids: Vec<String>,
}

#[derive(Serialize)]
struct ReconcileResponse {
    worker_id: String,
    present_connection_ids: Vec<String>,
}

pub struct QuicTransport {
    service: Weak<WebSocketService>,
    endpoint: quinn::Endpoint,
    bearer: String,
    server_name: String,
    reconcile_bind_address: SocketAddr,
    connections: DashMap<String, quinn::Connection>,
}

impl QuicTransport {
    pub fn from_env(service: Weak<WebSocketService>) -> anyhow::Result<Option<Arc<Self>>> {
        let endpoint_value = std::env::var("FN0_WEBSOCKET_QUIC_ENDPOINT").unwrap_or_default();
        if endpoint_value.is_empty() {
            return Ok(None);
        }
        let advertised_address: SocketAddr = endpoint_value.parse()?;
        let bind_address = std::env::var("FN0_WEBSOCKET_QUIC_BIND")
            .ok()
            .map(|value| value.parse())
            .transpose()?
            .unwrap_or_else(|| {
                SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), advertised_address.port())
            });
        let certificate_pem = crate::read_pem_env("FN0_WEBSOCKET_QUIC_CERT_PEM")
            .ok_or_else(|| anyhow::anyhow!("FN0_WEBSOCKET_QUIC_CERT_PEM is required"))?;
        let key_pem = crate::read_pem_env("FN0_WEBSOCKET_QUIC_KEY_PEM")
            .ok_or_else(|| anyhow::anyhow!("FN0_WEBSOCKET_QUIC_KEY_PEM is required"))?;
        let bearer = std::env::var("FN0_WEBSOCKET_QUIC_BEARER")?;
        let server_name = std::env::var("FN0_WEBSOCKET_QUIC_SERVER_NAME")
            .unwrap_or_else(|_| "fn0-worker.internal".to_string());
        let certificates: Vec<CertificateDer<'static>> =
            rustls_pemfile::certs(&mut certificate_pem.as_bytes()).collect::<Result<_, _>>()?;
        let private_key: PrivateKeyDer<'static> =
            rustls_pemfile::private_key(&mut key_pem.as_bytes())?
                .ok_or_else(|| anyhow::anyhow!("QUIC private key is missing"))?;

        let mut server_crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certificates.clone(), private_key)?;
        server_crypto.alpn_protocols = vec![ALPN.to_vec()];
        let mut server_config =
            quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));
        Arc::get_mut(&mut server_config.transport)
            .expect("unique transport config")
            .max_concurrent_uni_streams(0_u8.into());
        let mut endpoint = quinn::Endpoint::server(server_config, bind_address)?;

        let mut roots = rustls::RootCertStore::empty();
        for certificate in certificates {
            roots.add(certificate)?;
        }
        let mut client_crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        client_crypto.alpn_protocols = vec![ALPN.to_vec()];
        endpoint.set_default_client_config(quinn::ClientConfig::new(Arc::new(
            QuicClientConfig::try_from(client_crypto)?,
        )));

        Ok(Some(Arc::new(Self {
            service,
            endpoint,
            bearer,
            server_name,
            reconcile_bind_address: bind_address,
            connections: DashMap::new(),
        })))
    }

    pub fn spawn_server(self: &Arc<Self>) {
        let reconcile_transport = self.clone();
        tokio::spawn(async move {
            if let Err(error) = reconcile_transport.serve_reconcile().await {
                tracing::error!(%error, "websocket reconciliation server failed");
            }
        });
        let transport = self.clone();
        tokio::spawn(async move {
            while let Some(incoming) = transport.endpoint.accept().await {
                let transport = transport.clone();
                tokio::spawn(async move {
                    match incoming.await {
                        Ok(connection) => transport.serve_connection(connection).await,
                        Err(error) => tracing::warn!(%error, "websocket QUIC handshake failed"),
                    }
                });
            }
        });
    }

    async fn serve_reconcile(self: Arc<Self>) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.reconcile_bind_address).await?;
        loop {
            let (socket, _) = listener.accept().await?;
            let transport = self.clone();
            tokio::spawn(async move {
                let service = service_fn(move |request| {
                    let transport = transport.clone();
                    async move { transport.handle_reconcile(request).await }
                });
                if let Err(error) = http1::Builder::new()
                    .serve_connection(TokioIo::new(socket), service)
                    .await
                {
                    tracing::warn!(%error, "websocket reconciliation request failed");
                }
            });
        }
    }

    async fn handle_reconcile(
        &self,
        request: hyper::Request<hyper::body::Incoming>,
    ) -> Result<hyper::Response<Full<bytes::Bytes>>, std::convert::Infallible> {
        let authorized = request
            .headers()
            .get(hyper::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value == format!("Bearer {}", self.bearer));
        if request.method() != hyper::Method::POST
            || request.uri().path() != RECONCILE_PATH
            || !authorized
        {
            return Ok(empty_response(hyper::StatusCode::NOT_FOUND));
        }
        let body = match request.into_body().collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => return Ok(empty_response(hyper::StatusCode::BAD_REQUEST)),
        };
        if body.len() > RECONCILE_BODY_LIMIT {
            return Ok(empty_response(hyper::StatusCode::PAYLOAD_TOO_LARGE));
        }
        let request: ReconcileRequest = match serde_json::from_slice::<ReconcileRequest>(&body) {
            Ok(request) if request.connection_ids.len() <= RECONCILE_CONNECTION_LIMIT => request,
            _ => return Ok(empty_response(hyper::StatusCode::BAD_REQUEST)),
        };
        let Some(service) = self.service.upgrade() else {
            return Ok(empty_response(hyper::StatusCode::SERVICE_UNAVAILABLE));
        };
        let present_connection_ids = if request.worker_id == service.worker_id() {
            request
                .connection_ids
                .into_iter()
                .filter(|connection_id| service.has_connection(connection_id))
                .collect()
        } else {
            Vec::new()
        };
        let body = serde_json::to_vec(&ReconcileResponse {
            worker_id: service.worker_id().to_string(),
            present_connection_ids,
        })
        .unwrap_or_default();
        Ok(hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .body(Full::new(bytes::Bytes::from(body)))
            .expect("valid reconciliation response"))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn send(
        &self,
        endpoint: &str,
        caller_project_id: String,
        connection_id: String,
        target_worker_id: String,
        message_kind: WebSocketMessageKind,
        mut body: Body,
        remaining: Duration,
    ) -> Result<(), WebSocketCommandError> {
        let connection = self.connection(endpoint).await?;
        let (mut send_stream, mut receive_stream) = connection
            .open_bi()
            .await
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?;
        let header = CommandHeader {
            version: PROTOCOL_VERSION,
            bearer: self.bearer.clone(),
            caller_project_id,
            connection_id,
            target_worker_id,
            command: CommandKind::Send,
            message_kind: Some(message_kind.into()),
            deadline_unix_millis: unix_millis()
                .saturating_add(remaining.as_millis().try_into().unwrap_or(u64::MAX)),
        };
        write_packet(&mut send_stream, &header)
            .await
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?;
        expect_ready(&mut receive_stream).await?;
        while let Some(frame_result) = body.frame().await {
            let frame = frame_result
                .map_err(|_| WebSocketCommandError::unknown(WebSocketCommandErrorKind::Internal))?;
            if let Ok(data) = frame.into_data() {
                send_stream.write_all(&data).await.map_err(|_| {
                    WebSocketCommandError::unknown(WebSocketCommandErrorKind::Transport)
                })?;
            }
        }
        send_stream
            .finish()
            .map_err(|_| WebSocketCommandError::unknown(WebSocketCommandErrorKind::Transport))?;
        read_complete(&mut receive_stream).await
    }

    pub async fn disconnect(
        &self,
        endpoint: &str,
        caller_project_id: String,
        connection_id: String,
        target_worker_id: String,
        remaining: Duration,
    ) -> Result<(), WebSocketCommandError> {
        let connection = self.connection(endpoint).await?;
        let (mut send_stream, mut receive_stream) = connection
            .open_bi()
            .await
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?;
        let header = CommandHeader {
            version: PROTOCOL_VERSION,
            bearer: self.bearer.clone(),
            caller_project_id,
            connection_id,
            target_worker_id,
            command: CommandKind::Disconnect,
            message_kind: None,
            deadline_unix_millis: unix_millis()
                .saturating_add(remaining.as_millis().try_into().unwrap_or(u64::MAX)),
        };
        write_packet(&mut send_stream, &header)
            .await
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?;
        send_stream
            .finish()
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?;
        expect_ready(&mut receive_stream).await?;
        read_complete(&mut receive_stream).await
    }

    async fn connection(&self, endpoint: &str) -> Result<quinn::Connection, WebSocketCommandError> {
        if let Some(connection) = self.connections.get(endpoint)
            && connection.close_reason().is_none()
        {
            return Ok(connection.clone());
        }
        let address: SocketAddr = endpoint
            .parse()
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?;
        let connecting = self
            .endpoint
            .connect(address, &self.server_name)
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?;
        let connection = tokio::time::timeout(DIAL_TIMEOUT, connecting)
            .await
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?
            .map_err(|_| WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport))?;
        self.connections
            .insert(endpoint.to_string(), connection.clone());
        Ok(connection)
    }

    async fn serve_connection(self: Arc<Self>, connection: quinn::Connection) {
        loop {
            let streams = match connection.accept_bi().await {
                Ok(streams) => streams,
                Err(_) => return,
            };
            let transport = self.clone();
            tokio::spawn(async move {
                if let Err(error) = transport.serve_command(streams.0, streams.1).await {
                    tracing::warn!(%error, "websocket QUIC command failed");
                }
            });
        }
    }

    async fn serve_command(
        &self,
        mut send_stream: quinn::SendStream,
        mut receive_stream: quinn::RecvStream,
    ) -> anyhow::Result<()> {
        let header: CommandHeader = read_packet(&mut receive_stream).await?;
        if header.version != PROTOCOL_VERSION
            || header.bearer != self.bearer
            || self
                .service
                .upgrade()
                .is_none_or(|service| service.worker_id() != header.target_worker_id)
        {
            write_rejected(
                &mut send_stream,
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::ConnectionNotFound),
            )
            .await?;
            return Ok(());
        }
        let Some(service) = self.service.upgrade() else {
            write_rejected(
                &mut send_stream,
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Transport),
            )
            .await?;
            return Ok(());
        };
        match header.command {
            CommandKind::Send => {
                let Some(message_kind) = header.message_kind else {
                    write_rejected(
                        &mut send_stream,
                        WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal),
                    )
                    .await?;
                    return Ok(());
                };
                let body_stream = ReaderStream::new(receive_stream)
                    .map_ok(http_body::Frame::data)
                    .map_err(anyhow::Error::from);
                let body = StreamBody::new(body_stream).boxed_unsync();
                let admitted = match service.admit_local_send(
                    &header.caller_project_id,
                    &header.connection_id,
                    message_kind.into(),
                    body,
                    deadline_instant(header.deadline_unix_millis),
                ) {
                    Ok(admitted) => admitted,
                    Err(error) => {
                        write_rejected(&mut send_stream, error).await?;
                        return Ok(());
                    }
                };
                if admitted.ready_receiver.await.is_err() {
                    let result = admitted.response_receiver.await.unwrap_or_else(|_| {
                        Err(WebSocketCommandError::unknown(
                            WebSocketCommandErrorKind::Transport,
                        ))
                    });
                    write_rejected(
                        &mut send_stream,
                        result.err().unwrap_or_else(|| {
                            WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal)
                        }),
                    )
                    .await?;
                    return Ok(());
                }
                write_packet(
                    &mut send_stream,
                    &CommandResponse {
                        stage: ResponseStage::Ready,
                        error: None,
                    },
                )
                .await?;
                let result = admitted.response_receiver.await.unwrap_or_else(|_| {
                    Err(WebSocketCommandError::unknown(
                        WebSocketCommandErrorKind::Transport,
                    ))
                });
                write_result(&mut send_stream, result).await?;
            }
            CommandKind::Disconnect => {
                let disconnect_future =
                    service.disconnect_local(&header.caller_project_id, &header.connection_id);
                write_packet(
                    &mut send_stream,
                    &CommandResponse {
                        stage: ResponseStage::Ready,
                        error: None,
                    },
                )
                .await?;
                let result = disconnect_future.await;
                write_result(&mut send_stream, result).await?;
            }
        }
        send_stream.finish()?;
        Ok(())
    }
}

impl From<WebSocketMessageKind> for MessageKind {
    fn from(value: WebSocketMessageKind) -> Self {
        match value {
            WebSocketMessageKind::Text => Self::Text,
            WebSocketMessageKind::Binary => Self::Binary,
        }
    }
}

impl From<MessageKind> for WebSocketMessageKind {
    fn from(value: MessageKind) -> Self {
        match value {
            MessageKind::Text => Self::Text,
            MessageKind::Binary => Self::Binary,
        }
    }
}

impl From<WebSocketCommandError> for WireError {
    fn from(value: WebSocketCommandError) -> Self {
        Self {
            kind: match value.kind {
                WebSocketCommandErrorKind::ConnectionNotFound => WireErrorKind::ConnectionNotFound,
                WebSocketCommandErrorKind::Backpressure => WireErrorKind::Backpressure,
                WebSocketCommandErrorKind::DeadlineExceeded => WireErrorKind::DeadlineExceeded,
                WebSocketCommandErrorKind::Transport => WireErrorKind::Transport,
                WebSocketCommandErrorKind::InvalidText => WireErrorKind::InvalidText,
                WebSocketCommandErrorKind::Internal => WireErrorKind::Internal,
            },
            delivery: match value.delivery {
                WebSocketDeliveryState::NotSent => WireDeliveryState::NotSent,
                WebSocketDeliveryState::Unknown => WireDeliveryState::Unknown,
            },
        }
    }
}

impl From<WireError> for WebSocketCommandError {
    fn from(value: WireError) -> Self {
        Self {
            kind: match value.kind {
                WireErrorKind::ConnectionNotFound => WebSocketCommandErrorKind::ConnectionNotFound,
                WireErrorKind::Backpressure => WebSocketCommandErrorKind::Backpressure,
                WireErrorKind::DeadlineExceeded => WebSocketCommandErrorKind::DeadlineExceeded,
                WireErrorKind::Transport => WebSocketCommandErrorKind::Transport,
                WireErrorKind::InvalidText => WebSocketCommandErrorKind::InvalidText,
                WireErrorKind::Internal => WebSocketCommandErrorKind::Internal,
            },
            delivery: match value.delivery {
                WireDeliveryState::NotSent => WebSocketDeliveryState::NotSent,
                WireDeliveryState::Unknown => WebSocketDeliveryState::Unknown,
            },
        }
    }
}

fn empty_response(status: hyper::StatusCode) -> hyper::Response<Full<bytes::Bytes>> {
    hyper::Response::builder()
        .status(status)
        .body(Full::new(bytes::Bytes::new()))
        .expect("valid empty response")
}

async fn expect_ready(receive_stream: &mut quinn::RecvStream) -> Result<(), WebSocketCommandError> {
    let response: CommandResponse = read_packet(receive_stream)
        .await
        .map_err(|_| WebSocketCommandError::unknown(WebSocketCommandErrorKind::Transport))?;
    match response.stage {
        ResponseStage::Ready => Ok(()),
        ResponseStage::Rejected | ResponseStage::Complete => Err(response
            .error
            .map(WebSocketCommandError::from)
            .unwrap_or_else(|| {
                WebSocketCommandError::not_sent(WebSocketCommandErrorKind::Internal)
            })),
    }
}

async fn read_complete(
    receive_stream: &mut quinn::RecvStream,
) -> Result<(), WebSocketCommandError> {
    let response: CommandResponse = read_packet(receive_stream)
        .await
        .map_err(|_| WebSocketCommandError::unknown(WebSocketCommandErrorKind::Transport))?;
    match response.stage {
        ResponseStage::Complete => response.error.map_or(Ok(()), |error| Err(error.into())),
        ResponseStage::Ready | ResponseStage::Rejected => Err(WebSocketCommandError::unknown(
            WebSocketCommandErrorKind::Internal,
        )),
    }
}

async fn write_rejected(
    send_stream: &mut quinn::SendStream,
    error: WebSocketCommandError,
) -> anyhow::Result<()> {
    write_packet(
        send_stream,
        &CommandResponse {
            stage: ResponseStage::Rejected,
            error: Some(error.into()),
        },
    )
    .await?;
    send_stream.finish()?;
    Ok(())
}

async fn write_result(
    send_stream: &mut quinn::SendStream,
    result: Result<(), WebSocketCommandError>,
) -> anyhow::Result<()> {
    write_packet(
        send_stream,
        &CommandResponse {
            stage: ResponseStage::Complete,
            error: result.err().map(WireError::from),
        },
    )
    .await
}

async fn write_packet<Value: Serialize>(
    send_stream: &mut quinn::SendStream,
    value: &Value,
) -> anyhow::Result<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > HEADER_LIMIT {
        return Err(anyhow::anyhow!("QUIC command packet too large"));
    }
    send_stream
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .await?;
    send_stream.write_all(&bytes).await?;
    Ok(())
}

async fn read_packet<Value: for<'de> Deserialize<'de>>(
    receive_stream: &mut quinn::RecvStream,
) -> anyhow::Result<Value> {
    let mut length_bytes = [0_u8; 4];
    receive_stream.read_exact(&mut length_bytes).await?;
    let length = u32::from_be_bytes(length_bytes) as usize;
    if length > HEADER_LIMIT {
        return Err(anyhow::anyhow!("QUIC command packet too large"));
    }
    let mut bytes = vec![0_u8; length];
    receive_stream.read_exact(&mut bytes).await?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn deadline_instant(deadline_unix_millis: u64) -> tokio::time::Instant {
    let remaining_millis = deadline_unix_millis.saturating_sub(unix_millis());
    tokio::time::Instant::now() + std::time::Duration::from_millis(remaining_millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_error_preserves_delivery_state() {
        let error = WebSocketCommandError::unknown(WebSocketCommandErrorKind::Transport);
        let decoded = WebSocketCommandError::from(WireError::from(error));
        assert_eq!(decoded.kind, WebSocketCommandErrorKind::Transport);
        assert_eq!(decoded.delivery, WebSocketDeliveryState::Unknown);
    }
}
