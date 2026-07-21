use anyhow::Result;
use std::path::PathBuf;

use super::project_config::read_project_id;

pub async fn add(project_dir: PathBuf, domain: String) -> Result<()> {
    let id = read_project_id(&project_dir)?;
    fn0_deploy::domain_add(&id, &domain).await
}

pub async fn remove(project_dir: PathBuf) -> Result<()> {
    let id = read_project_id(&project_dir)?;
    fn0_deploy::domain_remove(&id).await
}

pub async fn status(project_dir: PathBuf) -> Result<()> {
    let id = read_project_id(&project_dir)?;
    fn0_deploy::domain_status(&id).await
}
