use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the source boot volume replica which the boot volume will be created from. The boot volume replica shoulbe be in the same availability domain as the boot volume. Only one volume can be created from a replica at the same time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootVolumeSourceFromBootVolumeReplicaDetails {
    /// The OCID of the boot volume replica.
    pub id: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for BootVolumeSourceFromBootVolumeReplicaDetails
pub struct BootVolumeSourceFromBootVolumeReplicaDetailsRequired {
    /// The OCID of the boot volume replica.
    pub id: String,

    pub r#type: String,
}

impl BootVolumeSourceFromBootVolumeReplicaDetails {
    /// Create a new BootVolumeSourceFromBootVolumeReplicaDetails with required fields
    pub fn new(required: BootVolumeSourceFromBootVolumeReplicaDetailsRequired) -> Self {
        Self {
            id: required.id,

            r#type: required.r#type,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
