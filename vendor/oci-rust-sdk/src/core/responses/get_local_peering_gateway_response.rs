use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLocalPeeringGatewayResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.LocalPeeringGateway instance.
    pub local_peering_gateway: LocalPeeringGateway,
}

/// Required fields for GetLocalPeeringGatewayResponse
pub struct GetLocalPeeringGatewayResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.LocalPeeringGateway instance.
    pub local_peering_gateway: LocalPeeringGateway,
}

impl GetLocalPeeringGatewayResponse {
    /// Create a new GetLocalPeeringGatewayResponse with required fields
    pub fn new(required: GetLocalPeeringGatewayResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            local_peering_gateway: required.local_peering_gateway,
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

    /// Set local_peering_gateway
    pub fn set_local_peering_gateway(mut self, value: LocalPeeringGateway) -> Self {
        self.local_peering_gateway = value;
        self
    }
}
