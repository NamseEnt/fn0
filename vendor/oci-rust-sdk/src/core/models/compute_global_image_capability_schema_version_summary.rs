use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Summary information for a compute global image capability schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeGlobalImageCapabilitySchemaVersionSummary {
    /// The compute global image capability schema version name
    pub name: String,

    /// The OCID of the compute global image capability schema
    pub compute_global_image_capability_schema_id: String,

    /// The date and time the compute global image capability schema version was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Required fields for ComputeGlobalImageCapabilitySchemaVersionSummary
pub struct ComputeGlobalImageCapabilitySchemaVersionSummaryRequired {
    /// The compute global image capability schema version name
    pub name: String,

    /// The OCID of the compute global image capability schema
    pub compute_global_image_capability_schema_id: String,

    /// The date and time the compute global image capability schema version was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl ComputeGlobalImageCapabilitySchemaVersionSummary {
    /// Create a new ComputeGlobalImageCapabilitySchemaVersionSummary with required fields
    pub fn new(required: ComputeGlobalImageCapabilitySchemaVersionSummaryRequired) -> Self {
        Self {
            name: required.name,

            compute_global_image_capability_schema_id: required
                .compute_global_image_capability_schema_id,

            time_created: required.time_created,

            display_name: None,
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
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }
}
