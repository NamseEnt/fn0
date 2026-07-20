use anyhow::{Result, anyhow};
use fn0_deploy::credentials;
use fn0_deploy::env;
use std::path::Path;

const LEGACY_DOTENV: &str = ".env";

pub async fn set(project_dir: &Path, key: String, value: String, secret: bool) -> Result<()> {
    let kind = if secret {
        let creds = credentials::load()?.ok_or_else(|| {
            anyhow!(
                "not signed in. Run `forte login` first (credentials at {}).",
                credentials::path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            )
        })?;
        env::set_secret(project_dir, &key, &value, &creds).await?;
        "secret"
    } else {
        env::set_plain(project_dir, &key, &value)?;
        "plain"
    };
    let path = env::env_yaml_path(project_dir);
    println!("set {key} ({kind}) in {}", path.display());
    Ok(())
}

pub fn migrate(project_dir: &Path) -> Result<()> {
    let legacy_path = project_dir.join(LEGACY_DOTENV);
    let content = std::fs::read_to_string(&legacy_path)
        .map_err(|_| anyhow!("no {} in {}", LEGACY_DOTENV, project_dir.display()))?;

    let target_path = env::env_local_yaml_path(project_dir);
    if target_path.exists() {
        return Err(anyhow!(
            "{} already exists — merge {} into it by hand",
            target_path.display(),
            LEGACY_DOTENV
        ));
    }

    let mut migrated = 0;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        env::set_plain_local(project_dir, key.trim(), value.trim())?;
        migrated += 1;
    }

    println!("migrated {migrated} entries to {}", target_path.display());
    println!("{} is no longer read — delete it.", legacy_path.display());
    Ok(())
}

/// A `.env` alongside no `env.local.yaml` means the project predates the YAML
/// env format and its values would silently go missing.
pub fn reject_unmigrated_dotenv(project_dir: &Path) -> Result<()> {
    if project_dir.join(LEGACY_DOTENV).exists() && !env::env_local_yaml_path(project_dir).exists() {
        return Err(anyhow!(
            "{} is no longer read. Run `forte env migrate` to convert it to env.local.yaml.",
            LEGACY_DOTENV
        ));
    }
    Ok(())
}
