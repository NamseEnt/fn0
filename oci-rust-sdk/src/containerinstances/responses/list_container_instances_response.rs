use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListContainerInstancesResponse {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// Pagination of a list of items. When paging through a list, if this header appears in the response, then a partial list might have been returned. Include this value as the {@code page} parameter for the subsequent GET request to get the next batch of items.
    pub opc_next_page: String,

    /// The returned model.ContainerInstanceCollection instance.
    pub container_instance_collection: ContainerInstanceCollection,
}


/// Required fields for ListContainerInstancesResponse
pub struct ListContainerInstancesResponseRequired {
    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// Pagination of a list of items. When paging through a list, if this header appears in the response, then a partial list might have been returned. Include this value as the {@code page} parameter for the subsequent GET request to get the next batch of items.
    pub opc_next_page: String,

    /// The returned model.ContainerInstanceCollection instance.
    pub container_instance_collection: ContainerInstanceCollection,
}

impl ListContainerInstancesResponse {
    /// Create a new ListContainerInstancesResponse with required fields
    pub fn new(required: ListContainerInstancesResponseRequired) -> Self {
        Self {
            opc_request_id: required.opc_request_id,

            opc_next_page: required.opc_next_page,

            container_instance_collection: required.container_instance_collection,
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

    /// Set container_instance_collection
    pub fn set_container_instance_collection(mut self, value: ContainerInstanceCollection) -> Self {
        self.container_instance_collection = value;
        self
    }
}


