use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A point-in-time copy of a volume group that can then be used to create a new volume group or restore a volume group. For more information, see [Volume Groups](https://docs.oracle.com/iaas/Content/Block/Concepts/volumegroups.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm). <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeGroupBackup {
    /// The OCID of the compartment that contains the volume group backup.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the volume group backup.
    pub id: String,

    /// The current state of a volume group backup.
    pub lifecycle_state: VolumeGroupBackupLifecycleState,

    /// The date and time the volume group backup was created. This is the time the actual point-in-time image of the volume group data was taken. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The type of backup.
    #[serde(rename = "type")]
    pub r#type: VolumeGroupBackupType,

    /// OCIDs for the volume backups in this volume group backup.
    pub volume_backup_ids: Vec<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The date and time the volume group backup will expire and be automatically deleted. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). This parameter will always be present for volume group backups that were created automatically by a scheduled-backup policy. For manually created volume group backups, it will be absent, signifying that there is no expiration time and the backup will last forever until manually deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<DateTime<Utc>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The aggregate size of the volume group backup, in MBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_m_bs: Option<i64>,

    /// The aggregate size of the volume group backup, in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_g_bs: Option<i64>,

    /// Specifies whether the volume group backup was created manually, or via scheduled backup policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<VolumeGroupBackupSourceType>,

    /// The date and time the request to create the volume group backup was received. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_request_received: Option<DateTime<Utc>>,

    /// The aggregate size used by the volume group backup, in MBs. <p> It is typically smaller than sizeInMBs, depending on the spaceconsumed on the volume group and whether the volume backup is full or incremental. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_size_in_mbs: Option<i64>,

    /// The aggregate size used by the volume group backup, in GBs. <p> It is typically smaller than sizeInGBs, depending on the spaceconsumed on the volume group and whether the volume backup is full or incremental. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_size_in_gbs: Option<i64>,

    /// The OCID of the source volume group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_group_id: Option<String>,

    /// The OCID of the source volume group backup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_volume_group_backup_id: Option<String>,
}

/// Required fields for VolumeGroupBackup
pub struct VolumeGroupBackupRequired {
    /// The OCID of the compartment that contains the volume group backup.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the volume group backup.
    pub id: String,

    /// The current state of a volume group backup.
    pub lifecycle_state: VolumeGroupBackupLifecycleState,

    /// The date and time the volume group backup was created. This is the time the actual point-in-time image of the volume group data was taken. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The type of backup.
    pub r#type: VolumeGroupBackupType,

    /// OCIDs for the volume backups in this volume group backup.
    pub volume_backup_ids: Vec<String>,
}

impl VolumeGroupBackup {
    /// Create a new VolumeGroupBackup with required fields
    pub fn new(required: VolumeGroupBackupRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            display_name: required.display_name,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            r#type: required.r#type,

            volume_backup_ids: required.volume_backup_ids,

            defined_tags: None,

            expiration_time: None,

            freeform_tags: None,

            size_in_m_bs: None,

            size_in_g_bs: None,

            source_type: None,

            time_request_received: None,

            unique_size_in_mbs: None,

            unique_size_in_gbs: None,

            volume_group_id: None,

            source_volume_group_backup_id: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set expiration_time
    pub fn set_expiration_time(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.expiration_time = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: VolumeGroupBackupLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set size_in_m_bs
    pub fn set_size_in_m_bs(mut self, value: Option<i64>) -> Self {
        self.size_in_m_bs = value;
        self
    }

    /// Set size_in_g_bs
    pub fn set_size_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.size_in_g_bs = value;
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: Option<VolumeGroupBackupSourceType>) -> Self {
        self.source_type = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_request_received
    pub fn set_time_request_received(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_request_received = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: VolumeGroupBackupType) -> Self {
        self.r#type = value;
        self
    }

    /// Set unique_size_in_mbs
    pub fn set_unique_size_in_mbs(mut self, value: Option<i64>) -> Self {
        self.unique_size_in_mbs = value;
        self
    }

    /// Set unique_size_in_gbs
    pub fn set_unique_size_in_gbs(mut self, value: Option<i64>) -> Self {
        self.unique_size_in_gbs = value;
        self
    }

    /// Set volume_backup_ids
    pub fn set_volume_backup_ids(mut self, value: Vec<String>) -> Self {
        self.volume_backup_ids = value;
        self
    }

    /// Set volume_group_id
    pub fn set_volume_group_id(mut self, value: Option<String>) -> Self {
        self.volume_group_id = value;
        self
    }

    /// Set source_volume_group_backup_id
    pub fn set_source_volume_group_backup_id(mut self, value: Option<String>) -> Self {
        self.source_volume_group_backup_id = value;
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set expiration_time (unwraps Option)
    pub fn with_expiration_time(mut self, value: DateTime<Utc>) -> Self {
        self.expiration_time = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set size_in_m_bs (unwraps Option)
    pub fn with_size_in_m_bs(mut self, value: i64) -> Self {
        self.size_in_m_bs = Some(value);
        self
    }

    /// Set size_in_g_bs (unwraps Option)
    pub fn with_size_in_g_bs(mut self, value: i64) -> Self {
        self.size_in_g_bs = Some(value);
        self
    }

    /// Set source_type (unwraps Option)
    pub fn with_source_type(mut self, value: VolumeGroupBackupSourceType) -> Self {
        self.source_type = Some(value);
        self
    }

    /// Set time_request_received (unwraps Option)
    pub fn with_time_request_received(mut self, value: DateTime<Utc>) -> Self {
        self.time_request_received = Some(value);
        self
    }

    /// Set unique_size_in_mbs (unwraps Option)
    pub fn with_unique_size_in_mbs(mut self, value: i64) -> Self {
        self.unique_size_in_mbs = Some(value);
        self
    }

    /// Set unique_size_in_gbs (unwraps Option)
    pub fn with_unique_size_in_gbs(mut self, value: i64) -> Self {
        self.unique_size_in_gbs = Some(value);
        self
    }

    /// Set volume_group_id (unwraps Option)
    pub fn with_volume_group_id(mut self, value: impl Into<String>) -> Self {
        self.volume_group_id = Some(value.into());
        self
    }

    /// Set source_volume_group_backup_id (unwraps Option)
    pub fn with_source_volume_group_backup_id(mut self, value: impl Into<String>) -> Self {
        self.source_volume_group_backup_id = Some(value.into());
        self
    }
}
