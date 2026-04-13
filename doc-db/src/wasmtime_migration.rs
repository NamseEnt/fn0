use super::*;

const STATE_PK: &str = "wasmtime_state";
const REBUILD_PK_PREFIX: &str = "wasmtime_rebuild:";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct WasmtimeState {
    pub active_version: Option<String>,
    pub known_versions: Vec<String>,
}

impl Default for WasmtimeState {
    fn default() -> Self {
        Self {
            active_version: None,
            known_versions: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RebuildStatus {
    Pending,
    Done,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RebuildEntry {
    pub subdomain: String,
    pub code_id: u64,
    pub code_version: u64,
    pub status: RebuildStatus,
}

impl DocDb {
    pub async fn ensure_wasmtime_tables(&self) -> Result<()> {
        Ok(())
    }

    pub async fn get_wasmtime_state(&self) -> Result<WasmtimeState> {
        let conn = self.db.connect()?;
        let row = conn
            .query(
                "SELECT value FROM docs WHERE pk = ? AND sk = 0",
                libsql::params!(STATE_PK),
            )
            .await?
            .next()
            .await?;

        match row {
            Some(row) => {
                let bytes: Vec<u8> = row.get(0)?;
                let state: WasmtimeState =
                    serde_json::from_slice(&bytes).unwrap_or_default();
                Ok(state)
            }
            None => Ok(WasmtimeState::default()),
        }
    }

    async fn write_wasmtime_state(&self, state: &WasmtimeState) -> Result<()> {
        let conn = self.db.connect()?;
        let bytes = serde_json::to_vec(state).unwrap();
        conn.execute(
            "INSERT INTO docs (pk, sk, value) VALUES (?, 0, ?) \
             ON CONFLICT(pk, sk) DO UPDATE SET value = excluded.value",
            libsql::params!(STATE_PK, bytes),
        )
        .await?;
        Ok(())
    }

    pub async fn record_known_wasmtime_version(&self, version: &str) -> Result<WasmtimeState> {
        let mut state = self.get_wasmtime_state().await?;
        if !state.known_versions.iter().any(|v| v == version) {
            state.known_versions.push(version.to_string());
            self.write_wasmtime_state(&state).await?;
        }
        Ok(state)
    }

    pub async fn set_active_wasmtime_version(&self, version: &str) -> Result<WasmtimeState> {
        let mut state = self.get_wasmtime_state().await?;
        state.active_version = Some(version.to_string());
        if !state.known_versions.iter().any(|v| v == version) {
            state.known_versions.push(version.to_string());
        }
        self.write_wasmtime_state(&state).await?;
        Ok(state)
    }

    pub async fn forget_wasmtime_version(&self, version: &str) -> Result<()> {
        let mut state = self.get_wasmtime_state().await?;
        state.known_versions.retain(|v| v != version);
        if state.active_version.as_deref() == Some(version) {
            state.active_version = None;
        }
        self.write_wasmtime_state(&state).await?;
        Ok(())
    }

    pub async fn upsert_rebuild_entries(
        &self,
        target_version: &str,
        entries: &[RebuildEntry],
    ) -> Result<()> {
        let conn = self.db.connect()?;
        let pk = format!("{REBUILD_PK_PREFIX}{target_version}");

        for entry in entries {
            let existing = conn
                .query(
                    "SELECT sk FROM docs WHERE pk = ? AND json_extract(value, '$.subdomain') = ?",
                    libsql::params!(pk.clone(), entry.subdomain.clone()),
                )
                .await?
                .next()
                .await?;

            if existing.is_some() {
                continue;
            }

            let next_sk: u64 = conn
                .query(
                    "SELECT COALESCE(MAX(sk), 0) + 1 FROM docs WHERE pk = ?",
                    libsql::params!(pk.clone()),
                )
                .await?
                .next()
                .await?
                .map(|row| row.get::<u64>(0).unwrap())
                .unwrap_or(1);

            let bytes = serde_json::to_vec(entry).unwrap();
            conn.execute(
                "INSERT INTO docs (pk, sk, value) VALUES (?, ?, ?)",
                libsql::params!(pk.clone(), next_sk, bytes),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn list_rebuild_entries(
        &self,
        target_version: &str,
    ) -> Result<Vec<RebuildEntry>> {
        let conn = self.db.connect()?;
        let pk = format!("{REBUILD_PK_PREFIX}{target_version}");

        let mut rows = conn
            .query(
                "SELECT value FROM docs WHERE pk = ? ORDER BY sk ASC",
                libsql::params!(pk),
            )
            .await?;

        let mut entries = Vec::new();
        while let Some(row) = rows.next().await? {
            let bytes: Vec<u8> = row.get(0)?;
            if let Ok(entry) = serde_json::from_slice::<RebuildEntry>(&bytes) {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    pub async fn pending_rebuild_count(&self, target_version: &str) -> Result<u64> {
        let entries = self.list_rebuild_entries(target_version).await?;
        Ok(entries
            .iter()
            .filter(|e| matches!(e.status, RebuildStatus::Pending))
            .count() as u64)
    }

    pub async fn mark_rebuild_done(
        &self,
        target_version: &str,
        subdomain: &str,
    ) -> Result<()> {
        let conn = self.db.connect()?;
        let pk = format!("{REBUILD_PK_PREFIX}{target_version}");

        let row = conn
            .query(
                "SELECT sk, value FROM docs WHERE pk = ? AND json_extract(value, '$.subdomain') = ?",
                libsql::params!(pk.clone(), subdomain),
            )
            .await?
            .next()
            .await?;

        let Some(row) = row else {
            return Ok(());
        };

        let sk: u64 = row.get(0)?;
        let bytes: Vec<u8> = row.get(1)?;
        let mut entry: RebuildEntry = serde_json::from_slice(&bytes)
            .map_err(|e| libsql::Error::Misuse(format!("decode rebuild entry: {e}")))?;
        entry.status = RebuildStatus::Done;
        let new_bytes = serde_json::to_vec(&entry).unwrap();

        conn.execute(
            "UPDATE docs SET value = ? WHERE pk = ? AND sk = ?",
            libsql::params!(new_bytes, pk, sk),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_rebuild_table(&self, version: &str) -> Result<()> {
        let conn = self.db.connect()?;
        let pk = format!("{REBUILD_PK_PREFIX}{version}");
        conn.execute(
            "DELETE FROM docs WHERE pk = ?",
            libsql::params!(pk),
        )
        .await?;
        Ok(())
    }

    pub async fn insert_job(
        &self,
        id: &str,
        task_name: &str,
        payload: &str,
    ) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "INSERT INTO hq_jobs (id, task_name, payload, status, retry_count, max_retries, created_at, updated_at) \
             VALUES (?, ?, ?, 'pending', 0, 3, datetime('now'), datetime('now')) \
             ON CONFLICT(id) DO NOTHING",
            libsql::params!(id, task_name, payload),
        )
        .await?;
        Ok(())
    }

    pub async fn job_exists_for_task(&self, task_name: &str) -> Result<bool> {
        let conn = self.db.connect()?;
        let row = conn
            .query(
                "SELECT 1 FROM hq_jobs WHERE task_name = ? LIMIT 1",
                libsql::params!(task_name),
            )
            .await?
            .next()
            .await?;
        Ok(row.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_db() -> DocDb {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();
        db.ensure_wasmtime_tables().await.unwrap();
        db
    }

    #[tokio::test]
    async fn state_starts_empty() {
        let db = make_db().await;
        let state = db.get_wasmtime_state().await.unwrap();
        assert!(state.active_version.is_none());
        assert!(state.known_versions.is_empty());
    }

    #[tokio::test]
    async fn set_active_marks_known() {
        let db = make_db().await;
        let s = db.set_active_wasmtime_version("42").await.unwrap();
        assert_eq!(s.active_version.as_deref(), Some("42"));
        assert_eq!(s.known_versions, vec!["42".to_string()]);

        let s = db.set_active_wasmtime_version("43").await.unwrap();
        assert_eq!(s.active_version.as_deref(), Some("43"));
        assert_eq!(
            s.known_versions,
            vec!["42".to_string(), "43".to_string()]
        );
    }

    #[tokio::test]
    async fn record_known_no_duplicates() {
        let db = make_db().await;
        db.record_known_wasmtime_version("42").await.unwrap();
        db.record_known_wasmtime_version("42").await.unwrap();
        let s = db.get_wasmtime_state().await.unwrap();
        assert_eq!(s.known_versions, vec!["42".to_string()]);
    }

    #[tokio::test]
    async fn rebuild_entries_lifecycle() {
        let db = make_db().await;
        let entries = vec![
            RebuildEntry {
                subdomain: "alice-app".into(),
                code_id: 1,
                code_version: 1,
                status: RebuildStatus::Pending,
            },
            RebuildEntry {
                subdomain: "bob-app".into(),
                code_id: 2,
                code_version: 5,
                status: RebuildStatus::Pending,
            },
        ];

        db.upsert_rebuild_entries("43", &entries).await.unwrap();
        assert_eq!(db.pending_rebuild_count("43").await.unwrap(), 2);

        db.upsert_rebuild_entries("43", &entries).await.unwrap();
        assert_eq!(
            db.list_rebuild_entries("43").await.unwrap().len(),
            2,
            "upsert is idempotent"
        );

        db.mark_rebuild_done("43", "alice-app").await.unwrap();
        assert_eq!(db.pending_rebuild_count("43").await.unwrap(), 1);

        db.mark_rebuild_done("43", "bob-app").await.unwrap();
        assert_eq!(db.pending_rebuild_count("43").await.unwrap(), 0);

        db.delete_rebuild_table("43").await.unwrap();
        assert_eq!(db.list_rebuild_entries("43").await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn forget_version_clears_active() {
        let db = make_db().await;
        db.set_active_wasmtime_version("42").await.unwrap();
        db.set_active_wasmtime_version("43").await.unwrap();
        db.forget_wasmtime_version("42").await.unwrap();
        let s = db.get_wasmtime_state().await.unwrap();
        assert_eq!(s.active_version.as_deref(), Some("43"));
        assert_eq!(s.known_versions, vec!["43".to_string()]);

        db.forget_wasmtime_version("43").await.unwrap();
        let s = db.get_wasmtime_state().await.unwrap();
        assert!(s.active_version.is_none());
        assert!(s.known_versions.is_empty());
    }

    #[tokio::test]
    async fn insert_job_is_idempotent() {
        let db = make_db().await;
        db.insert_job("j1", "wasmtime_rebuild", "{}").await.unwrap();
        db.insert_job("j1", "wasmtime_rebuild", "{}").await.unwrap();
        assert!(db.job_exists_for_task("wasmtime_rebuild").await.unwrap());
        let job = db.claim_job().await.unwrap().unwrap();
        assert_eq!(job.id, "j1");
        db.complete_job("j1").await.unwrap();
        assert!(!db.job_exists_for_task("wasmtime_rebuild").await.unwrap());
    }
}
