use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVolumeBackupPolicyAssignmentDetails {
    /// The OCID of the volume to assign the policy to.
    pub asset_id: String,

    /// The OCID of the volume backup policy to assign to the volume.
    pub policy_id: String,

    /// The OCID of the Vault service key which is the master encryption key for the block / boot volume cross region backups, which will be used in the destination region to encrypt the backup's encryption keys. For more information about the Vault service and encryption keys, see [Overview of Vault service](https://docs.oracle.com/iaas/Content/KeyManagement/Concepts/keyoverview.htm) and [Using Keys](https://docs.oracle.com/iaas/Content/KeyManagement/Tasks/usingkeys.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xrc_kms_key_id: Option<String>,
}

/// Required fields for CreateVolumeBackupPolicyAssignmentDetails
pub struct CreateVolumeBackupPolicyAssignmentDetailsRequired {
    /// The OCID of the volume to assign the policy to.
    pub asset_id: String,

    /// The OCID of the volume backup policy to assign to the volume.
    pub policy_id: String,
}

impl CreateVolumeBackupPolicyAssignmentDetails {
    /// Create a new CreateVolumeBackupPolicyAssignmentDetails with required fields
    pub fn new(required: CreateVolumeBackupPolicyAssignmentDetailsRequired) -> Self {
        Self {
            asset_id: required.asset_id,

            policy_id: required.policy_id,

            xrc_kms_key_id: None,
        }
    }

    /// Set asset_id
    pub fn set_asset_id(mut self, value: String) -> Self {
        self.asset_id = value;
        self
    }

    /// Set policy_id
    pub fn set_policy_id(mut self, value: String) -> Self {
        self.policy_id = value;
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
