use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteVolumeGroupRequest {
    /// The Oracle Cloud ID (OCID) that uniquely identifies the volume group.
    pub volume_group_id: String,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for DeleteVolumeGroupRequest
pub struct DeleteVolumeGroupRequestRequired {
    /// The Oracle Cloud ID (OCID) that uniquely identifies the volume group.
    pub volume_group_id: String,
}

impl DeleteVolumeGroupRequest {
    /// Create a new DeleteVolumeGroupRequest with required fields
    pub fn new(required: DeleteVolumeGroupRequestRequired) -> Self {
        Self {
            volume_group_id: required.volume_group_id,

            if_match: None,
}
    }

    /// Set volume_group_id
    pub fn set_volume_group_id(mut self, value: String) -> Self {
        self.volume_group_id = value;
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


