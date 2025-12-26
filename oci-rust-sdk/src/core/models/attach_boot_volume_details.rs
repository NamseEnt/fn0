use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachBootVolumeDetails {
    /// The OCID of the  boot volume.
    pub boot_volume_id: String,

    /// The OCID of the instance.
    pub instance_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Refer the top-level definition of encryptionInTransitType. The default value is NONE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_in_transit_type: Option<EncryptionInTransitType>,
}

/// Required fields for AttachBootVolumeDetails
pub struct AttachBootVolumeDetailsRequired {
    /// The OCID of the  boot volume.
    pub boot_volume_id: String,

    /// The OCID of the instance.
    pub instance_id: String,
}

impl AttachBootVolumeDetails {
    /// Create a new AttachBootVolumeDetails with required fields
    pub fn new(required: AttachBootVolumeDetailsRequired) -> Self {
        Self {
            boot_volume_id: required.boot_volume_id,

            instance_id: required.instance_id,

            display_name: None,

            encryption_in_transit_type: None,
        }
    }

    /// Set boot_volume_id
    pub fn set_boot_volume_id(mut self, value: String) -> Self {
        self.boot_volume_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }

    /// Set encryption_in_transit_type
    pub fn set_encryption_in_transit_type(
        mut self,
        value: Option<EncryptionInTransitType>,
    ) -> Self {
        self.encryption_in_transit_type = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set encryption_in_transit_type (unwraps Option)
    pub fn with_encryption_in_transit_type(mut self, value: EncryptionInTransitType) -> Self {
        self.encryption_in_transit_type = Some(value);
        self
    }
}
