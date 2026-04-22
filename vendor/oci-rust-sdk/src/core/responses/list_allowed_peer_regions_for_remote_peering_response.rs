use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAllowedPeerRegionsForRemotePeeringResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// A list of PeerRegionForRemotePeering instances.
    pub items: Vec<PeerRegionForRemotePeering>,
}

/// Required fields for ListAllowedPeerRegionsForRemotePeeringResponse
pub struct ListAllowedPeerRegionsForRemotePeeringResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// A list of PeerRegionForRemotePeering instances.
    pub items: Vec<PeerRegionForRemotePeering>,
}

impl ListAllowedPeerRegionsForRemotePeeringResponse {
    /// Create a new ListAllowedPeerRegionsForRemotePeeringResponse with required fields
    pub fn new(required: ListAllowedPeerRegionsForRemotePeeringResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            items: required.items,
        }
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<PeerRegionForRemotePeering>) -> Self {
        self.items = value;
        self
    }
}
