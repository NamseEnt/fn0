use crate::common::admin;
use fn0_shared_schema::{
    DbRequest, WorkerHostStatusDoc, WorkerHostStatusDocDelete, WorkerHostStatusDocQuery,
};
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

pub const ZOMBIE_TIMEOUT_SECS: i64 = 60;
pub const ORPHAN_DNS_GRACE_SECS: i64 = 300;

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub enum Output {
    Ok {
        scanned_instances_count: u64,
        terminated_instances_count: u64,
        cleaned_dns_records_count: u64,
        orphan_dns_records_cleaned_count: u64,
    },
    Unauthorized,
    Error {
        message: String,
    },
}

pub struct SweepStats {
    pub scanned_instances: u64,
    pub terminated_instances: u64,
    pub cleaned_dns_records: u64,
    pub orphan_dns_records_cleaned: u64,
}

pub async fn run_sweep() -> anyhow::Result<SweepStats> {
    let api_token = std::env::var("FN0_WORKER_DNS_API_TOKEN")
        .map_err(|_| anyhow::anyhow!("FN0_WORKER_DNS_API_TOKEN not set"))?;
    let zone_id = std::env::var("FN0_WORKER_DNS_ZONE_ID")
        .map_err(|_| anyhow::anyhow!("FN0_WORKER_DNS_ZONE_ID not set"))?;
    let hostnames_raw = std::env::var("FN0_WORKER_DNS_HOSTNAMES")
        .map_err(|_| anyhow::anyhow!("FN0_WORKER_DNS_HOSTNAMES not set"))?;
    let hostnames: Vec<String> = hostnames_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if hostnames.is_empty() {
        anyhow::bail!("FN0_WORKER_DNS_HOSTNAMES is empty");
    }

    let db = doc_db::turso();
    let docs: Vec<WorkerHostStatusDoc> = WorkerHostStatusDocQuery {
        host_id: None,
        limit: None,
    }
    .send_with(&db)
    .await?;

    let scanned_instances = docs.len() as u64;
    let now = chrono::Utc::now().timestamp();
    let stale: Vec<WorkerHostStatusDoc> = docs
        .into_iter()
        .filter(|d| now - d.reported_at > ZOMBIE_TIMEOUT_SECS)
        .collect();

    let client = http::Client::new();
    let mut cleaned_dns_records: u64 = 0;

    for d in stale {
        let mut all_hostnames_ok = true;
        let mut deleted_for_host: u64 = 0;
        for hostname in &hostnames {
            match delete_a_records_for_addr(
                &client,
                &api_token,
                &zone_id,
                hostname,
                &d.addr,
            )
            .await
            {
                Ok(n) => {
                    deleted_for_host += n as u64;
                }
                Err(e) => {
                    tracing::error!(
                        host_id = %d.host_id,
                        addr = %d.addr,
                        hostname = %hostname,
                        error = %e,
                        "zombie_sweep dns delete failed",
                    );
                    all_hostnames_ok = false;
                }
            }
        }
        cleaned_dns_records += deleted_for_host;

        if !all_hostnames_ok {
            continue;
        }

        if let Err(e) = (WorkerHostStatusDocDelete {
            host_id: d.host_id.clone(),
        })
        .send_with(&db)
        .await
        {
            tracing::error!(
                host_id = %d.host_id,
                error = %e,
                "zombie_sweep doc delete failed",
            );
            continue;
        }

        tracing::info!(
            host_id = %d.host_id,
            addr = %d.addr,
            deleted_dns_records = deleted_for_host,
            "zombie_sweep reaped host",
        );
    }

    // Reverse sweep: catch A records whose doc was deleted but DNS deregister failed.
    // Order matters — list DNS first, then re-query docs. worker-agent writes doc
    // before DNS register, so a record snapshotted here that has no matching live
    // doc (and predates the grace window) is a true orphan, never a startup race.
    let mut dns_by_hostname: Vec<(String, Vec<DnsRecord>)> = Vec::new();
    for hostname in &hostnames {
        let records = list_a_records_for_hostname(&client, &api_token, &zone_id, hostname).await?;
        dns_by_hostname.push((hostname.clone(), records));
    }
    let live_docs: Vec<WorkerHostStatusDoc> = WorkerHostStatusDocQuery {
        host_id: None,
        limit: None,
    }
    .send_with(&db)
    .await?;
    let live_addrs: HashSet<String> = live_docs.into_iter().map(|d| d.addr).collect();

    let mut orphan_dns_records_cleaned: u64 = 0;
    for (hostname, records) in dns_by_hostname {
        for record in records {
            if live_addrs.contains(&record.content) {
                continue;
            }
            if now - record.created_on.timestamp() < ORPHAN_DNS_GRACE_SECS {
                continue;
            }
            match delete_a_record_by_id(&client, &api_token, &zone_id, &record.id).await {
                Ok(()) => {
                    orphan_dns_records_cleaned += 1;
                    tracing::info!(
                        hostname = %hostname,
                        addr = %record.content,
                        record_id = %record.id,
                        "zombie_sweep deleted orphan A record",
                    );
                }
                Err(e) => {
                    tracing::error!(
                        hostname = %hostname,
                        addr = %record.content,
                        record_id = %record.id,
                        error = %e,
                        "zombie_sweep orphan delete failed",
                    );
                }
            }
        }
    }

    Ok(SweepStats {
        scanned_instances,
        terminated_instances: 0,
        cleaned_dns_records,
        orphan_dns_records_cleaned,
    })
}

#[derive(Deserialize)]
struct DnsListResponse {
    result: Option<Vec<DnsRecord>>,
}

#[derive(Deserialize)]
struct DnsRecord {
    id: String,
    content: String,
    created_on: chrono::DateTime<chrono::Utc>,
}

async fn list_a_records_for_hostname(
    client: &http::Client,
    api_token: &str,
    zone_id: &str,
    hostname: &str,
) -> anyhow::Result<Vec<DnsRecord>> {
    let list_url = format!(
        "{CLOUDFLARE_API_BASE}/zones/{zone_id}/dns_records?type=A&name={hostname}&per_page=100"
    );
    let list_req = http::Request::builder()
        .method("GET")
        .uri(list_url)
        .header("Authorization", format!("Bearer {api_token}"))
        .body(Vec::new())?;
    let list_resp = client.send(list_req).await?;
    if !list_resp.status().is_success() {
        anyhow::bail!("cloudflare list dns_records status {}", list_resp.status());
    }
    let body = list_resp.into_body().bytes().await.to_vec();
    let parsed: DnsListResponse = serde_json::from_slice(&body)?;
    Ok(parsed.result.unwrap_or_default())
}

async fn delete_a_record_by_id(
    client: &http::Client,
    api_token: &str,
    zone_id: &str,
    record_id: &str,
) -> anyhow::Result<()> {
    let delete_url =
        format!("{CLOUDFLARE_API_BASE}/zones/{zone_id}/dns_records/{record_id}");
    let delete_req = http::Request::builder()
        .method("DELETE")
        .uri(delete_url)
        .header("Authorization", format!("Bearer {api_token}"))
        .body(Vec::new())?;
    let resp = client.send(delete_req).await?;
    if !resp.status().is_success() {
        anyhow::bail!(
            "cloudflare delete dns_record {} status {}",
            record_id,
            resp.status()
        );
    }
    Ok(())
}

async fn delete_a_records_for_addr(
    client: &http::Client,
    api_token: &str,
    zone_id: &str,
    hostname: &str,
    addr: &str,
) -> anyhow::Result<usize> {
    let records = list_a_records_for_hostname(client, api_token, zone_id, hostname).await?;
    let mut deleted = 0usize;
    for record in records.into_iter().filter(|r| r.content == addr) {
        delete_a_record_by_id(client, api_token, zone_id, &record.id).await?;
        deleted += 1;
    }
    Ok(deleted)
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }
    match run_sweep().await {
        Ok(stats) => Output::Ok {
            scanned_instances_count: stats.scanned_instances,
            terminated_instances_count: stats.terminated_instances,
            cleaned_dns_records_count: stats.cleaned_dns_records,
            orphan_dns_records_cleaned_count: stats.orphan_dns_records_cleaned,
        },
        Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
