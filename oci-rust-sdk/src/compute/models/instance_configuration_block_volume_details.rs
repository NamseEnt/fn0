use serde::{Deserialize, Serialize};

/// Create new block volumes or attach to an existing volume. Specify either createDetails or volumeId.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationBlockVolumeDetails {
    /// Volume attachment details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attach_details: Option<super::InstanceConfigurationAttachVolumeDetails>,

    /// Details for creating a new block volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_details: Option<super::InstanceConfigurationCreateVolumeDetails>,

    /// The OCID of the volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_id: Option<String>,
}

impl InstanceConfigurationBlockVolumeDetails {
    pub fn new() -> Self {
        Self {
            attach_details: None,
            create_details: None,
            volume_id: None,
        }
    }

    pub fn with_attach_details(
        mut self,
        details: super::InstanceConfigurationAttachVolumeDetails,
    ) -> Self {
        self.attach_details = Some(details);
        self
    }

    pub fn with_create_details(
        mut self,
        details: super::InstanceConfigurationCreateVolumeDetails,
    ) -> Self {
        self.create_details = Some(details);
        self
    }

    pub fn with_volume_id(mut self, id: impl Into<String>) -> Self {
        self.volume_id = Some(id.into());
        self
    }

    /// Set attach_details
    pub fn set_attach_details(mut self, attach_details: Option<super::InstanceConfigurationAttachVolumeDetails>) -> Self {
        self.attach_details = attach_details;
        self
    }

    /// Set create_details
    pub fn set_create_details(mut self, create_details: Option<super::InstanceConfigurationCreateVolumeDetails>) -> Self {
        self.create_details = create_details;
        self
    }

    /// Set volume_id
    pub fn set_volume_id(mut self, volume_id: Option<String>) -> Self {
        self.volume_id = volume_id;
        self
    }
}

impl Default for InstanceConfigurationBlockVolumeDetails {
    fn default() -> Self {
        Self::new()
    }
}
