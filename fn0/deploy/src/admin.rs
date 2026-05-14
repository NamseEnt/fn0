use anyhow::{Result, anyhow};

pub struct AdminRunOutput {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

pub async fn admin_run(
    _project_id: &str,
    _task: &str,
    _input_body: Vec<u8>,
    _timeout_secs: u64,
) -> Result<AdminRunOutput> {
    Err(anyhow!(
        "admin run is not yet migrated to control. See GitHub issue #4."
    ))
}
