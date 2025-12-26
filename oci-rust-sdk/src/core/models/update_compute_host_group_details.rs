use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Update details information for a compute host group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComputeHostGroupDetails {
    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// A list of HostGroupConfiguration objects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configurations: Option<Vec<HostGroupConfiguration>>,

    /// A flag that allows customers to restrict placement for hosts attached to the group. If true, the only way to place on hosts is to target the specific host group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_targeted_placement_required: Option<bool>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

impl UpdateComputeHostGroupDetails {
    /// Create a new UpdateComputeHostGroupDetails
    pub fn new() -> Self {
        Self {
            display_name: None,

            configurations: None,

            is_targeted_placement_required: None,

            defined_tags: None,

            freeform_tags: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set configurations
    pub fn set_configurations(mut self, value: Option<Vec<HostGroupConfiguration>>) -> Self {
        self.configurations = value;
        self
    }

    /// Set is_targeted_placement_required
    pub fn set_is_targeted_placement_required(mut self, value: Option<bool>) -> Self {
        self.is_targeted_placement_required = value;
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

    /// Set configurations (unwraps Option)
    pub fn with_configurations(mut self, value: Vec<HostGroupConfiguration>) -> Self {
        self.configurations = Some(value);
        self
    }

    /// Set is_targeted_placement_required (unwraps Option)
    pub fn with_is_targeted_placement_required(mut self, value: bool) -> Self {
        self.is_targeted_placement_required = Some(value);
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

impl Default for UpdateComputeHostGroupDetails {
    fn default() -> Self {
        Self::new()
    }
}
