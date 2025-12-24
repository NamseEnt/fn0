use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the volume backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationVolumeSourceFromVolumeBackupDetails {
    #[serde(rename = "type")]
    pub r#type: String,

    /// The OCID of the volume backup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Required fields for InstanceConfigurationVolumeSourceFromVolumeBackupDetails
pub struct InstanceConfigurationVolumeSourceFromVolumeBackupDetailsRequired {
    pub r#type: String,
}

impl InstanceConfigurationVolumeSourceFromVolumeBackupDetails {
    /// Create a new InstanceConfigurationVolumeSourceFromVolumeBackupDetails with required fields
    pub fn new(required: InstanceConfigurationVolumeSourceFromVolumeBackupDetailsRequired) -> Self {
        Self {
            r#type: required.r#type,

            id: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
        self.id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }
}
