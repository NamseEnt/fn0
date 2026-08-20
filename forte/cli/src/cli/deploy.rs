use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

use super::build::{BuildOptions, read_websocket_singletons, run_build};
use super::cron;
use super::project_config::{read_optional_domain, read_optional_project_id};
use fn0_deploy::{CloudflareConnection, DomainStatus};

pub async fn run(project_dir: PathBuf) -> Result<()> {
    let creds = fn0_deploy::credentials::load()?.ok_or_else(|| {
        anyhow!(
            "not signed in. Run `forte login` first (credentials at {}).",
            fn0_deploy::credentials::path()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        )
    })?;

    let project_id = read_optional_project_id(&project_dir)?.ok_or_else(|| {
        anyhow!("this project has not been set up yet. Run `forte cloud init` first.")
    })?;
    match fn0_deploy::fetch_cloudflare_connection(&project_id).await? {
        CloudflareConnection::Connected { .. } => {}
        CloudflareConnection::NotConnected => {
            return Err(anyhow!(
                "project '{project_id}' has no Cloudflare account behind it, so there is nowhere \
                 to put its bundle or its assets. Run `forte cloud init`."
            ));
        }
        CloudflareConnection::NotFound => {
            return Err(anyhow!(
                "project '{project_id}' not found or not owned by you."
            ));
        }
    }
    check_domain_matches(&creds, &project_id, &project_dir).await?;

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
    let websocket_singletons = read_websocket_singletons(&project_dir)?;

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
        &websocket_singletons,
    )
    .await?;

    let resolved = fn0_deploy::resolve_app_url(&project_id).await?;
    if let Some(note) = &resolved.note {
        eprintln!("note: {note}");
    }
    match resolved.url {
        Some(url) => println!("app URL: {url}"),
        None => println!("app URL: none yet"),
    }

    Ok(())
}

/// Forte.toml declares the domain; control decides it. They only drift if the
/// file was hand-edited, and deploying past that would silently ship to the old
/// hostname. Changing a domain means signing a certificate, which needs a
/// Cloudflare token this command deliberately never asks for.
async fn check_domain_matches(
    creds: &fn0_deploy::credentials::Credentials,
    project_id: &str,
    project_dir: &Path,
) -> Result<()> {
    let Some(declared) = read_optional_domain(project_dir)? else {
        return Ok(());
    };
    let live = match fn0_deploy::fetch_domain_status(creds, project_id).await? {
        DomainStatus::SelfHosted { domain, .. } => Some(domain),
        _ => None,
    };
    match live {
        Some(live) if live == declared => Ok(()),
        Some(live) => Err(anyhow!(
            "Forte.toml says the domain is '{declared}', but project '{project_id}' answers on \
             '{live}'. Run `forte cloud init` to move it."
        )),
        None => Err(anyhow!(
            "Forte.toml says the domain is '{declared}', but project '{project_id}' has none. \
             Run `forte cloud init`."
        )),
    }
}
