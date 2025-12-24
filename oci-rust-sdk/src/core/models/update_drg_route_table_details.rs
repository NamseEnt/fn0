use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Details used in a request to update a DRG route table. <p> You can't assign a table to a virtual circuit or IPSec tunnel attachment if there is a static route rule for an RPC attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDrgRouteTableDetails {
    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the import route distribution used to specify how incoming route advertisements through referenced attachements are inserted into the DRG route table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_drg_route_distribution_id: Option<String>,

    /// If you want traffic to be routed using ECMP across your virtual circuits or IPSec tunnels to your on-prem networks, set this value to true on the route table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_ecmp_enabled: Option<bool>,
}

impl UpdateDrgRouteTableDetails {
    /// Create a new UpdateDrgRouteTableDetails
    pub fn new() -> Self {
        Self {
            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            import_drg_route_distribution_id: None,

            is_ecmp_enabled: None,
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

    /// Set import_drg_route_distribution_id
    pub fn set_import_drg_route_distribution_id(mut self, value: Option<String>) -> Self {
        self.import_drg_route_distribution_id = value;
        self
    }

    /// Set is_ecmp_enabled
    pub fn set_is_ecmp_enabled(mut self, value: Option<bool>) -> Self {
        self.is_ecmp_enabled = value;
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

    /// Set import_drg_route_distribution_id (unwraps Option)
    pub fn with_import_drg_route_distribution_id(mut self, value: impl Into<String>) -> Self {
        self.import_drg_route_distribution_id = Some(value.into());
        self
    }

    /// Set is_ecmp_enabled (unwraps Option)
    pub fn with_is_ecmp_enabled(mut self, value: bool) -> Self {
        self.is_ecmp_enabled = Some(value);
        self
    }
}

impl Default for UpdateDrgRouteTableDetails {
    fn default() -> Self {
        Self::new()
    }
}
