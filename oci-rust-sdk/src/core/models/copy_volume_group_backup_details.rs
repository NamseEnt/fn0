use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyVolumeGroupBackupDetails {
    /// The name of the destination region. <p> Example: {@code us-ashburn-1}
    pub destination_region: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The OCID of the Vault service key in the destination region which will be the master encryption key for the copied volume group backup. If you do not specify this attribute the volume group backup will be encrypted with the Oracle-provided encryption key when it is copied to the destination region. <p> For more information about the Vault service and encryption keys, see [Overview of Vault service](https://docs.oracle.com/iaas/Content/KeyManagement/Concepts/keyoverview.htm) and [Using Keys](https://docs.oracle.com/iaas/Content/KeyManagement/Tasks/usingkeys.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_id: Option<String>,
}

/// Required fields for CopyVolumeGroupBackupDetails
pub struct CopyVolumeGroupBackupDetailsRequired {
    /// The name of the destination region. <p> Example: {@code us-ashburn-1}
    pub destination_region: String,
}

impl CopyVolumeGroupBackupDetails {
    /// Create a new CopyVolumeGroupBackupDetails with required fields
    pub fn new(required: CopyVolumeGroupBackupDetailsRequired) -> Self {
        Self {
            destination_region: required.destination_region,

            display_name: None,

            kms_key_id: None,
        }
    }

    /// Set destination_region
    pub fn set_destination_region(mut self, value: String) -> Self {
        self.destination_region = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set kms_key_id
    pub fn set_kms_key_id(mut self, value: Option<String>) -> Self {
        self.kms_key_id = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set kms_key_id (unwraps Option)
    pub fn with_kms_key_id(mut self, value: impl Into<String>) -> Self {
        self.kms_key_id = Some(value.into());
        self
    }
}
