use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBlockVolumeReplicaResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.BlockVolumeReplica instance.
    pub block_volume_replica: BlockVolumeReplica,
}


/// Required fields for GetBlockVolumeReplicaResponse
pub struct GetBlockVolumeReplicaResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.BlockVolumeReplica instance.
    pub block_volume_replica: BlockVolumeReplica,
}

impl GetBlockVolumeReplicaResponse {
    /// Create a new GetBlockVolumeReplicaResponse with required fields
    pub fn new(required: GetBlockVolumeReplicaResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            block_volume_replica: required.block_volume_replica,
}
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set block_volume_replica
    pub fn set_block_volume_replica(mut self, value: BlockVolumeReplica) -> Self {
        self.block_volume_replica = value;
        self
    }
}


