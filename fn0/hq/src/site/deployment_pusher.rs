use crate::deployment_cache::DeploymentCache;
use crate::doc_db::Deployment;
use crate::ssh_pool::SshPool;
use base64::Engine;
use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use tracing::*;

#[derive(Serialize)]
struct DeploymentsFile {
    generation: u64,
    deployments: Vec<DeploymentEntry>,
}

#[derive(Serialize)]
struct DeploymentEntry {
    subdomain: String,
    code_id: u64,
    code_version: u64,
}

pub async fn push_to_addr(
    ssh_pool: &SshPool,
    deployment_cache: &DeploymentCache,
    addr: &str,
) -> Result<()> {
    let payload = build_payload(deployment_cache);
    let json = serde_json::to_string(&payload)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(json.as_bytes());

    let cmd = format!(
        "echo '{b64}' | base64 -d > /etc/fn0-worker/deployments.json.tmp && \
         mv /etc/fn0-worker/deployments.json.tmp /etc/fn0-worker/deployments.json"
    );

    let (code, output) = ssh_pool.exec(addr, &cmd).await?;
    if code != 0 {
        return Err(eyre!(
            "deployments.json write failed on {addr}: exit={code}, output={}",
            output.trim()
        ));
    }
    Ok(())
}

pub async fn push_to_all(
    ssh_pool: &SshPool,
    deployment_cache: &DeploymentCache,
    addrs: Vec<String>,
) {
    let mut tasks = Vec::with_capacity(addrs.len());
    for addr in addrs {
        let pool = ssh_pool.clone();
        let cache = deployment_cache.clone();
        tasks.push(tokio::spawn(async move {
            if let Err(err) = push_to_addr(&pool, &cache, &addr).await {
                warn!(%err, %addr, "deployments.json push failed");
            }
        }));
    }
    for t in tasks {
        let _ = t.await;
    }
}

fn build_payload(deployment_cache: &DeploymentCache) -> DeploymentsFile {
    use crate::deployment_cache::DeploymentId;
    let generation = deployment_cache.last_deployment_id();
    let latest_per_subdomain = deployment_cache.slice_updates(DeploymentId(0));
    let deployments = latest_per_subdomain
        .into_iter()
        .filter_map(|d| match d {
            Deployment::Deploy {
                subdomain,
                code_id,
                code_version,
            } => Some(DeploymentEntry {
                subdomain,
                code_id,
                code_version,
            }),
            Deployment::Undeploy { .. } => None,
        })
        .collect();
    DeploymentsFile {
        generation,
        deployments,
    }
}
