use std::num::NonZeroUsize;

use libsql::Row;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Copy, Serialize, Deserialize)]
/// max instances = min(instance_per_gb * memory, instance_per_core * cores)
pub struct ScaleConfig {
    pub instances_per_gb: NonZeroUsize,

    /// Physical core
    pub instances_per_core: NonZeroUsize,

    /// 1~100
    pub scale_out_threshold_percent: NonZeroUsize,

    /// 1~100, scale_in_threshold_percent < scale_out_threshold_percent
    pub scale_in_threshold_percent: NonZeroUsize,

    pub scale_out_cooldown_secs: NonZeroUsize,

    pub scale_in_threshold_ticks: NonZeroUsize,

    pub scale_in_cooldown_secs: NonZeroUsize,

    pub max_hosts: NonZeroUsize,

    /// min_hosts <= max_hosts
    pub min_hosts: NonZeroUsize,
}

impl DocDb {
    pub async fn get_scale_config(&self, site_name: &str) -> Result<Option<ScaleConfig>> {
        let conn = self.db.connect()?;
        let pk = format!("scale-config:{site_name}");
        let mut rows = conn
            .query(
                "SELECT value FROM docs WHERE pk = ? AND sk = 0",
                libsql::params![pk],
            )
            .await?;
        Ok(rows.next().await?.map(|row| row.into()))
    }
}

impl From<Row> for ScaleConfig {
    fn from(row: Row) -> Self {
        let json: String = row.get(0).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
