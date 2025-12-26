use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeGlobalImageCapabilitySchemaResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeGlobalImageCapabilitySchema instance.
    pub compute_global_image_capability_schema: ComputeGlobalImageCapabilitySchema,
}

/// Required fields for GetComputeGlobalImageCapabilitySchemaResponse
pub struct GetComputeGlobalImageCapabilitySchemaResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeGlobalImageCapabilitySchema instance.
    pub compute_global_image_capability_schema: ComputeGlobalImageCapabilitySchema,
}

impl GetComputeGlobalImageCapabilitySchemaResponse {
    /// Create a new GetComputeGlobalImageCapabilitySchemaResponse with required fields
    pub fn new(required: GetComputeGlobalImageCapabilitySchemaResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            compute_global_image_capability_schema: required.compute_global_image_capability_schema,
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

    /// Set compute_global_image_capability_schema
    pub fn set_compute_global_image_capability_schema(
        mut self,
        value: ComputeGlobalImageCapabilitySchema,
    ) -> Self {
        self.compute_global_image_capability_schema = value;
        self
    }
}
