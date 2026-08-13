use crate::cache::S3BundleCache;
use crate::storage_resolver::ManifestStorageResolver;
use crate::websocket::WebSocketService;
use doc_db::{Database, DbRequest};
use fn0_shared_schema::{STATIC_CACHE_STATE_ACTIVE, WorkerManifestDocGet};
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

pub async fn run(
    db: Database,
    cache: S3BundleCache,
    storage_resolver: Arc<ManifestStorageResolver>,
    manifest_loaded: Arc<AtomicBool>,
    websocket_service: Arc<WebSocketService>,
) {
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

        // Before registering the projects: a bundle that becomes servable
        // before its storage target is in place would sign its first writes
        // against the platform account.
        storage_resolver
            .apply(
                manifest
                    .project_manifests
                    .iter()
                    .filter_map(|(project_id, project_manifest)| {
                        project_manifest
                            .storage
                            .as_ref()
                            .map(|storage| (project_id, storage))
                    })
                    .collect(),
            )
            .await;

        for (project_id, project_manifest) in &manifest.project_manifests {
            if known_projects
                .get(project_id)
                .is_some_and(|known_version| *known_version != project_manifest.code_version)
            {
                websocket_service.close_project(project_id).await;
            }
            new_projects.insert(project_id.clone(), project_manifest.code_version);
            cache
                .register(
                    project_id,
                    project_manifest.code_version,
                    project_manifest.static_cache_state == STATIC_CACHE_STATE_ACTIVE,
                )
                .await;
            // An empty domain is a pre-rename manifest row; such a project
            // cannot receive a request and must not claim a route.
            if !project_manifest.domain.is_empty() {
                new_domains.insert(project_manifest.domain.clone(), project_id.clone());
                cache
                    .register_domain(&project_manifest.domain, project_id)
                    .await;
            }
        }

        for project_id in known_projects.keys() {
            if !new_projects.contains_key(project_id) {
                websocket_service.close_project(project_id).await;
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
