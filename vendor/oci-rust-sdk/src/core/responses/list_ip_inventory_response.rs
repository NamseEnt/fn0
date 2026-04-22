use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIpInventoryResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// For list pagination. A pagination token to get the total number of results available.
    pub opc_total_items: i64,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the request.
    pub opc_work_request_id: String,

    /// The IpInventory API current state.
    pub lifecycle_state: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the resource. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the resource.
    pub data_request_id: String,

    /// The returned model.IpInventoryCollection instance.
    pub ip_inventory_collection: IpInventoryCollection,
}

/// Required fields for ListIpInventoryResponse
pub struct ListIpInventoryResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// For list pagination. A pagination token to get the total number of results available.
    pub opc_total_items: i64,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the request.
    pub opc_work_request_id: String,

    /// The IpInventory API current state.
    pub lifecycle_state: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the resource. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the resource.
    pub data_request_id: String,

    /// The returned model.IpInventoryCollection instance.
    pub ip_inventory_collection: IpInventoryCollection,
}

impl ListIpInventoryResponse {
    /// Create a new ListIpInventoryResponse with required fields
    pub fn new(required: ListIpInventoryResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_next_page: required.opc_next_page,

            opc_request_id: required.opc_request_id,

            opc_total_items: required.opc_total_items,

            opc_work_request_id: required.opc_work_request_id,

            lifecycle_state: required.lifecycle_state,

            data_request_id: required.data_request_id,

            ip_inventory_collection: required.ip_inventory_collection,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_next_page
    pub fn set_opc_next_page(mut self, value: String) -> Self {
        self.opc_next_page = value;
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

    /// Set opc_work_request_id
    pub fn set_opc_work_request_id(mut self, value: String) -> Self {
        self.opc_work_request_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: String) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set data_request_id
    pub fn set_data_request_id(mut self, value: String) -> Self {
        self.data_request_id = value;
        self
    }

    /// Set ip_inventory_collection
    pub fn set_ip_inventory_collection(mut self, value: IpInventoryCollection) -> Self {
        self.ip_inventory_collection = value;
        self
    }
}
