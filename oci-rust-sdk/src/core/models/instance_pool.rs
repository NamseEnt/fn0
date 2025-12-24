use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// An instance pool is a set of instances within the same region that are managed as a group. For more information about instance pools and instance configurations, see [Managing Compute Instances](https://docs.oracle.com/iaas/Content/Compute/Concepts/instancemanagement.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancePool {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the instance pool.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance configuration associated with the instance pool.
    pub instance_configuration_id: String,

    /// The current state of the instance pool.
    pub lifecycle_state: InstancePoolLifecycleState,

    /// The placement configurations for the instance pool.
    pub placement_configurations: Vec<InstancePoolPlacementConfiguration>,

    /// The number of instances that should be in the instance pool. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size: i64,

    /// The date and time the instance pool was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
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

    /// The load balancers attached to the instance pool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_balancers: Option<Vec<InstancePoolLoadBalancerAttachment>>,

    /// A user-friendly formatter for the instance pool's instances. Instance displaynames follow the format. The formatter does not retroactively change instance's displaynames, only instance displaynames in the future follow the format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_display_name_formatter: Option<String>,

    /// A user-friendly formatter for the instance pool's instances. Instance hostnames follow the format. The formatter does not retroactively change instance's hostnames, only instance hostnames in the future follow the format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_hostname_formatter: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_management: Option<InstancePoolLifecycleManagementDetails>,
}

/// Required fields for InstancePool
pub struct InstancePoolRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance pool.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the instance pool.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance configuration associated with the instance pool.
    pub instance_configuration_id: String,

    /// The current state of the instance pool.
    pub lifecycle_state: InstancePoolLifecycleState,

    /// The placement configurations for the instance pool.
    pub placement_configurations: Vec<InstancePoolPlacementConfiguration>,

    /// The number of instances that should be in the instance pool. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size: i64,

    /// The date and time the instance pool was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl InstancePool {
    /// Create a new InstancePool with required fields
    pub fn new(required: InstancePoolRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            instance_configuration_id: required.instance_configuration_id,

            lifecycle_state: required.lifecycle_state,

            placement_configurations: required.placement_configurations,

            size: required.size,

            time_created: required.time_created,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            load_balancers: None,

            instance_display_name_formatter: None,

            instance_hostname_formatter: None,

            lifecycle_management: None,
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

    /// Set instance_configuration_id
    pub fn set_instance_configuration_id(mut self, value: String) -> Self {
        self.instance_configuration_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: InstancePoolLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set placement_configurations
    pub fn set_placement_configurations(
        mut self,
        value: Vec<InstancePoolPlacementConfiguration>,
    ) -> Self {
        self.placement_configurations = value;
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

    /// Set load_balancers
    pub fn set_load_balancers(
        mut self,
        value: Option<Vec<InstancePoolLoadBalancerAttachment>>,
    ) -> Self {
        self.load_balancers = value;
        self
    }

    /// Set instance_display_name_formatter
    pub fn set_instance_display_name_formatter(mut self, value: Option<String>) -> Self {
        self.instance_display_name_formatter = value;
        self
    }

    /// Set instance_hostname_formatter
    pub fn set_instance_hostname_formatter(mut self, value: Option<String>) -> Self {
        self.instance_hostname_formatter = value;
        self
    }

    /// Set lifecycle_management
    pub fn set_lifecycle_management(
        mut self,
        value: Option<InstancePoolLifecycleManagementDetails>,
    ) -> Self {
        self.lifecycle_management = value;
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

    /// Set load_balancers (unwraps Option)
    pub fn with_load_balancers(mut self, value: Vec<InstancePoolLoadBalancerAttachment>) -> Self {
        self.load_balancers = Some(value);
        self
    }

    /// Set instance_display_name_formatter (unwraps Option)
    pub fn with_instance_display_name_formatter(mut self, value: impl Into<String>) -> Self {
        self.instance_display_name_formatter = Some(value.into());
        self
    }

    /// Set instance_hostname_formatter (unwraps Option)
    pub fn with_instance_hostname_formatter(mut self, value: impl Into<String>) -> Self {
        self.instance_hostname_formatter = Some(value.into());
        self
    }

    /// Set lifecycle_management (unwraps Option)
    pub fn with_lifecycle_management(
        mut self,
        value: InstancePoolLifecycleManagementDetails,
    ) -> Self {
        self.lifecycle_management = Some(value);
        self
    }
}
