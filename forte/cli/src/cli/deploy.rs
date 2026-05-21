use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::build::{BuildOptions, run_build};
use super::cron;

const DEFAULT_STATIC_BASE_DOMAIN: &str = "static.fn0.dev";

#[derive(Default, Deserialize, Serialize)]
struct ForteConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    project_id: Option<String>,
}

pub async fn run(project_dir: PathBuf, name_arg: Option<String>) -> Result<()> {
    let config_path = project_dir.join("Forte.toml");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|_| anyhow!("Forte.toml not found. Are you in a Forte project directory?"))?;
    let mut config: ForteConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse Forte.toml: {}", e))?;

    let creds = fn0_deploy::credentials::load()?.ok_or_else(|| {
        anyhow!(
            "not signed in. Run `forte login` first (credentials at {}).",
            fn0_deploy::credentials::path()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        )
    })?;
    let client = reqwest::Client::new();

    let project_id = match config.project_id.clone() {
        Some(id) => id,
        None => {
            let project_name = match name_arg {
                Some(n) => n,
                None => prompt_project_name()?,
            };
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

    let code_version: u64 = chrono::Utc::now()
        .timestamp_millis()
        .try_into()
        .expect("system clock returns positive timestamp");
    let static_base_domain = std::env::var("FORTE_STATIC_BASE_DOMAIN")
        .unwrap_or_else(|_| DEFAULT_STATIC_BASE_DOMAIN.to_string());
    let static_base_url = format!("https://{project_id}.{static_base_domain}/{code_version}/");
    println!("project_id: {project_id}");
    println!("code_version: {code_version}");
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
    fn0_deploy::deploy_forte(
        &creds.control_url,
        &creds.token,
        &project_id,
        code_version,
        &fe_dist,
        &bundle_path,
        &jobs,
        &cron_updated_at,
    )
    .await?;

    Ok(())
}

fn prompt_project_name() -> Result<String> {
    let name = inquire::Text::new("Project name:")
        .with_help_message("Used as a display label in the control plane.")
        .prompt()?;
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(anyhow!("project name cannot be empty"));
    }
    Ok(trimmed)
}
