use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeployJobPhase {
    Queued,
    EnvUploaded,
    CwasmCompiled,
    DbEnsured,
    Versioned,
    Deployed,
    BuildRegistered,
    R2GcEnqueued,
    Pushed,
    Done,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeployJob {
    pub job_id: String,
    pub subdomain: String,
    pub code_id: u64,
    pub build_id: Option<String>,
    pub env_ciphertext: Option<String>,
    pub phase: DeployJobPhase,
    pub code_version: Option<u64>,
    pub old_build_ids: Option<Vec<String>>,
    pub generation: Option<u64>,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub heartbeat_at: Option<String>,
}

impl DeployJob {
    pub fn is_terminal(&self) -> bool {
        matches!(self.phase, DeployJobPhase::Done | DeployJobPhase::Failed)
    }
}

fn pk(job_id: &str) -> String {
    format!("deploy_job:{job_id}")
}

impl DocDb {
    pub async fn insert_deploy_job(&self, job: &DeployJob) -> Result<()> {
        let conn = self.db.connect()?;
        let value = serde_json::to_string(job).expect("DeployJob serializes");
        conn.execute(
            "INSERT INTO docs (pk, sk, value) VALUES (?, 0, ?)",
            libsql::params!(pk(&job.job_id), value),
        )
        .await?;
        Ok(())
    }

    pub async fn get_deploy_job(&self, job_id: &str) -> Result<Option<DeployJob>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT value FROM docs WHERE pk = ? AND sk = 0",
                libsql::params!(pk(job_id)),
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        let value: String = row.get(0)?;
        let job: DeployJob = serde_json::from_str(&value)
            .map_err(|e| libsql::Error::Misuse(format!("bad deploy_job: {e}")))?;
        Ok(Some(job))
    }

    pub async fn update_deploy_job(&self, job: &DeployJob) -> Result<()> {
        let conn = self.db.connect()?;
        let value = serde_json::to_string(job).expect("DeployJob serializes");
        conn.execute(
            "UPDATE docs SET value = ? WHERE pk = ? AND sk = 0",
            libsql::params!(value, pk(&job.job_id)),
        )
        .await?;
        Ok(())
    }

    pub async fn list_active_deploy_jobs(&self) -> Result<Vec<DeployJob>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT value FROM docs WHERE pk LIKE 'deploy_job:%' AND sk = 0",
                libsql::params!(),
            )
            .await?;
        let mut jobs = Vec::new();
        while let Some(row) = rows.next().await? {
            let value: String = row.get(0)?;
            match serde_json::from_str::<DeployJob>(&value) {
                Ok(job) => {
                    if !job.is_terminal() {
                        jobs.push(job);
                    }
                }
                Err(err) => {
                    tracing::warn!(%err, "skipping undecodable deploy_job row");
                }
            }
        }
        Ok(jobs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(job_id: &str) -> DeployJob {
        DeployJob {
            job_id: job_id.to_string(),
            subdomain: "alice-app".to_string(),
            code_id: 7,
            build_id: Some("b1".to_string()),
            env_ciphertext: None,
            phase: DeployJobPhase::Queued,
            code_version: None,
            old_build_ids: None,
            generation: None,
            attempts: 0,
            last_error: None,
            created_at: "2026-04-23T00:00:00Z".to_string(),
            updated_at: "2026-04-23T00:00:00Z".to_string(),
            heartbeat_at: None,
        }
    }

    #[tokio::test]
    async fn insert_get_update_roundtrip() {
        let db = DocDb::new_test_db().await.unwrap();
        let mut job = sample("j1");
        db.insert_deploy_job(&job).await.unwrap();

        let fetched = db.get_deploy_job("j1").await.unwrap().unwrap();
        assert_eq!(fetched.phase, DeployJobPhase::Queued);

        job.phase = DeployJobPhase::Deployed;
        job.code_version = Some(42);
        db.update_deploy_job(&job).await.unwrap();

        let fetched = db.get_deploy_job("j1").await.unwrap().unwrap();
        assert_eq!(fetched.phase, DeployJobPhase::Deployed);
        assert_eq!(fetched.code_version, Some(42));
    }

    #[tokio::test]
    async fn list_active_excludes_terminal() {
        let db = DocDb::new_test_db().await.unwrap();
        let mut running = sample("running");
        let mut done = sample("done");
        done.phase = DeployJobPhase::Done;
        let mut failed = sample("failed");
        failed.phase = DeployJobPhase::Failed;
        db.insert_deploy_job(&running).await.unwrap();
        db.insert_deploy_job(&done).await.unwrap();
        db.insert_deploy_job(&failed).await.unwrap();

        let active = db.list_active_deploy_jobs().await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].job_id, "running");

        running.phase = DeployJobPhase::Done;
        db.update_deploy_job(&running).await.unwrap();
        let active = db.list_active_deploy_jobs().await.unwrap();
        assert!(active.is_empty());
    }
}
