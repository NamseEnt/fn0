use forte_sdk::*;
use serde::{Deserialize, Serialize};

const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

pub struct CloudflareClient {
    api_token: String,
    account_id: String,
    zone_id: Option<String>,
}

impl CloudflareClient {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            api_token: std::env::var("FN0_CLOUDFLARE_API_TOKEN")
                .map_err(|_| anyhow::anyhow!("FN0_CLOUDFLARE_API_TOKEN not set"))?,
            account_id: std::env::var("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID")
                .map_err(|_| anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID not set"))?,
            zone_id: std::env::var("FN0_CLOUDFLARE_ZONE_ID").ok(),
        })
    }

    async fn call(
        &self,
        method: &str,
        path: &str,
        body: Vec<u8>,
    ) -> anyhow::Result<(u16, Vec<u8>)> {
        let url = format!("{CLOUDFLARE_API_BASE}{path}");
        let req = http::Request::builder()
            .uri(url)
            .method(method)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .body(body)?;
        let resp = http::Client::new().send(req).await?;
        let status = resp.status().as_u16();
        let body = resp.into_body().bytes().await.to_vec();
        Ok((status, body))
    }

    pub async fn create_r2_bucket(
        &self,
        bucket_name: &str,
        location_hint: &str,
    ) -> anyhow::Result<()> {
        #[derive(Serialize)]
        struct Body<'a> {
            name: &'a str,
            #[serde(rename = "locationHint")]
            location_hint: &'a str,
        }
        let payload = serde_json::to_vec(&Body {
            name: bucket_name,
            location_hint,
        })?;
        let path = format!("/accounts/{}/r2/buckets", self.account_id);
        let (status, body) = self.call("POST", &path, payload).await?;
        if (200..300).contains(&status) {
            return Ok(());
        }
        if response_indicates_already_exists(&body) {
            return Ok(());
        }
        anyhow::bail!(
            "create_r2_bucket {bucket_name} failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    pub async fn r2_bucket_exists(&self, bucket_name: &str) -> anyhow::Result<bool> {
        let path = format!("/accounts/{}/r2/buckets/{}", self.account_id, bucket_name);
        let (status, body) = self.call("GET", &path, Vec::new()).await?;
        if (200..300).contains(&status) {
            return Ok(true);
        }
        if status == 404 {
            return Ok(false);
        }
        anyhow::bail!(
            "r2_bucket_exists {bucket_name} failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    pub async fn delete_r2_bucket(&self, bucket_name: &str) -> anyhow::Result<()> {
        let path = format!("/accounts/{}/r2/buckets/{}", self.account_id, bucket_name);
        let (status, body) = self.call("DELETE", &path, Vec::new()).await?;
        if (200..300).contains(&status) || status == 404 {
            return Ok(());
        }
        anyhow::bail!(
            "delete_r2_bucket {bucket_name} failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    pub async fn put_r2_bucket_cors(
        &self,
        bucket_name: &str,
        methods: &[&str],
        allow_origin: &str,
        expose_headers: &[&str],
    ) -> anyhow::Result<()> {
        #[derive(Serialize)]
        struct Body<'a> {
            rules: Vec<Rule<'a>>,
        }
        #[derive(Serialize)]
        struct Rule<'a> {
            allowed: Allowed<'a>,
            #[serde(rename = "exposeHeaders")]
            expose_headers: Vec<&'a str>,
            #[serde(rename = "maxAgeSeconds")]
            max_age_seconds: u32,
        }
        #[derive(Serialize)]
        struct Allowed<'a> {
            methods: Vec<&'a str>,
            origins: Vec<&'a str>,
            headers: Vec<&'a str>,
        }
        let payload = serde_json::to_vec(&Body {
            rules: vec![Rule {
                allowed: Allowed {
                    methods: methods.to_vec(),
                    origins: vec![allow_origin],
                    headers: vec!["*"],
                },
                expose_headers: expose_headers.to_vec(),
                max_age_seconds: 86400,
            }],
        })?;
        let path = format!(
            "/accounts/{}/r2/buckets/{}/cors",
            self.account_id, bucket_name
        );
        let (status, body) = self.call("PUT", &path, payload).await?;
        if (200..300).contains(&status) {
            return Ok(());
        }
        anyhow::bail!(
            "put_r2_bucket_cors {bucket_name} failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }

    pub async fn list_r2_bucket_names(&self, name_contains: &str) -> anyhow::Result<Vec<String>> {
        #[derive(Deserialize)]
        struct Envelope {
            result: Option<ListResult>,
            #[serde(default)]
            result_info: Option<ListResultInfo>,
        }
        #[derive(Deserialize)]
        struct ListResult {
            buckets: Vec<Bucket>,
        }
        #[derive(Deserialize)]
        struct Bucket {
            name: String,
        }
        #[derive(Deserialize)]
        struct ListResultInfo {
            cursor: Option<String>,
        }

        const PER_PAGE: usize = 1000;
        let mut names = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let mut path = format!(
                "/accounts/{}/r2/buckets?name_contains={name_contains}&per_page={PER_PAGE}",
                self.account_id
            );
            if let Some(cursor) = &cursor {
                path.push_str(&format!("&cursor={cursor}"));
            }
            let (status, body) = self.call("GET", &path, Vec::new()).await?;
            if !(200..300).contains(&status) {
                anyhow::bail!(
                    "list_r2_bucket_names failed (status={status}): {}",
                    String::from_utf8_lossy(&body)
                );
            }
            let envelope: Envelope = serde_json::from_slice(&body)?;
            let buckets = envelope
                .result
                .ok_or_else(|| anyhow::anyhow!("list_r2_bucket_names: missing result"))?
                .buckets;
            let page_len = buckets.len();
            names.extend(buckets.into_iter().map(|b| b.name));
            if page_len < PER_PAGE {
                break;
            }
            match envelope.result_info.and_then(|i| i.cursor) {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        Ok(names)
    }

    pub async fn purge_cache_tags(&self, tags: &[&str]) -> anyhow::Result<()> {
        #[derive(Serialize)]
        struct Body<'a> {
            tags: &'a [&'a str],
        }
        let payload = serde_json::to_vec(&Body { tags })?;
        let zone_id = self
            .zone_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("FN0_CLOUDFLARE_ZONE_ID not set"))?;
        let path = format!("/zones/{zone_id}/purge_cache");
        let (status, body) = self.call("POST", &path, payload).await?;
        if (200..300).contains(&status) {
            return Ok(());
        }
        anyhow::bail!(
            "purge_cache_tags failed (status={status}): {}",
            String::from_utf8_lossy(&body)
        );
    }
}

#[derive(Deserialize)]
struct CloudflareEnvelope {
    #[serde(default)]
    errors: Vec<CloudflareError>,
}

#[derive(Deserialize)]
struct CloudflareError {
    #[serde(default)]
    message: String,
}

fn response_indicates_already_exists(body: &[u8]) -> bool {
    let Ok(env) = serde_json::from_slice::<CloudflareEnvelope>(body) else {
        return false;
    };
    env.errors.iter().any(|e| {
        let m = e.message.to_lowercase();
        m.contains("already exists")
            || m.contains("already configured")
            || m.contains("already in use")
            || m.contains("duplicate")
    })
}
