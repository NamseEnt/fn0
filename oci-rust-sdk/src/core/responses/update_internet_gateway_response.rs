use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInternetGatewayResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InternetGateway instance.
    pub internet_gateway: InternetGateway,
}


/// Required fields for UpdateInternetGatewayResponse
pub struct UpdateInternetGatewayResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InternetGateway instance.
    pub internet_gateway: InternetGateway,
}

impl UpdateInternetGatewayResponse {
    /// Create a new UpdateInternetGatewayResponse with required fields
    pub fn new(required: UpdateInternetGatewayResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            internet_gateway: required.internet_gateway,
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

    /// Set internet_gateway
    pub fn set_internet_gateway(mut self, value: InternetGateway) -> Self {
        self.internet_gateway = value;
        self
    }
}


