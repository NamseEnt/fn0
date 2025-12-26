use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIPSecConnectionTunnelResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IPSecConnectionTunnel instance.
    pub i_psec_connection_tunnel: IPSecConnectionTunnel,
}

/// Required fields for GetIPSecConnectionTunnelResponse
pub struct GetIPSecConnectionTunnelResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IPSecConnectionTunnel instance.
    pub i_psec_connection_tunnel: IPSecConnectionTunnel,
}

impl GetIPSecConnectionTunnelResponse {
    /// Create a new GetIPSecConnectionTunnelResponse with required fields
    pub fn new(required: GetIPSecConnectionTunnelResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            i_psec_connection_tunnel: required.i_psec_connection_tunnel,
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

    /// Set i_psec_connection_tunnel
    pub fn set_i_psec_connection_tunnel(mut self, value: IPSecConnectionTunnel) -> Self {
        self.i_psec_connection_tunnel = value;
        self
    }
}
