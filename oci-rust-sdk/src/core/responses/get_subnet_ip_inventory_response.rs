use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSubnetIpInventoryResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IpInventorySubnetResourceCollection instance.
    pub ip_inventory_subnet_resource_collection: IpInventorySubnetResourceCollection,
}


/// Required fields for GetSubnetIpInventoryResponse
pub struct GetSubnetIpInventoryResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.IpInventorySubnetResourceCollection instance.
    pub ip_inventory_subnet_resource_collection: IpInventorySubnetResourceCollection,
}

impl GetSubnetIpInventoryResponse {
    /// Create a new GetSubnetIpInventoryResponse with required fields
    pub fn new(required: GetSubnetIpInventoryResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            ip_inventory_subnet_resource_collection: required.ip_inventory_subnet_resource_collection,
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

    /// Set ip_inventory_subnet_resource_collection
    pub fn set_ip_inventory_subnet_resource_collection(mut self, value: IpInventorySubnetResourceCollection) -> Self {
        self.ip_inventory_subnet_resource_collection = value;
        self
    }
}


