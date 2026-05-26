use anyhow::{Result, anyhow};
use fn0_deploy::credentials;
use fn0_deploy::env;
use std::path::Path;

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
