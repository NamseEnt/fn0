use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Create new block volumes or attach to an existing volume. Specify either createDetails or volumeId.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationBlockVolumeDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attach_details: Option<InstanceConfigurationIscsiAttachVolumeDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_details: Option<InstanceConfigurationCreateVolumeDetails>,

    /// The OCID of the volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_id: Option<String>,
}

impl InstanceConfigurationBlockVolumeDetails {
    /// Create a new InstanceConfigurationBlockVolumeDetails
    pub fn new() -> Self {
        Self {
            attach_details: None,

            create_details: None,

            volume_id: None,
        }
    }

    /// Set attach_details
    pub fn set_attach_details(
        mut self,
        value: Option<InstanceConfigurationIscsiAttachVolumeDetails>,
    ) -> Self {
        self.attach_details = value;
        self
    }

    /// Set create_details
    pub fn set_create_details(
        mut self,
        value: Option<InstanceConfigurationCreateVolumeDetails>,
    ) -> Self {
        self.create_details = value;
        self
    }

    /// Set volume_id
    pub fn set_volume_id(mut self, value: Option<String>) -> Self {
        self.volume_id = value;
        self
    }

    /// Set attach_details (unwraps Option)
    pub fn with_attach_details(
        mut self,
        value: InstanceConfigurationIscsiAttachVolumeDetails,
    ) -> Self {
        self.attach_details = Some(value);
        self
    }

    /// Set create_details (unwraps Option)
    pub fn with_create_details(mut self, value: InstanceConfigurationCreateVolumeDetails) -> Self {
        self.create_details = Some(value);
        self
    }

    /// Set volume_id (unwraps Option)
    pub fn with_volume_id(mut self, value: impl Into<String>) -> Self {
        self.volume_id = Some(value.into());
        self
    }
}

impl Default for InstanceConfigurationBlockVolumeDetails {
    fn default() -> Self {
        Self::new()
    }
}
