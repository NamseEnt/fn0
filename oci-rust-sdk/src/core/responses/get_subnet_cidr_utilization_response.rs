use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSubnetCidrUtilizationResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// For list pagination. A pagination token to get the total number of results available.
    pub opc_total_items: i64,

    /// The returned model.IpInventoryCidrUtilizationCollection instance.
    pub ip_inventory_cidr_utilization_collection: IpInventoryCidrUtilizationCollection,
}

/// Required fields for GetSubnetCidrUtilizationResponse
pub struct GetSubnetCidrUtilizationResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// For list pagination. A pagination token to get the total number of results available.
    pub opc_total_items: i64,

    /// The returned model.IpInventoryCidrUtilizationCollection instance.
    pub ip_inventory_cidr_utilization_collection: IpInventoryCidrUtilizationCollection,
}

impl GetSubnetCidrUtilizationResponse {
    /// Create a new GetSubnetCidrUtilizationResponse with required fields
    pub fn new(required: GetSubnetCidrUtilizationResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            opc_total_items: required.opc_total_items,

            ip_inventory_cidr_utilization_collection: required
                .ip_inventory_cidr_utilization_collection,
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

    /// Set opc_total_items
    pub fn set_opc_total_items(mut self, value: i64) -> Self {
        self.opc_total_items = value;
        self
    }

    /// Set ip_inventory_cidr_utilization_collection
    pub fn set_ip_inventory_cidr_utilization_collection(
        mut self,
        value: IpInventoryCidrUtilizationCollection,
    ) -> Self {
        self.ip_inventory_cidr_utilization_collection = value;
        self
    }
}
