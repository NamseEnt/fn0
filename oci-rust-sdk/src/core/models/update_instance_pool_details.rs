use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The data to update an instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstancePoolDetails {
    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance configuration associated with the instance pool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_configuration_id: Option<String>,

    /// The placement configurations for the instance pool. Provide one placement configuration for each availability domain. <p> To use the instance pool with a regional subnet, provide a placement configuration for each availability domain, and include the regional subnet in each placement configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_configurations: Option<Vec<UpdateInstancePoolPlacementConfigurationDetails>>,

    /// The number of instances that should be in the instance pool. <p> To determine whether capacity is available for a specific shape before you resize an instance pool, use the {@link #createComputeCapacityReport(CreateComputeCapacityReportRequest) createComputeCapacityReport} operation. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    /// A user-friendly formatter for the instance pool's instances. Instance displaynames follow the format. The formatter does not retroactively change instance's displaynames, only instance displaynames in the future follow the format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_display_name_formatter: Option<String>,

    /// A user-friendly formatter for the instance pool's instances. Instance hostnames follow the format. The formatter does not retroactively change instance's hostnames, only instance hostnames in the future follow the format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_hostname_formatter: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_management: Option<InstancePoolLifecycleManagementDetails>,
}

impl UpdateInstancePoolDetails {
    /// Create a new UpdateInstancePoolDetails
    pub fn new() -> Self {
        Self {
            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            instance_configuration_id: None,

            placement_configurations: None,

            size: None,

            instance_display_name_formatter: None,

            instance_hostname_formatter: None,

            lifecycle_management: None,
        }
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
    pub fn set_instance_configuration_id(mut self, value: Option<String>) -> Self {
        self.instance_configuration_id = value;
        self
    }

    /// Set placement_configurations
    pub fn set_placement_configurations(
        mut self,
        value: Option<Vec<UpdateInstancePoolPlacementConfigurationDetails>>,
    ) -> Self {
        self.placement_configurations = value;
        self
    }

    /// Set size
    pub fn set_size(mut self, value: Option<i64>) -> Self {
        self.size = value;
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

    /// Set instance_configuration_id (unwraps Option)
    pub fn with_instance_configuration_id(mut self, value: impl Into<String>) -> Self {
        self.instance_configuration_id = Some(value.into());
        self
    }

    /// Set placement_configurations (unwraps Option)
    pub fn with_placement_configurations(
        mut self,
        value: Vec<UpdateInstancePoolPlacementConfigurationDetails>,
    ) -> Self {
        self.placement_configurations = Some(value);
        self
    }

    /// Set size (unwraps Option)
    pub fn with_size(mut self, value: i64) -> Self {
        self.size = Some(value);
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

impl Default for UpdateInstancePoolDetails {
    fn default() -> Self {
        Self::new()
    }
}
