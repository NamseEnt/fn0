use super::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    Idle,
    Running,
    Prepared,
    Failed,
}

impl MigrationStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Prepared => "prepared",
            Self::Failed => "failed",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "running" => Self::Running,
            "prepared" => Self::Prepared,
            "failed" => Self::Failed,
            _ => Self::Idle,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompiledArtifactStatus {
    Pending,
    Processing,
    Ready,
    Failed,
}

impl CompiledArtifactStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "processing" => Self::Processing,
            "ready" => Self::Ready,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MigrationState {
    pub active_wasmtime_id: String,
    pub preparing_wasmtime_id: Option<String>,
    pub migration_status: MigrationStatus,
    pub updated_at: String,
}

#[derive(Clone, Debug)]
pub struct LatestCode {
    pub code_id: u64,
    pub subdomain: String,
    pub latest_code_version: u64,
    pub latest_source_bundle_key: String,
    pub updated_at: String,
}

#[derive(Clone, Debug)]
pub struct DeployUploadSession {
    pub deploy_job_id: String,
    pub code_id: u64,
    pub subdomain: String,
    pub source_bundle_key: String,
}

#[derive(Clone, Debug)]
pub struct MigrationProgress {
    pub state: MigrationState,
    pub total_latest_codes: u64,
    pub ready_count: u64,
    pub failed_count: u64,
    pub retrying_count: u64,
    pub recent_errors: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompileJobPayload {
    pub code_id: u64,
    pub subdomain: String,
    pub code_version: u64,
    pub source_bundle_key: String,
    pub target_wasmtime_id: String,
    pub output_artifact_key: String,
}

impl CompileJobPayload {
    pub fn job_id(&self) -> String {
        format!(
            "compile:{}:{}:{}",
            self.code_id, self.code_version, self.target_wasmtime_id
        )
    }
}

impl DocDb {
    pub async fn ensure_wasmtime_tables(&self, initial_active_wasmtime_id: &str) -> Result<()> {
        let conn = self.db.connect()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS deploy_upload_sessions (deploy_job_id TEXT PRIMARY KEY, code_id INTEGER NOT NULL, subdomain TEXT NOT NULL, source_bundle_key TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
            libsql::params!(),
        )
        .await?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS latest_code_versions (code_id INTEGER PRIMARY KEY, subdomain TEXT NOT NULL, latest_code_version INTEGER NOT NULL, latest_source_bundle_key TEXT NOT NULL, updated_at TEXT NOT NULL)",
            libsql::params!(),
        )
        .await?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS compiled_artifacts (code_id INTEGER NOT NULL, code_version INTEGER NOT NULL, wasmtime_id TEXT NOT NULL, status TEXT NOT NULL, artifact_key TEXT NOT NULL, attempt_count INTEGER NOT NULL DEFAULT 0, last_error TEXT, updated_at TEXT NOT NULL, PRIMARY KEY (code_id, code_version, wasmtime_id))",
            libsql::params!(),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_compiled_artifacts_wasmtime_status ON compiled_artifacts (wasmtime_id, status)",
            libsql::params!(),
        )
        .await?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS migration_state (singleton_key INTEGER PRIMARY KEY CHECK (singleton_key = 1), active_wasmtime_id TEXT NOT NULL, preparing_wasmtime_id TEXT, migration_status TEXT NOT NULL, updated_at TEXT NOT NULL)",
            libsql::params!(),
        )
        .await?;

        conn.execute(
            "INSERT OR IGNORE INTO migration_state (singleton_key, active_wasmtime_id, preparing_wasmtime_id, migration_status, updated_at) VALUES (1, ?, NULL, ?, datetime('now'))",
            libsql::params!(initial_active_wasmtime_id, MigrationStatus::Idle.as_str()),
        )
        .await?;

        Ok(())
    }

    pub async fn insert_deploy_upload_session(
        &self,
        deploy_job_id: &str,
        code_id: u64,
        subdomain: &str,
        source_bundle_key: &str,
    ) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "INSERT OR REPLACE INTO deploy_upload_sessions (deploy_job_id, code_id, subdomain, source_bundle_key, created_at, updated_at) VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))",
            libsql::params!(deploy_job_id, code_id, subdomain, source_bundle_key),
        )
        .await?;
        Ok(())
    }

    pub async fn get_deploy_upload_session(
        &self,
        deploy_job_id: &str,
    ) -> Result<Option<DeployUploadSession>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT deploy_job_id, code_id, subdomain, source_bundle_key FROM deploy_upload_sessions WHERE deploy_job_id = ?",
                libsql::params!(deploy_job_id),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(DeployUploadSession {
            deploy_job_id: row.get(0)?,
            code_id: row.get(1)?,
            subdomain: row.get(2)?,
            source_bundle_key: row.get(3)?,
        }))
    }

    pub async fn list_latest_codes(&self) -> Result<Vec<LatestCode>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT code_id, subdomain, latest_code_version, latest_source_bundle_key, updated_at FROM latest_code_versions ORDER BY code_id ASC",
                libsql::params!(),
            )
            .await?;

        let mut items = Vec::new();
        while let Some(row) = rows.next().await? {
            items.push(LatestCode {
                code_id: row.get(0)?,
                subdomain: row.get(1)?,
                latest_code_version: row.get(2)?,
                latest_source_bundle_key: row.get(3)?,
                updated_at: row.get(4)?,
            });
        }
        Ok(items)
    }

    pub async fn get_latest_code_version(&self, code_id: u64) -> Result<Option<u64>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT latest_code_version FROM latest_code_versions WHERE code_id = ?",
                libsql::params!(code_id),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(row.get(0)?))
    }

    pub async fn get_migration_state(&self) -> Result<MigrationState> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT active_wasmtime_id, preparing_wasmtime_id, migration_status, updated_at FROM migration_state WHERE singleton_key = 1",
                libsql::params!(),
            )
            .await?;

        let row = rows.next().await?.expect("migration_state must be initialized");

        let status: String = row.get(2)?;
        Ok(MigrationState {
            active_wasmtime_id: row.get(0)?,
            preparing_wasmtime_id: row.get(1)?,
            migration_status: MigrationStatus::from_str(&status),
            updated_at: row.get(3)?,
        })
    }

    pub async fn compare_and_start_migration(&self, target_wasmtime_id: &str) -> Result<bool> {
        let conn = self.db.connect()?;
        conn.execute(
            "UPDATE migration_state SET preparing_wasmtime_id = ?, migration_status = ?, updated_at = datetime('now') WHERE singleton_key = 1 AND migration_status != ?",
            libsql::params!(
                target_wasmtime_id,
                MigrationStatus::Running.as_str(),
                MigrationStatus::Running.as_str()
            ),
        )
        .await?;

        let mut rows = conn
            .query(
                "SELECT preparing_wasmtime_id, migration_status FROM migration_state WHERE singleton_key = 1",
                libsql::params!(),
            )
            .await?;
        let row = rows.next().await?.expect("migration_state must exist");
        let preparing: Option<String> = row.get(0)?;
        let status: String = row.get(1)?;
        Ok(
            preparing.as_deref() == Some(target_wasmtime_id)
                && MigrationStatus::from_str(&status) == MigrationStatus::Running,
        )
    }

    pub async fn mark_migration_prepared(&self, target_wasmtime_id: &str) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "UPDATE migration_state SET migration_status = ?, updated_at = datetime('now') WHERE singleton_key = 1 AND preparing_wasmtime_id = ?",
            libsql::params!(MigrationStatus::Prepared.as_str(), target_wasmtime_id),
        )
        .await?;
        Ok(())
    }

    pub async fn mark_migration_failed(&self, target_wasmtime_id: &str) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "UPDATE migration_state SET migration_status = ?, updated_at = datetime('now') WHERE singleton_key = 1 AND preparing_wasmtime_id = ?",
            libsql::params!(MigrationStatus::Failed.as_str(), target_wasmtime_id),
        )
        .await?;
        Ok(())
    }

    pub async fn ensure_compile_jobs(&self, payloads: &[CompileJobPayload]) -> Result<u64> {
        if payloads.is_empty() {
            return Ok(0);
        }

        let conn = self.db.connect()?;
        let tx = conn.transaction().await?;
        let mut enqueued_count = 0;

        for payload in payloads {
            if plan_compile_job(&tx, payload).await? {
                enqueued_count += 1;
            }
        }

        tx.commit().await?;
        Ok(enqueued_count)
    }

    pub async fn finalize_deploy(
        &self,
        deploy_job_id: &str,
        subdomain: &str,
        code_id: u64,
        code_version: u64,
        source_bundle_key: &str,
        compile_targets: &[CompileJobPayload],
    ) -> Result<u64> {
        let conn = self.db.connect()?;
        let tx = conn.transaction().await?;

        tx.execute(
            "INSERT INTO latest_code_versions (code_id, subdomain, latest_code_version, latest_source_bundle_key, updated_at) VALUES (?, ?, ?, ?, datetime('now')) ON CONFLICT(code_id) DO UPDATE SET subdomain = excluded.subdomain, latest_code_version = excluded.latest_code_version, latest_source_bundle_key = excluded.latest_source_bundle_key, updated_at = datetime('now')",
            libsql::params!(code_id, subdomain, code_version, source_bundle_key),
        )
        .await?;

        let next_sk: u64 = tx
            .query(
                "SELECT COALESCE(MAX(sk), 0) + 1 FROM docs WHERE pk = 'deployments'",
                libsql::params!(),
            )
            .await?
            .next()
            .await?
            .map(|row| row.get::<u64>(0).unwrap())
            .unwrap_or(1);

        let deployment = serde_json::to_string(&Deployment::Deploy {
            subdomain: subdomain.to_string(),
            code_id,
            code_version,
        })
        .unwrap();

        tx.execute(
            "INSERT INTO docs (pk, sk, value) VALUES ('deployments', ?, ?)",
            libsql::params!(next_sk, deployment),
        )
        .await?;

        for payload in compile_targets {
            plan_compile_job(&tx, payload).await?;
        }

        tx.execute(
            "DELETE FROM deploy_upload_sessions WHERE deploy_job_id = ?",
            libsql::params!(deploy_job_id),
        )
        .await?;

        tx.commit().await?;
        Ok(next_sk)
    }

    pub async fn mark_compile_processing(&self, payload: &CompileJobPayload) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "UPDATE compiled_artifacts SET status = ?, attempt_count = attempt_count + 1, last_error = NULL, updated_at = datetime('now') WHERE code_id = ? AND code_version = ? AND wasmtime_id = ?",
            libsql::params!(
                CompiledArtifactStatus::Processing.as_str(),
                payload.code_id,
                payload.code_version,
                payload.target_wasmtime_id.clone()
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn mark_compile_ready(&self, payload: &CompileJobPayload) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "UPDATE compiled_artifacts SET status = ?, artifact_key = ?, last_error = NULL, updated_at = datetime('now') WHERE code_id = ? AND code_version = ? AND wasmtime_id = ?",
            libsql::params!(
                CompiledArtifactStatus::Ready.as_str(),
                payload.output_artifact_key.clone(),
                payload.code_id,
                payload.code_version,
                payload.target_wasmtime_id.clone()
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn mark_compile_pending_error(
        &self,
        payload: &CompileJobPayload,
        error: &str,
    ) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "UPDATE compiled_artifacts SET status = ?, last_error = ?, updated_at = datetime('now') WHERE code_id = ? AND code_version = ? AND wasmtime_id = ?",
            libsql::params!(
                CompiledArtifactStatus::Pending.as_str(),
                error,
                payload.code_id,
                payload.code_version,
                payload.target_wasmtime_id.clone()
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn mark_compile_failed(
        &self,
        payload: &CompileJobPayload,
        error: &str,
    ) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "UPDATE compiled_artifacts SET status = ?, last_error = ?, updated_at = datetime('now') WHERE code_id = ? AND code_version = ? AND wasmtime_id = ?",
            libsql::params!(
                CompiledArtifactStatus::Failed.as_str(),
                error,
                payload.code_id,
                payload.code_version,
                payload.target_wasmtime_id.clone()
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn get_migration_progress(&self) -> Result<MigrationProgress> {
        let state = self.get_migration_state().await?;
        let Some(preparing_wasmtime_id) = state.preparing_wasmtime_id.clone() else {
            return Ok(MigrationProgress {
                state,
                total_latest_codes: 0,
                ready_count: 0,
                failed_count: 0,
                retrying_count: 0,
                recent_errors: Vec::new(),
            });
        };

        let conn = self.db.connect()?;
        let total_latest_codes = query_count(
            conn.query(
                "SELECT COUNT(*) FROM latest_code_versions",
                libsql::params!(),
            )
            .await?,
        )
        .await?;

        let ready_count = query_count(
            conn.query(
                "SELECT COUNT(*) FROM latest_code_versions l INNER JOIN compiled_artifacts c ON c.code_id = l.code_id AND c.code_version = l.latest_code_version AND c.wasmtime_id = ? WHERE c.status = ?",
                libsql::params!(
                    preparing_wasmtime_id.clone(),
                    CompiledArtifactStatus::Ready.as_str()
                ),
            )
            .await?,
        )
        .await?;

        let failed_count = query_count(
            conn.query(
                "SELECT COUNT(*) FROM latest_code_versions l INNER JOIN compiled_artifacts c ON c.code_id = l.code_id AND c.code_version = l.latest_code_version AND c.wasmtime_id = ? WHERE c.status = ?",
                libsql::params!(
                    preparing_wasmtime_id.clone(),
                    CompiledArtifactStatus::Failed.as_str()
                ),
            )
            .await?,
        )
        .await?;

        let retrying_count = query_count(
            conn.query(
                "SELECT COUNT(*) FROM latest_code_versions l INNER JOIN compiled_artifacts c ON c.code_id = l.code_id AND c.code_version = l.latest_code_version AND c.wasmtime_id = ? WHERE c.status = ?",
                libsql::params!(
                    preparing_wasmtime_id.clone(),
                    CompiledArtifactStatus::Processing.as_str()
                ),
            )
            .await?,
        )
        .await?;

        let mut recent_errors_rows = conn
            .query(
                "SELECT l.code_id, l.latest_code_version, l.subdomain, c.last_error FROM latest_code_versions l INNER JOIN compiled_artifacts c ON c.code_id = l.code_id AND c.code_version = l.latest_code_version AND c.wasmtime_id = ? WHERE c.last_error IS NOT NULL ORDER BY c.updated_at DESC LIMIT 5",
                libsql::params!(preparing_wasmtime_id),
            )
            .await?;

        let mut recent_errors = Vec::new();
        while let Some(row) = recent_errors_rows.next().await? {
            let code_id: u64 = row.get(0)?;
            let code_version: u64 = row.get(1)?;
            let subdomain: String = row.get(2)?;
            let error: String = row.get(3)?;
            recent_errors.push(format!(
                "code_id={code_id} subdomain={subdomain} code_version={code_version}: {error}"
            ));
        }

        Ok(MigrationProgress {
            state,
            total_latest_codes,
            ready_count,
            failed_count,
            retrying_count,
            recent_errors,
        })
    }
}

async fn plan_compile_job(
    tx: &libsql::Transaction,
    payload: &CompileJobPayload,
) -> Result<bool> {
    let mut rows = tx
        .query(
            "SELECT status, artifact_key FROM compiled_artifacts WHERE code_id = ? AND code_version = ? AND wasmtime_id = ?",
            libsql::params!(
                payload.code_id,
                payload.code_version,
                payload.target_wasmtime_id.clone()
            ),
        )
        .await?;

    if let Some(row) = rows.next().await? {
        let status: String = row.get(0)?;
        let artifact_key: String = row.get(1)?;
        if CompiledArtifactStatus::from_str(&status) == CompiledArtifactStatus::Ready
            && artifact_key == payload.output_artifact_key
        {
            return Ok(false);
        }
    }

    tx.execute(
        "INSERT INTO compiled_artifacts (code_id, code_version, wasmtime_id, status, artifact_key, attempt_count, last_error, updated_at) VALUES (?, ?, ?, ?, ?, 0, NULL, datetime('now')) ON CONFLICT(code_id, code_version, wasmtime_id) DO UPDATE SET status = excluded.status, artifact_key = excluded.artifact_key, last_error = NULL, updated_at = datetime('now')",
        libsql::params!(
            payload.code_id,
            payload.code_version,
            payload.target_wasmtime_id.clone(),
            CompiledArtifactStatus::Pending.as_str(),
            payload.output_artifact_key.clone()
        ),
    )
    .await?;

    let job_payload = serde_json::to_string(payload).unwrap();
    tx.execute(
        "INSERT OR IGNORE INTO hq_jobs (id, task_name, payload, status, retry_count, max_retries, created_at, updated_at) VALUES (?, 'compile_for_wasmtime', ?, 'pending', 0, 3, datetime('now'), datetime('now'))",
        libsql::params!(payload.job_id(), job_payload),
    )
    .await?;

    Ok(true)
}

async fn query_count(mut rows: libsql::Rows) -> Result<u64> {
    Ok(rows
        .next()
        .await?
        .map(|row| row.get::<u64>(0).unwrap())
        .unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migration_state_bootstraps_and_progress_defaults() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();
        db.ensure_wasmtime_tables("wasmtime-1").await.unwrap();

        let state = db.get_migration_state().await.unwrap();
        assert_eq!(state.active_wasmtime_id, "wasmtime-1");
        assert_eq!(state.migration_status, MigrationStatus::Idle);

        let progress = db.get_migration_progress().await.unwrap();
        assert_eq!(progress.total_latest_codes, 0);
        assert_eq!(progress.ready_count, 0);
    }

    #[tokio::test]
    async fn finalize_deploy_tracks_latest_and_enqueues_jobs() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();
        db.ensure_wasmtime_tables("wasmtime-1").await.unwrap();

        db.insert_deploy_upload_session("deploy-1", 7, "alice-app", "uploads/deploy-1.raw.tar")
            .await
            .unwrap();

        let payload = CompileJobPayload {
            code_id: 7,
            subdomain: "alice-app".to_string(),
            code_version: 3,
            source_bundle_key: "sources/7/3/bundle.raw.tar".to_string(),
            target_wasmtime_id: "wasmtime-1".to_string(),
            output_artifact_key: "bundles/alice-app.tar.zst".to_string(),
        };

        db.finalize_deploy(
            "deploy-1",
            "alice-app",
            7,
            3,
            "sources/7/3/bundle.raw.tar",
            std::slice::from_ref(&payload),
        )
        .await
        .unwrap();

        let latest = db.list_latest_codes().await.unwrap();
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].latest_code_version, 3);

        let job = db.claim_job().await.unwrap().unwrap();
        assert_eq!(job.task_name, "compile_for_wasmtime");

        assert!(db
            .get_deploy_upload_session("deploy-1")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn ensure_compile_jobs_skips_ready_artifacts() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();
        db.ensure_wasmtime_tables("wasmtime-1").await.unwrap();

        let payload = CompileJobPayload {
            code_id: 1,
            subdomain: "app".to_string(),
            code_version: 1,
            source_bundle_key: "sources/1/1/bundle.raw.tar".to_string(),
            target_wasmtime_id: "wasmtime-2".to_string(),
            output_artifact_key: "compiled/wasmtime-2/1/1/bundle.tar.zst".to_string(),
        };

        assert_eq!(db.ensure_compile_jobs(std::slice::from_ref(&payload)).await.unwrap(), 1);

        db.mark_compile_processing(&payload).await.unwrap();
        db.mark_compile_ready(&payload).await.unwrap();

        assert_eq!(db.ensure_compile_jobs(std::slice::from_ref(&payload)).await.unwrap(), 0);
    }
}
