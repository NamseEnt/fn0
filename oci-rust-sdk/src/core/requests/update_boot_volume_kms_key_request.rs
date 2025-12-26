use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBootVolumeKmsKeyRequest {
    /// The OCID of the boot volume.
    pub boot_volume_id: String,

    /// Updates the Vault service master encryption key assigned to the specified boot volume.
    pub update_boot_volume_kms_key_details: UpdateBootVolumeKmsKeyDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateBootVolumeKmsKeyRequest
pub struct UpdateBootVolumeKmsKeyRequestRequired {
    /// The OCID of the boot volume.
    pub boot_volume_id: String,

    /// Updates the Vault service master encryption key assigned to the specified boot volume.
    pub update_boot_volume_kms_key_details: UpdateBootVolumeKmsKeyDetails,
}

impl UpdateBootVolumeKmsKeyRequest {
    /// Create a new UpdateBootVolumeKmsKeyRequest with required fields
    pub fn new(required: UpdateBootVolumeKmsKeyRequestRequired) -> Self {
        Self {
            boot_volume_id: required.boot_volume_id,

            update_boot_volume_kms_key_details: required.update_boot_volume_kms_key_details,

            if_match: None,
}
    }

    /// Set boot_volume_id
    pub fn set_boot_volume_id(mut self, value: String) -> Self {
        self.boot_volume_id = value;
        self
    }

    /// Set update_boot_volume_kms_key_details
    pub fn set_update_boot_volume_kms_key_details(mut self, value: UpdateBootVolumeKmsKeyDetails) -> Self {
        self.update_boot_volume_kms_key_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}


