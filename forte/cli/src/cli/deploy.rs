use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use super::build::{BuildOptions, run_build};
use super::cron;

const DEFAULT_STATIC_BASE_DOMAIN: &str = "static.fn0.dev";

#[derive(Deserialize, Serialize)]
struct ForteConfig {
    name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    project_id: Option<String>,
}

pub async fn run(project_dir: PathBuf) -> Result<()> {
    let config_path = project_dir.join("Forte.toml");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|_| anyhow!("Forte.toml not found. Are you in a Forte project directory?"))?;
    let mut config: ForteConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse Forte.toml: {}", e))?;

    let project_name = config
        .name
        .clone()
        .ok_or_else(|| anyhow!("'name' field missing in Forte.toml"))?;

    let creds = fn0_deploy::credentials::require()?;
    let client = reqwest::Client::new();

    let project_id = match config.project_id.clone() {
        Some(id) => id,
        None => {
            let mut maybe = None;
            let id = fn0_deploy::ensure_project_id(
                &client,
                &creds.control_url,
                &creds.token,
                &project_name,
                &mut maybe,
            )
            .await?;
            config.project_id = Some(id.clone());
            std::fs::write(
                &config_path,
                toml::to_string_pretty(&config)
                    .map_err(|e| anyhow!("Failed to serialize Forte.toml: {}", e))?,
            )?;
            println!("Saved project_id to Forte.toml");
            id
        }
    };

    let build_id = Uuid::new_v4().to_string();
    let static_base_domain = std::env::var("FORTE_STATIC_BASE_DOMAIN")
        .unwrap_or_else(|_| DEFAULT_STATIC_BASE_DOMAIN.to_string());
    let static_base_url = format!("https://{project_id}.{static_base_domain}/{build_id}/");
    println!("project_id: {project_id}");
    println!("build_id:   {build_id}");
    println!("static base URL: {static_base_url}");

    run_build(BuildOptions {
        project_dir: project_dir.clone(),
        static_base_url: Some(static_base_url),
    })
    .await?;

    let dist_dir = project_dir.join("dist");
    let bundle_path = dist_dir.join("bundle.raw.tar");
    let env_yaml = fn0_deploy::read_env_yaml(&project_dir)?;
    fn0_deploy::create_raw_bundle_forte(&dist_dir, env_yaml.as_deref(), &bundle_path)?;

    let jobs = cron::read_and_validate(&project_dir)?;
    let cron_updated_at = chrono::Utc::now().to_rfc3339();

    let fe_dist = project_dir.join("fe/dist");
    let mut maybe_id = Some(project_id);
    fn0_deploy::deploy_forte(
        &creds.control_url,
        &creds.token,
        &project_name,
        &mut maybe_id,
        &build_id,
        &fe_dist,
        &bundle_path,
        &jobs,
        &cron_updated_at,
    )
    .await?;

    Ok(())
}
