use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the volume that the volume backup policy is assigned to. <p> For more information about Oracle defined backup policies and custom backup policies, see [Policy-Based Backups](https://docs.oracle.com/iaas/Content/Block/Tasks/schedulingvolumebackups.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeBackupPolicyAssignment {
    /// The OCID of the volume the policy has been assigned to.
    pub asset_id: String,

    /// The OCID of the volume backup policy assignment.
    pub id: String,

    /// The OCID of the volume backup policy that has been assigned to the volume.
    pub policy_id: String,

    /// The date and time the volume backup policy was assigned to the volume. The format is defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,

    /// The OCID of the Vault service key which is the master encryption key for the block / boot volume cross region backups, which will be used in the destination region to encrypt the backup's encryption keys. For more information about the Vault service and encryption keys, see [Overview of Vault service](https://docs.oracle.com/iaas/Content/KeyManagement/Concepts/keyoverview.htm) and [Using Keys](https://docs.oracle.com/iaas/Content/KeyManagement/Tasks/usingkeys.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xrc_kms_key_id: Option<String>,
}

/// Required fields for VolumeBackupPolicyAssignment
pub struct VolumeBackupPolicyAssignmentRequired {
    /// The OCID of the volume the policy has been assigned to.
    pub asset_id: String,

    /// The OCID of the volume backup policy assignment.
    pub id: String,

    /// The OCID of the volume backup policy that has been assigned to the volume.
    pub policy_id: String,

    /// The date and time the volume backup policy was assigned to the volume. The format is defined by [RFC3339](https://tools.ietf.org/html/rfc3339).
    pub time_created: DateTime<Utc>,
}

impl VolumeBackupPolicyAssignment {
    /// Create a new VolumeBackupPolicyAssignment with required fields
    pub fn new(required: VolumeBackupPolicyAssignmentRequired) -> Self {
        Self {
            asset_id: required.asset_id,

            id: required.id,

            policy_id: required.policy_id,

            time_created: required.time_created,

            xrc_kms_key_id: None,
        }
    }

    /// Set asset_id
    pub fn set_asset_id(mut self, value: String) -> Self {
        self.asset_id = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set policy_id
    pub fn set_policy_id(mut self, value: String) -> Self {
        self.policy_id = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set xrc_kms_key_id
    pub fn set_xrc_kms_key_id(mut self, value: Option<String>) -> Self {
        self.xrc_kms_key_id = value;
        self
    }

    /// Set xrc_kms_key_id (unwraps Option)
    pub fn with_xrc_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.xrc_kms_key_id = Some(value.into());
        self
    }
}
