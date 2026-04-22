use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Compute Global Image Capability Schema Version is a set of all possible capabilities for a collection of images.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeGlobalImageCapabilitySchemaVersion {
    /// The name of the compute global image capability schema version
    pub name: String,

    /// The ocid of the compute global image capability schema
    pub compute_global_image_capability_schema_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The map of each capability name to its ImageCapabilityDescriptor.
    pub schema_data: HashMap<String, ImageCapabilitySchemaDescriptor>,

    /// The date and time the compute global image capability schema version was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

/// Required fields for ComputeGlobalImageCapabilitySchemaVersion
pub struct ComputeGlobalImageCapabilitySchemaVersionRequired {
    /// The name of the compute global image capability schema version
    pub name: String,

    /// The ocid of the compute global image capability schema
    pub compute_global_image_capability_schema_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The map of each capability name to its ImageCapabilityDescriptor.
    pub schema_data: HashMap<String, ImageCapabilitySchemaDescriptor>,

    /// The date and time the compute global image capability schema version was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl ComputeGlobalImageCapabilitySchemaVersion {
    /// Create a new ComputeGlobalImageCapabilitySchemaVersion with required fields
    pub fn new(required: ComputeGlobalImageCapabilitySchemaVersionRequired) -> Self {
        Self {
            name: required.name,

            compute_global_image_capability_schema_id: required
                .compute_global_image_capability_schema_id,

            display_name: required.display_name,

            schema_data: required.schema_data,

            time_created: required.time_created,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set compute_global_image_capability_schema_id
    pub fn set_compute_global_image_capability_schema_id(mut self, value: String) -> Self {
        self.compute_global_image_capability_schema_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
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
}
