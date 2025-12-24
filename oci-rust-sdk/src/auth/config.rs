use crate::auth::provider::AuthProvider;
use crate::core::region::Region;
use configparser::ini::Ini;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Configuration file based authentication provider
///
/// Reads credentials from OCI configuration file (default: ~/.oci/config)
#[derive(Debug, Clone)]
pub struct ConfigFileAuthProvider {
    tenancy: String,
    user: String,
    fingerprint: String,
    key_content: String,
    passphrase: Option<String>,
    region: Option<Region>,
}

impl ConfigFileAuthProvider {
    /// Create a new ConfigFileAuthProvider from a configuration file
    ///
    /// # Arguments
    /// * `config_path` - Path to the OCI config file (default: ~/.oci/config)
    /// * `profile` - Profile name to use (default: DEFAULT)
    pub fn from_file(config_path: impl AsRef<Path>, profile: &str) -> crate::core::Result<Self> {
        let config_path = expand_tilde(config_path.as_ref());
        let mut ini = Ini::new();
        ini.load(&config_path).map_err(|e| {
            crate::core::OciError::ConfigError(format!("Failed to load config file: {}", e))
        })?;

        let tenancy = ini.get(profile, "tenancy").ok_or_else(|| {
            crate::core::OciError::ConfigError(format!(
                "Missing 'tenancy' in profile '{}'",
                profile
            ))
        })?;

        let user = ini.get(profile, "user").ok_or_else(|| {
            crate::core::OciError::ConfigError(format!("Missing 'user' in profile '{}'", profile))
        })?;

        let fingerprint = ini.get(profile, "fingerprint").ok_or_else(|| {
            crate::core::OciError::ConfigError(format!(
                "Missing 'fingerprint' in profile '{}'",
                profile
            ))
        })?;

        let key_file = ini.get(profile, "key_file").ok_or_else(|| {
            crate::core::OciError::ConfigError(format!(
                "Missing 'key_file' in profile '{}'",
                profile
            ))
        })?;

        let key_path = expand_tilde(Path::new(&key_file));
        let key_content = fs::read_to_string(&key_path).map_err(|e| {
            crate::core::OciError::ConfigError(format!("Failed to read key file: {}", e))
        })?;

        let passphrase = ini.get(profile, "pass_phrase");

        let region = ini
            .get(profile, "region")
            .and_then(|r| Region::from_str(&r).ok());

        Ok(Self {
            tenancy,
            user,
            fingerprint,
            key_content,
            passphrase,
            region,
        })
    }

    /// Create from default config file path (~/.oci/config) and DEFAULT profile
    pub fn from_default() -> crate::core::Result<Self> {
        let config_path = dirs::home_dir()
            .ok_or_else(|| {
                crate::core::OciError::ConfigError("Could not determine home directory".to_string())
            })?
            .join(".oci")
            .join("config");

        Self::from_file(config_path, "DEFAULT")
    }

    /// Get the region (if specified in config)
    pub fn region(&self) -> Option<Region> {
        self.region
    }
}

impl AuthProvider for ConfigFileAuthProvider {
    fn get_key_id(&self) -> String {
        format!("{}/{}/{}", self.tenancy, self.user, self.fingerprint)
    }

    fn get_private_key(&self) -> &str {
        &self.key_content
    }

    fn get_passphrase(&self) -> Option<&str> {
        self.passphrase.as_deref()
    }
}

/// Expand tilde (~) in paths to home directory
fn expand_tilde(path: &Path) -> PathBuf {
    if let Ok(p) = path.strip_prefix("~")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(p);
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let path = Path::new("~/.oci/config");
        let expanded = expand_tilde(path);
        assert!(!expanded.to_str().unwrap().contains('~'));
    }
}
