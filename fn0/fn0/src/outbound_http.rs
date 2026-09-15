//! Sends the application's own outbound HTTP requests through the destination policy and the
//! egress budget.
//!
//! This mirrors `wasmtime_wasi_http::p3::default_send_request` (HTTP/1.1, webpki roots, the same
//! timeout options) with two differences: the TCP connection goes through [`OutboundDialer`], and
//! the request body is charged to the project's egress budget as it streams. The default function
//! resolves and connects in one call, which leaves no place to reject a private address.

use crate::egress::{EgressBudget, EgressDenied, EgressMeteredBody};
use crate::outbound_destination::{OutboundDialError, OutboundDialer, PrivateDestinationAccess};
use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::http;
use hyper_util::rt::TokioIo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use wasmtime_wasi_http::p3::RequestOptions;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(600);

pub type OutboundHttpBody = UnsyncBoxBody<Bytes, ErrorCode>;
pub type OutboundHttpTransmit = Pin<Box<dyn Future<Output = Result<(), ErrorCode>> + Send>>;

pub struct GuestOutboundHttp {
    application_dialer: OutboundDialer,
    control_dialer: OutboundDialer,
    control_project_id: String,
    egress_budget: Arc<dyn EgressBudget>,
}

impl GuestOutboundHttp {
    /// The control project is platform code, not application code: it probes workers on their
    /// private endpoints, so it always dials with private destinations allowed.
    pub fn new(
        application_dialer: OutboundDialer,
        egress_budget: Arc<dyn EgressBudget>,
        control_project_id: String,
    ) -> Self {
        let control_dialer =
            application_dialer.with_private_destination_access(PrivateDestinationAccess::Allowed);
        Self {
            application_dialer,
            control_dialer,
            control_project_id,
            egress_budget,
        }
    }

    pub async fn send(
        &self,
        project_id: &str,
        request: http::Request<OutboundHttpBody>,
        options: Option<RequestOptions>,
    ) -> Result<(http::Response<OutboundHttpBody>, OutboundHttpTransmit), ErrorCode> {
        if self.egress_budget.known_exhausted(project_id) {
            return Err(ErrorCode::HttpRequestDenied);
        }
        let connect_timeout = options
            .and_then(|options| options.connect_timeout)
            .unwrap_or(DEFAULT_TIMEOUT);
        let first_byte_timeout = options
            .and_then(|options| options.first_byte_timeout)
            .unwrap_or(DEFAULT_TIMEOUT);
        let between_bytes_timeout = options
            .and_then(|options| options.between_bytes_timeout)
            .unwrap_or(DEFAULT_TIMEOUT);

        let (mut parts, body) = request.into_parts();
        let use_tls = parts.uri.scheme() == Some(&http::uri::Scheme::HTTPS);
        let host = parts
            .uri
            .host()
            .ok_or(ErrorCode::HttpRequestUriInvalid)?
            .to_string();
        let port = parts
            .uri
            .port_u16()
            .unwrap_or(if use_tls { 443 } else { 80 });

        let dialer = if project_id == self.control_project_id {
            &self.control_dialer
        } else {
            &self.application_dialer
        };
        let tcp_stream = dialer
            .connect(&host, port, connect_timeout)
            .await
            .map_err(dial_error_code)?;

        let metered_body = EgressMeteredBody::new(
            body,
            self.egress_budget.clone(),
            project_id.to_string(),
            egress_denied_error_code,
        )
        .boxed_unsync();

        parts.uri = http::Uri::builder()
            .path_and_query(
                parts
                    .uri
                    .path_and_query()
                    .map(|path_and_query| path_and_query.as_str())
                    .unwrap_or("/"),
            )
            .build()
            .map_err(|_| ErrorCode::HttpRequestUriInvalid)?;
        let request = http::Request::from_parts(parts, metered_body);

        if use_tls {
            let server_name = tls_server_name(&host)?;
            let tls_stream = tokio::time::timeout(
                connect_timeout,
                outbound_tls_connector().connect(server_name, tcp_stream),
            )
            .await
            .map_err(|_| ErrorCode::ConnectionTimeout)?
            .map_err(|_| ErrorCode::TlsProtocolError)?;
            send_over(
                tls_stream,
                request,
                connect_timeout,
                first_byte_timeout,
                between_bytes_timeout,
            )
            .await
        } else {
            send_over(
                tcp_stream,
                request,
                connect_timeout,
                first_byte_timeout,
                between_bytes_timeout,
            )
            .await
        }
    }
}

fn dial_error_code(error: OutboundDialError) -> ErrorCode {
    match error {
        OutboundDialError::DestinationForbidden => ErrorCode::DestinationIpProhibited,
        OutboundDialError::NameResolution(_) | OutboundDialError::NoAddresses => {
            ErrorCode::DnsError(
                wasmtime_wasi_http::p3::bindings::http::types::DnsErrorPayload {
                    rcode: Some("address not available".to_string()),
                    info_code: Some(0),
                },
            )
        }
        OutboundDialError::Connect(_) => ErrorCode::ConnectionRefused,
        OutboundDialError::Timeout => ErrorCode::ConnectionTimeout,
    }
}

fn response_error_code(error: hyper::Error) -> ErrorCode {
    if error.is_timeout() {
        return ErrorCode::HttpResponseTimeout;
    }
    if let Some(error_code) =
        std::error::Error::source(&error).and_then(|cause| cause.downcast_ref::<ErrorCode>())
    {
        return error_code.clone();
    }
    ErrorCode::HttpProtocolError
}

fn egress_denied_error_code(_denied: EgressDenied) -> ErrorCode {
    ErrorCode::HttpRequestDenied
}

fn tls_server_name(host: &str) -> Result<rustls::pki_types::ServerName<'static>, ErrorCode> {
    let unbracketed_host = host
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
        .unwrap_or(host);
    if let Ok(address) = unbracketed_host.parse::<std::net::IpAddr>() {
        return Ok(rustls::pki_types::ServerName::from(address));
    }
    rustls::pki_types::ServerName::try_from(unbracketed_host.to_string()).map_err(|_| {
        ErrorCode::DnsError(
            wasmtime_wasi_http::p3::bindings::http::types::DnsErrorPayload {
                rcode: Some("invalid dns name".to_string()),
                info_code: Some(0),
            },
        )
    })
}

fn outbound_tls_connector() -> tokio_rustls::TlsConnector {
    let root_certificates = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_certificates)
        .with_no_client_auth();
    tokio_rustls::TlsConnector::from(Arc::new(config))
}

async fn send_over<Stream>(
    stream: Stream,
    request: http::Request<OutboundHttpBody>,
    connect_timeout: Duration,
    first_byte_timeout: Duration,
    between_bytes_timeout: Duration,
) -> Result<(http::Response<OutboundHttpBody>, OutboundHttpTransmit), ErrorCode>
where
    Stream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Unpin + 'static,
{
    let (mut sender, connection) = tokio::time::timeout(
        connect_timeout,
        hyper::client::conn::http1::Builder::new().handshake(TokioIo::new(stream)),
    )
    .await
    .map_err(|_| ErrorCode::ConnectionTimeout)?
    .map_err(ErrorCode::from_hyper_request_error)?;

    let send = async move {
        let response = tokio::time::timeout(first_byte_timeout, sender.send_request(request))
            .await
            .map_err(|_| ErrorCode::ConnectionReadTimeout)?
            .map_err(ErrorCode::from_hyper_request_error)?;
        let mut between_bytes_interval = tokio::time::interval(between_bytes_timeout);
        between_bytes_interval.reset();
        Ok::<_, ErrorCode>(response.map(|incoming| {
            IncomingResponseBody {
                incoming,
                between_bytes_interval,
            }
            .boxed_unsync()
        }))
    };
    let mut send = std::pin::pin!(send);
    let mut connection = Some(connection);
    let response = std::future::poll_fn(|context| match send.as_mut().poll(context) {
        Poll::Ready(result) => Poll::Ready(result),
        Poll::Pending => {
            let Some(connection_future) = connection.as_mut() else {
                return Poll::Pending;
            };
            let connection_result = std::task::ready!(Pin::new(connection_future).poll(context));
            connection = None;
            match connection_result {
                Ok(()) => send.as_mut().poll(context),
                Err(error) => Poll::Ready(Err(ErrorCode::from_hyper_request_error(error))),
            }
        }
    })
    .await?;
    let transmit: OutboundHttpTransmit = Box::pin(async move {
        let Some(connection) = connection.take() else {
            return Ok(());
        };
        connection.await.map_err(response_error_code)
    });
    Ok((response, transmit))
}

struct IncomingResponseBody {
    incoming: hyper::body::Incoming,
    between_bytes_interval: tokio::time::Interval,
}

impl http_body::Body for IncomingResponseBody {
    type Data = Bytes;
    type Error = ErrorCode;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Bytes>, ErrorCode>>> {
        match Pin::new(&mut self.incoming).poll_frame(context) {
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(error))) => Poll::Ready(Some(Err(response_error_code(error)))),
            Poll::Ready(Some(Ok(frame))) => {
                self.between_bytes_interval.reset();
                Poll::Ready(Some(Ok(frame)))
            }
            Poll::Pending => {
                std::task::ready!(self.between_bytes_interval.poll_tick(context));
                Poll::Ready(Some(Err(ErrorCode::ConnectionReadTimeout)))
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        self.incoming.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.incoming.size_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::egress::test_budget::CountingBudget;
    use http_body_util::Full;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn request_to(uri: &str, body: &'static [u8]) -> http::Request<OutboundHttpBody> {
        http::Request::builder()
            .method(http::Method::POST)
            .uri(uri)
            .body(
                Full::new(Bytes::from_static(body))
                    .map_err(|never: std::convert::Infallible| match never {})
                    .boxed_unsync(),
            )
            .unwrap()
    }

    #[tokio::test]
    async fn blocked_policy_refuses_loopback_before_connecting() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let listener_port = listener.local_addr().unwrap().port();
        let budget = CountingBudget::with_remaining(1_000);
        let outbound_http = GuestOutboundHttp::new(
            OutboundDialer::system(PrivateDestinationAccess::Blocked),
            budget.clone(),
            "fn0-control".to_string(),
        );
        let result = outbound_http
            .send(
                "project",
                request_to(&format!("http://127.0.0.1:{listener_port}/"), b"secret"),
                None,
            )
            .await;
        assert!(matches!(result, Err(ErrorCode::DestinationIpProhibited)));
        assert!(
            tokio::time::timeout(Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
        assert!(budget.charges.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn control_project_reaches_private_worker_endpoints_under_blocked_policy() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let listener_port = listener.local_addr().unwrap().port();
        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buffer = [0_u8; 1024];
            let _ = stream.read(&mut buffer).await.unwrap();
            stream
                .write_all(b"HTTP/1.1 204 No Content\r\n\r\n")
                .await
                .unwrap();
        });
        let outbound_http = GuestOutboundHttp::new(
            OutboundDialer::system(PrivateDestinationAccess::Blocked),
            CountingBudget::with_remaining(0),
            "fn0-control".to_string(),
        );
        let (response, transmit) = outbound_http
            .send(
                "fn0-control",
                request_to(&format!("http://127.0.0.1:{listener_port}/reconcile"), b""),
                None,
            )
            .await
            .unwrap();
        let transmit_task = tokio::spawn(transmit);
        assert_eq!(response.status(), http::StatusCode::NO_CONTENT);
        server_task.await.unwrap();
        transmit_task.abort();
    }

    #[tokio::test]
    async fn known_exhausted_budget_refuses_before_connecting() {
        let budget = Arc::new(CountingBudget {
            remaining_bytes: std::sync::Mutex::new(0),
            charges: std::sync::Mutex::new(Vec::new()),
            exhausted: true,
        });
        let outbound_http = GuestOutboundHttp::new(
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            budget,
            "fn0-control".to_string(),
        );
        let result = outbound_http
            .send("project", request_to("http://127.0.0.1:9/", b""), None)
            .await;
        assert!(matches!(result, Err(ErrorCode::HttpRequestDenied)));
    }

    #[tokio::test]
    async fn allowed_request_body_is_charged_and_delivered() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let listener_port = listener.local_addr().unwrap().port();
        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut received = Vec::new();
            let mut buffer = [0_u8; 1024];
            while !received.ends_with(b"payload") {
                let read_count = stream.read(&mut buffer).await.unwrap();
                if read_count == 0 {
                    break;
                }
                received.extend_from_slice(&buffer[..read_count]);
            }
            stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\nok")
                .await
                .unwrap();
            received
        });
        let budget = CountingBudget::with_remaining(1_000);
        let outbound_http = GuestOutboundHttp::new(
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            budget.clone(),
            "fn0-control".to_string(),
        );
        let (response, transmit) = outbound_http
            .send(
                "project",
                request_to(
                    &format!("http://127.0.0.1:{listener_port}/path"),
                    b"payload",
                ),
                None,
            )
            .await
            .unwrap();
        let transmit_task = tokio::spawn(transmit);
        assert_eq!(response.status(), http::StatusCode::OK);
        let response_body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(response_body, Bytes::from_static(b"ok"));
        let received = server_task.await.unwrap();
        assert!(received.starts_with(b"POST /path HTTP/1.1\r\n"));
        assert_eq!(
            budget.charges.lock().unwrap().as_slice(),
            [("project".to_string(), 7)]
        );
        transmit_task.abort();
    }

    #[tokio::test]
    async fn request_body_over_budget_fails_the_request() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let listener_port = listener.local_addr().unwrap().port();
        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut received = Vec::new();
            let mut buffer = [0_u8; 1024];
            while let Ok(read_count) = stream.read(&mut buffer).await {
                if read_count == 0 {
                    break;
                }
                received.extend_from_slice(&buffer[..read_count]);
            }
            received
        });
        let budget = CountingBudget::with_remaining(3);
        let outbound_http = GuestOutboundHttp::new(
            OutboundDialer::system(PrivateDestinationAccess::Allowed),
            budget,
            "fn0-control".to_string(),
        );
        let result = outbound_http
            .send(
                "project",
                request_to(&format!("http://127.0.0.1:{listener_port}/"), b"payload"),
                None,
            )
            .await;
        assert!(result.is_err());
        let received = server_task.await.unwrap();
        assert!(!received.windows(7).any(|window| window == b"payload"));
    }
}
