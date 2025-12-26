use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeImageCapabilitySchemaResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeImageCapabilitySchema instance.
    pub compute_image_capability_schema: ComputeImageCapabilitySchema,
}


/// Required fields for GetComputeImageCapabilitySchemaResponse
pub struct GetComputeImageCapabilitySchemaResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeImageCapabilitySchema instance.
    pub compute_image_capability_schema: ComputeImageCapabilitySchema,
}

impl GetComputeImageCapabilitySchemaResponse {
    /// Create a new GetComputeImageCapabilitySchemaResponse with required fields
    pub fn new(required: GetComputeImageCapabilitySchemaResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            compute_image_capability_schema: required.compute_image_capability_schema,
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

    /// Set compute_image_capability_schema
    pub fn set_compute_image_capability_schema(mut self, value: ComputeImageCapabilitySchema) -> Self {
        self.compute_image_capability_schema = value;
        self
    }
}


