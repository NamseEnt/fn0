use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Summary information for an instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancePoolSummary {
    /// The OCID of the instance pool.
    pub id: String,

    /// The OCID of the compartment containing the instance pool.
    pub compartment_id: String,

    /// The OCID of the instance configuration associated with the instance pool.
    pub instance_configuration_id: String,

    /// The current state of the instance pool.
    pub lifecycle_state: InstancePoolSummaryLifecycleState,

    /// The availability domains for the instance pool.
    pub availability_domains: Vec<String>,

    /// The number of instances that should be in the instance pool. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size: i64,

    /// The date and time the instance pool was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

/// Required fields for InstancePoolSummary
pub struct InstancePoolSummaryRequired {
    /// The OCID of the instance pool.
    pub id: String,

    /// The OCID of the compartment containing the instance pool.
    pub compartment_id: String,

    /// The OCID of the instance configuration associated with the instance pool.
    pub instance_configuration_id: String,

    /// The current state of the instance pool.
    pub lifecycle_state: InstancePoolSummaryLifecycleState,

    /// The availability domains for the instance pool.
    pub availability_domains: Vec<String>,

    /// The number of instances that should be in the instance pool. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size: i64,

    /// The date and time the instance pool was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl InstancePoolSummary {
    /// Create a new InstancePoolSummary with required fields
    pub fn new(required: InstancePoolSummaryRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            instance_configuration_id: required.instance_configuration_id,

            lifecycle_state: required.lifecycle_state,

            availability_domains: required.availability_domains,

            size: required.size,

            time_created: required.time_created,

            display_name: None,

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
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set instance_configuration_id
    pub fn set_instance_configuration_id(mut self, value: String) -> Self {
        self.instance_configuration_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: InstancePoolSummaryLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set availability_domains
    pub fn set_availability_domains(mut self, value: Vec<String>) -> Self {
        self.availability_domains = value;
        self
    }

    /// Set size
    pub fn set_size(mut self, value: i64) -> Self {
        self.size = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
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

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
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

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }
}
