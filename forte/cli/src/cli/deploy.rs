use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::path::PathBuf;

use super::build::{BuildOptions, run_build};

#[derive(Deserialize)]
struct ForteConfig {
    name: Option<String>,
}

pub async fn run(project_dir: PathBuf) -> Result<()> {
    let config_path = project_dir.join("Forte.toml");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|_| anyhow!("Forte.toml not found. Are you in a Forte project directory?"))?;
    let config: ForteConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse Forte.toml: {}", e))?;

    let project_name = config
        .name
        .ok_or_else(|| anyhow!("'name' field missing in Forte.toml"))?;

    run_build(BuildOptions {
        project_dir: project_dir.clone(),
    })
    .await?;

    let dist_dir = project_dir.join("dist");
    let bundle_path = dist_dir.join("bundle.raw.tar");

    fn0_deploy::create_raw_bundle_forte(&dist_dir, &bundle_path)?;

    let env_content = fn0_deploy::read_env_content(&project_dir)?;

    fn0_deploy::deploy(&project_name, &bundle_path, env_content).await?;

    Ok(())
}
