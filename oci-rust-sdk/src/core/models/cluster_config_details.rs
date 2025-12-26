use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The HPC cluster configuration requested when launching instances in a compute capacity reservation. <p> If the parameter is provided, the reservation is created with the HPC island and a list of HPC blocks that you specify. If a list of HPC blocks are missing or not provided, the reservation is created with any HPC blocks in the HPC island that you specify. If the values of HPC island or HPC block that you provide are not valid, an error is returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterConfigDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the HPC island.
    pub hpc_island_id: String,

    /// The list of OCIDs of the network blocks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_block_ids: Option<Vec<String>>,
}

/// Required fields for ClusterConfigDetails
pub struct ClusterConfigDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the HPC island.
    pub hpc_island_id: String,
}

impl ClusterConfigDetails {
    /// Create a new ClusterConfigDetails with required fields
    pub fn new(required: ClusterConfigDetailsRequired) -> Self {
        Self {
            hpc_island_id: required.hpc_island_id,

            network_block_ids: None,
        }
    }

    /// Set hpc_island_id
    pub fn set_hpc_island_id(mut self, value: String) -> Self {
        self.hpc_island_id = value;
        self
    }

    /// Set network_block_ids
    pub fn set_network_block_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.network_block_ids = value;
        self
    }

    /// Set network_block_ids (unwraps Option)
    pub fn with_network_block_ids(mut self, value: Vec<String>) -> Self {
        self.network_block_ids = Some(value);
        self
    }
}
