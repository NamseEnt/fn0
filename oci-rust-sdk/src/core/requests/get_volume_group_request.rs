use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeGroupRequest {
    /// The Oracle Cloud ID (OCID) that uniquely identifies the volume group.
    pub volume_group_id: String,
}


/// Required fields for GetVolumeGroupRequest
pub struct GetVolumeGroupRequestRequired {
    /// The Oracle Cloud ID (OCID) that uniquely identifies the volume group.
    pub volume_group_id: String,
}

impl GetVolumeGroupRequest {
    /// Create a new GetVolumeGroupRequest with required fields
    pub fn new(required: GetVolumeGroupRequestRequired) -> Self {
        Self {
            volume_group_id: required.volume_group_id,
}
    }

    /// Set volume_group_id
    pub fn set_volume_group_id(mut self, value: String) -> Self {
        self.volume_group_id = value;
        self
    }
}


