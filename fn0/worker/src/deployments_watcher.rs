use crate::cache::S3BundleCache;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

const POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Deserialize, Clone)]
struct DeploymentsFile {
    generation: u64,
    deployments: Vec<DeploymentEntry>,
    #[serde(default)]
    custom_domains: BTreeMap<String, String>,
}

#[derive(Deserialize, Clone)]
struct DeploymentEntry {
    subdomain: String,
    code_id: u64,
    code_version: u64,
}

pub async fn run(path: &Path, generation: Arc<AtomicU64>, cache: S3BundleCache) {
    let mut last_mtime: Option<SystemTime> = None;
    let mut known: HashMap<String, (u64, u64)> = HashMap::new();
    let mut known_domains: BTreeMap<String, String> = BTreeMap::new();

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        let mtime = match tokio::fs::metadata(path).await {
            Ok(m) => m.modified().ok(),
            Err(_) => continue,
        };

        if mtime == last_mtime {
            continue;
        }

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
        last_mtime = mtime;

        let target: HashMap<String, (u64, u64)> = parsed
            .deployments
            .iter()
            .map(|d| (d.subdomain.clone(), (d.code_id, d.code_version)))
            .collect();

        for (sub, ver) in &target {
            cache.register(sub, ver.0, ver.1).await;
        }

        let stale: Vec<String> = known
            .keys()
            .filter(|s| !target.contains_key(*s))
            .cloned()
            .collect();
        for sub in stale {
            cache.unregister(&sub).await;
        }

        for (domain, subdomain) in &parsed.custom_domains {
            cache.register_domain(domain, subdomain).await;
        }
        let stale_domains: Vec<String> = known_domains
            .keys()
            .filter(|d| !parsed.custom_domains.contains_key(*d))
            .cloned()
            .collect();
        for d in stale_domains {
            cache.unregister_domain(&d).await;
        }

        known = target;
        known_domains = parsed.custom_domains;
        generation.store(parsed.generation, Ordering::Relaxed);
    }
}
