use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the volumes in a volume group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeGroupSourceFromVolumesDetails {
    /// OCIDs for the volumes in this volume group.
    pub volume_ids: Vec<String>,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for VolumeGroupSourceFromVolumesDetails
pub struct VolumeGroupSourceFromVolumesDetailsRequired {
    /// OCIDs for the volumes in this volume group.
    pub volume_ids: Vec<String>,

    pub r#type: String,
}

impl VolumeGroupSourceFromVolumesDetails {
    /// Create a new VolumeGroupSourceFromVolumesDetails with required fields
    pub fn new(required: VolumeGroupSourceFromVolumesDetailsRequired) -> Self {
        Self {
            volume_ids: required.volume_ids,

            r#type: required.r#type,
        }
    }

    /// Set volume_ids
    pub fn set_volume_ids(mut self, value: Vec<String>) -> Self {
        self.volume_ids = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
