use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIPSecConnectionTunnelErrorResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IPSecConnectionTunnelErrorDetails instance.
    pub i_psec_connection_tunnel_error_details: IPSecConnectionTunnelErrorDetails,
}


/// Required fields for GetIPSecConnectionTunnelErrorResponse
pub struct GetIPSecConnectionTunnelErrorResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IPSecConnectionTunnelErrorDetails instance.
    pub i_psec_connection_tunnel_error_details: IPSecConnectionTunnelErrorDetails,
}

impl GetIPSecConnectionTunnelErrorResponse {
    /// Create a new GetIPSecConnectionTunnelErrorResponse with required fields
    pub fn new(required: GetIPSecConnectionTunnelErrorResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            i_psec_connection_tunnel_error_details: required.i_psec_connection_tunnel_error_details,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set i_psec_connection_tunnel_error_details
    pub fn set_i_psec_connection_tunnel_error_details(mut self, value: IPSecConnectionTunnelErrorDetails) -> Self {
        self.i_psec_connection_tunnel_error_details = value;
        self
    }
}


