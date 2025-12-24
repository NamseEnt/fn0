use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A point-in-time copy of a boot volume that can then be used to create a new boot volume or recover a boot volume. For more information, see [Overview of Boot Volume Backups](https://docs.oracle.com/iaas/Content/Block/Concepts/bootvolumebackups.htm) To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm). <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootVolumeBackup {
    /// The OCID of the compartment that contains the boot volume backup.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the boot volume backup.
    pub id: String,

    /// The current state of a boot volume backup.
    pub lifecycle_state: BootVolumeBackupLifecycleState,

    /// The date and time the boot volume backup was created. This is the time the actual point-in-time image of the volume data was taken. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The OCID of the boot volume.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// System tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"foo-namespace\": {\"bar-key\": \"value\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The date and time the volume backup will expire and be automatically deleted. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). This parameter will always be present for backups that were created automatically by a scheduled-backup policy. For manually created backups, it will be absent, signifying that there is no expiration time and the backup will last forever until manually deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<DateTime<Utc>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The image OCID used to create the boot volume the backup is taken from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,

    /// The OCID of the Vault service master encryption assigned to the boot volume backup. For more information about the Vault service and encryption keys, see [Overview of Vault service](https://docs.oracle.com/iaas/Content/KeyManagement/Concepts/keyoverview.htm) and [Using Keys](https://docs.oracle.com/iaas/Content/KeyManagement/Tasks/usingkeys.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,

    /// The size of the boot volume, in GBs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_g_bs: Option<i64>,

    /// The OCID of the source boot volume backup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_boot_volume_backup_id: Option<String>,

    /// Specifies whether the backup was created manually, or via scheduled backup policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<BootVolumeBackupSourceType>,

    /// The date and time the request to create the boot volume backup was received. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_request_received: Option<DateTime<Utc>>,

    /// The type of a volume backup.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<BootVolumeBackupType>,

    /// The size used by the backup, in GBs. It is typically smaller than sizeInGBs, depending on the space consumed on the boot volume and whether the backup is full or incremental. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_size_in_g_bs: Option<i64>,
}

/// Required fields for BootVolumeBackup
pub struct BootVolumeBackupRequired {
    /// The OCID of the compartment that contains the boot volume backup.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the boot volume backup.
    pub id: String,

    /// The current state of a boot volume backup.
    pub lifecycle_state: BootVolumeBackupLifecycleState,

    /// The date and time the boot volume backup was created. This is the time the actual point-in-time image of the volume data was taken. Format defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,
}

impl BootVolumeBackup {
    /// Create a new BootVolumeBackup with required fields
    pub fn new(required: BootVolumeBackupRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            display_name: required.display_name,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            boot_volume_id: None,

            defined_tags: None,

            system_tags: None,

            expiration_time: None,

            freeform_tags: None,

            image_id: None,

            kms_key_id: None,

            size_in_g_bs: None,

            source_boot_volume_backup_id: None,

            source_type: None,

            time_request_received: None,

            r#type: None,

            unique_size_in_g_bs: None,
        }
    }

    /// Set boot_volume_id
    pub fn set_boot_volume_id(mut self, value: Option<String>) -> Self {
        self.boot_volume_id = value;
        self
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

    /// Set system_tags
    pub fn set_system_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.system_tags = value;
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

    /// Set image_id
    pub fn set_image_id(mut self, value: Option<String>) -> Self {
        self.image_id = value;
        self
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, value: Option<String>) -> Self {
        self.kms_key_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: BootVolumeBackupLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set size_in_g_bs
    pub fn set_size_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.size_in_g_bs = value;
        self
    }

    /// Set source_boot_volume_backup_id
    pub fn set_source_boot_volume_backup_id(mut self, value: Option<String>) -> Self {
        self.source_boot_volume_backup_id = value;
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: Option<BootVolumeBackupSourceType>) -> Self {
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
    pub fn set_type(mut self, value: Option<BootVolumeBackupType>) -> Self {
        self.r#type = value;
        self
    }

    /// Set unique_size_in_g_bs
    pub fn set_unique_size_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.unique_size_in_g_bs = value;
        self
    }

    /// Set boot_volume_id (unwraps Option)
    pub fn with_boot_volume_id(mut self, value: impl Into<String>) -> Self {
        self.boot_volume_id = Some(value.into());
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

    /// Set system_tags (unwraps Option)
    pub fn with_system_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.system_tags = Some(value);
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

    /// Set image_id (unwraps Option)
    pub fn with_image_id(mut self, value: impl Into<String>) -> Self {
        self.image_id = Some(value.into());
        self
    }

    /// Set kms_key_id (unwraps Option)
    pub fn with_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.kms_key_id = Some(value.into());
        self
    }

    /// Set size_in_g_bs (unwraps Option)
    pub fn with_size_in_g_bs(mut self, value: i64) -> Self {
        self.size_in_g_bs = Some(value);
        self
    }

    /// Set source_boot_volume_backup_id (unwraps Option)
    pub fn with_source_boot_volume_backup_id(mut self, value: impl Into<String>) -> Self {
        self.source_boot_volume_backup_id = Some(value.into());
        self
    }

    /// Set source_type (unwraps Option)
    pub fn with_source_type(mut self, value: BootVolumeBackupSourceType) -> Self {
        self.source_type = Some(value);
        self
    }

    /// Set time_request_received (unwraps Option)
    pub fn with_time_request_received(mut self, value: DateTime<Utc>) -> Self {
        self.time_request_received = Some(value);
        self
    }

    /// Set r#type (unwraps Option)
    pub fn with_type(mut self, value: BootVolumeBackupType) -> Self {
        self.r#type = Some(value);
        self
    }

    /// Set unique_size_in_g_bs (unwraps Option)
    pub fn with_unique_size_in_g_bs(mut self, value: i64) -> Self {
        self.unique_size_in_g_bs = Some(value);
        self
    }
}
