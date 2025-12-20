use color_eyre::eyre::Result;
use futures::TryStreamExt;
use std::{sync::Arc, time::Duration};
use tokio::time::MissedTickBehavior;
use tracing::warn;

#[derive(Clone)]
pub struct DeploymentDb {
    cache: Arc<boxcar::Vec<Deployment>>,
    pool: sqlx::SqlitePool,
}

impl DeploymentDb {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect(url).await?;
        let cache = Arc::new(load_all(&pool).await?);
        Ok(Self { cache, pool })
    }

    pub async fn run_sync(&self) {
        let mut interval = tokio::time::interval(deployment_id_sync_interval_ms());
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            let mut stream = sqlx::query_as::<_, DeploymentRow>(
                "SELECT * FROM deployments WHERE id > ? ORDER BY id ASC",
            )
            .bind(self.cache.count() as i64)
            .fetch(&self.pool);

            loop {
                match stream.try_next().await {
                    Ok(Some(deployment)) => {
                        assert_eq!(deployment.id, self.cache.count() as u64);

                        self.cache.push(Deployment {
                            code_id: deployment.code_id,
                            code_version: deployment.code_version,
                        });
                    }
                    Ok(None) => break,
                    Err(err) => {
                        warn!(%err, "Failed to fetch deployments");
                        break;
                    }
                }
            }
        }
    }

    pub fn slice_updates(&self, deployment_id_start_excluded: DeploymentId) -> Vec<Deployment> {
        assert!(deployment_id_start_excluded.0 as usize <= self.cache.count());
        self.cache
            .iter()
            .skip(deployment_id_start_excluded.0 as usize)
            .map(|(_, deployment)| *deployment)
            .collect()
    }

    pub fn last_deployment_id(&self) -> u64 {
        self.cache.count() as _
    }
}

#[derive(sqlx::FromRow, serde::Deserialize)]
struct DeploymentRow {
    id: u64,
    code_id: u64,
    code_version: u64,
}

async fn load_all(pool: &sqlx::SqlitePool) -> Result<boxcar::Vec<Deployment>> {
    let mut out = Vec::new();

    let mut stream =
        sqlx::query_as::<_, DeploymentRow>("SELECT * FROM deployments ORDER BY id ASC").fetch(pool);

    while let Some(deployment) = stream.try_next().await? {
        out.push(Deployment {
            code_id: deployment.code_id,
            code_version: deployment.code_version,
        });
    }

    Ok(boxcar::Vec::from_iter(out))
}

#[derive(Clone, Copy)]
pub struct Deployment {
    pub code_id: u64,
    pub code_version: u64,
}

#[derive(Eq, Hash, PartialEq, Clone, Debug, Ord, PartialOrd)]
pub struct DeploymentId(pub u64);

fn deployment_id_sync_interval_ms() -> Duration {
    match std::env::var("DEPLOYMENT_ID_SYNC_INTERVAL_MS") {
        Ok(s) => match s.parse() {
            Ok(v) => return Duration::from_millis(v),
            Err(err) => warn!(%err, "DEPLOYMENT_ID_SYNC_INTERVAL_MS is not a valid number"),
        },
        Err(err) => warn!(%err, "Fail to get DEPLOYMENT_ID_SYNC_INTERVAL_MS from env"),
    }
    Duration::from_secs(2)
}
