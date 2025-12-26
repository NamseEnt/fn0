use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationParavirtualizedAttachVolumeDetails {
    #[serde(rename = "type")]
    pub r#type: String,

    /// Whether to enable in-transit encryption for the data volume's paravirtualized attachment. The default value is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pv_encryption_in_transit_enabled: Option<bool>,
}

/// Required fields for InstanceConfigurationParavirtualizedAttachVolumeDetails
pub struct InstanceConfigurationParavirtualizedAttachVolumeDetailsRequired {
    pub r#type: String,
}

impl InstanceConfigurationParavirtualizedAttachVolumeDetails {
    /// Create a new InstanceConfigurationParavirtualizedAttachVolumeDetails with required fields
    pub fn new(required: InstanceConfigurationParavirtualizedAttachVolumeDetailsRequired) -> Self {
        Self {
            r#type: required.r#type,

            is_pv_encryption_in_transit_enabled: None,
        }
    }

    /// Set is_pv_encryption_in_transit_enabled
    pub fn set_is_pv_encryption_in_transit_enabled(mut self, value: Option<bool>) -> Self {
        self.is_pv_encryption_in_transit_enabled = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set is_pv_encryption_in_transit_enabled (unwraps Option)
    pub fn with_is_pv_encryption_in_transit_enabled(mut self, value: bool) -> Self {
        self.is_pv_encryption_in_transit_enabled = Some(value);
        self
    }
}
