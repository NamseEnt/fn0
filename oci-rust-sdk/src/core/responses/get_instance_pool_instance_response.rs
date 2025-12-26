use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstancePoolInstanceResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstancePoolInstance instance.
    pub instance_pool_instance: InstancePoolInstance,
}


/// Required fields for GetInstancePoolInstanceResponse
pub struct GetInstancePoolInstanceResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.InstancePoolInstance instance.
    pub instance_pool_instance: InstancePoolInstance,
}

impl GetInstancePoolInstanceResponse {
    /// Create a new GetInstancePoolInstanceResponse with required fields
    pub fn new(required: GetInstancePoolInstanceResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            instance_pool_instance: required.instance_pool_instance,
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

    /// Set instance_pool_instance
    pub fn set_instance_pool_instance(mut self, value: InstancePoolInstance) -> Self {
        self.instance_pool_instance = value;
        self
    }
}


