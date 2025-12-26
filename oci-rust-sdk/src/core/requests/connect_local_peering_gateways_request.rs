use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectLocalPeeringGatewaysRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the local peering gateway.
    pub local_peering_gateway_id: String,

    /// Details regarding the local peering gateway to connect.
    pub connect_local_peering_gateways_details: ConnectLocalPeeringGatewaysDetails,
}


/// Required fields for ConnectLocalPeeringGatewaysRequest
pub struct ConnectLocalPeeringGatewaysRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the local peering gateway.
    pub local_peering_gateway_id: String,

    /// Details regarding the local peering gateway to connect.
    pub connect_local_peering_gateways_details: ConnectLocalPeeringGatewaysDetails,
}

impl ConnectLocalPeeringGatewaysRequest {
    /// Create a new ConnectLocalPeeringGatewaysRequest with required fields
    pub fn new(required: ConnectLocalPeeringGatewaysRequestRequired) -> Self {
        Self {
            local_peering_gateway_id: required.local_peering_gateway_id,

            connect_local_peering_gateways_details: required.connect_local_peering_gateways_details,
}
    }

    /// Set local_peering_gateway_id
    pub fn set_local_peering_gateway_id(mut self, value: String) -> Self {
        self.local_peering_gateway_id = value;
        self
    }

    /// Set connect_local_peering_gateways_details
    pub fn set_connect_local_peering_gateways_details(mut self, value: ConnectLocalPeeringGatewaysDetails) -> Self {
        self.connect_local_peering_gateways_details = value;
        self
    }
}


