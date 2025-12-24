use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A route distribution establishes how routes get imported into DRG route tables and exported through the DRG attachments. <p> A route distribution is a list of statements. Each statement consists of a set of matches, all of which must be {@code True} for the statement's action to take place. Each statement determines which routes are propagated. <p> You can assign a route distribution as a route table's import distribution. The statements in an import route distribution specify how how incoming route advertisements through a referenced attachment or all attachments of a certain type are inserted into the route table. <p> You can assign a route distribution as a DRG attachment's export distribution unless the attachment has the type {@code VCN}. Exporting routes through a VCN attachment is unsupported. Export route distribution statements specify how routes in a DRG attachment's assigned table are advertised out through the attachment. When a DRG is created, a route distribution is created with a single ACCEPT statement with match criteria MATCH_ALL. By default, all DRG attachments (except for those of type VCN), are assigned this distribution. You can't create a new export route distribution, one is created for you when the DRG is created. <p> The two auto-generated DRG route tables (one as the default for VCN attachments, and the other for all other types of attachments) are each assigned an auto generated import route distribution. The default VCN table's import distribution has a single statement with match criteria MATCH_ALL to import routes from each DRG attachment type. The other table's import distribution has a statement to import routes from attachments with the VCN type. <p> The route distribution is always in the same compartment as the DRG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrgRouteDistribution {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG that contains this route distribution.
    pub drg_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the route distribution.
    pub compartment_id: String,

    /// The route distribution's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The route distribution's current state.
    pub lifecycle_state: DrgRouteDistributionLifecycleState,

    /// The date and time the route distribution was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// Whether this distribution defines how routes get imported into route tables or exported through DRG attachments.
    pub distribution_type: DrgRouteDistributionDistributionType,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

/// Required fields for DrgRouteDistribution
pub struct DrgRouteDistributionRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DRG that contains this route distribution.
    pub drg_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the route distribution.
    pub compartment_id: String,

    /// The route distribution's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The route distribution's current state.
    pub lifecycle_state: DrgRouteDistributionLifecycleState,

    /// The date and time the route distribution was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// Whether this distribution defines how routes get imported into route tables or exported through DRG attachments.
    pub distribution_type: DrgRouteDistributionDistributionType,
}

impl DrgRouteDistribution {
    /// Create a new DrgRouteDistribution with required fields
    pub fn new(required: DrgRouteDistributionRequired) -> Self {
        Self {
            drg_id: required.drg_id,

            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            distribution_type: required.distribution_type,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,
        }
    }

    /// Set drg_id
    pub fn set_drg_id(mut self, value: String) -> Self {
        self.drg_id = value;
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

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: DrgRouteDistributionLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set distribution_type
    pub fn set_distribution_type(mut self, value: DrgRouteDistributionDistributionType) -> Self {
        self.distribution_type = value;
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
}
