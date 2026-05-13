use serde::Deserialize;
use std::time::Duration;
use tracing::*;

const CLOUDFLARE_API: &str = "https://api.cloudflare.com/client/v4";
const PUBLIC_IP_PROBE_URL: &str = "https://checkip.amazonaws.com";
const RECORD_TTL_SECONDS: u32 = 60;

pub async fn register() -> Result<(), DnsRegisterError> {
    let args = DnsRegisterArgs::from_env();
    let public_ip = detect_public_ipv4().await?;
    let client = build_http_client()?;
    for hostname in &args.hostnames {
        let existing = list_a_records(&client, &args, hostname).await?;
        if existing.iter().any(|r| r.content == public_ip) {
            info!(%public_ip, %hostname, "DNS already registered");
            continue;
        }
        create_a_record(&client, &args, hostname, &public_ip).await?;
        info!(%public_ip, %hostname, "DNS registered");
    }
    Ok(())
}

pub async fn deregister() -> Result<(), DnsRegisterError> {
    let args = DnsRegisterArgs::from_env();
    let public_ip = detect_public_ipv4().await?;
    let client = build_http_client()?;
    for hostname in &args.hostnames {
        let existing = list_a_records(&client, &args, hostname).await?;
        let mut deleted = 0usize;
        for record in existing.into_iter().filter(|r| r.content == public_ip) {
            delete_record(&client, &args, &record.id).await?;
            deleted += 1;
        }
        info!(%public_ip, %hostname, deleted, "DNS deregistered");
    }
    Ok(())
}

async fn list_a_records(
    client: &reqwest::Client,
    args: &DnsRegisterArgs,
    hostname: &str,
) -> Result<Vec<DnsRecord>, DnsRegisterError> {
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
        .query(&[("type", "A"), ("name", hostname)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(resp.result.unwrap_or_default())
}

async fn create_a_record(
    client: &reqwest::Client,
    args: &DnsRegisterArgs,
    hostname: &str,
    ip: &str,
) -> Result<(), DnsRegisterError> {
    client
        .post(format!(
            "{CLOUDFLARE_API}/zones/{}/dns_records",
            args.zone_id
        ))
        .header("Authorization", format!("Bearer {}", args.api_token))
        .json(&serde_json::json!({
            "type": "A",
            "name": hostname,
            "content": ip,
            "ttl": RECORD_TTL_SECONDS,
            "proxied": true,
        }))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

async fn delete_record(
    client: &reqwest::Client,
    args: &DnsRegisterArgs,
    record_id: &str,
) -> Result<(), DnsRegisterError> {
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

pub async fn detect_public_ipv4() -> Result<String, DnsRegisterError> {
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
        return Err(DnsRegisterError::EmptyPublicIp);
    }
    Ok(ip)
}

fn build_http_client() -> Result<reqwest::Client, DnsRegisterError> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?)
}

#[derive(Deserialize, Clone)]
struct DnsRecord {
    id: String,
    content: String,
}

struct DnsRegisterArgs {
    api_token: String,
    zone_id: String,
    hostnames: Vec<String>,
}

impl DnsRegisterArgs {
    fn from_env() -> Self {
        let raw = std::env::var("FN0_AGENT_DNS_HOSTNAMES")
            .expect("FN0_AGENT_DNS_HOSTNAMES must be set");
        let hostnames: Vec<String> = raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if hostnames.is_empty() {
            panic!("FN0_AGENT_DNS_HOSTNAMES is empty");
        }
        Self {
            api_token: std::env::var("FN0_AGENT_DNS_API_TOKEN")
                .expect("FN0_AGENT_DNS_API_TOKEN must be set"),
            zone_id: std::env::var("FN0_AGENT_DNS_ZONE_ID")
                .expect("FN0_AGENT_DNS_ZONE_ID must be set"),
            hostnames,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DnsRegisterError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("public IP probe returned empty body")]
    EmptyPublicIp,
}
