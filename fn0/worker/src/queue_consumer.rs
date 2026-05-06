use anyhow::{Result, anyhow};
use base64::Engine;
use bytes::Bytes;
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Full};
use oci_rust_sdk::auth::{RequestSigner, SimpleAuthProvider, SimpleAuthProviderRequiredFields};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

use crate::worker_pool::{self, DispatchError, RequestEnvelope};

const PUT_MESSAGES_API_VERSION: &str = "20210201";
const LONG_POLL_TIMEOUT_SECS: u32 = 30;
const VISIBILITY_SECS: u32 = 60;
const MAX_MESSAGES_PER_FETCH: u32 = 8;

pub struct QueueConsumerConfig {
    pub queue_ocid: String,
    pub messages_endpoint: String,
    pub tenancy: String,
    pub user: String,
    pub fingerprint: String,
    pub private_key_pem: String,
}

impl QueueConsumerConfig {
    pub fn from_env() -> Result<Option<Self>> {
        let Ok(messages_endpoint) = std::env::var("FN0_QUEUE_MESSAGES_ENDPOINT") else {
            return Ok(None);
        };
        let queue_ocid =
            std::env::var("FN0_QUEUE_OCID").map_err(|_| anyhow!("FN0_QUEUE_OCID is required"))?;
        let tenancy = std::env::var("FN0_QUEUE_OCI_TENANCY_ID")
            .map_err(|_| anyhow!("FN0_QUEUE_OCI_TENANCY_ID is required"))?;
        let user = std::env::var("FN0_QUEUE_OCI_USER_ID")
            .map_err(|_| anyhow!("FN0_QUEUE_OCI_USER_ID is required"))?;
        let fingerprint = std::env::var("FN0_QUEUE_OCI_FINGERPRINT")
            .map_err(|_| anyhow!("FN0_QUEUE_OCI_FINGERPRINT is required"))?;
        let private_key_b64 = std::env::var("FN0_QUEUE_OCI_PRIVATE_KEY_BASE64")
            .map_err(|_| anyhow!("FN0_QUEUE_OCI_PRIVATE_KEY_BASE64 is required"))?;
        let private_key_pem = base64::engine::general_purpose::STANDARD
            .decode(private_key_b64.as_bytes())
            .map_err(|e| anyhow!("queue private key base64 decode: {e}"))
            .and_then(|b| {
                String::from_utf8(b).map_err(|e| anyhow!("queue private key utf8: {e}"))
            })?;
        Ok(Some(Self {
            queue_ocid,
            messages_endpoint,
            tenancy,
            user,
            fingerprint,
            private_key_pem,
        }))
    }
}

struct Consumer {
    queue_ocid: String,
    messages_host: String,
    signer: Arc<RequestSigner>,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct GetMessagesResponse {
    #[serde(default)]
    messages: Vec<IncomingMessage>,
}

#[derive(Deserialize)]
struct IncomingMessage {
    receipt: String,
    content: String,
}

#[derive(Deserialize)]
struct WrappedMessage {
    project_id: String,
    task_name: String,
    payload: serde_json::Value,
}

pub async fn run(
    config: QueueConsumerConfig,
    worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
) {
    let consumer = match Consumer::new(config) {
        Ok(c) => c,
        Err(err) => {
            tracing::error!(?err, "queue consumer init failed; consumer disabled");
            return;
        }
    };

    loop {
        match consumer.fetch_messages().await {
            Ok(messages) if messages.is_empty() => {}
            Ok(messages) => {
                for msg in messages {
                    let consumer = &consumer;
                    let worker_senders = worker_senders.clone();
                    if let Err(err) = consumer.dispatch_one(msg, worker_senders).await {
                        tracing::warn!(?err, "queue dispatch failed; leaving for redelivery");
                    }
                }
            }
            Err(err) => {
                tracing::warn!(?err, "queue fetch failed");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

impl Consumer {
    fn new(config: QueueConsumerConfig) -> Result<Self> {
        let url = url::Url::parse(&config.messages_endpoint)
            .map_err(|e| anyhow!("queue endpoint parse: {e}"))?;
        let host = url
            .host_str()
            .ok_or_else(|| anyhow!("queue endpoint missing host"))?;
        let messages_host = if let Some(port) = url.port() {
            format!("{host}:{port}")
        } else {
            host.to_string()
        };

        let provider: Arc<SimpleAuthProvider> = Arc::new(
            SimpleAuthProvider::builder(SimpleAuthProviderRequiredFields {
                tenancy: config.tenancy,
                user: config.user,
                fingerprint: config.fingerprint,
                private_key: config.private_key_pem,
            })
            .build(),
        );
        let signer = Arc::new(
            RequestSigner::new(provider as Arc<dyn oci_rust_sdk::auth::AuthProvider>)
                .map_err(|e| anyhow!("queue signer init failed: {e:?}"))?,
        );

        Ok(Self {
            queue_ocid: config.queue_ocid,
            messages_host,
            signer,
            http: reqwest::Client::new(),
        })
    }

    fn messages_url(&self, suffix: &str) -> String {
        format!(
            "https://{}/{}/queues/{}/messages{}",
            self.messages_host, PUT_MESSAGES_API_VERSION, self.queue_ocid, suffix
        )
    }

    async fn fetch_messages(&self) -> Result<Vec<IncomingMessage>> {
        let url_str = self.messages_url(&format!(
            "?limit={MAX_MESSAGES_PER_FETCH}&timeoutInSeconds={LONG_POLL_TIMEOUT_SECS}&visibilityInSeconds={VISIBILITY_SECS}"
        ));
        let url = url::Url::parse(&url_str).map_err(|e| anyhow!("queue url parse: {e}"))?;

        let mut headers = reqwest::header::HeaderMap::new();
        self.signer
            .sign_request("GET", &url, &mut headers, None)
            .map_err(|e| anyhow!("queue sign: {e:?}"))?;

        let resp = self
            .http
            .get(url_str)
            .headers(headers)
            .send()
            .await
            .map_err(|e| anyhow!("queue GetMessages: {e}"))?;

        let status = resp.status();
        let body_bytes = resp
            .bytes()
            .await
            .map_err(|e| anyhow!("queue GetMessages body: {e}"))?;

        if !status.is_success() {
            return Err(anyhow!(
                "queue GetMessages failed status={status} body={}",
                String::from_utf8_lossy(&body_bytes)
            ));
        }

        let parsed: GetMessagesResponse =
            serde_json::from_slice(&body_bytes).map_err(|e| anyhow!("queue resp parse: {e}"))?;
        Ok(parsed.messages)
    }

    async fn delete_message(&self, receipt: &str) -> Result<()> {
        let url_str = self.messages_url(&format!("/{receipt}"));
        let url = url::Url::parse(&url_str).map_err(|e| anyhow!("queue url parse: {e}"))?;

        let mut headers = reqwest::header::HeaderMap::new();
        self.signer
            .sign_request("DELETE", &url, &mut headers, None)
            .map_err(|e| anyhow!("queue sign: {e:?}"))?;

        let resp = self
            .http
            .delete(url_str)
            .headers(headers)
            .send()
            .await
            .map_err(|e| anyhow!("queue DeleteMessage: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "queue DeleteMessage failed status={status} body={body}"
            ));
        }
        Ok(())
    }

    async fn dispatch_one(
        &self,
        msg: IncomingMessage,
        worker_senders: Arc<Vec<mpsc::Sender<RequestEnvelope>>>,
    ) -> Result<()> {
        let content_bytes = base64::engine::general_purpose::STANDARD
            .decode(msg.content.as_bytes())
            .map_err(|e| anyhow!("queue content base64: {e}"))?;
        let wrapped: WrappedMessage = serde_json::from_slice(&content_bytes)
            .map_err(|e| anyhow!("queue content parse: {e}"))?;

        let inner_body = serde_json::to_vec(&serde_json::json!({
            "task_name": wrapped.task_name,
            "payload": wrapped.payload,
        }))
        .map_err(|e| anyhow!("queue inner body: {e}"))?;

        let req: hyper::Request<UnsyncBoxBody<Bytes, anyhow::Error>> = hyper::Request::builder()
            .method("POST")
            .uri("/__fn0_queue_task/execute")
            .header("host", format!("{}.fn0.dev", wrapped.project_id))
            .header("content-type", "application/json")
            .body(
                Full::new(Bytes::from(inner_body))
                    .map_err(|never: std::convert::Infallible| match never {})
                    .boxed_unsync(),
            )
            .map_err(|e| anyhow!("queue req build: {e}"))?;

        let (resp_tx, resp_rx) = oneshot::channel();
        let envelope = RequestEnvelope {
            code_id: wrapped.project_id.clone(),
            req,
            resp_tx,
        };

        if let Err(err) = worker_pool::dispatch(&worker_senders, envelope) {
            return match err {
                DispatchError::Full => {
                    Err(anyhow!("worker queue full for {}", wrapped.project_id))
                }
                DispatchError::Closed => Err(anyhow!("worker pool closed")),
            };
        }

        let result = match resp_rx.await {
            Ok(r) => r,
            Err(_) => return Err(anyhow!("worker dropped response channel")),
        };

        let resp = result.map_err(|e| anyhow!("queue task exec: {e:?}"))?;
        if !resp.status().is_success() {
            return Err(anyhow!("queue task returned status {}", resp.status()));
        }

        self.delete_message(&msg.receipt).await
    }
}
