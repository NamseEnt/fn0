use anyhow::{Result, anyhow};
use std::path::PathBuf;

use super::build::{BuildOptions, run_build};
use super::cron;
use super::project_config::{read_optional_project_id, write_project_id};

pub async fn run(project_dir: PathBuf, name_arg: Option<String>) -> Result<()> {
    let existing_project_id = read_optional_project_id(&project_dir)?;

    let creds = fn0_deploy::credentials::load()?.ok_or_else(|| {
        anyhow!(
            "not signed in. Run `forte login` first (credentials at {}).",
            fn0_deploy::credentials::path()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        )
    })?;
    let client = reqwest::Client::new();

    let project_id = match existing_project_id {
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
            write_project_id(&project_dir, &id)?;
            println!("Saved project_id to Forte.toml");
            id
        }
    };

    let code_version: u64 = chrono::Utc::now()
        .timestamp_millis()
        .try_into()
        .expect("system clock returns positive timestamp");
    let static_base_url = match std::env::var("FORTE_STATIC_BASE_DOMAIN") {
        Ok(domain) => format!("https://{domain}/{project_id}/{code_version}/"),
        Err(_) => fn0_deploy::resolve_asset_base_url(&project_id, code_version).await?,
    };
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

    let resolved = fn0_deploy::resolve_app_url(&project_id).await?;
    if let Some(note) = &resolved.note {
        eprintln!("note: {note}");
    }
    println!("app URL: {}", resolved.url);

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
