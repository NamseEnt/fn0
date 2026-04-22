use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIPSecConnectionResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IPSecConnection instance.
    pub i_psec_connection: IPSecConnection,
}

/// Required fields for GetIPSecConnectionResponse
pub struct GetIPSecConnectionResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IPSecConnection instance.
    pub i_psec_connection: IPSecConnection,
}

impl GetIPSecConnectionResponse {
    /// Create a new GetIPSecConnectionResponse with required fields
    pub fn new(required: GetIPSecConnectionResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            i_psec_connection: required.i_psec_connection,
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

    /// Set i_psec_connection
    pub fn set_i_psec_connection(mut self, value: IPSecConnection) -> Self {
        self.i_psec_connection = value;
        self
    }
}
