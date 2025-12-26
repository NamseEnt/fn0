use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRemotePeeringConnectionResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.RemotePeeringConnection instance.
    pub remote_peering_connection: RemotePeeringConnection,
}


/// Required fields for GetRemotePeeringConnectionResponse
pub struct GetRemotePeeringConnectionResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.RemotePeeringConnection instance.
    pub remote_peering_connection: RemotePeeringConnection,
}

impl GetRemotePeeringConnectionResponse {
    /// Create a new GetRemotePeeringConnectionResponse with required fields
    pub fn new(required: GetRemotePeeringConnectionResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            remote_peering_connection: required.remote_peering_connection,
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

    /// Set remote_peering_connection
    pub fn set_remote_peering_connection(mut self, value: RemotePeeringConnection) -> Self {
        self.remote_peering_connection = value;
        self
    }
}


