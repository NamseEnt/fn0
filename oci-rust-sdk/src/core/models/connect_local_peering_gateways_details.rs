use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about the other local peering gateway (LPG).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectLocalPeeringGatewaysDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the LPG you want to peer with.
    pub peer_id: String,
}

/// Required fields for ConnectLocalPeeringGatewaysDetails
pub struct ConnectLocalPeeringGatewaysDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the LPG you want to peer with.
    pub peer_id: String,
}

impl ConnectLocalPeeringGatewaysDetails {
    /// Create a new ConnectLocalPeeringGatewaysDetails with required fields
    pub fn new(required: ConnectLocalPeeringGatewaysDetailsRequired) -> Self {
        Self {
            peer_id: required.peer_id,
        }
    }

    /// Set peer_id
    pub fn set_peer_id(mut self, value: String) -> Self {
        self.peer_id = value;
        self
    }
}
