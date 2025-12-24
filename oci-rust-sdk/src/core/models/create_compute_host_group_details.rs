use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Detail information for a compute host group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComputeHostGroupDetails {
    /// The availability domain of a host group. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the compartment that contains host group.
    pub compartment_id: String,

    /// A flag that allows customers to restrict placement for hosts attached to the group. If true, the only way to place on hosts is to target the specific host group.
    pub is_targeted_placement_required: bool,

    /// A list of HostGroupConfiguration objects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configurations: Option<Vec<HostGroupConfiguration>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

/// Required fields for CreateComputeHostGroupDetails
pub struct CreateComputeHostGroupDetailsRequired {
    /// The availability domain of a host group. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The OCID of the compartment that contains host group.
    pub compartment_id: String,

    /// A flag that allows customers to restrict placement for hosts attached to the group. If true, the only way to place on hosts is to target the specific host group.
    pub is_targeted_placement_required: bool,
}

impl CreateComputeHostGroupDetails {
    /// Create a new CreateComputeHostGroupDetails with required fields
    pub fn new(required: CreateComputeHostGroupDetailsRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            display_name: required.display_name,

            compartment_id: required.compartment_id,

            is_targeted_placement_required: required.is_targeted_placement_required,

            configurations: None,

            defined_tags: None,

            freeform_tags: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set configurations
    pub fn set_configurations(mut self, value: Option<Vec<HostGroupConfiguration>>) -> Self {
        self.configurations = value;
        self
    }

    /// Set is_targeted_placement_required
    pub fn set_is_targeted_placement_required(mut self, value: bool) -> Self {
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

    /// Set configurations (unwraps Option)
    pub fn with_configurations(mut self, value: Vec<HostGroupConfiguration>) -> Self {
        self.configurations = Some(value);
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
