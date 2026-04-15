use libsql::Row;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerTarget {
    pub image_registry: String,
    pub image_repository: String,
    pub image_tag: String,
}

impl WorkerTarget {
    pub fn image_ref(&self) -> String {
        format!(
            "{}/{}:{}",
            self.image_registry, self.image_repository, self.image_tag
        )
    }
}

fn pk(site_name: &str) -> String {
    format!("worker-target:{site_name}")
}

impl DocDb {
    pub async fn get_worker_target(&self, site_name: &str) -> Result<Option<WorkerTarget>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT value FROM docs WHERE pk = ? AND sk = 0",
                libsql::params![pk(site_name)],
            )
            .await?;
        Ok(rows.next().await?.map(|row| row.into()))
    }

    pub async fn set_worker_target(&self, site_name: &str, target: &WorkerTarget) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "REPLACE INTO docs (pk, sk, value) VALUES (?, 0, ?)",
            libsql::params![pk(site_name), serde_json::to_string(target).unwrap()],
        )
        .await?;
        Ok(())
    }
}

impl From<Row> for WorkerTarget {
    fn from(row: Row) -> Self {
        let json: String = row.get(0).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
