use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClusterNetworkResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ClusterNetwork instance.
    pub cluster_network: ClusterNetwork,
}

/// Required fields for UpdateClusterNetworkResponse
pub struct UpdateClusterNetworkResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ClusterNetwork instance.
    pub cluster_network: ClusterNetwork,
}

impl UpdateClusterNetworkResponse {
    /// Create a new UpdateClusterNetworkResponse with required fields
    pub fn new(required: UpdateClusterNetworkResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            cluster_network: required.cluster_network,
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

    /// Set cluster_network
    pub fn set_cluster_network(mut self, value: ClusterNetwork) -> Self {
        self.cluster_network = value;
        self
    }
}
