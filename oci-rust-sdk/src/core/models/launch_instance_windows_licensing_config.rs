use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The default windows licensing config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInstanceWindowsLicensingConfig {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for LaunchInstanceWindowsLicensingConfig
pub struct LaunchInstanceWindowsLicensingConfigRequired {
    pub r#type: String,
}

impl LaunchInstanceWindowsLicensingConfig {
    /// Create a new LaunchInstanceWindowsLicensingConfig with required fields
    pub fn new(required: LaunchInstanceWindowsLicensingConfigRequired) -> Self {
        Self {
            r#type: required.r#type,
        }
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
