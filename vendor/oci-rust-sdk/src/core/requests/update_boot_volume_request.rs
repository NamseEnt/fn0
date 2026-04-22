use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBootVolumeRequest {
    /// The OCID of the boot volume.
    pub boot_volume_id: String,

    /// Update boot volume's display name.
    pub update_boot_volume_details: UpdateBootVolumeDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateBootVolumeRequest
pub struct UpdateBootVolumeRequestRequired {
    /// The OCID of the boot volume.
    pub boot_volume_id: String,

    /// Update boot volume's display name.
    pub update_boot_volume_details: UpdateBootVolumeDetails,
}

impl UpdateBootVolumeRequest {
    /// Create a new UpdateBootVolumeRequest with required fields
    pub fn new(required: UpdateBootVolumeRequestRequired) -> Self {
        Self {
            boot_volume_id: required.boot_volume_id,

            update_boot_volume_details: required.update_boot_volume_details,

            if_match: None,
        }
    }

    /// Set boot_volume_id
    pub fn set_boot_volume_id(mut self, value: String) -> Self {
        self.boot_volume_id = value;
        self
    }

    /// Set update_boot_volume_details
    pub fn set_update_boot_volume_details(mut self, value: UpdateBootVolumeDetails) -> Self {
        self.update_boot_volume_details = value;
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
