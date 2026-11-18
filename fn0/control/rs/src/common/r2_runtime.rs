use crate::common::cloudflare::CachePurgeFile;
use forte_sdk::http;
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct RuntimeGrant {
    url: String,
    project_id: String,
    grant: String,
}

#[derive(Clone, Deserialize)]
pub struct Session {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    pub expires_at: i64,
}

#[derive(Serialize)]
pub struct AssetPresign {
    pub key: String,
    pub method: &'static str,
    pub expires_seconds: u32,
    pub content_length: String,
    pub cache_assets: bool,
}

#[derive(Clone)]
pub struct SessionCache {
    grant: RuntimeGrant,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl RuntimeGrant {
    pub fn new(url: &str, project_id: &str, grant: String) -> anyhow::Result<Self> {
        let host = url
            .strip_prefix("https://")
            .and_then(|value| value.strip_suffix(".workers.dev"));
        let label_is_valid = |label: &str| {
            (1..=63).contains(&label.len())
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        };
        anyhow::ensure!(
            host.and_then(|name| name.split_once('.'))
                .is_some_and(|(worker, account)| {
                    (worker == "fn0-broker" || worker.starts_with("fn0-broker-"))
                        && label_is_valid(worker)
                        && label_is_valid(account)
                }),
            "invalid runtime broker URL"
        );
        anyhow::ensure!(
            grant.len() == 64 && grant.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "invalid runtime broker grant"
        );
        Ok(Self {
            url: url.to_string(),
            project_id: project_id.to_string(),
            grant,
        })
    }

    pub fn storage(self) -> SessionCache {
        SessionCache {
            grant: self,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn post<Body: Serialize, Reply: for<'de> Deserialize<'de>>(
        &self,
        action: &str,
        body: Body,
    ) -> anyhow::Result<Reply> {
        let request = http::Request::builder()
            .method("POST")
            .uri(format!("{}/v1/runtime/{action}", self.url))
            .header("Authorization", format!("Bearer {}", self.grant))
            .header("Content-Type", "application/json")
            .header("User-Agent", "fn0-control/1")
            .body(serde_json::to_vec(&body)?)?;
        let response = http::Client::new().send(request).await?;
        anyhow::ensure!(
            response.status().is_success(),
            "runtime broker {action} returned {}",
            response.status()
        );
        let raw = response.into_body().bytes_limited(512 * 1024).await?;
        serde_json::from_slice(&raw).map_err(|_| anyhow::anyhow!("invalid runtime broker response"))
    }

    async fn purge(&self, action: &str, request: serde_json::Value) -> anyhow::Result<()> {
        #[derive(Deserialize)]
        struct ResultBody {
            ok: bool,
        }
        let result: ResultBody = self.post(action, request).await?;
        anyhow::ensure!(result.ok, "runtime broker rejected {action}");
        Ok(())
    }

    pub async fn verify_purge(&self) -> anyhow::Result<()> {
        self.purge(
            "verify-purge",
            serde_json::json!({"project_id": self.project_id}),
        )
        .await
    }

    pub async fn purge_files(&self, files: &[CachePurgeFile]) -> anyhow::Result<()> {
        self.purge(
            "purge",
            serde_json::json!({"project_id": self.project_id, "files": files}),
        )
        .await
    }

    pub async fn purge_tags(&self, tags: &[&str]) -> anyhow::Result<()> {
        self.purge(
            "purge",
            serde_json::json!({"project_id": self.project_id, "tags": tags}),
        )
        .await
    }
}

impl SessionCache {
    pub async fn presign_assets(
        &self,
        account_id: &str,
        bucket: &str,
        requests: &[AssetPresign],
    ) -> anyhow::Result<Vec<String>> {
        #[derive(Deserialize)]
        struct Reply {
            urls: Vec<String>,
        }
        let expected = format!("https://{bucket}.{account_id}.r2.cloudflarestorage.com/");
        let mut urls = Vec::with_capacity(requests.len());
        for entries in requests.chunks(80) {
            let reply: Reply = self
                .grant
                .post(
                    "presign",
                    serde_json::json!({
                        "project_id": self.grant.project_id,
                        "bucket": bucket,
                        "entries": entries,
                    }),
                )
                .await?;
            anyhow::ensure!(
                reply.urls.len() == entries.len()
                    && reply.urls.iter().all(|url| {
                        url.starts_with(&expected)
                            && !url.to_ascii_lowercase().contains("x-amz-security-token=")
                    }),
                "broker returned invalid asset presigns"
            );
            urls.extend(reply.urls);
        }
        Ok(urls)
    }

    pub async fn lease(&self, bucket: &str, validity: i64) -> anyhow::Result<Session> {
        let mut sessions = self.sessions.lock().await;
        let deadline = || forte_sdk::now().timestamp() + validity;
        if let Some(cached) = sessions.get(bucket)
            && cached.expires_at > deadline() + 30
        {
            return Ok(cached.clone());
        }
        let fresh: anyhow::Result<Session> = self
            .grant
            .post(
                "session",
                serde_json::json!({"project_id": self.grant.project_id, "bucket": bucket, "validity_seconds": validity}),
            )
            .await;
        match fresh {
            Ok(session)
                if session.expires_at > deadline() + 5
                    && !session.session_token.is_empty()
                    && !session.secret_access_key.is_empty()
                    && !session.access_key_id.is_empty() =>
            {
                sessions.insert(bucket.to_string(), session.clone());
                Ok(session)
            }
            _ if let Some(cached) = sessions.get(bucket)
                && cached.expires_at > deadline() + 5 =>
            {
                Ok(cached.clone())
            }
            Ok(_) => anyhow::bail!("runtime broker returned an incomplete or expiring session"),
            Err(error) => Err(error),
        }
    }
}
