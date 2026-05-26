use color_eyre::Result;
use color_eyre::eyre::eyre;
use fn0_deploy::credentials;
use fn0_deploy::env::{self, EntryKind};

pub async fn set(key: String, value: String, secret: bool) -> Result<()> {
    let project_dir = std::env::current_dir()?;
    let kind = if secret {
        let creds = credentials::require().map_err(|e| eyre!("{e}"))?;
        env::set_secret(&project_dir, &key, &value, &creds)
            .await
            .map_err(|e| eyre!("{e}"))?;
        "secret"
    } else {
        env::set_plain(&project_dir, &key, &value).map_err(|e| eyre!("{e}"))?;
        "plain"
    };
    let path = env::env_yaml_path(&project_dir);
    println!("set {key} ({kind}) in {}", path.display());
    Ok(())
}

pub async fn list() -> Result<()> {
    let project_dir = std::env::current_dir()?;
    let path = env::env_yaml_path(&project_dir);
    let entries = env::list_entries(&project_dir).map_err(|e| eyre!("{e}"))?;
    if entries.is_empty() {
        if !path.exists() {
            println!("(no {} in this project)", path.display());
        } else {
            println!("(no entries)");
        }
        return Ok(());
    }
    for (name, kind) in entries {
        let kind_str = match kind {
            EntryKind::Plain => "plain",
            EntryKind::Secret => "secret",
        };
        println!("{name}\t{kind_str}");
    }
    Ok(())
}

pub async fn unset(key: String) -> Result<()> {
    let project_dir = std::env::current_dir()?;
    env::unset(&project_dir, &key).map_err(|e| eyre!("{e}"))?;
    let path = env::env_yaml_path(&project_dir);
    println!("unset {key} in {}", path.display());
    Ok(())
}
