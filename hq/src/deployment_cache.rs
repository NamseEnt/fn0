use crate::telemetry;
use color_eyre::eyre::Result;
use doc_db::{Deployment, DocDb};
use std::{sync::Arc, time::Duration};
use tokio::time::MissedTickBehavior;
use tracing::warn;

#[derive(Clone)]
pub struct DeploymentCache {
    cache: Arc<boxcar::Vec<Deployment>>,
    doc_db: DocDb,
}

impl DeploymentCache {
    pub async fn new(doc_db: DocDb) -> Result<Self> {
        let cache = Arc::new(boxcar::Vec::from_iter(doc_db.all_deployments().await?));
        telemetry::deployment_cache_cached_count(cache.count());
        Ok(Self { cache, doc_db })
    }

    pub async fn run_sync(&self) {
        let mut interval = tokio::time::interval(deployment_id_sync_interval_ms());
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let sk = self.cache.count() as u64;

            let deployments = match self.doc_db.deployments_after(sk).await {
                Ok(stream) => {
                    telemetry::deployment_cache_fetch_status(true);
                    stream
                }
                Err(err) => {
                    warn!(%err, "Failed to fetch deployments");
                    telemetry::deployment_cache_fetch_status(false);
                    continue;
                }
            };

            let new_count = deployments.len();

            if new_count > 0 {
                telemetry::deployment_cache_new_loaded(new_count);
            }

            for deployment in deployments {
                self.cache.push(deployment);
            }

            telemetry::deployment_cache_cached_count(self.cache.count());
        }
    }

    pub fn slice_updates(&self, deployment_id_start_excluded: DeploymentId) -> Vec<Deployment> {
        if deployment_id_start_excluded.0 as usize == self.cache.count() {
            return Vec::new();
        }
        assert!((deployment_id_start_excluded.0 as usize) < self.cache.count());

        let mut last_idx_by_subdomain: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let skip = deployment_id_start_excluded.0 as usize;
        let collected: Vec<Deployment> = self
            .cache
            .iter()
            .skip(skip)
            .map(|(_, d)| d.clone())
            .collect();
        for (idx, d) in collected.iter().enumerate() {
            let subdomain = match d {
                Deployment::Deploy { subdomain, .. } => subdomain,
                Deployment::Undeploy { subdomain } => subdomain,
            };
            last_idx_by_subdomain.insert(subdomain.clone(), idx);
        }
        collected
            .into_iter()
            .enumerate()
            .filter_map(|(idx, d)| {
                let subdomain = match &d {
                    Deployment::Deploy { subdomain, .. } => subdomain,
                    Deployment::Undeploy { subdomain } => subdomain,
                };
                if last_idx_by_subdomain.get(subdomain) == Some(&idx) {
                    Some(d)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn last_deployment_id(&self) -> u64 {
        self.cache.count() as _
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug, Ord, PartialOrd)]
pub struct DeploymentId(pub u64);

fn deployment_id_sync_interval_ms() -> Duration {
    match std::env::var("DEPLOYMENT_ID_SYNC_INTERVAL_MS") {
        Ok(s) => s.parse().map(Duration::from_millis).unwrap_or(Duration::from_secs(2)),
        Err(_) => Duration::from_secs(2),
    }
}
