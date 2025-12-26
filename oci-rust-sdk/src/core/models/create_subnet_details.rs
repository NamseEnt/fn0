use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubnetDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the subnet.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN to contain the subnet.
    pub vcn_id: String,

    /// Controls whether the subnet is regional or specific to an availability domain. Oracle recommends creating regional subnets because they're more flexible and make it easier to implement failover across availability domains. Originally, AD-specific subnets were the only kind available to use. <p> To create a regional subnet, omit this attribute. Then any resources later created in this subnet (such as a Compute instance) can be created in any availability domain in the region. <p> To instead create an AD-specific subnet, set this attribute to the availability domain you want this subnet to be in. Then any resources later created in this subnet can only be created in that availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// The CIDR IP address range of the subnet. The CIDR must maintain the following rules - <p> a. The CIDR block is valid and correctly formatted. b. The new range is within one of the parent VCN ranges. <p> Example: {@code 10.0.1.0/24}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_block: Option<String>,

    /// The list of all IPv4 CIDR blocks for the subnet that meets the following criteria: - Ipv4 CIDR blocks must be valid. - Multiple Ipv4 CIDR blocks must not overlap each other or the on-premises network CIDR block. - The number of prefixes must not exceed the limit of IPv4 prefixes allowed to a subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_cidr_blocks: Option<Vec<String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the set of DHCP options the subnet will use. If you don't provide a value, the subnet uses the VCN's default set of DHCP options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhcp_options_id: Option<String>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// A DNS label for the subnet, used in conjunction with the VNIC's hostname and VCN's DNS label to form a fully qualified domain name (FQDN) for each VNIC within this subnet (for example, {@code bminstance1.subnet123.vcn1.oraclevcn.com}). Must be an alphanumeric string that begins with a letter and is unique within the VCN. The value cannot be changed. <p> This value must be set if you want to use the Internet and VCN Resolver to resolve the hostnames of instances in the subnet. It can only be set if the VCN itself was created with a DNS label. <p> For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm). <p> Example: {@code subnet123}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_label: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Use this to enable IPv6 addressing for this subnet. The VCN must be enabled for IPv6. You can't change this subnet characteristic later. All subnets are /64 in size. The subnet portion of the IPv6 address is the fourth hextet from the left (1111 in the following example). <p> For important details about IPv6 addressing in a VCN, see [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 2001:0db8:0123:1111::/64}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_block: Option<String>,

    /// The list of all IPv6 prefixes (Oracle allocated IPv6 GUA, ULA or private IPv6 prefixes, BYOIPv6 prefixes) for the subnet that meets the following criteria: - The prefixes must be valid. - Multiple prefixes must not overlap each other or the on-premises network prefix. - The number of prefixes must not exceed the limit of IPv6 prefixes allowed to a subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_blocks: Option<Vec<String>>,

    /// Whether to disallow ingress internet traffic to VNICs within this subnet. Defaults to false. <p> For IPv6, if {@code prohibitInternetIngress} is set to {@code true}, internet access is not allowed for any IPv6s assigned to VNICs in the subnet. Otherwise, ingress internet traffic is allowed by default. <p> {@code prohibitPublicIpOnVnic} will be set to the value of {@code prohibitInternetIngress} to dictate IPv4 behavior in this subnet. Only one or the other flag should be specified. <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prohibit_internet_ingress: Option<bool>,

    /// Whether VNICs within this subnet can have public IP addresses. Defaults to false, which means VNICs created in this subnet will automatically be assigned public IP addresses unless specified otherwise during instance launch or VNIC creation (with the {@code assignPublicIp} flag in {@link CreateVnicDetails}). If {@code prohibitPublicIpOnVnic} is set to true, VNICs created in this subnet cannot have public IP addresses (that is, it's a private subnet). <p> If you intend to use an IPv6 prefix, you should use the flag {@code prohibitInternetIngress} to specify ingress internet traffic behavior of the subnet. <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prohibit_public_ip_on_vnic: Option<bool>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the subnet will use. If you don't provide a value, the subnet uses the VCN's default route table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,

    /// The OCIDs of the security list or lists the subnet will use. If you don't provide a value, the subnet uses the VCN's default security list. Remember that security lists are associated *with the subnet*, but the rules are applied to the individual VNICs in the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_list_ids: Option<Vec<String>>,
}

/// Required fields for CreateSubnetDetails
pub struct CreateSubnetDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the subnet.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN to contain the subnet.
    pub vcn_id: String,
}

impl CreateSubnetDetails {
    /// Create a new CreateSubnetDetails with required fields
    pub fn new(required: CreateSubnetDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            vcn_id: required.vcn_id,

            availability_domain: None,

            cidr_block: None,

            ipv4_cidr_blocks: None,

            defined_tags: None,

            dhcp_options_id: None,

            display_name: None,

            dns_label: None,

            freeform_tags: None,

            ipv6_cidr_block: None,

            ipv6_cidr_blocks: None,

            prohibit_internet_ingress: None,

            prohibit_public_ip_on_vnic: None,

            route_table_id: None,

            security_list_ids: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: Option<String>) -> Self {
        self.cidr_block = value;
        self
    }

    /// Set ipv4_cidr_blocks
    pub fn set_ipv4_cidr_blocks(mut self, value: Option<Vec<String>>) -> Self {
        self.ipv4_cidr_blocks = value;
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

    /// Set dhcp_options_id
    pub fn set_dhcp_options_id(mut self, value: Option<String>) -> Self {
        self.dhcp_options_id = value;
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

    /// Set ipv6_cidr_block
    pub fn set_ipv6_cidr_block(mut self, value: Option<String>) -> Self {
        self.ipv6_cidr_block = value;
        self
    }

    /// Set ipv6_cidr_blocks
    pub fn set_ipv6_cidr_blocks(mut self, value: Option<Vec<String>>) -> Self {
        self.ipv6_cidr_blocks = value;
        self
    }

    /// Set prohibit_internet_ingress
    pub fn set_prohibit_internet_ingress(mut self, value: Option<bool>) -> Self {
        self.prohibit_internet_ingress = value;
        self
    }

    /// Set prohibit_public_ip_on_vnic
    pub fn set_prohibit_public_ip_on_vnic(mut self, value: Option<bool>) -> Self {
        self.prohibit_public_ip_on_vnic = value;
        self
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set security_list_ids
    pub fn set_security_list_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.security_list_ids = value;
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

    /// Set cidr_block (unwraps Option)
    pub fn with_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.cidr_block = Some(value.into());
        self
    }

    /// Set ipv4_cidr_blocks (unwraps Option)
    pub fn with_ipv4_cidr_blocks(mut self, value: Vec<String>) -> Self {
        self.ipv4_cidr_blocks = Some(value);
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

    /// Set dhcp_options_id (unwraps Option)
    pub fn with_dhcp_options_id(mut self, value: impl Into<String>) -> Self {
        self.dhcp_options_id = Some(value.into());
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

    /// Set ipv6_cidr_block (unwraps Option)
    pub fn with_ipv6_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.ipv6_cidr_block = Some(value.into());
        self
    }

    /// Set ipv6_cidr_blocks (unwraps Option)
    pub fn with_ipv6_cidr_blocks(mut self, value: Vec<String>) -> Self {
        self.ipv6_cidr_blocks = Some(value);
        self
    }

    /// Set prohibit_internet_ingress (unwraps Option)
    pub fn with_prohibit_internet_ingress(mut self, value: bool) -> Self {
        self.prohibit_internet_ingress = Some(value);
        self
    }

    /// Set prohibit_public_ip_on_vnic (unwraps Option)
    pub fn with_prohibit_public_ip_on_vnic(mut self, value: bool) -> Self {
        self.prohibit_public_ip_on_vnic = Some(value);
        self
    }

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }

    /// Set security_list_ids (unwraps Option)
    pub fn with_security_list_ids(mut self, value: Vec<String>) -> Self {
        self.security_list_ids = Some(value);
        self
    }
}
