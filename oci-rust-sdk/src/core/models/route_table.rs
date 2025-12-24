use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A collection of {@code RouteRule} objects, which are used to route packets based on destination IP to a particular network entity. For more information, see [Overview of the Networking Service](https://docs.oracle.com/iaas/Content/Network/Concepts/overview.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteTable {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the route table.
    pub compartment_id: String,

    /// The route table's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The route table's current state.
    pub lifecycle_state: RouteTableLifecycleState,

    /// The collection of rules for routing destination IPs to network devices.
    pub route_rules: Vec<RouteRule>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the route table list belongs to.
    pub vcn_id: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The date and time the route table was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,
}

/// Required fields for RouteTable
pub struct RouteTableRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the route table.
    pub compartment_id: String,

    /// The route table's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The route table's current state.
    pub lifecycle_state: RouteTableLifecycleState,

    /// The collection of rules for routing destination IPs to network devices.
    pub route_rules: Vec<RouteRule>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the route table list belongs to.
    pub vcn_id: String,
}

impl RouteTable {
    /// Create a new RouteTable with required fields
    pub fn new(required: RouteTableRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            route_rules: required.route_rules,

            vcn_id: required.vcn_id,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            time_created: None,
        }
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
    pub fn set_lifecycle_state(mut self, value: RouteTableLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set route_rules
    pub fn set_route_rules(mut self, value: Vec<RouteRule>) -> Self {
        self.route_rules = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: String) -> Self {
        self.vcn_id = value;
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

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }
}
