use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDrgRouteTableResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DrgRouteTable instance.
    pub drg_route_table: DrgRouteTable,
}

/// Required fields for GetDrgRouteTableResponse
pub struct GetDrgRouteTableResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.DrgRouteTable instance.
    pub drg_route_table: DrgRouteTable,
}

impl GetDrgRouteTableResponse {
    /// Create a new GetDrgRouteTableResponse with required fields
    pub fn new(required: GetDrgRouteTableResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            drg_route_table: required.drg_route_table,
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

    /// Set drg_route_table
    pub fn set_drg_route_table(mut self, value: DrgRouteTable) -> Self {
        self.drg_route_table = value;
        self
    }
}
