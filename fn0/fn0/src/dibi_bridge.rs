use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use dibi_protocol::{
    MAX_FRAME_SIZE, Opcode, RequestOperation, Status, decode_request_frame, decode_response_frame,
    decode_response_payload, encode_request_frame, validate_tenant,
};
use rustls::pki_types::CertificateDer;
use tokio::{net::lookup_host, sync::Mutex, time::timeout};

const DEFAULT_TARGET_PORT: u16 = 4433;
const DEFAULT_PLACEHOLDER_URL: &str = "dibi://fn0-db.fn0.dev";
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const ALPN_PROTOCOL: &[u8] = b"dibi/2";

#[derive(Clone, Debug)]
pub struct DibiBridgeConfig {
    pub placeholder_url: String,
    pub target_host: String,
    pub target_port: u16,
    pub server_name: String,
    pub worker_token: Vec<u8>,
    pub ca_certificate: Option<Vec<u8>>,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum DibiBridgeError {
    #[error("Dibi bridge configuration error: {0}")]
    Configuration(String),
    #[error("Dibi endpoint is not allowed")]
    EndpointNotAllowed,
    #[error("Dibi request is invalid")]
    InvalidRequest,
    #[error("Dibi worker authentication failed")]
    AuthenticationFailed,
    #[error("Dibi bridge is unavailable: {0}")]
    Unavailable(String),
    #[error("Dibi bridge request timed out")]
    Timeout,
    #[error("Dibi response is too large")]
    ResponseTooLarge,
    #[error("Dibi protocol error: {0}")]
    Protocol(String),
}

pub struct DibiBridge {
    config: DibiBridgeConfig,
    endpoint: quinn::Endpoint,
    connection: Mutex<Option<quinn::Connection>>,
    next_request_id: AtomicU64,
}

impl DibiBridge {
    pub fn from_env() -> Result<Option<Arc<Self>>, DibiBridgeError> {
        let Some(target_host) = std::env::var("FN0_DIBI_TARGET_HOST")
            .ok()
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };
        let worker_token = std::env::var("FN0_DIBI_WORKER_TOKEN")
            .map_err(|_| {
                DibiBridgeError::Configuration("FN0_DIBI_WORKER_TOKEN is required".to_owned())
            })?
            .into_bytes();
        if worker_token.is_empty() {
            return Err(DibiBridgeError::Configuration(
                "FN0_DIBI_WORKER_TOKEN must be non-empty".to_owned(),
            ));
        }
        let target_port = parse_env_u16("FN0_DIBI_TARGET_PORT", DEFAULT_TARGET_PORT)?;
        let server_name =
            std::env::var("FN0_DIBI_SERVER_NAME").unwrap_or_else(|_| target_host.clone());
        let placeholder_url = std::env::var("FN0_DIBI_PLACEHOLDER_URL")
            .unwrap_or_else(|_| DEFAULT_PLACEHOLDER_URL.to_owned());
        let ca_certificate = read_certificate_env()?;
        Ok(Some(Arc::new(Self::new(DibiBridgeConfig {
            placeholder_url,
            target_host,
            target_port,
            server_name,
            worker_token,
            ca_certificate,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
        })?)))
    }

    pub fn new(config: DibiBridgeConfig) -> Result<Self, DibiBridgeError> {
        if config.placeholder_url.is_empty()
            || config.target_host.is_empty()
            || config.server_name.is_empty()
            || config.worker_token.is_empty()
        {
            return Err(DibiBridgeError::Configuration(
                "placeholder, target host, server name, and worker token are required".to_owned(),
            ));
        }
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let roots = build_root_store(config.ca_certificate.as_deref())?;
        let mut client_crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        client_crypto.alpn_protocols = vec![ALPN_PROTOCOL.to_vec()];
        let client_crypto = quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto)
            .map_err(|error| DibiBridgeError::Configuration(error.to_string()))?;
        let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().map_err(|error| {
            DibiBridgeError::Configuration(format!("invalid local endpoint: {error}"))
        })?)
        .map_err(|error| DibiBridgeError::Configuration(error.to_string()))?;
        endpoint.set_default_client_config(quinn::ClientConfig::new(Arc::new(client_crypto)));
        Ok(Self {
            config,
            endpoint,
            connection: Mutex::new(None),
            next_request_id: AtomicU64::new(1),
        })
    }

    pub fn placeholder_url(&self) -> &str {
        &self.config.placeholder_url
    }

    pub async fn request(
        &self,
        project_id: &str,
        endpoint: &str,
        frame: &[u8],
    ) -> Result<Vec<u8>, DibiBridgeError> {
        if endpoint != self.config.placeholder_url {
            return Err(DibiBridgeError::EndpointNotAllowed);
        }
        let request = decode_request_frame(frame).map_err(|_| DibiBridgeError::InvalidRequest)?;
        let opcode = request.operation.opcode();
        if opcode == Opcode::Auth || (opcode.is_tenant_scoped() && !request.tenant.is_empty()) {
            return Err(DibiBridgeError::InvalidRequest);
        }
        let network_tenant = if opcode.is_tenant_scoped() {
            validate_tenant(project_id).map_err(|_| DibiBridgeError::InvalidRequest)?;
            project_id
        } else {
            ""
        };
        let network_frame =
            encode_request_frame(request.request_id, network_tenant, &request.operation);
        let connection = self.connection().await?;
        let response = match timeout(
            self.config.request_timeout,
            exchange(&connection, &network_frame),
        )
        .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(error)) => {
                self.invalidate_connection(&connection).await;
                return Err(error);
            }
            Err(_) => {
                self.invalidate_connection(&connection).await;
                return Err(DibiBridgeError::Timeout);
            }
        };
        validate_response(&response, request.request_id, request.operation.opcode())?;
        Ok(response)
    }

    async fn connection(&self) -> Result<quinn::Connection, DibiBridgeError> {
        let mut guard = self.connection.lock().await;
        if let Some(connection) = guard.as_ref() {
            if connection.close_reason().is_none() {
                return Ok(connection.clone());
            }
            *guard = None;
        }
        let connection = self.connect_authenticated().await?;
        *guard = Some(connection.clone());
        Ok(connection)
    }

    async fn connect_authenticated(&self) -> Result<quinn::Connection, DibiBridgeError> {
        let target = timeout(
            self.config.connect_timeout,
            lookup_host((self.config.target_host.as_str(), self.config.target_port)),
        )
        .await
        .map_err(|_| DibiBridgeError::Timeout)?
        .map_err(|error| DibiBridgeError::Unavailable(error.to_string()))?
        .next()
        .ok_or_else(|| {
            DibiBridgeError::Unavailable("target address was not resolved".to_owned())
        })?;
        let connecting = self
            .endpoint
            .connect(target, &self.config.server_name)
            .map_err(|error| DibiBridgeError::Unavailable(error.to_string()))?;
        let connection = timeout(self.config.connect_timeout, connecting)
            .await
            .map_err(|_| DibiBridgeError::Timeout)?
            .map_err(|error| DibiBridgeError::Unavailable(error.to_string()))?;
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let frame = encode_request_frame(
            request_id,
            "",
            &RequestOperation::Auth {
                worker_token: self.config.worker_token.clone(),
            },
        );
        let response = timeout(self.config.connect_timeout, exchange(&connection, &frame))
            .await
            .map_err(|_| DibiBridgeError::Timeout)??;
        let response_frame = decode_response_frame(&response)
            .map_err(|error| DibiBridgeError::Protocol(error.to_string()))?;
        if response_frame.request_id != request_id {
            return Err(DibiBridgeError::Protocol(
                "AUTH request ID mismatch".to_owned(),
            ));
        }
        decode_response_payload(Opcode::Auth, response_frame.status, &response_frame.payload)
            .map_err(|error| DibiBridgeError::Protocol(error.to_string()))?;
        if response_frame.status != Status::Ok {
            return Err(DibiBridgeError::AuthenticationFailed);
        }
        Ok(connection)
    }

    async fn invalidate_connection(&self, connection: &quinn::Connection) {
        let mut guard = self.connection.lock().await;
        if guard
            .as_ref()
            .is_some_and(|cached| cached.stable_id() == connection.stable_id())
        {
            *guard = None;
        }
    }
}

async fn exchange(
    connection: &quinn::Connection,
    frame: &[u8],
) -> Result<Vec<u8>, DibiBridgeError> {
    let (mut send, mut receive) = connection
        .open_bi()
        .await
        .map_err(|error| DibiBridgeError::Unavailable(error.to_string()))?;
    send.write_all(frame)
        .await
        .map_err(|error| DibiBridgeError::Unavailable(error.to_string()))?;
    send.finish()
        .map_err(|error| DibiBridgeError::Unavailable(error.to_string()))?;
    read_frame(&mut receive).await
}

async fn read_frame(receive: &mut quinn::RecvStream) -> Result<Vec<u8>, DibiBridgeError> {
    let mut header = [0_u8; dibi_protocol::FRAME_HEADER_SIZE];
    receive
        .read_exact(&mut header)
        .await
        .map_err(|error| DibiBridgeError::Unavailable(error.to_string()))?;
    let payload_len = u32::from_be_bytes(
        header[16..20]
            .try_into()
            .map_err(|_| DibiBridgeError::Protocol("invalid response header".to_owned()))?,
    ) as usize;
    let frame_len = dibi_protocol::FRAME_HEADER_SIZE
        .checked_add(payload_len)
        .ok_or(DibiBridgeError::ResponseTooLarge)?;
    if frame_len > MAX_FRAME_SIZE {
        return Err(DibiBridgeError::ResponseTooLarge);
    }
    let mut frame = Vec::with_capacity(frame_len);
    frame.extend_from_slice(&header);
    let mut payload = vec![0_u8; payload_len];
    receive
        .read_exact(&mut payload)
        .await
        .map_err(|error| DibiBridgeError::Unavailable(error.to_string()))?;
    frame.extend_from_slice(&payload);
    let mut trailing = [0_u8; 1];
    match receive
        .read(&mut trailing)
        .await
        .map_err(|error| DibiBridgeError::Unavailable(error.to_string()))?
    {
        Some(0) | None => Ok(frame),
        Some(_) => Err(DibiBridgeError::Protocol(
            "trailing response bytes".to_owned(),
        )),
    }
}

fn validate_response(
    response: &[u8],
    request_id: u64,
    opcode: Opcode,
) -> Result<(), DibiBridgeError> {
    let response_frame = decode_response_frame(response)
        .map_err(|error| DibiBridgeError::Protocol(error.to_string()))?;
    if response_frame.request_id != request_id {
        return Err(DibiBridgeError::Protocol(
            "response request ID mismatch".to_owned(),
        ));
    }
    decode_response_payload(opcode, response_frame.status, &response_frame.payload)
        .map_err(|error| DibiBridgeError::Protocol(error.to_string()))?;
    Ok(())
}

fn parse_env_u16(name: &str, default: u16) -> Result<u16, DibiBridgeError> {
    let value = std::env::var(name).unwrap_or_else(|_| default.to_string());
    value.parse().map_err(|error| {
        DibiBridgeError::Configuration(format!("{name} must be a valid port: {error}"))
    })
}

fn read_certificate_env() -> Result<Option<Vec<u8>>, DibiBridgeError> {
    if let Ok(value) = std::env::var("FN0_DIBI_CA_CERT") {
        return Ok(Some(value.into_bytes()));
    }
    let Ok(value) = std::env::var("FN0_DIBI_CA_CERT_BASE64") else {
        return Ok(None);
    };
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(value)
        .map(Some)
        .map_err(|error| {
            DibiBridgeError::Configuration(format!("invalid FN0_DIBI_CA_CERT_BASE64: {error}"))
        })
}

fn build_root_store(
    ca_certificate: Option<&[u8]>,
) -> Result<rustls::RootCertStore, DibiBridgeError> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    if let Some(ca_certificate) = ca_certificate {
        let mut certificate_reader = ca_certificate;
        let certificates = rustls_pemfile::certs(&mut certificate_reader)
            .collect::<Result<Vec<CertificateDer<'static>>, _>>()
            .map_err(|error| {
                DibiBridgeError::Configuration(format!("invalid custom CA: {error}"))
            })?;
        for certificate in certificates {
            roots.add(certificate).map_err(|error| {
                DibiBridgeError::Configuration(format!("invalid custom CA: {error}"))
            })?;
        }
    }
    Ok(roots)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;

    use super::*;
    use dibi_protocol::{
        RequestOperation, ResponsePayload, Status, decode_request_frame, encode_request_frame,
        encode_response_frame,
    };
    use rcgen::generate_simple_self_signed;
    use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
    use tokio::task::JoinHandle;

    #[derive(Clone, Copy)]
    enum MockBehavior {
        Normal,
        CloseAfterResponse,
        CloseBeforeResponse,
    }

    struct MockState {
        connection_count: AtomicUsize,
        authentication_count: AtomicUsize,
        request_count: AtomicUsize,
        tenants: Mutex<Vec<String>>,
    }

    struct MockServer {
        endpoint: quinn::Endpoint,
        address: std::net::SocketAddr,
        certificate_pem: String,
        state: Arc<MockState>,
        task: JoinHandle<()>,
    }

    impl MockServer {
        async fn start(behavior: MockBehavior) -> Self {
            let certificate = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
            let certificate_pem = certificate.cert.pem();
            let certificate_der = certificate.cert.der().clone();
            let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                certificate.signing_key.serialize_der(),
            ));
            let mut server_crypto = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![certificate_der], private_key)
                .unwrap();
            server_crypto.alpn_protocols = vec![ALPN_PROTOCOL.to_vec()];
            let server_crypto =
                quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto).unwrap();
            let endpoint = quinn::Endpoint::server(
                quinn::ServerConfig::with_crypto(Arc::new(server_crypto)),
                "127.0.0.1:0".parse().unwrap(),
            )
            .unwrap();
            let address = endpoint.local_addr().unwrap();
            let state = Arc::new(MockState {
                connection_count: AtomicUsize::new(0),
                authentication_count: AtomicUsize::new(0),
                request_count: AtomicUsize::new(0),
                tenants: Mutex::new(Vec::new()),
            });
            let task_endpoint = endpoint.clone();
            let task_state = state.clone();
            let task = tokio::spawn(async move {
                while let Some(incoming) = task_endpoint.accept().await {
                    let task_state = task_state.clone();
                    tokio::spawn(async move {
                        let connection = incoming.await.unwrap();
                        task_state.connection_count.fetch_add(1, Ordering::Relaxed);
                        loop {
                            let Ok((mut send, mut receive)) = connection.accept_bi().await else {
                                return;
                            };
                            let Ok(frame) = read_frame(&mut receive).await else {
                                return;
                            };
                            let request = decode_request_frame(&frame).unwrap();
                            match request.operation {
                                RequestOperation::Auth { worker_token } => {
                                    assert_eq!(worker_token, b"worker-token");
                                    task_state
                                        .authentication_count
                                        .fetch_add(1, Ordering::Relaxed);
                                    let response = encode_response_frame(
                                        request.request_id,
                                        Status::Ok,
                                        &ResponsePayload::Empty,
                                    )
                                    .unwrap();
                                    send.write_all(&response).await.unwrap();
                                    send.finish().unwrap();
                                }
                                RequestOperation::Get { .. } => {
                                    task_state.request_count.fetch_add(1, Ordering::Relaxed);
                                    task_state.tenants.lock().await.push(request.tenant.clone());
                                    if matches!(behavior, MockBehavior::CloseBeforeResponse) {
                                        connection.close(0_u32.into(), b"request received");
                                        return;
                                    }
                                    let response = encode_response_frame(
                                        request.request_id,
                                        Status::Ok,
                                        &ResponsePayload::Found {
                                            found: true,
                                            data: Some(request.tenant.into_bytes()),
                                        },
                                    )
                                    .unwrap();
                                    send.write_all(&response).await.unwrap();
                                    send.finish().unwrap();
                                    if matches!(behavior, MockBehavior::CloseAfterResponse) {
                                        tokio::time::sleep(Duration::from_millis(20)).await;
                                        connection.close(0_u32.into(), b"connection rotation");
                                        return;
                                    }
                                }
                                _ => return,
                            }
                        }
                    });
                }
            });
            Self {
                endpoint,
                address,
                certificate_pem,
                state,
                task,
            }
        }

        async fn shutdown(self) {
            self.endpoint.close(0_u32.into(), b"test complete");
            self.task.abort();
            let _ = self.task.await;
        }
    }

    fn test_bridge() -> DibiBridge {
        DibiBridge::new(DibiBridgeConfig {
            placeholder_url: "dibi://fn0-db.fn0.dev".to_owned(),
            target_host: "127.0.0.1".to_owned(),
            target_port: 4433,
            server_name: "localhost".to_owned(),
            worker_token: b"worker-token".to_vec(),
            ca_certificate: None,
            connect_timeout: Duration::from_millis(50),
            request_timeout: Duration::from_millis(50),
        })
        .unwrap()
    }

    #[tokio::test]
    async fn rejects_spoofed_tenant_before_network_access() {
        let bridge = test_bridge();
        let frame = encode_request_frame(
            1,
            "project-b",
            &RequestOperation::Get {
                pk: "User".to_owned(),
                sk: "1".to_owned(),
            },
        );
        assert!(matches!(
            bridge
                .request("project-a", "dibi://fn0-db.fn0.dev", &frame)
                .await,
            Err(DibiBridgeError::InvalidRequest)
        ));
        assert!(bridge.connection.lock().await.is_none());
    }

    #[tokio::test]
    async fn rejects_arbitrary_endpoint_before_network_access() {
        let bridge = test_bridge();
        let frame = encode_request_frame(
            1,
            "",
            &RequestOperation::Get {
                pk: "User".to_owned(),
                sk: "1".to_owned(),
            },
        );
        assert!(matches!(
            bridge
                .request("project-a", "dibi://other.example", &frame)
                .await,
            Err(DibiBridgeError::EndpointNotAllowed)
        ));
        assert!(bridge.connection.lock().await.is_none());
    }

    fn guest_get_frame(request_id: u64) -> Vec<u8> {
        encode_request_frame(
            request_id,
            "",
            &RequestOperation::Get {
                pk: "User".to_owned(),
                sk: "1".to_owned(),
            },
        )
    }

    fn bridge_for_server(server: &MockServer) -> DibiBridge {
        DibiBridge::new(DibiBridgeConfig {
            placeholder_url: "dibi://fn0-db.fn0.dev".to_owned(),
            target_host: "127.0.0.1".to_owned(),
            target_port: server.address.port(),
            server_name: "localhost".to_owned(),
            worker_token: b"worker-token".to_vec(),
            ca_certificate: Some(server.certificate_pem.as_bytes().to_vec()),
            connect_timeout: Duration::from_secs(1),
            request_timeout: Duration::from_secs(1),
        })
        .unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn reuses_authenticated_connection_across_tenants() {
        let server = MockServer::start(MockBehavior::Normal).await;
        let bridge = bridge_for_server(&server);

        let first = bridge
            .request("project-a", "dibi://fn0-db.fn0.dev", &guest_get_frame(1))
            .await
            .unwrap();
        let second = bridge
            .request("project-b", "dibi://fn0-db.fn0.dev", &guest_get_frame(2))
            .await
            .unwrap();
        let first_response = decode_response_frame(&first).unwrap();
        let second_response = decode_response_frame(&second).unwrap();
        assert_eq!(first_response.status, Status::Ok);
        assert_eq!(second_response.status, Status::Ok);
        assert_eq!(
            decode_response_payload(Opcode::Get, Status::Ok, &first_response.payload).unwrap(),
            ResponsePayload::Found {
                found: true,
                data: Some(b"project-a".to_vec()),
            }
        );
        assert_eq!(
            decode_response_payload(Opcode::Get, Status::Ok, &second_response.payload).unwrap(),
            ResponsePayload::Found {
                found: true,
                data: Some(b"project-b".to_vec()),
            }
        );
        assert_eq!(server.state.connection_count.load(Ordering::Relaxed), 1);
        assert_eq!(server.state.authentication_count.load(Ordering::Relaxed), 1);
        assert_eq!(server.state.request_count.load(Ordering::Relaxed), 2);
        assert_eq!(
            &*server.state.tenants.lock().await,
            &["project-a".to_owned(), "project-b".to_owned()]
        );
        server.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn reconnects_and_authenticates_after_cached_connection_closes() {
        let server = MockServer::start(MockBehavior::CloseAfterResponse).await;
        let bridge = bridge_for_server(&server);

        bridge
            .request("project-a", "dibi://fn0-db.fn0.dev", &guest_get_frame(1))
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let closed = bridge
                    .connection
                    .lock()
                    .await
                    .as_ref()
                    .is_some_and(|connection| connection.close_reason().is_some());
                if closed {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        bridge
            .request("project-b", "dibi://fn0-db.fn0.dev", &guest_get_frame(2))
            .await
            .unwrap();
        assert_eq!(server.state.connection_count.load(Ordering::Relaxed), 2);
        assert_eq!(server.state.authentication_count.load(Ordering::Relaxed), 2);
        assert_eq!(server.state.request_count.load(Ordering::Relaxed), 2);
        server.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn does_not_replay_a_request_after_remote_connection_failure() {
        let server = MockServer::start(MockBehavior::CloseBeforeResponse).await;
        let bridge = bridge_for_server(&server);

        assert!(
            bridge
                .request("project-a", "dibi://fn0-db.fn0.dev", &guest_get_frame(1))
                .await
                .is_err()
        );
        assert_eq!(server.state.connection_count.load(Ordering::Relaxed), 1);
        assert_eq!(server.state.authentication_count.load(Ordering::Relaxed), 1);
        assert_eq!(server.state.request_count.load(Ordering::Relaxed), 1);
        server.shutdown().await;
    }
}
