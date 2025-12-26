use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBootVolumeRequest {
    /// The OCID of the boot volume.
    pub boot_volume_id: String,
}

/// Required fields for GetBootVolumeRequest
pub struct GetBootVolumeRequestRequired {
    /// The OCID of the boot volume.
    pub boot_volume_id: String,
}

impl GetBootVolumeRequest {
    /// Create a new GetBootVolumeRequest with required fields
    pub fn new(required: GetBootVolumeRequestRequired) -> Self {
        Self {
            boot_volume_id: required.boot_volume_id,
        }
    }

    /// Set boot_volume_id
    pub fn set_boot_volume_id(mut self, value: String) -> Self {
        self.boot_volume_id = value;
        self
    }
}
