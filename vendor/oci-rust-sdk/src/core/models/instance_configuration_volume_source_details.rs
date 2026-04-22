use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationVolumeSourceDetails {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for InstanceConfigurationVolumeSourceDetails
pub struct InstanceConfigurationVolumeSourceDetailsRequired {
    pub r#type: String,
}

impl InstanceConfigurationVolumeSourceDetails {
    /// Create a new InstanceConfigurationVolumeSourceDetails with required fields
    pub fn new(required: InstanceConfigurationVolumeSourceDetailsRequired) -> Self {
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
