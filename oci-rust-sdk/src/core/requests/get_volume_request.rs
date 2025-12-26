use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeRequest {
    /// The OCID of the volume.
    pub volume_id: String,
}


/// Required fields for GetVolumeRequest
pub struct GetVolumeRequestRequired {
    /// The OCID of the volume.
    pub volume_id: String,
}

impl GetVolumeRequest {
    /// Create a new GetVolumeRequest with required fields
    pub fn new(required: GetVolumeRequestRequired) -> Self {
        Self {
            volume_id: required.volume_id,
}
    }

    /// Set volume_id
    pub fn set_volume_id(mut self, value: String) -> Self {
        self.volume_id = value;
        self
    }
}


