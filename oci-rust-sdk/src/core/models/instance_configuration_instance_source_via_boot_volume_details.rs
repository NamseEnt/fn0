use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationInstanceSourceViaBootVolumeDetails {
    pub source_type: String,

    /// The OCID of the boot volume used to boot the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_id: Option<String>,
}

/// Required fields for InstanceConfigurationInstanceSourceViaBootVolumeDetails
pub struct InstanceConfigurationInstanceSourceViaBootVolumeDetailsRequired {
    pub source_type: String,
}

impl InstanceConfigurationInstanceSourceViaBootVolumeDetails {
    /// Create a new InstanceConfigurationInstanceSourceViaBootVolumeDetails with required fields
    pub fn new(required: InstanceConfigurationInstanceSourceViaBootVolumeDetailsRequired) -> Self {
        Self {
            source_type: required.source_type,

            boot_volume_id: None,
        }
    }

    /// Set boot_volume_id
    pub fn set_boot_volume_id(mut self, value: Option<String>) -> Self {
        self.boot_volume_id = value;
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: String) -> Self {
        self.source_type = value;
        self
    }

    /// Set boot_volume_id (unwraps Option)
    pub fn with_boot_volume_id(mut self, value: impl Into<String>) -> Self {
        self.boot_volume_id = Some(value.into());
        self
    }
}
