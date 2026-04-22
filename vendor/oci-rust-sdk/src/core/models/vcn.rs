use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A virtual cloud network (VCN). For more information, see [Overview of the Networking Service](https://docs.oracle.com/iaas/Content/Network/Concepts/overview.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vcn {
    /// Deprecated. The first CIDR IP address from cidrBlocks. <p> Example: {@code 172.16.0.0/16}
    pub cidr_block: String,

    /// The list of IPv4 CIDR blocks the VCN will use.
    pub cidr_blocks: Vec<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the VCN.
    pub compartment_id: String,

    /// The VCN's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The VCN's current state.
    pub lifecycle_state: VcnLifecycleState,

    /// The list of BYOIPv6 prefixes required to create a VCN that uses BYOIPv6 ranges.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byoipv6_cidr_blocks: Option<Vec<String>>,

    /// For an IPv6-enabled VCN, this is the list of Private IPv6 prefixes for the VCN's IP address space.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_private_cidr_blocks: Option<Vec<String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the VCN's default set of DHCP options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_dhcp_options_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the VCN's default route table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_route_table_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the VCN's default security list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_security_list_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// A DNS label for the VCN, used in conjunction with the VNIC's hostname and subnet's DNS label to form a fully qualified domain name (FQDN) for each VNIC within this subnet (for example, {@code bminstance1.subnet123.vcn1.oraclevcn.com}). Must be an alphanumeric string that begins with a letter. The value cannot be changed. <p> The absence of this parameter means the Internet and VCN Resolver will not work for this VCN. <p> For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm). <p> Example: {@code vcn1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_label: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// [Security attributes](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/zpr-artifacts.htm#security-attributes) are labels for a resource that can be referenced in a [Zero Trust Packet Routing](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/overview.htm) (ZPR) policy to control access to ZPR-supported resources. <p> Example: {@code {\"Oracle-DataSecurity-ZPR\": {\"MaxEgressCount\": {\"value\":\"42\",\"mode\":\"audit\"}}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_attributes: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// For an IPv6-enabled VCN, this is the list of IPv6 prefixes for the VCN's IP address space. The prefixes are provided by Oracle and the sizes are always /56.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_blocks: Option<Vec<String>>,

    /// The date and time the VCN was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The VCN's domain name, which consists of the VCN's DNS label, and the {@code oraclevcn.com} domain. <p> For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm). <p> Example: {@code vcn1.oraclevcn.com}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcn_domain_name: Option<String>,

    /// Indicates whether ZPR Only mode is enforced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_zpr_only: Option<bool>,
}

/// Required fields for Vcn
pub struct VcnRequired {
    /// Deprecated. The first CIDR IP address from cidrBlocks. <p> Example: {@code 172.16.0.0/16}
    pub cidr_block: String,

    /// The list of IPv4 CIDR blocks the VCN will use.
    pub cidr_blocks: Vec<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the VCN.
    pub compartment_id: String,

    /// The VCN's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The VCN's current state.
    pub lifecycle_state: VcnLifecycleState,
}

impl Vcn {
    /// Create a new Vcn with required fields
    pub fn new(required: VcnRequired) -> Self {
        Self {
            cidr_block: required.cidr_block,

            cidr_blocks: required.cidr_blocks,

            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            byoipv6_cidr_blocks: None,

            ipv6_private_cidr_blocks: None,

            default_dhcp_options_id: None,

            default_route_table_id: None,

            default_security_list_id: None,

            defined_tags: None,

            display_name: None,

            dns_label: None,

            freeform_tags: None,

            security_attributes: None,

            ipv6_cidr_blocks: None,

            time_created: None,

            vcn_domain_name: None,

            is_zpr_only: None,
        }
    }

    /// Set byoipv6_cidr_blocks
    pub fn set_byoipv6_cidr_blocks(mut self, value: Option<Vec<String>>) -> Self {
        self.byoipv6_cidr_blocks = value;
        self
    }

    /// Set ipv6_private_cidr_blocks
    pub fn set_ipv6_private_cidr_blocks(mut self, value: Option<Vec<String>>) -> Self {
        self.ipv6_private_cidr_blocks = value;
        self
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: String) -> Self {
        self.cidr_block = value;
        self
    }

    /// Set cidr_blocks
    pub fn set_cidr_blocks(mut self, value: Vec<String>) -> Self {
        self.cidr_blocks = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set default_dhcp_options_id
    pub fn set_default_dhcp_options_id(mut self, value: Option<String>) -> Self {
        self.default_dhcp_options_id = value;
        self
    }

    /// Set default_route_table_id
    pub fn set_default_route_table_id(mut self, value: Option<String>) -> Self {
        self.default_route_table_id = value;
        self
    }

    /// Set default_security_list_id
    pub fn set_default_security_list_id(mut self, value: Option<String>) -> Self {
        self.default_security_list_id = value;
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

    /// Set dns_label
    pub fn set_dns_label(mut self, value: Option<String>) -> Self {
        self.dns_label = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set security_attributes
    pub fn set_security_attributes(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.security_attributes = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set ipv6_cidr_blocks
    pub fn set_ipv6_cidr_blocks(mut self, value: Option<Vec<String>>) -> Self {
        self.ipv6_cidr_blocks = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: VcnLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set vcn_domain_name
    pub fn set_vcn_domain_name(mut self, value: Option<String>) -> Self {
        self.vcn_domain_name = value;
        self
    }

    /// Set is_zpr_only
    pub fn set_is_zpr_only(mut self, value: Option<bool>) -> Self {
        self.is_zpr_only = value;
        self
    }

    /// Set byoipv6_cidr_blocks (unwraps Option)
    pub fn with_byoipv6_cidr_blocks(mut self, value: Vec<String>) -> Self {
        self.byoipv6_cidr_blocks = Some(value);
        self
    }

    /// Set ipv6_private_cidr_blocks (unwraps Option)
    pub fn with_ipv6_private_cidr_blocks(mut self, value: Vec<String>) -> Self {
        self.ipv6_private_cidr_blocks = Some(value);
        self
    }

    /// Set default_dhcp_options_id (unwraps Option)
    pub fn with_default_dhcp_options_id(mut self, value: impl Into<String>) -> Self {
        self.default_dhcp_options_id = Some(value.into());
        self
    }

    /// Set default_route_table_id (unwraps Option)
    pub fn with_default_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.default_route_table_id = Some(value.into());
        self
    }

    /// Set default_security_list_id (unwraps Option)
    pub fn with_default_security_list_id(mut self, value: impl Into<String>) -> Self {
        self.default_security_list_id = Some(value.into());
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

    /// Set dns_label (unwraps Option)
    pub fn with_dns_label(mut self, value: impl Into<String>) -> Self {
        self.dns_label = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set security_attributes (unwraps Option)
    pub fn with_security_attributes(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.security_attributes = Some(value);
        self
    }

    /// Set ipv6_cidr_blocks (unwraps Option)
    pub fn with_ipv6_cidr_blocks(mut self, value: Vec<String>) -> Self {
        self.ipv6_cidr_blocks = Some(value);
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set vcn_domain_name (unwraps Option)
    pub fn with_vcn_domain_name(mut self, value: impl Into<String>) -> Self {
        self.vcn_domain_name = Some(value.into());
        self
    }

    /// Set is_zpr_only (unwraps Option)
    pub fn with_is_zpr_only(mut self, value: bool) -> Self {
        self.is_zpr_only = Some(value);
        self
    }
}
