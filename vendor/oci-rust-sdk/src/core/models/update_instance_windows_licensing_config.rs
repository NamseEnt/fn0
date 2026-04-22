use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The default windows licensing config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceWindowsLicensingConfig {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for UpdateInstanceWindowsLicensingConfig
pub struct UpdateInstanceWindowsLicensingConfigRequired {
    pub r#type: String,
}

impl UpdateInstanceWindowsLicensingConfig {
    /// Create a new UpdateInstanceWindowsLicensingConfig with required fields
    pub fn new(required: UpdateInstanceWindowsLicensingConfigRequired) -> Self {
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
