use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDrgAttachmentDetails {
    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table that is assigned to this attachment. <p> The DRG route table manages traffic inside the DRG. <p> You can't remove a DRG route table from a DRG attachment, but you can reassign which DRG route table it uses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drg_route_table_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_details: Option<VcnDrgAttachmentNetworkUpdateDetails>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the export route distribution used to specify how routes in the assigned DRG route table are advertised out through the attachment. If this value is null, no routes are advertised through this attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_drg_route_distribution_id: Option<String>,

    /// This is the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table that is used to route the traffic as it enters a VCN through this attachment. <p> For information about why you would associate a route table with a DRG attachment, see: <p> [Transit Routing: Access to Multiple VCNs in Same Region](https://docs.oracle.com/iaas/Content/Network/Tasks/transitrouting.htm) * [Transit Routing: Private Access to Oracle Services](https://docs.oracle.com/iaas/Content/Network/Tasks/transitroutingoracleservices.htm)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,
}

impl UpdateDrgAttachmentDetails {
    /// Create a new UpdateDrgAttachmentDetails
    pub fn new() -> Self {
        Self {
            display_name: None,

            drg_route_table_id: None,

            network_details: None,

            defined_tags: None,

            freeform_tags: None,

            export_drg_route_distribution_id: None,

            route_table_id: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set drg_route_table_id
    pub fn set_drg_route_table_id(mut self, value: Option<String>) -> Self {
        self.drg_route_table_id = value;
        self
    }

    /// Set network_details
    pub fn set_network_details(
        mut self,
        value: Option<VcnDrgAttachmentNetworkUpdateDetails>,
    ) -> Self {
        self.network_details = value;
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

    /// Set export_drg_route_distribution_id
    pub fn set_export_drg_route_distribution_id(mut self, value: Option<String>) -> Self {
        self.export_drg_route_distribution_id = value;
        self
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set drg_route_table_id (unwraps Option)
    pub fn with_drg_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.drg_route_table_id = Some(value.into());
        self
    }

    /// Set network_details (unwraps Option)
    pub fn with_network_details(mut self, value: VcnDrgAttachmentNetworkUpdateDetails) -> Self {
        self.network_details = Some(value);
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

    /// Set export_drg_route_distribution_id (unwraps Option)
    pub fn with_export_drg_route_distribution_id(mut self, value: impl Into<String>) -> Self {
        self.export_drg_route_distribution_id = Some(value.into());
        self
    }

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }
}

impl Default for UpdateDrgAttachmentDetails {
    fn default() -> Self {
        Self::new()
    }
}
