use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNatGatewayResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.NatGateway instance.
    pub nat_gateway: NatGateway,
}

/// Required fields for CreateNatGatewayResponse
pub struct CreateNatGatewayResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.NatGateway instance.
    pub nat_gateway: NatGateway,
}

impl CreateNatGatewayResponse {
    /// Create a new CreateNatGatewayResponse with required fields
    pub fn new(required: CreateNatGatewayResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            nat_gateway: required.nat_gateway,
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

    /// Set nat_gateway
    pub fn set_nat_gateway(mut self, value: NatGateway) -> Self {
        self.nat_gateway = value;
        self
    }
}
