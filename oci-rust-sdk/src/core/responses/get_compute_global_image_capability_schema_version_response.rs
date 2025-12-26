use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeGlobalImageCapabilitySchemaVersionResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeGlobalImageCapabilitySchemaVersion instance.
    pub compute_global_image_capability_schema_version: ComputeGlobalImageCapabilitySchemaVersion,
}


/// Required fields for GetComputeGlobalImageCapabilitySchemaVersionResponse
pub struct GetComputeGlobalImageCapabilitySchemaVersionResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.ComputeGlobalImageCapabilitySchemaVersion instance.
    pub compute_global_image_capability_schema_version: ComputeGlobalImageCapabilitySchemaVersion,
}

impl GetComputeGlobalImageCapabilitySchemaVersionResponse {
    /// Create a new GetComputeGlobalImageCapabilitySchemaVersionResponse with required fields
    pub fn new(required: GetComputeGlobalImageCapabilitySchemaVersionResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            compute_global_image_capability_schema_version: required.compute_global_image_capability_schema_version,
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

    /// Set compute_global_image_capability_schema_version
    pub fn set_compute_global_image_capability_schema_version(mut self, value: ComputeGlobalImageCapabilitySchemaVersion) -> Self {
        self.compute_global_image_capability_schema_version = value;
        self
    }
}


