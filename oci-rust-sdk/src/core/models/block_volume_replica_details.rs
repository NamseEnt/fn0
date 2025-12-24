use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Contains the details for the block volume replica
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockVolumeReplicaDetails {
    /// The availability domain of the block volume replica. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The OCID of the Vault service key which is the master encryption key for the cross region block volume replicas, which will be used in the destination region to encrypt the block volume replica's encryption keys. For more information about the Vault service and encryption keys, see [Overview of Vault service](https://docs.oracle.com/iaas/Content/KeyManagement/Concepts/keyoverview.htm) and [Using Keys](https://docs.oracle.com/iaas/Content/KeyManagement/Tasks/usingkeys.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xrr_kms_key_id: Option<String>,
}

/// Required fields for BlockVolumeReplicaDetails
pub struct BlockVolumeReplicaDetailsRequired {
    /// The availability domain of the block volume replica. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,
}

impl BlockVolumeReplicaDetails {
    /// Create a new BlockVolumeReplicaDetails with required fields
    pub fn new(required: BlockVolumeReplicaDetailsRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            display_name: None,

            xrr_kms_key_id: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set xrr_kms_key_id
    pub fn set_xrr_kms_key_id(mut self, value: Option<String>) -> Self {
        self.xrr_kms_key_id = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set xrr_kms_key_id (unwraps Option)
    pub fn with_xrr_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.xrr_kms_key_id = Some(value.into());
        self
    }
}
