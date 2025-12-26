use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Compute Image Capability Schema is a set of capabilities that filter the compute global capability schema version for an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeImageCapabilitySchema {
    /// The id of the compute global image capability schema version
    pub id: String,

    /// The ocid of the compute global image capability schema
    pub compute_global_image_capability_schema_id: String,

    /// The name of the compute global image capability schema version
    pub compute_global_image_capability_schema_version_name: String,

    /// The OCID of the image associated with this compute image capability schema
    pub image_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The map of each capability name to its ImageCapabilityDescriptor.
    pub schema_data: HashMap<String, ImageCapabilitySchemaDescriptor>,

    /// The date and time the compute image capability schema was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The OCID of the compartment that contains the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// The ComputeImageCapabilitySchema current lifecycle state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<ComputeImageCapabilitySchemaLifecycleState>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

/// Required fields for ComputeImageCapabilitySchema
pub struct ComputeImageCapabilitySchemaRequired {
    /// The id of the compute global image capability schema version
    pub id: String,

    /// The ocid of the compute global image capability schema
    pub compute_global_image_capability_schema_id: String,

    /// The name of the compute global image capability schema version
    pub compute_global_image_capability_schema_version_name: String,

    /// The OCID of the image associated with this compute image capability schema
    pub image_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The map of each capability name to its ImageCapabilityDescriptor.
    pub schema_data: HashMap<String, ImageCapabilitySchemaDescriptor>,

    /// The date and time the compute image capability schema was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl ComputeImageCapabilitySchema {
    /// Create a new ComputeImageCapabilitySchema with required fields
    pub fn new(required: ComputeImageCapabilitySchemaRequired) -> Self {
        Self {
            id: required.id,

            compute_global_image_capability_schema_id: required
                .compute_global_image_capability_schema_id,

            compute_global_image_capability_schema_version_name: required
                .compute_global_image_capability_schema_version_name,

            image_id: required.image_id,

            display_name: required.display_name,

            schema_data: required.schema_data,

            time_created: required.time_created,

            compartment_id: None,

            lifecycle_state: None,

            defined_tags: None,

            freeform_tags: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set compute_global_image_capability_schema_id
    pub fn set_compute_global_image_capability_schema_id(mut self, value: String) -> Self {
        self.compute_global_image_capability_schema_id = value;
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

    /// Set lifecycle_state
    pub fn set_lifecycle_state(
        mut self,
        value: Option<ComputeImageCapabilitySchemaLifecycleState>,
    ) -> Self {
        self.lifecycle_state = value;
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

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
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

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(
        mut self,
        value: ComputeImageCapabilitySchemaLifecycleState,
    ) -> Self {
        self.lifecycle_state = Some(value);
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

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }
}
