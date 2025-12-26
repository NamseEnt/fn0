use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A resource to be used only with the Oracle Cloud VMware Solution. <p> Conceptually, a virtual LAN (VLAN) is a broadcast domain that is created by partitioning and isolating a network at the data link layer (a *layer 2 network*). VLANs work by using IEEE 802.1Q VLAN tags. Layer 2 traffic is forwarded within the VLAN based on MAC learning. <p> In the Networking service, a VLAN is an object within a VCN. You use VLANs to partition the VCN at the data link layer (layer 2). A VLAN is analagous to a subnet, which is an object for partitioning the VCN at the IP layer (layer 3).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vlan {
    /// The range of IPv4 addresses that will be used for layer 3 communication with hosts outside the VLAN. <p> Example: {@code 192.168.1.0/24}
    pub cidr_block: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the VLAN.
    pub compartment_id: String,

    /// The VLAN's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The VLAN's current state.
    pub lifecycle_state: VlanLifecycleState,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the VLAN is in.
    pub vcn_id: String,

    /// The VLAN's availability domain. This attribute will be null if this is a regional VLAN rather than an AD-specific VLAN. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// A list of the OCIDs of the network security groups (NSGs) to use with this VLAN. All VNICs in the VLAN belong to these NSGs. For more information about NSGs, see {@link NetworkSecurityGroup}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsg_ids: Option<Vec<String>>,

    /// The IEEE 802.1Q VLAN tag of this VLAN. <p> Example: {@code 100} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_tag: Option<i64>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table that the VLAN uses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,

    /// The date and time the VLAN was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,
}

/// Required fields for Vlan
pub struct VlanRequired {
    /// The range of IPv4 addresses that will be used for layer 3 communication with hosts outside the VLAN. <p> Example: {@code 192.168.1.0/24}
    pub cidr_block: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the VLAN.
    pub compartment_id: String,

    /// The VLAN's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The VLAN's current state.
    pub lifecycle_state: VlanLifecycleState,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the VLAN is in.
    pub vcn_id: String,
}

impl Vlan {
    /// Create a new Vlan with required fields
    pub fn new(required: VlanRequired) -> Self {
        Self {
            cidr_block: required.cidr_block,

            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            vcn_id: required.vcn_id,

            availability_domain: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            nsg_ids: None,

            vlan_tag: None,

            route_table_id: None,

            time_created: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: String) -> Self {
        self.cidr_block = value;
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
    pub fn set_lifecycle_state(mut self, value: VlanLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set nsg_ids
    pub fn set_nsg_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.nsg_ids = value;
        self
    }

    /// Set vlan_tag
    pub fn set_vlan_tag(mut self, value: Option<i64>) -> Self {
        self.vlan_tag = value;
        self
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
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

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
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

    /// Set nsg_ids (unwraps Option)
    pub fn with_nsg_ids(mut self, value: Vec<String>) -> Self {
        self.nsg_ids = Some(value);
        self
    }

    /// Set vlan_tag (unwraps Option)
    pub fn with_vlan_tag(mut self, value: i64) -> Self {
        self.vlan_tag = Some(value);
        self
    }

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }
}
