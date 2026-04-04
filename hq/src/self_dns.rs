use crate::args::SelfDnsArgs;
use color_eyre::eyre::{Result, eyre};
use tracing::*;

pub async fn register(args: SelfDnsArgs) -> Result<()> {
    let public_ip = detect_public_ip().await?;
    info!(%public_ip, hostname = %args.hostname, "Registering self DNS");

    let client = reqwest::Client::new();
    let api_url = "https://api.cloudflare.com/client/v4";

    #[derive(serde::Deserialize)]
    struct ListResponse {
        result: Option<Vec<RecordResult>>,
    }

    #[derive(serde::Deserialize)]
    struct RecordResult {
        id: String,
        content: String,
    }

    let list: ListResponse = client
        .get(format!("{api_url}/zones/{}/dns_records", args.cloudflare_zone_id))
        .header("Authorization", format!("Bearer {}", args.cloudflare_api_token))
        .query(&[
            ("type", "A"),
            ("name", &args.hostname),
        ])
        .send()
        .await?
        .json()
        .await?;

    if let Some(records) = &list.result {
        if let Some(record) = records.first() {
            if record.content == public_ip {
                info!("DNS already correct");
                return Ok(());
            }
            client
                .patch(format!(
                    "{api_url}/zones/{}/dns_records/{}",
                    args.cloudflare_zone_id, record.id
                ))
                .header("Authorization", format!("Bearer {}", args.cloudflare_api_token))
                .json(&serde_json::json!({
                    "content": public_ip,
                }))
                .send()
                .await?;
            info!("DNS updated");
            return Ok(());
        }
    }

    client
        .post(format!("{api_url}/zones/{}/dns_records", args.cloudflare_zone_id))
        .header("Authorization", format!("Bearer {}", args.cloudflare_api_token))
        .json(&serde_json::json!({
            "type": "A",
            "name": args.hostname,
            "content": public_ip,
            "ttl": 60,
            "proxied": false,
        }))
        .send()
        .await?;
    info!("DNS created");

    Ok(())
}

async fn detect_public_ip() -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let ip = client
        .get("https://ifconfig.me")
        .send()
        .await?
        .text()
        .await?
        .trim()
        .to_string();

    if ip.is_empty() {
        return Err(eyre!("Failed to detect public IP"));
    }

    Ok(ip)
}
