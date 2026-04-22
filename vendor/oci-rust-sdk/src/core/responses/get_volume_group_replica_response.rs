use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumeGroupReplicaResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeGroupReplica instance.
    pub volume_group_replica: VolumeGroupReplica,
}

/// Required fields for GetVolumeGroupReplicaResponse
pub struct GetVolumeGroupReplicaResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VolumeGroupReplica instance.
    pub volume_group_replica: VolumeGroupReplica,
}

impl GetVolumeGroupReplicaResponse {
    /// Create a new GetVolumeGroupReplicaResponse with required fields
    pub fn new(required: GetVolumeGroupReplicaResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            volume_group_replica: required.volume_group_replica,
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

    /// Set volume_group_replica
    pub fn set_volume_group_replica(mut self, value: VolumeGroupReplica) -> Self {
        self.volume_group_replica = value;
        self
    }
}
