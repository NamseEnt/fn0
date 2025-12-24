use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The details for updating the instance source from an existing boot volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceSourceViaBootVolumeDetails {
    /// The OCID of the boot volume used to boot the instance.
    pub boot_volume_id: String,

    pub source_type: String,
}

/// Required fields for UpdateInstanceSourceViaBootVolumeDetails
pub struct UpdateInstanceSourceViaBootVolumeDetailsRequired {
    /// The OCID of the boot volume used to boot the instance.
    pub boot_volume_id: String,

    pub source_type: String,
}

impl UpdateInstanceSourceViaBootVolumeDetails {
    /// Create a new UpdateInstanceSourceViaBootVolumeDetails with required fields
    pub fn new(required: UpdateInstanceSourceViaBootVolumeDetailsRequired) -> Self {
        Self {
            boot_volume_id: required.boot_volume_id,

            source_type: required.source_type,
        }
    }

    /// Set boot_volume_id
    pub fn set_boot_volume_id(mut self, value: String) -> Self {
        self.boot_volume_id = value;
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: String) -> Self {
        self.source_type = value;
        self
    }
}
