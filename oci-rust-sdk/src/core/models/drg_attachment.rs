use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A DRG attachment serves as a link between a DRG and a network resource. A DRG can be attached to a VCN, IPSec tunnel, remote peering connection, or virtual circuit. <p> For more information, see [Overview of the Networking Service](https://docs.oracle.com/iaas/Content/Network/Concepts/overview.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgAttachment {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the DRG attachment.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    pub drg_id: String,

    /// The DRG attachment's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The DRG attachment's current state.
    pub lifecycle_state: DrgAttachmentLifecycleState,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The date and time the DRG attachment was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table that is assigned to this attachment. <p> The DRG route table manages traffic inside the DRG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drg_route_table_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_details: Option<VcnDrgAttachmentNetworkDetails>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the DRG attachment is using. <p> For information about why you would associate a route table with a DRG attachment, see: <p> [Transit Routing: Access to Multiple VCNs in Same Region](https://docs.oracle.com/iaas/Content/Network/Tasks/transitrouting.htm) * [Transit Routing: Private Access to Oracle Services](https://docs.oracle.com/iaas/Content/Network/Tasks/transitroutingoracleservices.htm) <p> This field is deprecated. Instead, use the {@code networkDetails} field to view the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the attached resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN. This field is deprecated. Instead, use the {@code networkDetails} field to view the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the attached resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcn_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the export route distribution used to specify how routes in the assigned DRG route table are advertised to the attachment. If this value is null, no routes are advertised through this attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_drg_route_distribution_id: Option<String>,

    /// Indicates whether the DRG attachment and attached network live in a different tenancy than the DRG. <p> Example: {@code false}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_cross_tenancy: Option<bool>,
}

/// Required fields for DrgAttachment
pub struct DrgAttachmentRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the DRG attachment.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG.
    pub drg_id: String,

    /// The DRG attachment's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The DRG attachment's current state.
    pub lifecycle_state: DrgAttachmentLifecycleState,
}

impl DrgAttachment {
    /// Create a new DrgAttachment with required fields
    pub fn new(required: DrgAttachmentRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            drg_id: required.drg_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            display_name: None,

            time_created: None,

            drg_route_table_id: None,

            network_details: None,

            defined_tags: None,

            freeform_tags: None,

            route_table_id: None,

            vcn_id: None,

            export_drg_route_distribution_id: None,

            is_cross_tenancy: None,
        }
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

    /// Set drg_id
    pub fn set_drg_id(mut self, value: String) -> Self {
        self.drg_id = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: DrgAttachmentLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set drg_route_table_id
    pub fn set_drg_route_table_id(mut self, value: Option<String>) -> Self {
        self.drg_route_table_id = value;
        self
    }

    /// Set network_details
    pub fn set_network_details(mut self, value: Option<VcnDrgAttachmentNetworkDetails>) -> Self {
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

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: Option<String>) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set export_drg_route_distribution_id
    pub fn set_export_drg_route_distribution_id(mut self, value: Option<String>) -> Self {
        self.export_drg_route_distribution_id = value;
        self
    }

    /// Set is_cross_tenancy
    pub fn set_is_cross_tenancy(mut self, value: Option<bool>) -> Self {
        self.is_cross_tenancy = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set drg_route_table_id (unwraps Option)
    pub fn with_drg_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.drg_route_table_id = Some(value.into());
        self
    }

    /// Set network_details (unwraps Option)
    pub fn with_network_details(mut self, value: VcnDrgAttachmentNetworkDetails) -> Self {
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

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }

    /// Set vcn_id (unwraps Option)
    pub fn with_vcn_id(mut self, value: impl Into<String>) -> Self {
        self.vcn_id = Some(value.into());
        self
    }

    /// Set export_drg_route_distribution_id (unwraps Option)
    pub fn with_export_drg_route_distribution_id(mut self, value: impl Into<String>) -> Self {
        self.export_drg_route_distribution_id = Some(value.into());
        self
    }

    /// Set is_cross_tenancy (unwraps Option)
    pub fn with_is_cross_tenancy(mut self, value: bool) -> Self {
        self.is_cross_tenancy = Some(value);
        self
    }
}
