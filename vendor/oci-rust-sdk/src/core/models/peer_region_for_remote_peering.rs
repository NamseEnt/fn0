use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details about a region that supports remote VCN peering. For more information, see [VCN Peering](https://docs.oracle.com/iaas/Content/Network/Tasks/VCNpeering.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerRegionForRemotePeering {
    /// The region's name. <p> Example: {@code us-phoenix-1}
    pub name: String,
}

/// Required fields for PeerRegionForRemotePeering
pub struct PeerRegionForRemotePeeringRequired {
    /// The region's name. <p> Example: {@code us-phoenix-1}
    pub name: String,
}

impl PeerRegionForRemotePeering {
    /// Create a new PeerRegionForRemotePeering with required fields
    pub fn new(required: PeerRegionForRemotePeeringRequired) -> Self {
        Self {
            name: required.name,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }
}
