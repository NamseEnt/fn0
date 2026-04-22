use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// All routing inside the DRG is driven by the contents of DRG route tables. DRG route tables contain rules which route packets to a particular network destination, represented as a DRG attachment. The routing decision for a packet entering a DRG is determined by the rules in the DRG route table assigned to the attachment-of-entry. <p> Each DRG attachment can inject routes in any DRG route table, provided there is a statement corresponding to the attachment in the route table's {@code importDrgRouteDistribution}. You can also insert static routes into the DRG route tables. <p> The DRG route table is always in the same compartment as the DRG. There must always be a default DRG route table for each attachment type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgRouteTable {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment the DRG is in. The DRG route table is always in the same compartment as the DRG.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG the DRG that contains this route table.
    pub drg_id: String,

    /// The date and time the DRG route table was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The DRG route table's current state.
    pub lifecycle_state: DrgRouteTableLifecycleState,

    /// If you want traffic to be routed using ECMP across your virtual circuits or IPSec tunnels to your on-premises network, enable ECMP on the DRG route table to which these attachments import routes.
    pub is_ecmp_enabled: bool,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the import route distribution used to specify how incoming route advertisements from referenced attachments are inserted into the DRG route table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_drg_route_distribution_id: Option<String>,
}

/// Required fields for DrgRouteTable
pub struct DrgRouteTableRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG route table.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment the DRG is in. The DRG route table is always in the same compartment as the DRG.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG the DRG that contains this route table.
    pub drg_id: String,

    /// The date and time the DRG route table was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The DRG route table's current state.
    pub lifecycle_state: DrgRouteTableLifecycleState,

    /// If you want traffic to be routed using ECMP across your virtual circuits or IPSec tunnels to your on-premises network, enable ECMP on the DRG route table to which these attachments import routes.
    pub is_ecmp_enabled: bool,
}

impl DrgRouteTable {
    /// Create a new DrgRouteTable with required fields
    pub fn new(required: DrgRouteTableRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            drg_id: required.drg_id,

            time_created: required.time_created,

            lifecycle_state: required.lifecycle_state,

            is_ecmp_enabled: required.is_ecmp_enabled,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            import_drg_route_distribution_id: None,
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

    /// Set drg_id
    pub fn set_drg_id(mut self, value: String) -> Self {
        self.drg_id = value;
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

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: DrgRouteTableLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set import_drg_route_distribution_id
    pub fn set_import_drg_route_distribution_id(mut self, value: Option<String>) -> Self {
        self.import_drg_route_distribution_id = value;
        self
    }

    /// Set is_ecmp_enabled
    pub fn set_is_ecmp_enabled(mut self, value: bool) -> Self {
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
}
