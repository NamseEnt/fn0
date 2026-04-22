use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The platform configuration to be updated for the instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstancePlatformConfig {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for UpdateInstancePlatformConfig
pub struct UpdateInstancePlatformConfigRequired {
    pub r#type: String,
}

impl UpdateInstancePlatformConfig {
    /// Create a new UpdateInstancePlatformConfig with required fields
    pub fn new(required: UpdateInstancePlatformConfigRequired) -> Self {
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
