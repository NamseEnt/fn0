use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVolumeKmsKeyRequest {
    /// The OCID of the volume.
    pub volume_id: String,

    /// Updates the Vault service master encryption key assigned to the specified volume.
    pub update_volume_kms_key_details: UpdateVolumeKmsKeyDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateVolumeKmsKeyRequest
pub struct UpdateVolumeKmsKeyRequestRequired {
    /// The OCID of the volume.
    pub volume_id: String,

    /// Updates the Vault service master encryption key assigned to the specified volume.
    pub update_volume_kms_key_details: UpdateVolumeKmsKeyDetails,
}

impl UpdateVolumeKmsKeyRequest {
    /// Create a new UpdateVolumeKmsKeyRequest with required fields
    pub fn new(required: UpdateVolumeKmsKeyRequestRequired) -> Self {
        Self {
            volume_id: required.volume_id,

            update_volume_kms_key_details: required.update_volume_kms_key_details,

            if_match: None,
}
    }

    /// Set volume_id
    pub fn set_volume_id(mut self, value: String) -> Self {
        self.volume_id = value;
        self
    }

    /// Set update_volume_kms_key_details
    pub fn set_update_volume_kms_key_details(mut self, value: UpdateVolumeKmsKeyDetails) -> Self {
        self.update_volume_kms_key_details = value;
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


