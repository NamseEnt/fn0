use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WasmtimeRolloutPhase {
    Idle,
    Rebundling,
    ReadyToRoll,
    Rolling,
    Cleaning,
}

impl WasmtimeRolloutPhase {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Rebundling => "rebundling",
            Self::ReadyToRoll => "ready_to_roll",
            Self::Rolling => "rolling",
            Self::Cleaning => "cleaning",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "idle" => Some(Self::Idle),
            "rebundling" => Some(Self::Rebundling),
            "ready_to_roll" => Some(Self::ReadyToRoll),
            "rolling" => Some(Self::Rolling),
            "cleaning" => Some(Self::Cleaning),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmtimeRolloutState {
    pub site_name: String,
    pub current_version: String,
    pub target_version: String,
    pub previous_version: Option<String>,
    pub phase: WasmtimeRolloutPhase,
}

impl DocDb {
    pub async fn ensure_wasmtime_rollout_tables(&self) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS hq_wasmtime_rollouts (site_name TEXT PRIMARY KEY, current_version TEXT NOT NULL, target_version TEXT NOT NULL, previous_version TEXT, phase TEXT NOT NULL, updated_at TEXT NOT NULL)",
            libsql::params!(),
        )
        .await?;
        Ok(())
    }

    pub async fn get_wasmtime_rollout_state(
        &self,
        site_name: &str,
    ) -> Result<Option<WasmtimeRolloutState>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT current_version, target_version, previous_version, phase FROM hq_wasmtime_rollouts WHERE site_name = ?",
                libsql::params!(site_name),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        let phase_raw: String = row.get(3)?;
        let phase = WasmtimeRolloutPhase::from_str(&phase_raw).ok_or_else(|| {
            libsql::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown wasmtime rollout phase: {phase_raw}"),
            )))
        })?;

        Ok(Some(WasmtimeRolloutState {
            site_name: site_name.to_string(),
            current_version: row.get(0)?,
            target_version: row.get(1)?,
            previous_version: row.get(2)?,
            phase,
        }))
    }

    pub async fn init_wasmtime_rollout_state(
        &self,
        site_name: &str,
        current_version: &str,
    ) -> Result<WasmtimeRolloutState> {
        if let Some(state) = self.get_wasmtime_rollout_state(site_name).await? {
            return Ok(state);
        }

        let state = WasmtimeRolloutState {
            site_name: site_name.to_string(),
            current_version: current_version.to_string(),
            target_version: current_version.to_string(),
            previous_version: None,
            phase: WasmtimeRolloutPhase::Idle,
        };
        self.upsert_wasmtime_rollout_state(&state).await?;
        Ok(state)
    }

    pub async fn upsert_wasmtime_rollout_state(&self, state: &WasmtimeRolloutState) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute(
            "INSERT INTO hq_wasmtime_rollouts (site_name, current_version, target_version, previous_version, phase, updated_at) VALUES (?, ?, ?, ?, ?, datetime('now')) ON CONFLICT(site_name) DO UPDATE SET current_version = excluded.current_version, target_version = excluded.target_version, previous_version = excluded.previous_version, phase = excluded.phase, updated_at = excluded.updated_at",
            libsql::params!(
                state.site_name.clone(),
                state.current_version.clone(),
                state.target_version.clone(),
                state.previous_version.clone(),
                state.phase.as_str(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn has_active_wasmtime_rollout(&self) -> Result<bool> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT 1 FROM hq_wasmtime_rollouts WHERE phase != 'idle' LIMIT 1",
                libsql::params!(),
            )
            .await?;
        Ok(rows.next().await?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn init_and_read_rollout_state() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_wasmtime_rollout_tables().await.unwrap();

        let state = db
            .init_wasmtime_rollout_state("site-a", "42")
            .await
            .unwrap();
        assert_eq!(state.phase, WasmtimeRolloutPhase::Idle);
        assert_eq!(state.current_version, "42");
        assert_eq!(state.target_version, "42");

        let loaded = db
            .get_wasmtime_rollout_state("site-a")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(loaded, state);
    }

    #[tokio::test]
    async fn active_rollout_detection_works() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_wasmtime_rollout_tables().await.unwrap();

        let mut state = db
            .init_wasmtime_rollout_state("site-a", "42")
            .await
            .unwrap();
        assert!(!db.has_active_wasmtime_rollout().await.unwrap());

        state.target_version = "43".to_string();
        state.previous_version = Some("42".to_string());
        state.phase = WasmtimeRolloutPhase::Rebundling;
        db.upsert_wasmtime_rollout_state(&state).await.unwrap();

        assert!(db.has_active_wasmtime_rollout().await.unwrap());

        state.current_version = "43".to_string();
        state.previous_version = None;
        state.phase = WasmtimeRolloutPhase::Idle;
        db.upsert_wasmtime_rollout_state(&state).await.unwrap();

        assert!(!db.has_active_wasmtime_rollout().await.unwrap());
    }
}
