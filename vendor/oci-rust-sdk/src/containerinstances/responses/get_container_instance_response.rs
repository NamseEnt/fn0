use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetContainerInstanceResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ContainerInstance instance.
    pub container_instance: ContainerInstance,
}


/// Required fields for GetContainerInstanceResponse
pub struct GetContainerInstanceResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ContainerInstance instance.
    pub container_instance: ContainerInstance,
}

impl GetContainerInstanceResponse {
    /// Create a new GetContainerInstanceResponse with required fields
    pub fn new(required: GetContainerInstanceResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            container_instance: required.container_instance,
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

    /// Set container_instance
    pub fn set_container_instance(mut self, value: ContainerInstance) -> Self {
        self.container_instance = value;
        self
    }
}


