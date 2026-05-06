pub use fn0_deploy::credentials::{Credentials, default_control_url};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use std::path::PathBuf;

pub fn path() -> Result<PathBuf> {
    fn0_deploy::credentials::path().map_err(|e| eyre!("{e:#}"))
}

pub fn save(creds: &Credentials) -> Result<()> {
    fn0_deploy::credentials::save(creds).map_err(|e| eyre!("{e:#}"))
}

pub fn require() -> Result<Credentials> {
    fn0_deploy::credentials::require().map_err(|e| eyre!("{e:#}"))
}
