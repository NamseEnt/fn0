use forte_sdk::*;
use serde::{Deserialize, Serialize};

const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

pub struct CloudflareSaasClient {
    api_token: String,
    zone_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostnameStatus {
    Pending,
    Active,
    Moved,
    Deleted,
    Blocked,
    Other(String),
}

impl HostnameStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, HostnameStatus::Active)
    }

    fn from_str(value: &str) -> Self {
        match value {
            "pending" | "pending_validation" | "pending_issuance" | "pending_deployment"
            | "pending_deletion" | "pending_blocked" => HostnameStatus::Pending,
            "active" => HostnameStatus::Active,
            "moved" => HostnameStatus::Moved,
            "deleted" => HostnameStatus::Deleted,
            "blocked" => HostnameStatus::Blocked,
            other => HostnameStatus::Other(other.to_string()),
        }
    }
}

pub struct Hostname {
    pub id: String,
    pub status: HostnameStatus,
}

impl CloudflareSaasClient {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            api_token: std::env::var("FN0_CLOUDFLARE_SAAS_API_TOKEN")
                .map_err(|_| anyhow::anyhow!("FN0_CLOUDFLARE_SAAS_API_TOKEN not set"))?,
            zone_id: std::env::var("FN0_CLOUDFLARE_SAAS_ZONE_ID")
                .map_err(|_| anyhow::anyhow!("FN0_CLOUDFLARE_SAAS_ZONE_ID not set"))?,
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
        let bytes = resp.into_body().bytes().await.to_vec();
        Ok((status, bytes))
    }

    pub async fn create_or_adopt_hostname(&self, hostname: &str) -> anyhow::Result<Hostname> {
        #[derive(Serialize)]
        struct CreateBody<'a> {
            hostname: &'a str,
            ssl: Ssl,
        }
        #[derive(Serialize)]
        struct Ssl {
            method: &'static str,
            r#type: &'static str,
        }

        let body = serde_json::to_vec(&CreateBody {
            hostname,
            ssl: Ssl {
                method: "http",
                r#type: "dv",
            },
        })?;
        let path = format!("/zones/{}/custom_hostnames", self.zone_id);
        let (status, response_bytes) = self.call("POST", &path, body).await?;

        if (200..300).contains(&status) {
            return parse_single(&response_bytes);
        }

        if let Some(existing) = self.find_by_name(hostname).await? {
            return Ok(existing);
        }

        Err(anyhow::anyhow!(
            "cloudflare saas create_custom_hostname failed: status={status} body={}",
            String::from_utf8_lossy(&response_bytes)
        ))
    }

    pub async fn find_by_name(&self, hostname: &str) -> anyhow::Result<Option<Hostname>> {
        let path = format!(
            "/zones/{}/custom_hostnames?hostname={}",
            self.zone_id, hostname
        );
        let (status, response_bytes) = self.call("GET", &path, Vec::new()).await?;
        if !(200..300).contains(&status) {
            return Err(anyhow::anyhow!(
                "cloudflare saas list_custom_hostnames failed: status={status} body={}",
                String::from_utf8_lossy(&response_bytes)
            ));
        }
        let parsed: ListResponse = serde_json::from_slice(&response_bytes)?;
        if !parsed.success {
            return Err(anyhow::anyhow!(
                "cloudflare saas list_custom_hostnames not successful: body={}",
                String::from_utf8_lossy(&response_bytes)
            ));
        }
        Ok(parsed
            .result
            .unwrap_or_default()
            .into_iter()
            .next()
            .map(|r| Hostname {
                id: r.id,
                status: HostnameStatus::from_str(&r.status),
            }))
    }

    pub async fn delete_by_name(&self, hostname: &str) -> anyhow::Result<()> {
        let Some(found) = self.find_by_name(hostname).await? else {
            return Ok(());
        };
        let path = format!("/zones/{}/custom_hostnames/{}", self.zone_id, found.id);
        let (status, response_bytes) = self.call("DELETE", &path, Vec::new()).await?;
        if status == 404 {
            return Ok(());
        }
        if !(200..300).contains(&status) {
            if has_error_code(&response_bytes, 1436) {
                return Ok(());
            }
            return Err(anyhow::anyhow!(
                "cloudflare saas delete_custom_hostname failed: status={status} body={}",
                String::from_utf8_lossy(&response_bytes)
            ));
        }
        Ok(())
    }
}

fn parse_single(bytes: &[u8]) -> anyhow::Result<Hostname> {
    let parsed: SingleResponse = serde_json::from_slice(bytes)?;
    if !parsed.success {
        return Err(anyhow::anyhow!(
            "cloudflare saas response not successful: body={}",
            String::from_utf8_lossy(bytes)
        ));
    }
    let result = parsed
        .result
        .ok_or_else(|| anyhow::anyhow!("cloudflare saas response missing result"))?;
    Ok(Hostname {
        id: result.id,
        status: HostnameStatus::from_str(&result.status),
    })
}

fn has_error_code(bytes: &[u8], code: i64) -> bool {
    #[derive(Deserialize)]
    struct Errs {
        #[serde(default)]
        errors: Vec<Err>,
    }
    #[derive(Deserialize)]
    struct Err {
        #[serde(default)]
        code: Option<i64>,
    }
    serde_json::from_slice::<Errs>(bytes)
        .ok()
        .map(|e| {
            e.errors
                .into_iter()
                .filter_map(|e| e.code)
                .any(|c| c == code)
        })
        .unwrap_or(false)
}

#[derive(Deserialize)]
struct SingleResponse {
    success: bool,
    #[serde(default)]
    result: Option<HostnameResult>,
}

#[derive(Deserialize)]
struct ListResponse {
    success: bool,
    #[serde(default)]
    result: Option<Vec<HostnameResult>>,
}

#[derive(Deserialize)]
struct HostnameResult {
    id: String,
    status: String,
}
