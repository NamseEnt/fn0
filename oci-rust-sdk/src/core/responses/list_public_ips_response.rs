use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPublicIpsResponse {
    /// For list pagination. When this header appears in the response, additional pages of results remain. For important details about how pagination works, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine).
    pub opc_next_page: Option<String>,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: Option<String>,

    /// A list of PublicIp instances.
    pub items: Vec<PublicIp>,
}


/// Required fields for ListPublicIpsResponse
pub struct ListPublicIpsResponseRequired {
    /// A list of PublicIp instances.
    pub items: Vec<PublicIp>,
}

impl ListPublicIpsResponse {
    /// Create a new ListPublicIpsResponse with required fields
    pub fn new(required: ListPublicIpsResponseRequired) -> Self {
        Self {
            opc_next_page: None,
            opc_request_id: None,
            items: required.items,
        }
    }

    /// Set opc_next_page
    pub fn set_opc_next_page(mut self, value: Option<String>) -> Self {
        self.opc_next_page = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<PublicIp>) -> Self {
        self.items = value;
        self
    }

    /// Set opc_next_page (unwraps Option)
    pub fn with_opc_next_page(mut self, value: impl Into<String>) -> Self {
        self.opc_next_page = Some(value.into());
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}


