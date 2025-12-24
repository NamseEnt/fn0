use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Compute Global Image Capability Schema is a container for a set of compute global image capability schema versions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeGlobalImageCapabilitySchema {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute global image capability schema
    pub id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The date and time the compute global image capability schema was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The OCID of the compartment that contains the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// The name of the global capabilities version resource that is considered the current version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_version_name: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

/// Required fields for ComputeGlobalImageCapabilitySchema
pub struct ComputeGlobalImageCapabilitySchemaRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute global image capability schema
    pub id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The date and time the compute global image capability schema was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl ComputeGlobalImageCapabilitySchema {
    /// Create a new ComputeGlobalImageCapabilitySchema with required fields
    pub fn new(required: ComputeGlobalImageCapabilitySchemaRequired) -> Self {
        Self {
            id: required.id,

            display_name: required.display_name,

            time_created: required.time_created,

            compartment_id: None,

            current_version_name: None,

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

    /// Set current_version_name
    pub fn set_current_version_name(mut self, value: Option<String>) -> Self {
        self.current_version_name = value;
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

    /// Set current_version_name (unwraps Option)
    pub fn with_current_version_name(mut self, value: impl Into<String>) -> Self {
        self.current_version_name = Some(value.into());
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
