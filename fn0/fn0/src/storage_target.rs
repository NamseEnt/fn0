//! Per-project resolution of the R2 account, buckets and credentials a
//! request's object-storage traffic is signed against.
//!
//! Every storage hijack used to close over one credential set taken from the
//! process environment, which is correct only while every project's objects
//! live in the platform's own Cloudflare account. Projects that bring their own
//! account need the target chosen inside the request path, in the same trust
//! boundary that already knows the project id.
//!
//! [`StaticResolver`] preserves the single-tenant behaviour for the published
//! crate and for self-hosting: one target, handed to every project.

use serde::Deserialize;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::Mutex;
use wasmtime_wasi_http::p3::bindings::http::types::ErrorCode;

/// Credentials for one R2 endpoint.
#[derive(Clone, PartialEq, Eq)]
pub struct R2Credentials {
    pub endpoint_host: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    broker: Option<Arc<BrokerSessions>>,
}

impl R2Credentials {
    pub fn for_account(
        account_id: &str,
        region: String,
        access_key_id: String,
        secret_access_key: String,
    ) -> Self {
        Self {
            endpoint_host: format!("{account_id}.r2.cloudflarestorage.com"),
            region,
            access_key_id,
            secret_access_key,
            broker: None,
        }
    }

    pub fn for_broker(
        account_id: &str,
        region: String,
        broker_url: &str,
        project_id: &str,
        bucket: &str,
        grant: &str,
    ) -> Self {
        let mut credentials = Self::for_account(account_id, region, String::new(), String::new());
        credentials.broker = Some(Arc::new(BrokerSessions {
            endpoint: format!("{broker_url}/v1/runtime"),
            project_id: project_id.to_string(),
            bucket: bucket.to_string(),
            grant: grant.to_string(),
            cached: Mutex::new(None),
        }));
        credentials
    }

    pub(crate) async fn presign(
        &self,
        request: R2Presign<'_>,
    ) -> Result<Option<String>, ErrorCode> {
        let Some(broker) = &self.broker else {
            return Ok(None);
        };
        broker
            .presign(&self.endpoint_host, request)
            .await
            .map(Some)
            .map_err(|error| {
                tracing::warn!(%error, "could not sign R2 URL through broker");
                ErrorCode::InternalError(Some("R2 presigning unavailable".to_string()))
            })
    }

    pub(crate) async fn lease(&self, validity: u64) -> Result<R2SigningKeys, ErrorCode> {
        let Some(broker) = &self.broker else {
            return Ok(R2SigningKeys {
                access_key_id: self.access_key_id.clone(),
                secret_access_key: self.secret_access_key.clone(),
                session_token: None,
            });
        };
        broker.lease(validity).await.map_err(|error| {
            tracing::warn!(%error, "could not renew R2 session");
            ErrorCode::InternalError(Some("R2 credentials unavailable".to_string()))
        })
    }
}

pub(crate) struct R2Presign<'a> {
    pub(crate) path: &'a str,
    pub(crate) method: &'a str,
    pub(crate) expires: u64,
    pub(crate) content_type: Option<&'a str>,
    pub(crate) content_length: Option<u64>,
}

pub(crate) struct R2SigningKeys {
    pub(crate) access_key_id: String,
    pub(crate) secret_access_key: String,
    pub(crate) session_token: Option<String>,
}

#[derive(Clone, Deserialize)]
struct Session {
    access_key_id: String,
    secret_access_key: String,
    session_token: String,
    expires_at: i64,
}

struct BrokerSessions {
    endpoint: String,
    project_id: String,
    bucket: String,
    grant: String,
    cached: Mutex<Option<Session>>,
}

impl PartialEq for BrokerSessions {
    fn eq(&self, other: &Self) -> bool {
        self.endpoint == other.endpoint
            && self.project_id == other.project_id
            && self.bucket == other.bucket
            && self.grant == other.grant
    }
}

impl Eq for BrokerSessions {}

impl BrokerSessions {
    async fn lease(&self, validity: u64) -> anyhow::Result<R2SigningKeys> {
        let mut cached = self.cached.lock().await;
        let deadline = || chrono::Utc::now().timestamp() + validity as i64;
        if cached
            .as_ref()
            .is_none_or(|session| session.expires_at <= deadline() + 30)
        {
            match self.refresh(validity).await {
                Ok(session) if session.expires_at > deadline() + 5 => *cached = Some(session),
                Ok(_) | Err(_)
                    if cached
                        .as_ref()
                        .is_some_and(|session| session.expires_at > deadline() + 5) => {}
                Ok(_) => anyhow::bail!("broker session expires too soon"),
                Err(error) => return Err(error),
            }
        }
        let session = cached
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing R2 session"))?;
        Ok(R2SigningKeys {
            access_key_id: session.access_key_id.clone(),
            secret_access_key: session.secret_access_key.clone(),
            session_token: Some(session.session_token.clone()),
        })
    }

    async fn refresh(&self, validity: u64) -> anyhow::Result<Session> {
        let session: Session = self.request(
            "session",
            serde_json::json!({
                "project_id": self.project_id, "bucket": self.bucket, "validity_seconds": validity,
            }),
        ).await?.json().await?;
        anyhow::ensure!(
            !session.access_key_id.is_empty()
                && !session.secret_access_key.is_empty()
                && !session.session_token.is_empty(),
            "broker returned incomplete R2 session"
        );
        Ok(session)
    }

    async fn presign(
        &self,
        account_endpoint: &str,
        request: R2Presign<'_>,
    ) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct Reply {
            urls: Vec<String>,
        }
        let response: Reply = self
            .request(
                "presign",
                serde_json::json!({
                    "project_id": self.project_id,
                    "bucket": self.bucket,
                    "entries": [{
                        "key": request.path,
                        "encoded_path": true,
                        "method": request.method,
                        "expires_seconds": request.expires,
                        "content_type": request.content_type,
                        "content_length": request.content_length.map(|length| length.to_string()),
                    }],
                }),
            )
            .await?
            .json()
            .await?;
        let [url]: [String; 1] = response
            .urls
            .try_into()
            .map_err(|_| anyhow::anyhow!("broker returned an invalid presign count"))?;
        let parsed = reqwest::Url::parse(&url)?;
        let expected_host = format!("{}.{account_endpoint}", self.bucket);
        anyhow::ensure!(
            parsed.scheme() == "https"
                && parsed.host_str() == Some(expected_host.as_str())
                && !parsed
                    .query_pairs()
                    .any(|(name, _)| name.eq_ignore_ascii_case("x-amz-security-token")),
            "broker returned an invalid presign URL"
        );
        Ok(url)
    }

    async fn request(
        &self,
        action: &str,
        body: serde_json::Value,
    ) -> anyhow::Result<reqwest::Response> {
        static CLIENT: OnceLock<Result<reqwest::Client, reqwest::Error>> = OnceLock::new();
        let client = CLIENT
            .get_or_init(|| {
                reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .timeout(Duration::from_secs(8))
                    .user_agent("fn0-storage/1")
                    .build()
            })
            .as_ref()
            .map_err(|error| anyhow::anyhow!("broker HTTP client: {error}"))?;
        Ok(client
            .post(format!("{}/{action}", self.endpoint))
            .bearer_auth(&self.grant)
            .json(&body)
            .send()
            .await?
            .error_for_status()?)
    }
}

/// The bucket a project's private objects live in.
#[derive(Clone, PartialEq, Eq)]
pub struct PrivateObjectStorageTarget {
    pub credentials: R2Credentials,
    pub bucket: String,
}

/// The bucket a project's public objects live in, and the origin its CDN serves
/// them from.
#[derive(Clone, PartialEq, Eq)]
pub struct PublicStorageTarget {
    pub credentials: R2Credentials,
    pub bucket: String,
    /// Without a trailing slash.
    pub base_url: String,
}

/// Resolves where a project's private objects live, bucket included. The bucket
/// name is provisioned on the owner's own account and travels with the rest of
/// their configuration; deriving it here instead would put the naming rule in
/// two crates that version apart from each other.
pub trait ObjectStorageResolver: Send + Sync {
    fn resolve(&self, project_id: &str) -> Option<Arc<PrivateObjectStorageTarget>>;
}

pub trait PublicStorageResolver: Send + Sync {
    fn resolve(&self, project_id: &str) -> Option<Arc<PublicStorageTarget>>;
}

/// Hands the same target to every project.
pub struct StaticResolver<T> {
    target: Arc<T>,
}

impl<T> StaticResolver<T> {
    pub fn new(target: T) -> Self {
        Self {
            target: Arc::new(target),
        }
    }
}

impl ObjectStorageResolver for StaticResolver<PrivateObjectStorageTarget> {
    fn resolve(&self, _project_id: &str) -> Option<Arc<PrivateObjectStorageTarget>> {
        Some(self.target.clone())
    }
}

impl PublicStorageResolver for StaticResolver<PublicStorageTarget> {
    fn resolve(&self, _project_id: &str) -> Option<Arc<PublicStorageTarget>> {
        Some(self.target.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn broker_sessions_are_cached_and_renewed_for_longer_urls() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let broker = tokio::spawn(async move {
            for (credential, lifetime) in [("first", 100), ("renewed", 800)] {
                let (mut connection, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    request.push(connection.read_u8().await.unwrap());
                }
                let headers = String::from_utf8(request).unwrap();
                assert!(headers.starts_with("POST /v1/runtime/session HTTP/1.1"));
                assert!(headers.contains("authorization: Bearer scoped-grant"));
                let body = serde_json::json!({
                    "access_key_id": "r2-key",
                    "secret_access_key": "r2-secret",
                    "session_token": credential,
                    "expires_at": chrono::Utc::now().timestamp() + lifetime,
                });
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.to_string().len(),
                );
                connection.write_all(response.as_bytes()).await.unwrap();
                connection.shutdown().await.unwrap();
            }
        });

        let source = R2Credentials::for_broker(
            "account",
            "auto".to_string(),
            &origin,
            "project1",
            "bucket",
            "scoped-grant",
        );
        assert_eq!(
            source.lease(0).await.unwrap().session_token.as_deref(),
            Some("first")
        );
        assert_eq!(
            source.lease(10).await.unwrap().session_token.as_deref(),
            Some("first")
        );
        assert_eq!(
            source.lease(600).await.unwrap().session_token.as_deref(),
            Some("renewed")
        );
        broker.await.unwrap();
    }
}
