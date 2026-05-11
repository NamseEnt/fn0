use crate::cache::S3BundleCache;
use doc_db::{DbRequest, Database};
use fn0_shared_schema::WorkerManifestDocGet;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_secs(1);

pub fn build_database_from_env() -> anyhow::Result<Database> {
    let group_token = std::env::var("TURSO_GROUP_TOKEN")
        .map_err(|_| anyhow::anyhow!("TURSO_GROUP_TOKEN not set"))?;
    let host_suffix = std::env::var("TURSO_DB_HOST_SUFFIX")
        .map_err(|_| anyhow::anyhow!("TURSO_DB_HOST_SUFFIX not set"))?;
    let url = format!("https://fn0-control{host_suffix}");
    Ok(doc_db::turso_with_config(url, group_token))
}

pub async fn run(db: Database, cache: S3BundleCache, manifest_loaded: Arc<AtomicBool>) {
    let mut last_version: Option<u64> = None;
    let mut known_projects: HashMap<String, u64> = HashMap::new();
    let mut known_domains: HashMap<String, String> = HashMap::new();

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        let manifest = match (WorkerManifestDocGet {}).send_with(&db).await {
            Ok(Some(m)) => m,
            Ok(None) => continue,
            Err(err) => {
                tracing::warn!(%err, "manifest fetch failed");
                continue;
            }
        };

        if last_version == Some(manifest.manifest_version) {
            continue;
        }

        let mut new_projects: HashMap<String, u64> = HashMap::new();
        let mut new_domains: HashMap<String, String> = HashMap::new();

        for (project_id, project_manifest) in &manifest.project_manifests {
            new_projects.insert(project_id.clone(), project_manifest.code_version);
            cache
                .register(project_id, project_manifest.code_version)
                .await;
            if let Some(domain) = &project_manifest.custom_domain {
                new_domains.insert(domain.clone(), project_id.clone());
                cache.register_domain(domain, project_id).await;
            }
        }

        for project_id in known_projects.keys() {
            if !new_projects.contains_key(project_id) {
                cache.unregister(project_id).await;
            }
        }
        for domain in known_domains.keys() {
            if !new_domains.contains_key(domain) {
                cache.unregister_domain(domain).await;
            }
        }

        known_projects = new_projects;
        known_domains = new_domains;
        last_version = Some(manifest.manifest_version);
        manifest_loaded.store(true, Ordering::Release);

        tracing::info!(
            manifest_version = manifest.manifest_version,
            projects = known_projects.len(),
            domains = known_domains.len(),
            "manifest applied"
        );
    }
}
