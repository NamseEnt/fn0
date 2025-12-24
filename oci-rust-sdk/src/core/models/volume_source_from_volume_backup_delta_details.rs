use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the volume backups (first & second) and block size in bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeSourceFromVolumeBackupDeltaDetails {
    /// The OCID of the first volume backup.
    pub first_backup_id: String,

    /// The OCID of the second volume backup.
    pub second_backup_id: String,

    #[serde(rename = "type")]
    pub r#type: String,

    /// Block size in bytes to be considered while performing volume restore. The value must be a power of 2; ranging from 4KB (4096 bytes) to 1MB (1048576 bytes). If omitted, defaults to 4,096 bytes (4 KiB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_block_size_in_bytes: Option<i64>,
}

/// Required fields for VolumeSourceFromVolumeBackupDeltaDetails
pub struct VolumeSourceFromVolumeBackupDeltaDetailsRequired {
    /// The OCID of the first volume backup.
    pub first_backup_id: String,

    /// The OCID of the second volume backup.
    pub second_backup_id: String,

    pub r#type: String,
}

impl VolumeSourceFromVolumeBackupDeltaDetails {
    /// Create a new VolumeSourceFromVolumeBackupDeltaDetails with required fields
    pub fn new(required: VolumeSourceFromVolumeBackupDeltaDetailsRequired) -> Self {
        Self {
            first_backup_id: required.first_backup_id,

            second_backup_id: required.second_backup_id,

            r#type: required.r#type,

            change_block_size_in_bytes: None,
        }
    }

    /// Set first_backup_id
    pub fn set_first_backup_id(mut self, value: String) -> Self {
        self.first_backup_id = value;
        self
    }

    /// Set second_backup_id
    pub fn set_second_backup_id(mut self, value: String) -> Self {
        self.second_backup_id = value;
        self
    }

    /// Set change_block_size_in_bytes
    pub fn set_change_block_size_in_bytes(mut self, value: Option<i64>) -> Self {
        self.change_block_size_in_bytes = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set change_block_size_in_bytes (unwraps Option)
    pub fn with_change_block_size_in_bytes(mut self, value: i64) -> Self {
        self.change_block_size_in_bytes = Some(value);
        self
    }
}
