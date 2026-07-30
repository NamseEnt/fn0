use anyhow::Result;
use std::path::PathBuf;

use super::project_config::read_project_id;

pub async fn connect(
    project_dir: PathBuf,
    account_id: String,
    zone_id: String,
    api_token: String,
) -> Result<()> {
    let id = read_project_id(&project_dir)?;
    fn0_deploy::cloudflare_connect(&id, &account_id, &zone_id, &api_token).await
}

pub async fn status(project_dir: PathBuf) -> Result<()> {
    let id = read_project_id(&project_dir)?;
    fn0_deploy::cloudflare_status(&id).await
}
