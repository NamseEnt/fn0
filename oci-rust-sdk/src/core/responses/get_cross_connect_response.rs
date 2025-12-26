use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCrossConnectResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CrossConnect instance.
    pub cross_connect: CrossConnect,
}


/// Required fields for GetCrossConnectResponse
pub struct GetCrossConnectResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.CrossConnect instance.
    pub cross_connect: CrossConnect,
}

impl GetCrossConnectResponse {
    /// Create a new GetCrossConnectResponse with required fields
    pub fn new(required: GetCrossConnectResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            cross_connect: required.cross_connect,
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

    /// Set cross_connect
    pub fn set_cross_connect(mut self, value: CrossConnect) -> Self {
        self.cross_connect = value;
        self
    }
}


