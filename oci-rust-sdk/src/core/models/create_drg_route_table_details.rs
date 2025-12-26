use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Details used in a request to create a DRG route table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDrgRouteTableDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG the DRG route table belongs to.
    pub drg_id: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the import route distribution used to specify how incoming route advertisements through referenced attachments are inserted into the DRG route table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_drg_route_distribution_id: Option<String>,

    /// If you want traffic to be routed using ECMP across your virtual circuits or IPSec tunnels to your on-premises networks, enable ECMP on the DRG route table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_ecmp_enabled: Option<bool>,
}

/// Required fields for CreateDrgRouteTableDetails
pub struct CreateDrgRouteTableDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG the DRG route table belongs to.
    pub drg_id: String,
}

impl CreateDrgRouteTableDetails {
    /// Create a new CreateDrgRouteTableDetails with required fields
    pub fn new(required: CreateDrgRouteTableDetailsRequired) -> Self {
        Self {
            drg_id: required.drg_id,

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

    /// Set drg_id
    pub fn set_drg_id(mut self, value: String) -> Self {
        self.drg_id = value;
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
