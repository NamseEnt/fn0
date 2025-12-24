use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The data to update a [cluster network with instance pools](https://docs.oracle.com/iaas/Content/Compute/Tasks/managingclusternetworks.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClusterNetworkDetails {
    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The instance pools in the cluster network to update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_pools: Option<Vec<UpdateClusterNetworkInstancePoolDetails>>,
}

impl UpdateClusterNetworkDetails {
    /// Create a new UpdateClusterNetworkDetails
    pub fn new() -> Self {
        Self {
            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            instance_pools: None,
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

    /// Set instance_pools
    pub fn set_instance_pools(
        mut self,
        value: Option<Vec<UpdateClusterNetworkInstancePoolDetails>>,
    ) -> Self {
        self.instance_pools = value;
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

    /// Set instance_pools (unwraps Option)
    pub fn with_instance_pools(
        mut self,
        value: Vec<UpdateClusterNetworkInstancePoolDetails>,
    ) -> Self {
        self.instance_pools = Some(value);
        self
    }
}

impl Default for UpdateClusterNetworkDetails {
    fn default() -> Self {
        Self::new()
    }
}
