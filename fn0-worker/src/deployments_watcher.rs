use crate::bundle::BundleFetcher;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

const POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Deserialize)]
struct DeploymentsFile {
    generation: u64,
    deployments: Vec<DeploymentEntry>,
}

#[derive(Deserialize)]
struct DeploymentEntry {
    subdomain: String,
    code_id: u64,
    code_version: u64,
}

pub async fn run(
    path: &Path,
    generation: Arc<AtomicU64>,
    bundle_fetcher: Arc<BundleFetcher>,
) {
    let mut last_mtime: Option<SystemTime> = None;
    let mut current: HashMap<String, (u64, u64)> = HashMap::new();

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        let mtime = match tokio::fs::metadata(path).await {
            Ok(m) => m.modified().ok(),
            Err(_) => continue,
        };
        if mtime == last_mtime {
            continue;
        }
        last_mtime = mtime;

        let content = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(err) => {
                tracing::warn!(%err, "deployments.json read failed");
                continue;
            }
        };
        let parsed: DeploymentsFile = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(err) => {
                tracing::warn!(%err, "deployments.json parse failed");
                continue;
            }
        };

        let new_set: HashMap<String, (u64, u64)> = parsed
            .deployments
            .iter()
            .map(|d| (d.subdomain.clone(), (d.code_id, d.code_version)))
            .collect();

        for (sub, ver) in &new_set {
            if current.get(sub) == Some(ver) {
                continue;
            }
            if let Err(err) = bundle_fetcher
                .fetch_and_register(sub, ver.0, ver.1)
                .await
            {
                tracing::error!(%err, subdomain = %sub, "bundle fetch failed");
            }
        }

        for sub in current.keys() {
            if !new_set.contains_key(sub) {
                bundle_fetcher.unregister(sub).await;
            }
        }

        current = new_set;
        generation.store(parsed.generation, Ordering::Relaxed);
    }
}
