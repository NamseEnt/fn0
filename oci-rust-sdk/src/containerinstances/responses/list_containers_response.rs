use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListContainersResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// Pagination of a list of items. When paging through a list, if this header appears in the response, then a partial list might have been returned. Include this value as the {@code page} parameter for the subsequent GET request to get the next batch of items.
    pub opc_next_page: String,

    /// The returned model.ContainerCollection instance.
    pub container_collection: ContainerCollection,
}


/// Required fields for ListContainersResponse
pub struct ListContainersResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// Pagination of a list of items. When paging through a list, if this header appears in the response, then a partial list might have been returned. Include this value as the {@code page} parameter for the subsequent GET request to get the next batch of items.
    pub opc_next_page: String,

    /// The returned model.ContainerCollection instance.
    pub container_collection: ContainerCollection,
}

impl ListContainersResponse {
    /// Create a new ListContainersResponse with required fields
    pub fn new(required: ListContainersResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            opc_next_page: required.opc_next_page,

            container_collection: required.container_collection,
}
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_next_page
    pub fn set_opc_next_page(mut self, value: String) -> Self {
        self.opc_next_page = value;
        self
    }

    /// Set container_collection
    pub fn set_container_collection(mut self, value: ContainerCollection) -> Self {
        self.container_collection = value;
        self
    }
}


