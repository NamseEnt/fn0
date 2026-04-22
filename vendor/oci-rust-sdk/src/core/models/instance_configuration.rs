use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// An instance configuration is a template that defines the settings to use when creating Compute instances. For more information about instance configurations, see [Managing Compute Instances](https://docs.oracle.com/iaas/Content/Compute/Concepts/instancemanagement.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfiguration {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the instance configuration.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance configuration.
    pub id: String,

    /// The date and time the instance configuration was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_details: Option<ComputeInstanceOptions>,

    /// Parameters that were not specified when the instance configuration was created, but that are required to launch an instance from the instance configuration. See the {@link #launchInstanceConfiguration(LaunchInstanceConfigurationRequest) launchInstanceConfiguration} operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deferred_fields: Option<Vec<String>>,
}

/// Required fields for InstanceConfiguration
pub struct InstanceConfigurationRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the instance configuration.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance configuration.
    pub id: String,

    /// The date and time the instance configuration was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl InstanceConfiguration {
    /// Create a new InstanceConfiguration with required fields
    pub fn new(required: InstanceConfigurationRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            id: required.id,

            time_created: required.time_created,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            instance_details: None,

            deferred_fields: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
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
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set instance_details
    pub fn set_instance_details(mut self, value: Option<ComputeInstanceOptions>) -> Self {
        self.instance_details = value;
        self
    }

    /// Set deferred_fields
    pub fn set_deferred_fields(mut self, value: Option<Vec<String>>) -> Self {
        self.deferred_fields = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
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

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set instance_details (unwraps Option)
    pub fn with_instance_details(mut self, value: ComputeInstanceOptions) -> Self {
        self.instance_details = Some(value);
        self
    }

    /// Set deferred_fields (unwraps Option)
    pub fn with_deferred_fields(mut self, value: Vec<String>) -> Self {
        self.deferred_fields = Some(value);
        self
    }
}
