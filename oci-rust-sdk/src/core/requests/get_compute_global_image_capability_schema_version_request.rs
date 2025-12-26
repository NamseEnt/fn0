use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeGlobalImageCapabilitySchemaVersionRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute global image capability schema
    pub compute_global_image_capability_schema_id: String,

    /// The name of the compute global image capability schema version
    pub compute_global_image_capability_schema_version_name: String,
}


/// Required fields for GetComputeGlobalImageCapabilitySchemaVersionRequest
pub struct GetComputeGlobalImageCapabilitySchemaVersionRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute global image capability schema
    pub compute_global_image_capability_schema_id: String,

    /// The name of the compute global image capability schema version
    pub compute_global_image_capability_schema_version_name: String,
}

impl GetComputeGlobalImageCapabilitySchemaVersionRequest {
    /// Create a new GetComputeGlobalImageCapabilitySchemaVersionRequest with required fields
    pub fn new(required: GetComputeGlobalImageCapabilitySchemaVersionRequestRequired) -> Self {
        Self {
            compute_global_image_capability_schema_id: required.compute_global_image_capability_schema_id,

            compute_global_image_capability_schema_version_name: required.compute_global_image_capability_schema_version_name,
}
    }

    /// Set compute_global_image_capability_schema_id
    pub fn set_compute_global_image_capability_schema_id(mut self, value: String) -> Self {
        self.compute_global_image_capability_schema_id = value;
        self
    }

    /// Set compute_global_image_capability_schema_version_name
    pub fn set_compute_global_image_capability_schema_version_name(mut self, value: String) -> Self {
        self.compute_global_image_capability_schema_version_name = value;
        self
    }
}


