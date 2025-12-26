use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeImageCapabilitySchemaRequest {
    /// The id of the compute image capability schema or the image ocid
    pub compute_image_capability_schema_id: String,

    /// Merge the image capability schema with the global image capability schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_merge_enabled: Option<bool>,
}


/// Required fields for GetComputeImageCapabilitySchemaRequest
pub struct GetComputeImageCapabilitySchemaRequestRequired {
    /// The id of the compute image capability schema or the image ocid
    pub compute_image_capability_schema_id: String,
}

impl GetComputeImageCapabilitySchemaRequest {
    /// Create a new GetComputeImageCapabilitySchemaRequest with required fields
    pub fn new(required: GetComputeImageCapabilitySchemaRequestRequired) -> Self {
        Self {
            compute_image_capability_schema_id: required.compute_image_capability_schema_id,

            is_merge_enabled: None,
}
    }

    /// Set compute_image_capability_schema_id
    pub fn set_compute_image_capability_schema_id(mut self, value: String) -> Self {
        self.compute_image_capability_schema_id = value;
        self
    }

    /// Set is_merge_enabled
    pub fn set_is_merge_enabled(mut self, value: Option<bool>) -> Self {
        self.is_merge_enabled = value;
        self
    }

    /// Set is_merge_enabled (unwraps Option)
    pub fn with_is_merge_enabled(mut self, value: bool) -> Self {
        self.is_merge_enabled = Some(value);
        self
    }
}


