use anyhow::Result;
use base64::Engine;
use bytes::Bytes;
use dashmap::DashMap;
use http_body_util::BodyExt;
use http_body_util::Full;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::http::uri::Scheme;
use oci_rust_sdk::auth::{RequestSigner, SimpleAuthProvider, SimpleAuthProviderRequiredFields};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

const PUT_MESSAGES_API_VERSION: &str = "20210201";

pub struct LoopbackMessage {
    pub subdomain: String,
    pub task_name: String,
    pub payload: serde_json::Value,
}

#[derive(Clone)]
enum Backend {
    Oci {
        queue_ocid: String,
        messages_host: String,
        signer: Arc<RequestSigner>,
    },
    Loopback {
        tx: mpsc::UnboundedSender<LoopbackMessage>,
    },
}

#[derive(Clone)]
pub struct QueueHijack {
    pub placeholder_host: String,
    backend: Backend,
    usage: Arc<DashMap<String, AtomicU64>>,
}

#[derive(Deserialize)]
struct EnqueueBody {
    task_name: String,
    payload: serde_json::Value,
}

#[derive(Serialize)]
struct WrappedMessage<'a> {
    subdomain: &'a str,
    task_name: &'a str,
    payload: &'a serde_json::Value,
}

#[derive(Serialize)]
struct PutMessagesEntry {
    content: String,
}

#[derive(Serialize)]
struct PutMessagesRequest {
    messages: Vec<PutMessagesEntry>,
}

pub(crate) enum HijackAction {
    Forward(hyper::Request<UnsyncBoxBody<Bytes, ErrorCode>>),
    Synthesized(hyper::Response<UnsyncBoxBody<Bytes, ErrorCode>>),
}

impl QueueHijack {
    pub fn new_oci(
        placeholder_host: String,
        queue_ocid: String,
        messages_endpoint: String,
        tenancy: String,
        user: String,
        fingerprint: String,
        private_key_pem: String,
    ) -> Result<Self> {
        let messages_host = host_from_endpoint(&messages_endpoint)?;

        let provider: Arc<SimpleAuthProvider> = Arc::new(
            SimpleAuthProvider::builder(SimpleAuthProviderRequiredFields {
                tenancy,
                user,
                fingerprint,
                private_key: private_key_pem,
            })
            .build(),
        );
        let signer = Arc::new(
            RequestSigner::new(provider as Arc<dyn oci_rust_sdk::auth::AuthProvider>)
                .map_err(|e| anyhow::anyhow!("queue signer init failed: {e:?}"))?,
        );

        Ok(Self {
            placeholder_host,
            backend: Backend::Oci {
                queue_ocid,
                messages_host,
                signer,
            },
            usage: Arc::new(DashMap::new()),
        })
    }

    pub fn new_loopback(placeholder_host: String, tx: mpsc::UnboundedSender<LoopbackMessage>) -> Self {
        Self {
            placeholder_host,
            backend: Backend::Loopback { tx },
            usage: Arc::new(DashMap::new()),
        }
    }

    pub fn from_env() -> Result<Option<Self>> {
        let Ok(messages_endpoint) = std::env::var("FN0_QUEUE_MESSAGES_ENDPOINT") else {
            return Ok(None);
        };
        let placeholder_host = std::env::var("FN0_QUEUE_PLACEHOLDER_HOST")
            .unwrap_or_else(|_| "fn0-queue.fn0.dev".to_string());
        let queue_ocid = std::env::var("FN0_QUEUE_OCID")
            .map_err(|_| anyhow::anyhow!("FN0_QUEUE_OCID is required"))?;
        let tenancy = std::env::var("FN0_QUEUE_OCI_TENANCY_ID")
            .map_err(|_| anyhow::anyhow!("FN0_QUEUE_OCI_TENANCY_ID is required"))?;
        let user = std::env::var("FN0_QUEUE_OCI_USER_ID")
            .map_err(|_| anyhow::anyhow!("FN0_QUEUE_OCI_USER_ID is required"))?;
        let fingerprint = std::env::var("FN0_QUEUE_OCI_FINGERPRINT")
            .map_err(|_| anyhow::anyhow!("FN0_QUEUE_OCI_FINGERPRINT is required"))?;
        let private_key_b64 = std::env::var("FN0_QUEUE_OCI_PRIVATE_KEY_BASE64")
            .map_err(|_| anyhow::anyhow!("FN0_QUEUE_OCI_PRIVATE_KEY_BASE64 is required"))?;
        let private_key_pem = base64::engine::general_purpose::STANDARD
            .decode(private_key_b64.as_bytes())
            .map_err(|e| anyhow::anyhow!("queue private key base64 decode: {e}"))
            .and_then(|b| {
                String::from_utf8(b).map_err(|e| anyhow::anyhow!("queue private key utf8: {e}"))
            })?;

        Ok(Some(Self::new_oci(
            placeholder_host,
            queue_ocid,
            messages_endpoint,
            tenancy,
            user,
            fingerprint,
            private_key_pem,
        )?))
    }

    pub fn placeholder_url(&self) -> String {
        format!("http://{}", self.placeholder_host)
    }

    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host() == Some(self.placeholder_host.as_str())
    }

    pub(crate) fn record_usage(&self, subdomain: &str) {
        self.usage
            .entry(subdomain.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn usage_count(&self, subdomain: &str) -> u64 {
        self.usage
            .get(subdomain)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    pub(crate) fn handle_enqueue(
        &self,
        subdomain: &str,
        body_bytes: &[u8],
    ) -> Result<HijackAction, ErrorCode> {
        let parsed: EnqueueBody = serde_json::from_slice(body_bytes)
            .map_err(|e| ErrorCode::InternalError(Some(format!("enqueue body parse: {e}"))))?;

        match &self.backend {
            Backend::Oci {
                queue_ocid,
                messages_host,
                signer,
            } => {
                let req = build_put_messages_request(
                    queue_ocid,
                    messages_host,
                    signer,
                    subdomain,
                    &parsed,
                )?;
                Ok(HijackAction::Forward(req))
            }
            Backend::Loopback { tx } => {
                tx.send(LoopbackMessage {
                    subdomain: subdomain.to_string(),
                    task_name: parsed.task_name,
                    payload: parsed.payload,
                })
                .map_err(|_| ErrorCode::InternalError(Some("queue loopback closed".into())))?;
                let resp = hyper::Response::builder()
                    .status(200)
                    .body(
                        Full::new(Bytes::new())
                            .map_err(|never: std::convert::Infallible| match never {})
                            .boxed_unsync(),
                    )
                    .map_err(|e| ErrorCode::InternalError(Some(format!("synth resp: {e}"))))?;
                Ok(HijackAction::Synthesized(resp))
            }
        }
    }
}

fn build_put_messages_request(
    queue_ocid: &str,
    messages_host: &str,
    signer: &RequestSigner,
    subdomain: &str,
    parsed: &EnqueueBody,
) -> Result<hyper::Request<UnsyncBoxBody<Bytes, ErrorCode>>, ErrorCode> {
    let wrapped = WrappedMessage {
        subdomain,
        task_name: &parsed.task_name,
        payload: &parsed.payload,
    };
    let wrapped_json = serde_json::to_string(&wrapped)
        .map_err(|e| ErrorCode::InternalError(Some(format!("wrap message: {e}"))))?;
    let content = base64::engine::general_purpose::STANDARD.encode(wrapped_json.as_bytes());

    let put_body = serde_json::to_vec(&PutMessagesRequest {
        messages: vec![PutMessagesEntry { content }],
    })
    .map_err(|e| ErrorCode::InternalError(Some(format!("put body serialize: {e}"))))?;

    let path = format!("/{PUT_MESSAGES_API_VERSION}/queues/{queue_ocid}/messages");
    let url_str = format!("https://{messages_host}{path}");
    let url = url::Url::parse(&url_str)
        .map_err(|e| ErrorCode::InternalError(Some(format!("queue url parse: {e}"))))?;

    let mut headers = reqwest::header::HeaderMap::new();
    signer
        .sign_request("POST", &url, &mut headers, Some(&put_body))
        .map_err(|e| ErrorCode::InternalError(Some(format!("queue sign: {e:?}"))))?;

    let uri = hyper::Uri::builder()
        .scheme(Scheme::HTTPS)
        .authority(messages_host)
        .path_and_query(path.as_str())
        .build()
        .map_err(|_| ErrorCode::HttpRequestUriInvalid)?;

    let mut builder = hyper::Request::builder().method("POST").uri(uri);
    for (name, value) in headers.iter() {
        builder = builder.header(name.as_str(), value);
    }

    let body: UnsyncBoxBody<Bytes, ErrorCode> = Full::new(Bytes::from(put_body))
        .map_err(|never: std::convert::Infallible| match never {})
        .boxed_unsync();

    builder
        .body(body)
        .map_err(|e| ErrorCode::InternalError(Some(format!("queue request build: {e}"))))
}

fn host_from_endpoint(endpoint: &str) -> Result<String> {
    let url = url::Url::parse(endpoint)
        .map_err(|e| anyhow::anyhow!("queue endpoint parse: {e}"))?;
    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("queue endpoint missing host: {endpoint}"))?;
    if let Some(port) = url.port() {
        Ok(format!("{host}:{port}"))
    } else {
        Ok(host.to_string())
    }
}
