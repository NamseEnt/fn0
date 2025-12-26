use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the source volume replica which the volume group will be created from. The volume group replica shoulbe be in the same availability domain as the volume group. Only one volume group can be created from a volume group replica at the same time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeGroupSourceFromVolumeGroupReplicaDetails {
    /// The OCID of the volume group replica.
    pub volume_group_replica_id: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for VolumeGroupSourceFromVolumeGroupReplicaDetails
pub struct VolumeGroupSourceFromVolumeGroupReplicaDetailsRequired {
    /// The OCID of the volume group replica.
    pub volume_group_replica_id: String,

    pub r#type: String,
}

impl VolumeGroupSourceFromVolumeGroupReplicaDetails {
    /// Create a new VolumeGroupSourceFromVolumeGroupReplicaDetails with required fields
    pub fn new(required: VolumeGroupSourceFromVolumeGroupReplicaDetailsRequired) -> Self {
        Self {
            volume_group_replica_id: required.volume_group_replica_id,

            r#type: required.r#type,
        }
    }

    /// Set volume_group_replica_id
    pub fn set_volume_group_replica_id(mut self, value: String) -> Self {
        self.volume_group_replica_id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
