//! Fire-and-forget cross-project enqueue hijack.
//!
//! When the allowed caller (e.g. fn0-control) makes an HTTP request to
//! `placeholder_host`, the hijack catches it and routes the payload to
//! either an OCI queue (production) or an in-process loopback channel
//! (local). The caller does not wait for the eventual upstream worker.
//!
//! In contrast to [`crate::CrossProjectInvokeHijack`] (sync, direct
//! dispatch into the worker pool), this hijack is async / queued.

use anyhow::Result;
use base64::Engine;
use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::Full;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::http::uri::Scheme;
use oci_rust_sdk::auth::{RequestSigner, SimpleAuthProvider, SimpleAuthProviderRequiredFields};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

const PUT_MESSAGES_API_VERSION: &str = "20210201";

pub struct CrossProjectEnqueueMessage {
    pub project_id: String,
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
        tx: mpsc::UnboundedSender<CrossProjectEnqueueMessage>,
    },
}

#[derive(Clone)]
pub struct CrossProjectEnqueueHijack {
    pub placeholder_host: String,
    allowed_caller_project_id: String,
    backend: Backend,
}

#[derive(Deserialize)]
struct EnqueueBody {
    project_id: String,
    task_name: String,
    payload: serde_json::Value,
}

#[derive(Serialize)]
struct WrappedMessage<'a> {
    project_id: &'a str,
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

impl CrossProjectEnqueueHijack {
    #[allow(clippy::too_many_arguments)]
    pub fn new_oci(
        placeholder_host: String,
        allowed_caller_project_id: String,
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
                .map_err(|e| anyhow::anyhow!("cross project enqueue signer init failed: {e:?}"))?,
        );

        Ok(Self {
            placeholder_host,
            allowed_caller_project_id,
            backend: Backend::Oci {
                queue_ocid,
                messages_host,
                signer,
            },
        })
    }

    pub fn new_loopback(
        placeholder_host: String,
        allowed_caller_project_id: String,
        tx: mpsc::UnboundedSender<CrossProjectEnqueueMessage>,
    ) -> Self {
        Self {
            placeholder_host,
            allowed_caller_project_id,
            backend: Backend::Loopback { tx },
        }
    }

    pub fn from_env() -> Result<Self> {
        let messages_endpoint = std::env::var("FN0_CROSS_PROJECT_ENQUEUE_MESSAGES_ENDPOINT")
            .map_err(|_| {
                anyhow::anyhow!("FN0_CROSS_PROJECT_ENQUEUE_MESSAGES_ENDPOINT is required")
            })?;
        let placeholder_host = std::env::var("FN0_CROSS_PROJECT_ENQUEUE_PLACEHOLDER_HOST")
            .unwrap_or_else(|_| "fn0-cross-project-enqueue.fn0.dev".to_string());
        let allowed_caller_project_id =
            std::env::var("FN0_CROSS_PROJECT_ENQUEUE_ALLOWED_CALLER_PROJECT_ID").map_err(|_| {
                anyhow::anyhow!("FN0_CROSS_PROJECT_ENQUEUE_ALLOWED_CALLER_PROJECT_ID is required")
            })?;
        let queue_ocid = std::env::var("FN0_CROSS_PROJECT_ENQUEUE_OCID")
            .map_err(|_| anyhow::anyhow!("FN0_CROSS_PROJECT_ENQUEUE_OCID is required"))?;
        let tenancy = std::env::var("FN0_CROSS_PROJECT_ENQUEUE_OCI_TENANCY_ID")
            .map_err(|_| anyhow::anyhow!("FN0_CROSS_PROJECT_ENQUEUE_OCI_TENANCY_ID is required"))?;
        let user = std::env::var("FN0_CROSS_PROJECT_ENQUEUE_OCI_USER_ID")
            .map_err(|_| anyhow::anyhow!("FN0_CROSS_PROJECT_ENQUEUE_OCI_USER_ID is required"))?;
        let fingerprint =
            std::env::var("FN0_CROSS_PROJECT_ENQUEUE_OCI_FINGERPRINT").map_err(|_| {
                anyhow::anyhow!("FN0_CROSS_PROJECT_ENQUEUE_OCI_FINGERPRINT is required")
            })?;
        let private_key_b64 = std::env::var("FN0_CROSS_PROJECT_ENQUEUE_OCI_PRIVATE_KEY_BASE64")
            .map_err(|_| {
                anyhow::anyhow!("FN0_CROSS_PROJECT_ENQUEUE_OCI_PRIVATE_KEY_BASE64 is required")
            })?;
        let private_key_pem = base64::engine::general_purpose::STANDARD
            .decode(private_key_b64.as_bytes())
            .map_err(|e| anyhow::anyhow!("cross project enqueue private key base64 decode: {e}"))
            .and_then(|b| {
                String::from_utf8(b)
                    .map_err(|e| anyhow::anyhow!("cross project enqueue private key utf8: {e}"))
            })?;

        Self::new_oci(
            placeholder_host,
            allowed_caller_project_id,
            queue_ocid,
            messages_endpoint,
            tenancy,
            user,
            fingerprint,
            private_key_pem,
        )
    }

    pub fn placeholder_url(&self) -> String {
        format!("http://{}", self.placeholder_host)
    }

    pub fn allowed_caller_project_id(&self) -> &str {
        &self.allowed_caller_project_id
    }

    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host() == Some(self.placeholder_host.as_str())
    }

    pub(crate) fn handle_enqueue(
        &self,
        caller_project_id: &str,
        body_bytes: &[u8],
    ) -> Result<HijackAction, ErrorCode> {
        if caller_project_id != self.allowed_caller_project_id {
            let resp = hyper::Response::builder()
                .status(403)
                .body(
                    Full::new(Bytes::from_static(b"cross project enqueue forbidden"))
                        .map_err(|never: std::convert::Infallible| match never {})
                        .boxed_unsync(),
                )
                .map_err(|e| ErrorCode::InternalError(Some(format!("synth resp: {e}"))))?;
            return Ok(HijackAction::Synthesized(resp));
        }

        let parsed: EnqueueBody = serde_json::from_slice(body_bytes).map_err(|e| {
            ErrorCode::InternalError(Some(format!("cross project enqueue body parse: {e}")))
        })?;

        match &self.backend {
            Backend::Oci {
                queue_ocid,
                messages_host,
                signer,
            } => {
                let req = build_put_messages_request(queue_ocid, messages_host, signer, &parsed)?;
                Ok(HijackAction::Forward(req))
            }
            Backend::Loopback { tx } => {
                tx.send(CrossProjectEnqueueMessage {
                    project_id: parsed.project_id,
                    task_name: parsed.task_name,
                    payload: parsed.payload,
                })
                .map_err(|_| {
                    ErrorCode::InternalError(Some("cross project enqueue loopback closed".into()))
                })?;
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
    parsed: &EnqueueBody,
) -> Result<hyper::Request<UnsyncBoxBody<Bytes, ErrorCode>>, ErrorCode> {
    let wrapped = WrappedMessage {
        project_id: &parsed.project_id,
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
        .map_err(|e| anyhow::anyhow!("cross project enqueue endpoint parse: {e}"))?;
    let host = url.host_str().ok_or_else(|| {
        anyhow::anyhow!("cross project enqueue endpoint missing host: {endpoint}")
    })?;
    if let Some(port) = url.port() {
        Ok(format!("{host}:{port}"))
    } else {
        Ok(host.to_string())
    }
}
