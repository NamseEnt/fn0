use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeGroupReplicaRequest {
    /// The OCID of the volume replica group.
    pub volume_group_replica_id: String,
}


/// Required fields for GetVolumeGroupReplicaRequest
pub struct GetVolumeGroupReplicaRequestRequired {
    /// The OCID of the volume replica group.
    pub volume_group_replica_id: String,
}

impl GetVolumeGroupReplicaRequest {
    /// Create a new GetVolumeGroupReplicaRequest with required fields
    pub fn new(required: GetVolumeGroupReplicaRequestRequired) -> Self {
        Self {
            volume_group_replica_id: required.volume_group_replica_id,
}
    }

    /// Set volume_group_replica_id
    pub fn set_volume_group_replica_id(mut self, value: String) -> Self {
        self.volume_group_replica_id = value;
        self
    }
}


