use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct ForteConfig {
    project_id: Option<String>,
}

fn project_id(project_dir: &PathBuf) -> Result<String> {
    let config_path = project_dir.join("Forte.toml");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|_| anyhow!("Forte.toml not found. Are you in a Forte project directory?"))?;
    let config: ForteConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse Forte.toml: {}", e))?;
    config
        .project_id
        .ok_or_else(|| anyhow!("'project_id' field missing in Forte.toml. Run `forte deploy` first to register the project."))
}

pub async fn add(project_dir: PathBuf, domain: String) -> Result<()> {
    let id = project_id(&project_dir)?;
    fn0_deploy::domain_add(&id, &domain).await
}

pub async fn remove(project_dir: PathBuf) -> Result<()> {
    let id = project_id(&project_dir)?;
    fn0_deploy::domain_remove(&id).await
}

pub async fn status(project_dir: PathBuf) -> Result<()> {
    let id = project_id(&project_dir)?;
    fn0_deploy::domain_status(&id).await
}
