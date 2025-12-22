use serde::{Deserialize, Serialize};

/// The license config requested for the instance (polymorphic based on type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LaunchInstanceLicensingConfig {
    /// Windows licensing configuration.
    #[serde(rename = "WINDOWS")]
    Windows(LaunchInstanceWindowsLicensingConfig),
}

/// Windows licensing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInstanceWindowsLicensingConfig {
    /// License Type for the OS license.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_type: Option<LicenseType>,
}

/// License type for the OS license.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LicenseType {
    /// OCI provided license (e.g. metered $/OCPU-hour).
    OciProvided,
    /// Bring your own license.
    BringYourOwnLicense,
}

impl LaunchInstanceWindowsLicensingConfig {
    pub fn new() -> Self {
        Self {
            license_type: None,
        }
    }

    pub fn with_license_type(mut self, license_type: LicenseType) -> Self {
        self.license_type = Some(license_type);
        self
    }

    /// Set license_type
    pub fn set_license_type(mut self, license_type: Option<LicenseType>) -> Self {
        self.license_type = license_type;
        self
    }
}

impl Default for LaunchInstanceWindowsLicensingConfig {
    fn default() -> Self {
        Self::new()
    }
}
