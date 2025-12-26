use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetClusterNetworkRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cluster network.
    pub cluster_network_id: String,
}


/// Required fields for GetClusterNetworkRequest
pub struct GetClusterNetworkRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cluster network.
    pub cluster_network_id: String,
}

impl GetClusterNetworkRequest {
    /// Create a new GetClusterNetworkRequest with required fields
    pub fn new(required: GetClusterNetworkRequestRequired) -> Self {
        Self {
            cluster_network_id: required.cluster_network_id,
}
    }

    /// Set cluster_network_id
    pub fn set_cluster_network_id(mut self, value: String) -> Self {
        self.cluster_network_id = value;
        self
    }
}


