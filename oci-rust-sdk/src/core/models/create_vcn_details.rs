use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVcnDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the VCN.
    pub compartment_id: String,

    /// **Deprecated.** Do *not* set this value. Use {@code cidrBlocks} instead. Example: {@code 10.0.0.0/16}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_block: Option<String>,

    /// The list of one or more IPv4 CIDR blocks for the VCN that meet the following criteria: - The CIDR blocks must be valid. - They must not overlap with each other or with the on-premises network CIDR block. - The number of CIDR blocks must not exceed the limit of CIDR blocks allowed per VCN. <p> *Important:** Do *not* specify a value for {@code cidrBlock}. Use this parameter instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_blocks: Option<Vec<String>>,

    /// The list of one or more ULA or Private IPv6 prefixes for the VCN that meets the following criteria: - The CIDR blocks must be valid. - Multiple CIDR blocks must not overlap each other or the on-premises network prefix. - The number of CIDR blocks must not exceed the limit of IPv6 prefixes allowed to a VCN. <p> *Important:** Do *not* specify a value for {@code ipv6CidrBlock}. Use this parameter instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_private_cidr_blocks: Option<Vec<String>>,

    /// Specifies whether to skip Oracle allocated IPv6 GUA. By default, Oracle will allocate one GUA of /56 size for an IPv6 enabled VCN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_oracle_gua_allocation_enabled: Option<bool>,

    /// The list of BYOIPv6 OCIDs and BYOIPv6 prefixes required to create a VCN that uses BYOIPv6 address ranges.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byoipv6_cidr_details: Option<Vec<Byoipv6CidrDetails>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// A DNS label for the VCN, used in conjunction with the VNIC's hostname and subnet's DNS label to form a fully qualified domain name (FQDN) for each VNIC within this subnet (for example, {@code bminstance1.subnet123.vcn1.oraclevcn.com}). Not required to be unique, but it's a best practice to set unique DNS labels for VCNs in your tenancy. Must be an alphanumeric string that begins with a letter. The value cannot be changed. <p> You must set this value if you want instances to be able to use hostnames to resolve other instances in the VCN. Otherwise the Internet and VCN Resolver will not work. <p> For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm). <p> Example: {@code vcn1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_label: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// [Security attributes](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/zpr-artifacts.htm#security-attributes) are labels for a resource that can be referenced in a [Zero Trust Packet Routing](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/overview.htm) (ZPR) policy to control access to ZPR-supported resources. <p> Example: {@code {\"Oracle-DataSecurity-ZPR\": {\"MaxEgressCount\": {\"value\":\"42\",\"mode\":\"audit\"}}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_attributes: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Whether IPv6 is enabled for the VCN. Default is {@code false}. If enabled, Oracle will assign the VCN a IPv6 /56 CIDR block. You may skip having Oracle allocate the VCN a IPv6 /56 CIDR block by setting isOracleGuaAllocationEnabled to {@code false}. For important details about IPv6 addressing in a VCN, see [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_ipv6_enabled: Option<bool>,

    /// Indicates whether ZPR Only mode is enforced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_zpr_only: Option<bool>,
}

/// Required fields for CreateVcnDetails
pub struct CreateVcnDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the VCN.
    pub compartment_id: String,
}

impl CreateVcnDetails {
    /// Create a new CreateVcnDetails with required fields
    pub fn new(required: CreateVcnDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            cidr_block: None,

            cidr_blocks: None,

            ipv6_private_cidr_blocks: None,

            is_oracle_gua_allocation_enabled: None,

            byoipv6_cidr_details: None,

            defined_tags: None,

            display_name: None,

            dns_label: None,

            freeform_tags: None,

            security_attributes: None,

            is_ipv6_enabled: None,

            is_zpr_only: None,
        }
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: Option<String>) -> Self {
        self.cidr_block = value;
        self
    }

    /// Set cidr_blocks
    pub fn set_cidr_blocks(mut self, value: Option<Vec<String>>) -> Self {
        self.cidr_blocks = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set ipv6_private_cidr_blocks
    pub fn set_ipv6_private_cidr_blocks(mut self, value: Option<Vec<String>>) -> Self {
        self.ipv6_private_cidr_blocks = value;
        self
    }

    /// Set is_oracle_gua_allocation_enabled
    pub fn set_is_oracle_gua_allocation_enabled(mut self, value: Option<bool>) -> Self {
        self.is_oracle_gua_allocation_enabled = value;
        self
    }

    /// Set byoipv6_cidr_details
    pub fn set_byoipv6_cidr_details(mut self, value: Option<Vec<Byoipv6CidrDetails>>) -> Self {
        self.byoipv6_cidr_details = value;
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

    /// Set is_ipv6_enabled
    pub fn set_is_ipv6_enabled(mut self, value: Option<bool>) -> Self {
        self.is_ipv6_enabled = value;
        self
    }

    /// Set is_zpr_only
    pub fn set_is_zpr_only(mut self, value: Option<bool>) -> Self {
        self.is_zpr_only = value;
        self
    }

    /// Set cidr_block (unwraps Option)
    pub fn with_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.cidr_block = Some(value.into());
        self
    }

    /// Set cidr_blocks (unwraps Option)
    pub fn with_cidr_blocks(mut self, value: Vec<String>) -> Self {
        self.cidr_blocks = Some(value);
        self
    }

    /// Set ipv6_private_cidr_blocks (unwraps Option)
    pub fn with_ipv6_private_cidr_blocks(mut self, value: Vec<String>) -> Self {
        self.ipv6_private_cidr_blocks = Some(value);
        self
    }

    /// Set is_oracle_gua_allocation_enabled (unwraps Option)
    pub fn with_is_oracle_gua_allocation_enabled(mut self, value: bool) -> Self {
        self.is_oracle_gua_allocation_enabled = Some(value);
        self
    }

    /// Set byoipv6_cidr_details (unwraps Option)
    pub fn with_byoipv6_cidr_details(mut self, value: Vec<Byoipv6CidrDetails>) -> Self {
        self.byoipv6_cidr_details = Some(value);
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

    /// Set is_ipv6_enabled (unwraps Option)
    pub fn with_is_ipv6_enabled(mut self, value: bool) -> Self {
        self.is_ipv6_enabled = Some(value);
        self
    }

    /// Set is_zpr_only (unwraps Option)
    pub fn with_is_zpr_only(mut self, value: bool) -> Self {
        self.is_zpr_only = Some(value);
        self
    }
}
