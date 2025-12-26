use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVlanDetails {
    /// The range of IPv4 addresses that will be used for layer 3 communication with hosts outside the VLAN. The CIDR must maintain the following rules - <p> 1. The CIDR block is valid and correctly formatted. 2. The new range is within one of the parent VCN ranges. <p> Example: {@code 192.0.2.0/24}
    pub cidr_block: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the VLAN.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN to contain the VLAN.
    pub vcn_id: String,

    /// Controls whether the VLAN is regional or specific to an availability domain. A regional VLAN has the flexibility to implement failover across availability domains. Previously, all VLANs were AD-specific. <p> To create a regional VLAN, omit this attribute. Resources created subsequently in this VLAN (such as a Compute instance) can be created in any availability domain in the region. <p> To create an AD-specific VLAN, use this attribute to specify the availability domain. Resources created in this VLAN must be in that availability domain. <p> Example: {@code Uocm:PHX-AD-1}
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

    /// A list of the OCIDs of the network security groups (NSGs) to add all VNICs in the VLAN to. For more information about NSGs, see {@link NetworkSecurityGroup}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsg_ids: Option<Vec<String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the VLAN will use. If you don't provide a value, the VLAN uses the VCN's default route table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,

    /// The IEEE 802.1Q VLAN tag for this VLAN. The value must be unique across all VLANs in the VCN. If you don't provide a value, Oracle assigns one. You cannot change the value later. VLAN tag 0 is reserved for use by Oracle. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_tag: Option<i64>,
}

/// Required fields for CreateVlanDetails
pub struct CreateVlanDetailsRequired {
    /// The range of IPv4 addresses that will be used for layer 3 communication with hosts outside the VLAN. The CIDR must maintain the following rules - <p> 1. The CIDR block is valid and correctly formatted. 2. The new range is within one of the parent VCN ranges. <p> Example: {@code 192.0.2.0/24}
    pub cidr_block: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the VLAN.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN to contain the VLAN.
    pub vcn_id: String,
}

impl CreateVlanDetails {
    /// Create a new CreateVlanDetails with required fields
    pub fn new(required: CreateVlanDetailsRequired) -> Self {
        Self {
            cidr_block: required.cidr_block,

            compartment_id: required.compartment_id,

            vcn_id: required.vcn_id,

            availability_domain: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            nsg_ids: None,

            route_table_id: None,

            vlan_tag: None,
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

    /// Set nsg_ids
    pub fn set_nsg_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.nsg_ids = value;
        self
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: String) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set vlan_tag
    pub fn set_vlan_tag(mut self, value: Option<i64>) -> Self {
        self.vlan_tag = value;
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

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }

    /// Set vlan_tag (unwraps Option)
    pub fn with_vlan_tag(mut self, value: i64) -> Self {
        self.vlan_tag = Some(value);
        self
    }
}
