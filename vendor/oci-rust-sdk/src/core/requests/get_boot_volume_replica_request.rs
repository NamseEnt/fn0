use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBootVolumeReplicaRequest {
    /// The OCID of the boot volume replica.
    pub boot_volume_replica_id: String,
}

/// Required fields for GetBootVolumeReplicaRequest
pub struct GetBootVolumeReplicaRequestRequired {
    /// The OCID of the boot volume replica.
    pub boot_volume_replica_id: String,
}

impl GetBootVolumeReplicaRequest {
    /// Create a new GetBootVolumeReplicaRequest with required fields
    pub fn new(required: GetBootVolumeReplicaRequestRequired) -> Self {
        Self {
            boot_volume_replica_id: required.boot_volume_replica_id,
        }
    }

    /// Set boot_volume_replica_id
    pub fn set_boot_volume_replica_id(mut self, value: String) -> Self {
        self.boot_volume_replica_id = value;
        self
    }
}
