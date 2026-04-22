use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVolumeGroupRequest {
    /// The Oracle Cloud ID (OCID) that uniquely identifies the volume group.
    pub volume_group_id: String,

    /// Update volume group's set of volumes and/or display name
    pub update_volume_group_details: UpdateVolumeGroupDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,

    /// Specifies whether to disable or preserve the individual volume replication when removing a volume from the replication enabled volume group. When set to {@code true}, the individual volume replica is preserved. The default value is {@code true}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_volume_replica: Option<bool>,
}

/// Required fields for UpdateVolumeGroupRequest
pub struct UpdateVolumeGroupRequestRequired {
    /// The Oracle Cloud ID (OCID) that uniquely identifies the volume group.
    pub volume_group_id: String,

    /// Update volume group's set of volumes and/or display name
    pub update_volume_group_details: UpdateVolumeGroupDetails,
}

impl UpdateVolumeGroupRequest {
    /// Create a new UpdateVolumeGroupRequest with required fields
    pub fn new(required: UpdateVolumeGroupRequestRequired) -> Self {
        Self {
            volume_group_id: required.volume_group_id,

            update_volume_group_details: required.update_volume_group_details,

            if_match: None,

            preserve_volume_replica: None,
        }
    }

    /// Set volume_group_id
    pub fn set_volume_group_id(mut self, value: String) -> Self {
        self.volume_group_id = value;
        self
    }

    /// Set update_volume_group_details
    pub fn set_update_volume_group_details(mut self, value: UpdateVolumeGroupDetails) -> Self {
        self.update_volume_group_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set preserve_volume_replica
    pub fn set_preserve_volume_replica(mut self, value: Option<bool>) -> Self {
        self.preserve_volume_replica = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }

    /// Set preserve_volume_replica (unwraps Option)
    pub fn with_preserve_volume_replica(mut self, value: bool) -> Self {
        self.preserve_volume_replica = Some(value);
        self
    }
}
