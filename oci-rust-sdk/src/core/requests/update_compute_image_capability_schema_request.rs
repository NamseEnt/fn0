use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComputeImageCapabilitySchemaRequest {
    /// The id of the compute image capability schema or the image ocid
    pub compute_image_capability_schema_id: String,

    /// Updates the freeFormTags, definedTags, and display name of the image capability schema
    pub update_compute_image_capability_schema_details: UpdateComputeImageCapabilitySchemaDetails,

    /// For optimistic concurrency control. In the PUT or DELETE call for a resource, set the {@code if-match} parameter to the value of the etag from a previous GET or POST response for that resource. The resource will be updated or deleted only if the etag you provide matches the resource's current etag value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_match: Option<String>,
}


/// Required fields for UpdateComputeImageCapabilitySchemaRequest
pub struct UpdateComputeImageCapabilitySchemaRequestRequired {
    /// The id of the compute image capability schema or the image ocid
    pub compute_image_capability_schema_id: String,

    /// Updates the freeFormTags, definedTags, and display name of the image capability schema
    pub update_compute_image_capability_schema_details: UpdateComputeImageCapabilitySchemaDetails,
}

impl UpdateComputeImageCapabilitySchemaRequest {
    /// Create a new UpdateComputeImageCapabilitySchemaRequest with required fields
    pub fn new(required: UpdateComputeImageCapabilitySchemaRequestRequired) -> Self {
        Self {
            compute_image_capability_schema_id: required.compute_image_capability_schema_id,

            update_compute_image_capability_schema_details: required.update_compute_image_capability_schema_details,

            if_match: None,
}
    }

    /// Set compute_image_capability_schema_id
    pub fn set_compute_image_capability_schema_id(mut self, value: String) -> Self {
        self.compute_image_capability_schema_id = value;
        self
    }

    /// Set update_compute_image_capability_schema_details
    pub fn set_update_compute_image_capability_schema_details(mut self, value: UpdateComputeImageCapabilitySchemaDetails) -> Self {
        self.update_compute_image_capability_schema_details = value;
        self
    }

    /// Set if_match
    pub fn set_if_match(mut self, value: Option<String>) -> Self {
        self.if_match = value;
        self
    }

    /// Set if_match (unwraps Option)
    pub fn with_if_match(mut self, value: impl Into<String>) -> Self {
        self.if_match = Some(value.into());
        self
    }
}


