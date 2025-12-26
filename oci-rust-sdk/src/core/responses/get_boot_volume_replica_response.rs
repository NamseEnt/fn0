use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBootVolumeReplicaResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.BootVolumeReplica instance.
    pub boot_volume_replica: BootVolumeReplica,
}

/// Required fields for GetBootVolumeReplicaResponse
pub struct GetBootVolumeReplicaResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.BootVolumeReplica instance.
    pub boot_volume_replica: BootVolumeReplica,
}

impl GetBootVolumeReplicaResponse {
    /// Create a new GetBootVolumeReplicaResponse with required fields
    pub fn new(required: GetBootVolumeReplicaResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            boot_volume_replica: required.boot_volume_replica,
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

    /// Set boot_volume_replica
    pub fn set_boot_volume_replica(mut self, value: BootVolumeReplica) -> Self {
        self.boot_volume_replica = value;
        self
    }
}
