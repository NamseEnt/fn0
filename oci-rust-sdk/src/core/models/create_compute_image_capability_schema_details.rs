use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Create Image Capability Schema for an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComputeImageCapabilitySchemaDetails {
    /// The OCID of the compartment that contains the resource.
    pub compartment_id: String,

    /// The name of the compute global image capability schema version
    pub compute_global_image_capability_schema_version_name: String,

    /// The ocid of the image
    pub image_id: String,

    /// The map of each capability name to its ImageCapabilitySchemaDescriptor.
    pub schema_data: HashMap<String, ImageCapabilitySchemaDescriptor>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for CreateComputeImageCapabilitySchemaDetails
pub struct CreateComputeImageCapabilitySchemaDetailsRequired {
    /// The OCID of the compartment that contains the resource.
    pub compartment_id: String,

    /// The name of the compute global image capability schema version
    pub compute_global_image_capability_schema_version_name: String,

    /// The ocid of the image
    pub image_id: String,

    /// The map of each capability name to its ImageCapabilitySchemaDescriptor.
    pub schema_data: HashMap<String, ImageCapabilitySchemaDescriptor>,
}

impl CreateComputeImageCapabilitySchemaDetails {
    /// Create a new CreateComputeImageCapabilitySchemaDetails with required fields
    pub fn new(required: CreateComputeImageCapabilitySchemaDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            compute_global_image_capability_schema_version_name: required
                .compute_global_image_capability_schema_version_name,

            image_id: required.image_id,

            schema_data: required.schema_data,

            freeform_tags: None,

            display_name: None,

            defined_tags: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set compute_global_image_capability_schema_version_name
    pub fn set_compute_global_image_capability_schema_version_name(
        mut self,
        value: String,
    ) -> Self {
        self.compute_global_image_capability_schema_version_name = value;
        self
    }

    /// Set image_id
    pub fn set_image_id(mut self, value: String) -> Self {
        self.image_id = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set schema_data
    pub fn set_schema_data(
        mut self,
        value: HashMap<String, ImageCapabilitySchemaDescriptor>,
    ) -> Self {
        self.schema_data = value;
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }
}
