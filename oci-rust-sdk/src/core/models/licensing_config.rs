use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Configuration of the Operating System license.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicensingConfig {
    /// Operating System type of the Configuration.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<LicensingConfigType>,

    /// License Type for the OS license. * {@code OCI_PROVIDED} - OCI provided license (e.g. metered $/OCPU-hour). * {@code BRING_YOUR_OWN_LICENSE} - Bring your own license.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_type: Option<LicensingConfigLicenseType>,

    /// The Operating System version of the license config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
}

impl LicensingConfig {
    /// Create a new LicensingConfig
    pub fn new() -> Self {
        Self {
            r#type: None,

            license_type: None,

            os_version: None,
        }
    }

    /// Set r#type
    pub fn set_type(mut self, value: Option<LicensingConfigType>) -> Self {
        self.r#type = value;
        self
    }

    /// Set license_type
    pub fn set_license_type(mut self, value: Option<LicensingConfigLicenseType>) -> Self {
        self.license_type = value;
        self
    }

    /// Set os_version
    pub fn set_os_version(mut self, value: Option<String>) -> Self {
        self.os_version = value;
        self
    }

    /// Set r#type (unwraps Option)
    pub fn with_type(mut self, value: LicensingConfigType) -> Self {
        self.r#type = Some(value);
        self
    }

    /// Set license_type (unwraps Option)
    pub fn with_license_type(mut self, value: LicensingConfigLicenseType) -> Self {
        self.license_type = Some(value);
        self
    }

    /// Set os_version (unwraps Option)
    pub fn with_os_version(mut self, value: impl Into<String>) -> Self {
        self.os_version = Some(value.into());
        self
    }
}

impl Default for LicensingConfig {
    fn default() -> Self {
        Self::new()
    }
}
