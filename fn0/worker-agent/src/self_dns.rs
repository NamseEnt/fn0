use serde::Deserialize;
use std::time::Duration;
use tracing::*;

const CLOUDFLARE_API: &str = "https://api.cloudflare.com/client/v4";
const PUBLIC_IP_PROBE_URL: &str = "https://checkip.amazonaws.com";
const RECORD_TTL_SECONDS: u32 = 60;

pub async fn register() -> Result<(), SelfDnsError> {
    let Some(args) = SelfDnsArgs::from_env()? else {
        warn!("FN0_AGENT_DNS_* env not set; skipping self DNS register");
        return Ok(());
    };
    let public_ip = detect_public_ipv4().await?;
    let client = build_http_client()?;
    let existing = list_a_records(&client, &args).await?;
    if existing.iter().any(|r| r.content == public_ip) {
        info!(%public_ip, hostname = %args.hostname, "self DNS already registered");
        return Ok(());
    }
    create_a_record(&client, &args, &public_ip).await?;
    info!(%public_ip, hostname = %args.hostname, "self DNS registered");
    Ok(())
}

pub async fn deregister() -> Result<(), SelfDnsError> {
    let Some(args) = SelfDnsArgs::from_env()? else {
        return Ok(());
    };
    let public_ip = detect_public_ipv4().await?;
    let client = build_http_client()?;
    let existing = list_a_records(&client, &args).await?;
    let mut deleted = 0usize;
    for record in existing.into_iter().filter(|r| r.content == public_ip) {
        delete_record(&client, &args, &record.id).await?;
        deleted += 1;
    }
    info!(%public_ip, hostname = %args.hostname, deleted, "self DNS deregistered");
    Ok(())
}

async fn list_a_records(
    client: &reqwest::Client,
    args: &SelfDnsArgs,
) -> Result<Vec<DnsRecord>, SelfDnsError> {
    #[derive(Deserialize)]
    struct ListResponse {
        result: Option<Vec<DnsRecord>>,
    }
    let resp: ListResponse = client
        .get(format!(
            "{CLOUDFLARE_API}/zones/{}/dns_records",
            args.zone_id
        ))
        .header("Authorization", format!("Bearer {}", args.api_token))
        .query(&[("type", "A"), ("name", args.hostname.as_str())])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(resp.result.unwrap_or_default())
}

async fn create_a_record(
    client: &reqwest::Client,
    args: &SelfDnsArgs,
    ip: &str,
) -> Result<(), SelfDnsError> {
    client
        .post(format!(
            "{CLOUDFLARE_API}/zones/{}/dns_records",
            args.zone_id
        ))
        .header("Authorization", format!("Bearer {}", args.api_token))
        .json(&serde_json::json!({
            "type": "A",
            "name": args.hostname,
            "content": ip,
            "ttl": RECORD_TTL_SECONDS,
            "proxied": false,
        }))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

async fn delete_record(
    client: &reqwest::Client,
    args: &SelfDnsArgs,
    record_id: &str,
) -> Result<(), SelfDnsError> {
    client
        .delete(format!(
            "{CLOUDFLARE_API}/zones/{}/dns_records/{record_id}",
            args.zone_id
        ))
        .header("Authorization", format!("Bearer {}", args.api_token))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

pub async fn detect_public_ipv4() -> Result<String, SelfDnsError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let ip = client
        .get(PUBLIC_IP_PROBE_URL)
        .send()
        .await?
        .text()
        .await?
        .trim()
        .to_string();
    if ip.is_empty() {
        return Err(SelfDnsError::EmptyPublicIp);
    }
    Ok(ip)
}

fn build_http_client() -> Result<reqwest::Client, SelfDnsError> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?)
}

#[derive(Deserialize, Clone)]
struct DnsRecord {
    id: String,
    content: String,
}

struct SelfDnsArgs {
    api_token: String,
    zone_id: String,
    hostname: String,
}

impl SelfDnsArgs {
    fn from_env() -> Result<Option<Self>, SelfDnsError> {
        let Ok(api_token) = std::env::var("FN0_AGENT_DNS_API_TOKEN") else {
            return Ok(None);
        };
        let zone_id = std::env::var("FN0_AGENT_DNS_ZONE_ID")
            .map_err(|_| SelfDnsError::EnvMissing("FN0_AGENT_DNS_ZONE_ID"))?;
        let hostname = std::env::var("FN0_AGENT_DNS_HOSTNAME")
            .map_err(|_| SelfDnsError::EnvMissing("FN0_AGENT_DNS_HOSTNAME"))?;
        Ok(Some(Self {
            api_token,
            zone_id,
            hostname,
        }))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SelfDnsError {
    #[error("required env var not set: {0}")]
    EnvMissing(&'static str),
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("public IP probe returned empty body")]
    EmptyPublicIp,
}
