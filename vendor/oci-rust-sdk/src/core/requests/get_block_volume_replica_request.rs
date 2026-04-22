use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBlockVolumeReplicaRequest {
    /// The OCID of the block volume replica.
    pub block_volume_replica_id: String,
}

/// Required fields for GetBlockVolumeReplicaRequest
pub struct GetBlockVolumeReplicaRequestRequired {
    /// The OCID of the block volume replica.
    pub block_volume_replica_id: String,
}

impl GetBlockVolumeReplicaRequest {
    /// Create a new GetBlockVolumeReplicaRequest with required fields
    pub fn new(required: GetBlockVolumeReplicaRequestRequired) -> Self {
        Self {
            block_volume_replica_id: required.block_volume_replica_id,
        }
    }

    /// Set block_volume_replica_id
    pub fn set_block_volume_replica_id(mut self, value: String) -> Self {
        self.block_volume_replica_id = value;
        self
    }
}
