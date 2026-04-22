use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationIscsiAttachVolumeDetails {
    #[serde(rename = "type")]
    pub r#type: String,

    /// Whether to use CHAP authentication for the volume attachment. Defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_chap: Option<bool>,
}

/// Required fields for InstanceConfigurationIscsiAttachVolumeDetails
pub struct InstanceConfigurationIscsiAttachVolumeDetailsRequired {
    pub r#type: String,
}

impl InstanceConfigurationIscsiAttachVolumeDetails {
    /// Create a new InstanceConfigurationIscsiAttachVolumeDetails with required fields
    pub fn new(required: InstanceConfigurationIscsiAttachVolumeDetailsRequired) -> Self {
        Self {
            r#type: required.r#type,

            use_chap: None,
        }
    }

    /// Set use_chap
    pub fn set_use_chap(mut self, value: Option<bool>) -> Self {
        self.use_chap = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set use_chap (unwraps Option)
    pub fn with_use_chap(mut self, value: bool) -> Self {
        self.use_chap = Some(value);
        self
    }
}
