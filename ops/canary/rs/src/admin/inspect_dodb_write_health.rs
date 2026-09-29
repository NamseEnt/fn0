use crate::common::dodb_write_health::{PARTITION_KEY, SORT_KEY, State};
use crate::common::known_values::DOC_DB_WRITE_PARTITION_KEY;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input;

#[derive(Serialize)]
pub struct Output {
    pub legacy_write_probe_rows: usize,
    pub fixed_write_health_rows: usize,
    pub fixed_write_health_state: Option<StateSummary>,
}

#[derive(Serialize)]
pub struct StateSummary {
    pub format_version: u32,
    pub last_write_unix_ms: u64,
}

pub async fn handle(_input: Input) -> anyhow::Result<Output> {
    let database = doc_db::database();
    let legacy_rows = database
        .query(DOC_DB_WRITE_PARTITION_KEY, None::<&str>, 257)
        .await?;
    let fixed_rows = database.query(PARTITION_KEY, None::<&str>, 2).await?;
    let state = database.get(PARTITION_KEY, SORT_KEY).await?;
    let fixed_write_health_state = state
        .map(|bytes| serde_json::from_slice::<State>(&bytes))
        .transpose()?
        .map(|state| StateSummary {
            format_version: state.format_version,
            last_write_unix_ms: state.last_write_unix_ms,
        });

    Ok(Output {
        legacy_write_probe_rows: legacy_rows.len(),
        fixed_write_health_rows: fixed_rows.len(),
        fixed_write_health_state,
    })
}
