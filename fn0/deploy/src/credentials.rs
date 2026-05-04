use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_CONTROL_URL: &str = "https://fn0-control.fn0.dev";

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub token: String,
    pub control_url: String,
}

pub fn path() -> Result<PathBuf> {
    let dir = dirs::config_dir().ok_or_else(|| anyhow!("could not locate user config dir"))?;
    Ok(dir.join("fn0").join("credentials"))
}

pub fn load() -> Result<Option<Credentials>> {
    let p = path()?;
    if !p.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&p)?;
    let creds: Credentials = toml::from_str(&content)?;
    Ok(Some(creds))
}

pub fn require() -> Result<Credentials> {
    load()?.ok_or_else(|| {
        anyhow!(
            "not signed in. Run `fn0 login` first (credentials would be at {}).",
            path().map(|p| p.display().to_string()).unwrap_or_default()
        )
    })
}

pub fn default_control_url() -> String {
    std::env::var("FN0_CONTROL_URL").unwrap_or_else(|_| DEFAULT_CONTROL_URL.to_string())
}
