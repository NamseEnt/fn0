use crate::doc_db::DocDb;
use color_eyre::eyre::Result;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::MissedTickBehavior;
use tracing::warn;

#[derive(Clone, Default)]
pub struct CustomDomainSnapshot {
    pub domain_to_subdomain: BTreeMap<String, String>,
}

#[derive(Clone)]
pub struct CustomDomainCache {
    inner: Arc<RwLock<CustomDomainSnapshot>>,
    doc_db: DocDb,
}

impl CustomDomainCache {
    pub async fn new(doc_db: DocDb) -> Result<Self> {
        let cache = Self {
            inner: Arc::new(RwLock::new(CustomDomainSnapshot::default())),
            doc_db,
        };
        cache.refresh().await;
        Ok(cache)
    }

    pub async fn refresh(&self) {
        match self.doc_db.list_custom_domains().await {
            Ok(entries) => {
                let mut map = BTreeMap::new();
                for e in entries {
                    map.insert(e.domain, e.subdomain);
                }
                let mut w = self.inner.write().await;
                w.domain_to_subdomain = map;
            }
            Err(err) => {
                warn!(%err, "custom_domain_cache refresh failed");
            }
        }
    }

    pub async fn snapshot(&self) -> CustomDomainSnapshot {
        self.inner.read().await.clone()
    }

    pub async fn run_sync(&self) {
        let mut interval = tokio::time::interval(sync_interval());
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            self.refresh().await;
        }
    }
}

fn sync_interval() -> Duration {
    match std::env::var("CUSTOM_DOMAIN_SYNC_INTERVAL_MS") {
        Ok(s) => s
            .parse()
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_secs(2)),
        Err(_) => Duration::from_secs(2),
    }
}
