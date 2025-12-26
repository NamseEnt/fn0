use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the volume group to clone from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeGroupSourceFromVolumeGroupDetails {
    /// The OCID of the volume group to clone from.
    pub volume_group_id: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for VolumeGroupSourceFromVolumeGroupDetails
pub struct VolumeGroupSourceFromVolumeGroupDetailsRequired {
    /// The OCID of the volume group to clone from.
    pub volume_group_id: String,

    pub r#type: String,
}

impl VolumeGroupSourceFromVolumeGroupDetails {
    /// Create a new VolumeGroupSourceFromVolumeGroupDetails with required fields
    pub fn new(required: VolumeGroupSourceFromVolumeGroupDetailsRequired) -> Self {
        Self {
            volume_group_id: required.volume_group_id,

            r#type: required.r#type,
        }
    }

    /// Set volume_group_id
    pub fn set_volume_group_id(mut self, value: String) -> Self {
        self.volume_group_id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
