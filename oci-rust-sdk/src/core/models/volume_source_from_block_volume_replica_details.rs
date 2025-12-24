use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the source block volume replica which the block volume will be created from. The block volume replica shoulbe be in the same availability domain as the block volume. Only one volume can be created from a replica at the same time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeSourceFromBlockVolumeReplicaDetails {
    /// The OCID of the block volume replica.
    pub id: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for VolumeSourceFromBlockVolumeReplicaDetails
pub struct VolumeSourceFromBlockVolumeReplicaDetailsRequired {
    /// The OCID of the block volume replica.
    pub id: String,

    pub r#type: String,
}

impl VolumeSourceFromBlockVolumeReplicaDetails {
    /// Create a new VolumeSourceFromBlockVolumeReplicaDetails with required fields
    pub fn new(required: VolumeSourceFromBlockVolumeReplicaDetailsRequired) -> Self {
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
