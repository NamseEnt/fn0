use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct CloudflareSaas {
    client: reqwest::Client,
    api_url: String,
    zone_id: String,
    api_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomHostnameStatus {
    Pending,
    Active,
    Moved,
    Deleted,
    Blocked,
    Other(String),
}

impl CustomHostnameStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, CustomHostnameStatus::Active)
    }
    fn from_str(s: &str) -> Self {
        match s {
            "pending" | "pending_validation" | "pending_issuance" | "pending_deployment"
            | "pending_deletion" | "pending_blocked" => CustomHostnameStatus::Pending,
            "active" => CustomHostnameStatus::Active,
            "moved" => CustomHostnameStatus::Moved,
            "deleted" => CustomHostnameStatus::Deleted,
            "blocked" => CustomHostnameStatus::Blocked,
            other => CustomHostnameStatus::Other(other.to_string()),
        }
    }
}

pub struct CustomHostname {
    pub id: String,
    pub status: CustomHostnameStatus,
}

impl CloudflareSaas {
    pub fn new(zone_id: String, api_token: String, api_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .local_address("::".parse().ok())
                .build()
                .unwrap(),
            api_url: api_url.unwrap_or_else(|| "https://api.cloudflare.com/client/v4".to_string()),
            zone_id,
            api_token,
        }
    }

    pub async fn create_custom_hostname(&self, hostname: &str) -> Result<CustomHostname> {
        #[derive(Serialize)]
        struct Body<'a> {
            hostname: &'a str,
            ssl: Ssl,
        }
        #[derive(Serialize)]
        struct Ssl {
            method: &'static str,
            r#type: &'static str,
        }

        let url = format!("{}/zones/{}/custom_hostnames", self.api_url, self.zone_id);
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_token)
            .header("Content-Type", "application/json")
            .json(&Body {
                hostname,
                ssl: Ssl {
                    method: "http",
                    r#type: "dv",
                },
            })
            .timeout(DEFAULT_TIMEOUT)
            .send()
            .await?;

        let status_code = resp.status();
        let text = resp.text().await?;
        let parsed: CfResponse = serde_json::from_str(&text)
            .map_err(|e| eyre!("cloudflare_saas create parse failed: {e}: body={text}"))?;
        if !parsed.success {
            return Err(eyre!(
                "cloudflare_saas create_custom_hostname failed (status={status_code}): {text}"
            ));
        }
        let result = parsed
            .result
            .ok_or_else(|| eyre!("cloudflare_saas create: missing result: {text}"))?;
        Ok(CustomHostname {
            id: result.id,
            status: CustomHostnameStatus::from_str(&result.status),
        })
    }

    pub async fn get_custom_hostname(&self, id: &str) -> Result<Option<CustomHostname>> {
        let url = format!(
            "{}/zones/{}/custom_hostnames/{}",
            self.api_url, self.zone_id, id
        );
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_token)
            .timeout(DEFAULT_TIMEOUT)
            .send()
            .await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let text = resp.text().await?;
        let parsed: CfResponse = serde_json::from_str(&text)
            .map_err(|e| eyre!("cloudflare_saas get parse failed: {e}: body={text}"))?;
        if !parsed.success {
            if cf_error_codes(&text).contains(&1436) {
                return Ok(None);
            }
            return Err(eyre!("cloudflare_saas get_custom_hostname failed: {text}"));
        }
        let Some(r) = parsed.result else {
            return Ok(None);
        };
        Ok(Some(CustomHostname {
            id: r.id,
            status: CustomHostnameStatus::from_str(&r.status),
        }))
    }

    pub async fn delete_custom_hostname(&self, id: &str) -> Result<()> {
        let url = format!(
            "{}/zones/{}/custom_hostnames/{}",
            self.api_url, self.zone_id, id
        );
        let resp = self
            .client
            .delete(&url)
            .bearer_auth(&self.api_token)
            .timeout(DEFAULT_TIMEOUT)
            .send()
            .await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }
        let text = resp.text().await?;
        let parsed: CfDeleteResponse = serde_json::from_str(&text)
            .map_err(|e| eyre!("cloudflare_saas delete parse failed: {e}: body={text}"))?;
        if !parsed.success {
            if cf_error_codes(&text).contains(&1436) {
                return Ok(());
            }
            return Err(eyre!(
                "cloudflare_saas delete_custom_hostname failed: {text}"
            ));
        }
        Ok(())
    }

    pub async fn find_custom_hostname_by_name(
        &self,
        hostname: &str,
    ) -> Result<Option<CustomHostname>> {
        let url = format!("{}/zones/{}/custom_hostnames", self.api_url, self.zone_id);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_token)
            .query(&[("hostname", hostname)])
            .timeout(DEFAULT_TIMEOUT)
            .send()
            .await?;
        let text = resp.text().await?;
        let parsed: CfListResponse = serde_json::from_str(&text)
            .map_err(|e| eyre!("cloudflare_saas list parse failed: {e}: body={text}"))?;
        if !parsed.success {
            return Err(eyre!(
                "cloudflare_saas list_custom_hostnames failed: {text}"
            ));
        }
        Ok(parsed
            .result
            .unwrap_or_default()
            .into_iter()
            .next()
            .map(|r| CustomHostname {
                id: r.id,
                status: CustomHostnameStatus::from_str(&r.status),
            }))
    }
}

fn cf_error_codes(body: &str) -> Vec<i64> {
    #[derive(Deserialize)]
    struct Errs {
        errors: Option<Vec<CfErr>>,
    }
    #[derive(Deserialize)]
    struct CfErr {
        code: Option<i64>,
    }
    serde_json::from_str::<Errs>(body)
        .ok()
        .and_then(|e| e.errors)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|e| e.code)
        .collect()
}

#[derive(Deserialize)]
struct CfResponse {
    success: bool,
    #[serde(default)]
    result: Option<CfResult>,
}

#[derive(Deserialize)]
struct CfListResponse {
    success: bool,
    #[serde(default)]
    result: Option<Vec<CfResult>>,
}

#[derive(Deserialize)]
struct CfDeleteResponse {
    success: bool,
}

#[derive(Deserialize)]
struct CfResult {
    id: String,
    status: String,
}
