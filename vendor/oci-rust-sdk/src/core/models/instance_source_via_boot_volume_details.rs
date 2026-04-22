use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSourceViaBootVolumeDetails {
    /// The OCID of the boot volume used to boot the instance.
    pub boot_volume_id: String,

    pub source_type: String,
}

/// Required fields for InstanceSourceViaBootVolumeDetails
pub struct InstanceSourceViaBootVolumeDetailsRequired {
    /// The OCID of the boot volume used to boot the instance.
    pub boot_volume_id: String,

    pub source_type: String,
}

impl InstanceSourceViaBootVolumeDetails {
    /// Create a new InstanceSourceViaBootVolumeDetails with required fields
    pub fn new(required: InstanceSourceViaBootVolumeDetailsRequired) -> Self {
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
