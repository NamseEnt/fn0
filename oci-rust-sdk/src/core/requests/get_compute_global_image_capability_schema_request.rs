use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComputeGlobalImageCapabilitySchemaRequest {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute global image capability schema
    pub compute_global_image_capability_schema_id: String,
}


/// Required fields for GetComputeGlobalImageCapabilitySchemaRequest
pub struct GetComputeGlobalImageCapabilitySchemaRequestRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute global image capability schema
    pub compute_global_image_capability_schema_id: String,
}

impl GetComputeGlobalImageCapabilitySchemaRequest {
    /// Create a new GetComputeGlobalImageCapabilitySchemaRequest with required fields
    pub fn new(required: GetComputeGlobalImageCapabilitySchemaRequestRequired) -> Self {
        Self {
            compute_global_image_capability_schema_id: required.compute_global_image_capability_schema_id,
}
    }

    /// Set compute_global_image_capability_schema_id
    pub fn set_compute_global_image_capability_schema_id(mut self, value: String) -> Self {
        self.compute_global_image_capability_schema_id = value;
        self
    }
}


