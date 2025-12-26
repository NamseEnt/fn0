use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about the block volume replica in the destination availability domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockVolumeReplicaInfo {
    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The block volume replica's Oracle ID (OCID).
    pub block_volume_replica_id: String,

    /// The availability domain of the block volume replica. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the Vault service key to assign as the master encryption key for the block volume replica, see [Overview of Vault service](https://docs.oracle.com/iaas/Content/KeyManagement/Concepts/keyoverview.htm) and [Using Keys](https://docs.oracle.com/iaas/Content/KeyManagement/Tasks/usingkeys.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,
}

/// Required fields for BlockVolumeReplicaInfo
pub struct BlockVolumeReplicaInfoRequired {
    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The block volume replica's Oracle ID (OCID).
    pub block_volume_replica_id: String,

    /// The availability domain of the block volume replica. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,
}

impl BlockVolumeReplicaInfo {
    /// Create a new BlockVolumeReplicaInfo with required fields
    pub fn new(required: BlockVolumeReplicaInfoRequired) -> Self {
        Self {
            display_name: required.display_name,

            block_volume_replica_id: required.block_volume_replica_id,

            availability_domain: required.availability_domain,

            kms_key_id: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set block_volume_replica_id
    pub fn set_block_volume_replica_id(mut self, value: String) -> Self {
        self.block_volume_replica_id = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, value: Option<String>) -> Self {
        self.kms_key_id = value;
        self
    }

    /// Set kms_key_id (unwraps Option)
    pub fn with_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.kms_key_id = Some(value.into());
        self
    }
}
