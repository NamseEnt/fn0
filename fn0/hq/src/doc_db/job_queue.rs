use super::*;

#[derive(Clone)]
pub struct HqJob {
    pub id: String,
    pub task_name: String,
    pub payload: String,
    pub retry_count: i64,
    pub max_retries: i64,
}

impl DocDb {
    pub async fn ensure_job_tables(&self) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS hq_jobs (id TEXT PRIMARY KEY, task_name TEXT NOT NULL, payload TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending', retry_count INTEGER NOT NULL DEFAULT 0, max_retries INTEGER NOT NULL DEFAULT 3, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
            libsql::params!(),
        )
        .await?;
        Ok(())
    }

    pub async fn claim_job(&self) -> Result<Option<HqJob>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "UPDATE hq_jobs SET status = 'processing', updated_at = datetime('now') WHERE id = (SELECT id FROM hq_jobs WHERE status = 'pending' OR (status = 'processing' AND updated_at < datetime('now', '-60 seconds')) LIMIT 1) RETURNING id, task_name, payload, retry_count, max_retries",
                libsql::params!(),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(HqJob {
            id: row.get(0)?,
            task_name: row.get(1)?,
            payload: row.get(2)?,
            retry_count: row.get(3)?,
            max_retries: row.get(4)?,
        }))
    }

    pub async fn complete_job(&self, id: &str) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute("DELETE FROM hq_jobs WHERE id = ?", libsql::params!(id))
            .await?;
        Ok(())
    }

    pub async fn retry_job(&self, id: &str) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "UPDATE hq_jobs SET status = 'pending', retry_count = retry_count + 1, updated_at = datetime('now') WHERE id = ?",
            libsql::params!(id),
        )
        .await?;
        Ok(())
    }

    pub async fn fail_job(&self, id: &str) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute("DELETE FROM hq_jobs WHERE id = ?", libsql::params!(id))
            .await?;
        Ok(())
    }
}
