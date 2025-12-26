use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The HPC cluster configuration requested when launching instances of a cluster network. <p> If the parameter is provided, instances will only be placed within the HPC island and list of network blocks that you specify. If a list of network blocks are missing or not provided, the instances will be placed in any HPC blocks in the HPC island that you specify. If the values of HPC island or network block that you provide are not valid, an error is returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterConfigurationDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the HPC island.
    pub hpc_island_id: String,

    /// The list of network block OCIDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_block_ids: Option<Vec<String>>,
}

/// Required fields for ClusterConfigurationDetails
pub struct ClusterConfigurationDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the HPC island.
    pub hpc_island_id: String,
}

impl ClusterConfigurationDetails {
    /// Create a new ClusterConfigurationDetails with required fields
    pub fn new(required: ClusterConfigurationDetailsRequired) -> Self {
        Self {
            hpc_island_id: required.hpc_island_id,

            network_block_ids: None,
        }
    }

    /// Set network_block_ids
    pub fn set_network_block_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.network_block_ids = value;
        self
    }

    /// Set hpc_island_id
    pub fn set_hpc_island_id(mut self, value: String) -> Self {
        self.hpc_island_id = value;
        self
    }

    /// Set network_block_ids (unwraps Option)
    pub fn with_network_block_ids(mut self, value: Vec<String>) -> Self {
        self.network_block_ids = Some(value);
        self
    }
}
