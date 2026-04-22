use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIpInventoryRequest {
    /// Details required to list the IP Inventory data.
    pub list_ip_inventory_details: ListIpInventoryDetails,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}

/// Required fields for ListIpInventoryRequest
pub struct ListIpInventoryRequestRequired {
    /// Details required to list the IP Inventory data.
    pub list_ip_inventory_details: ListIpInventoryDetails,
}

impl ListIpInventoryRequest {
    /// Create a new ListIpInventoryRequest with required fields
    pub fn new(required: ListIpInventoryRequestRequired) -> Self {
        Self {
            list_ip_inventory_details: required.list_ip_inventory_details,

            opc_request_id: None,
        }
    }

    /// Set list_ip_inventory_details
    pub fn set_list_ip_inventory_details(mut self, value: ListIpInventoryDetails) -> Self {
        self.list_ip_inventory_details = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}
