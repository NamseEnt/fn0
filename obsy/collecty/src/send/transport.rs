use std::sync::Once;
use std::time::Duration;

use bytes::Bytes;
use http::header::{CONTENT_ENCODING, CONTENT_TYPE};
use http::{HeaderValue, Method, Request, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;

use super::{DeliverFuture, Outcome, Shipment, Transport};

/// Which collecty the segment came from, which of its three streams it
/// belongs to, and which segment of that stream it is. Together they are what
/// signy needs to skip what it already stored, so a resend after a crash costs
/// bandwidth and nothing else.
///
/// The signal is here rather than in front of every record because a segment
/// holds one signal's exports and no others. It is the same answer for the
/// whole body, so the body says it once.
pub const SENDER_HEADER: &str = "x-collecty-sender";
pub const SIGNAL_HEADER: &str = "x-collecty-signal";
pub const SEGMENT_HEADER: &str = "x-collecty-segment";
pub const ACCESS_CLIENT_ID_HEADER: &str = "CF-Access-Client-Id";
pub const ACCESS_CLIENT_SECRET_HEADER: &str = "CF-Access-Client-Secret";
const REASON_LIMIT: usize = 512;

type HttpsClient = Client<HttpsConnector<HttpConnector>, Full<Bytes>>;

pub struct HttpTransport {
    client: HttpsClient,
    base: String,
    timeout: Duration,
    access_client_id: Option<String>,
    access_client_secret: Option<String>,
}

impl HttpTransport {
    pub fn new(base: impl Into<String>, timeout: Duration) -> HttpTransport {
        ensure_crypto_provider();
        let connector = default_connector();
        Self::with_connector(base, timeout, connector, None, None)
    }

    pub fn with_access(
        base: impl Into<String>,
        timeout: Duration,
        access_client_id: Option<String>,
        access_client_secret: Option<String>,
    ) -> HttpTransport {
        ensure_crypto_provider();
        let connector = default_connector();
        Self::with_connector(
            base,
            timeout,
            connector,
            access_client_id,
            access_client_secret,
        )
    }

    #[cfg(test)]
    pub(crate) fn with_tls_config(
        base: impl Into<String>,
        timeout: Duration,
        access_client_id: Option<String>,
        access_client_secret: Option<String>,
        tls_config: rustls::ClientConfig,
    ) -> HttpTransport {
        ensure_crypto_provider();
        let connector = HttpsConnectorBuilder::new()
            .with_tls_config(tls_config)
            .https_or_http()
            .enable_http1()
            .wrap_connector(http_connector());
        Self::with_connector(
            base,
            timeout,
            connector,
            access_client_id,
            access_client_secret,
        )
    }

    fn with_connector(
        base: impl Into<String>,
        timeout: Duration,
        connector: HttpsConnector<HttpConnector>,
        access_client_id: Option<String>,
        access_client_secret: Option<String>,
    ) -> HttpTransport {
        let base = normalize_scheme(base.into());
        HttpTransport {
            client: Client::builder(TokioExecutor::new()).build(connector),
            base: base.trim_end_matches('/').to_string(),
            timeout,
            access_client_id,
            access_client_secret,
        }
    }

    pub fn route(&self) -> String {
        format!("{}/signy/api/v1/collect", self.base)
    }
}

static CRYPTO_PROVIDER: Once = Once::new();

fn ensure_crypto_provider() {
    CRYPTO_PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

fn http_connector() -> HttpConnector {
    let mut connector = HttpConnector::new();
    connector.enforce_http(false);
    connector.set_nodelay(true);
    connector
}

fn default_connector() -> HttpsConnector<HttpConnector> {
    HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_or_http()
        .enable_http1()
        .wrap_connector(http_connector())
}

fn normalize_scheme(value: String) -> String {
    let Some(separator) = value.find("://") else {
        return value;
    };
    let (scheme, rest) = value.split_at(separator);
    format!("{}{}", scheme.to_ascii_lowercase(), rest)
}

impl Transport for HttpTransport {
    fn deliver<'a>(&'a self, shipment: Shipment) -> DeliverFuture<'a> {
        Box::pin(async move {
            let uri = self.route();
            let request_builder = Request::builder()
                .method(Method::POST)
                .uri(&uri)
                .header(CONTENT_TYPE, "application/x-protobuf")
                .header(CONTENT_ENCODING, "zstd")
                .header(SENDER_HEADER, shipment.sender.to_string())
                .header(SIGNAL_HEADER, shipment.signal.as_str())
                .header(SEGMENT_HEADER, shipment.segment.to_string());
            let request = match add_access_headers(
                request_builder,
                &self.access_client_id,
                &self.access_client_secret,
            )
            .and_then(|builder| builder.body(Full::new(shipment.body)).map_err(|_| ()))
            {
                Ok(request) => request,
                Err(_) => {
                    return Outcome::Refused(format!("cannot build a request for {uri}"));
                }
            };

            let response =
                match tokio::time::timeout(self.timeout, self.client.request(request)).await {
                    Ok(Ok(response)) => response,
                    Ok(Err(error)) => return Outcome::Retry(format!("{uri}: {error}")),
                    Err(_) => {
                        return Outcome::Retry(format!(
                            "{uri}: no answer within {}ms",
                            self.timeout.as_millis()
                        ));
                    }
                };

            let status = response.status();
            let explanation = response
                .into_body()
                .collect()
                .await
                .map(|collected| {
                    String::from_utf8_lossy(&collected.to_bytes())
                        .trim()
                        .to_string()
                })
                .unwrap_or_default();
            let stored = stored_number(&explanation);
            let explanation = redact(
                &explanation,
                &self.access_client_id,
                &self.access_client_secret,
            )
            .chars()
            .take(REASON_LIMIT)
            .collect::<String>();

            if status.is_success() {
                return Outcome::Accepted(stored);
            }

            let reason = format!("{uri}: {status} {explanation}");

            if refuses_the_payload(status) {
                Outcome::Refused(reason)
            } else {
                Outcome::Retry(reason)
            }
        })
    }
}

fn add_access_headers(
    mut builder: http::request::Builder,
    access_client_id: &Option<String>,
    access_client_secret: &Option<String>,
) -> Result<http::request::Builder, ()> {
    if let Some(access_client_id) = access_client_id {
        builder = builder.header(
            ACCESS_CLIENT_ID_HEADER,
            HeaderValue::try_from(access_client_id).map_err(|_| ())?,
        );
    }
    if let Some(access_client_secret) = access_client_secret {
        builder = builder.header(
            ACCESS_CLIENT_SECRET_HEADER,
            HeaderValue::try_from(access_client_secret).map_err(|_| ())?,
        );
    }
    Ok(builder)
}

fn redact(
    text: &str,
    access_client_id: &Option<String>,
    access_client_secret: &Option<String>,
) -> String {
    let mut redacted = text.to_string();
    let mut credentials = [access_client_id.as_deref(), access_client_secret.as_deref()]
        .into_iter()
        .flatten()
        .filter(|credential| !credential.is_empty())
        .collect::<Vec<_>>();
    credentials.sort_by_key(|credential| std::cmp::Reverse(credential.len()));
    for credential in credentials {
        redacted = redacted.replace(credential, "[REDACTED]");
    }
    redacted
}

/// The segment signy says it now holds whole, out of `{"stored":n}`.
///
/// Zero for an answer that does not say, which leaves the sender committing
/// only the segment it just sent.
fn stored_number(body: &str) -> u64 {
    let Some(at) = body.find("\"stored\"") else {
        return 0;
    };
    let rest = &body[at + "\"stored\"".len()..];
    let Some(colon) = rest.find(':') else {
        return 0;
    };
    rest[colon + 1..]
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

fn refuses_the_payload(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::BAD_REQUEST
            | StatusCode::PAYLOAD_TOO_LARGE
            | StatusCode::UNSUPPORTED_MEDIA_TYPE
            | StatusCode::UNPROCESSABLE_ENTITY
    )
}
