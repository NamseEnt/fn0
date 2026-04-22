use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVolumeBackupRequest {
    /// The OCID of the volume backup.
    pub volume_backup_id: String,

    /// Update volume backup fields
    pub update_volume_backup_details: UpdateVolumeBackupDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}

/// Required fields for UpdateVolumeBackupRequest
pub struct UpdateVolumeBackupRequestRequired {
    /// The OCID of the volume backup.
    pub volume_backup_id: String,

    /// Update volume backup fields
    pub update_volume_backup_details: UpdateVolumeBackupDetails,
}

impl UpdateVolumeBackupRequest {
    /// Create a new UpdateVolumeBackupRequest with required fields
    pub fn new(required: UpdateVolumeBackupRequestRequired) -> Self {
        Self {
            volume_backup_id: required.volume_backup_id,

            update_volume_backup_details: required.update_volume_backup_details,

            if_match: None,
        }
    }

    /// Set volume_backup_id
    pub fn set_volume_backup_id(mut self, value: String) -> Self {
        self.volume_backup_id = value;
        self
    }

    /// Set update_volume_backup_details
    pub fn set_update_volume_backup_details(mut self, value: UpdateVolumeBackupDetails) -> Self {
        self.update_volume_backup_details = value;
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
