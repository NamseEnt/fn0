use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A logical subdivision of a VCN. Each subnet consists of a contiguous range of IP addresses that do not overlap with other subnets in the VCN. Example: 172.16.1.0/24. For more information, see [Overview of the Networking Service](https://docs.oracle.com/iaas/Content/Network/Concepts/overview.htm) and [VCNs and Subnets](https://docs.oracle.com/iaas/Content/Network/Tasks/managingVCNs.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subnet {
    /// The subnet's CIDR block. <p> Example: {@code 10.0.1.0/24}
    pub cidr_block: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the subnet.
    pub compartment_id: String,

    /// The subnet's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The subnet's current state.
    pub lifecycle_state: SubnetLifecycleState,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table that the subnet uses.
    pub route_table_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the subnet is in.
    pub vcn_id: String,

    /// The IP address of the virtual router. <p> Example: {@code 10.0.14.1}
    pub virtual_router_ip: String,

    /// The MAC address of the virtual router. <p> Example: {@code 00:00:00:00:00:01}
    pub virtual_router_mac: String,

    /// The subnet's availability domain. This attribute will be null if this is a regional subnet instead of an AD-specific subnet. Oracle recommends creating regional subnets. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// The list of all IPv4 CIDR blocks for the subnet that meets the following criteria: - Ipv4 CIDR blocks must be valid. - Multiple Ipv4 CIDR blocks must not overlap each other or the on-premises network CIDR block. - The number of prefixes must not exceed the limit of IPv4 prefixes allowed to a subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_cidr_blocks: Option<Vec<String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the set of DHCP options that the subnet uses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhcp_options_id: Option<String>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// A DNS label for the subnet, used in conjunction with the VNIC's hostname and VCN's DNS label to form a fully qualified domain name (FQDN) for each VNIC within this subnet (for example, {@code bminstance1.subnet123.vcn1.oraclevcn.com}). Must be an alphanumeric string that begins with a letter and is unique within the VCN. The value cannot be changed. <p> The absence of this parameter means the Internet and VCN Resolver will not resolve hostnames of instances in this subnet. <p> For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm). <p> Example: {@code subnet123}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_label: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// For an IPv6-enabled subnet, this is the IPv6 prefix for the subnet's IP address space. The subnet size is always /64. See [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> Example: {@code 2001:0db8:0123:1111::/64}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_block: Option<String>,

    /// The list of all IPv6 prefixes (Oracle allocated IPv6 GUA, ULA or private IPv6 prefixes, BYOIPv6 prefixes) for the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_blocks: Option<Vec<String>>,

    /// For an IPv6-enabled subnet, this is the IPv6 address of the virtual router. <p> Example: {@code 2001:0db8:0123:1111:89ab:cdef:1234:5678}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_virtual_router_ip: Option<String>,

    /// Whether to disallow ingress internet traffic to VNICs within this subnet. Defaults to false. <p> For IPV4, {@code prohibitInternetIngress} behaves similarly to {@code prohibitPublicIpOnVnic}. If it is set to false, VNICs created in this subnet will automatically be assigned public IP addresses unless specified otherwise during instance launch or VNIC creation (with the {@code assignPublicIp} flag in {@link CreateVnicDetails}). If {@code prohibitInternetIngress} is set to true, VNICs created in this subnet cannot have public IP addresses (that is, it's a privatesubnet). <p> For IPv6, if {@code prohibitInternetIngress} is set to {@code true}, internet access is not allowed for any IPv6s assigned to VNICs in the subnet. Otherwise, ingress internet traffic is allowed by default. <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prohibit_internet_ingress: Option<bool>,

    /// Whether VNICs within this subnet can have public IP addresses. Defaults to false, which means VNICs created in this subnet will automatically be assigned public IP addresses unless specified otherwise during instance launch or VNIC creation (with the {@code assignPublicIp} flag in {@link CreateVnicDetails}). If {@code prohibitPublicIpOnVnic} is set to true, VNICs created in this subnet cannot have public IP addresses (that is, it's a private subnet). <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prohibit_public_ip_on_vnic: Option<bool>,

    /// The OCIDs of the security list or lists that the subnet uses. Remember that security lists are associated *with the subnet*, but the rules are applied to the individual VNICs in the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_list_ids: Option<Vec<String>>,

    /// The subnet's domain name, which consists of the subnet's DNS label, the VCN's DNS label, and the {@code oraclevcn.com} domain. <p> For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm). <p> Example: {@code subnet123.vcn1.oraclevcn.com}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_domain_name: Option<String>,

    /// The date and time the subnet was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,
}

/// Required fields for Subnet
pub struct SubnetRequired {
    /// The subnet's CIDR block. <p> Example: {@code 10.0.1.0/24}
    pub cidr_block: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the subnet.
    pub compartment_id: String,

    /// The subnet's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// The subnet's current state.
    pub lifecycle_state: SubnetLifecycleState,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table that the subnet uses.
    pub route_table_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the subnet is in.
    pub vcn_id: String,

    /// The IP address of the virtual router. <p> Example: {@code 10.0.14.1}
    pub virtual_router_ip: String,

    /// The MAC address of the virtual router. <p> Example: {@code 00:00:00:00:00:01}
    pub virtual_router_mac: String,
}

impl Subnet {
    /// Create a new Subnet with required fields
    pub fn new(required: SubnetRequired) -> Self {
        Self {
            cidr_block: required.cidr_block,

            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            route_table_id: required.route_table_id,

            vcn_id: required.vcn_id,

            virtual_router_ip: required.virtual_router_ip,

            virtual_router_mac: required.virtual_router_mac,

            availability_domain: None,

            ipv4_cidr_blocks: None,

            defined_tags: None,

            dhcp_options_id: None,

            display_name: None,

            dns_label: None,

            freeform_tags: None,

            ipv6_cidr_block: None,

            ipv6_cidr_blocks: None,

            ipv6_virtual_router_ip: None,

            prohibit_internet_ingress: None,

            prohibit_public_ip_on_vnic: None,

            security_list_ids: None,

            subnet_domain_name: None,

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

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
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

    /// Set ipv6_virtual_router_ip
    pub fn set_ipv6_virtual_router_ip(mut self, value: Option<String>) -> Self {
        self.ipv6_virtual_router_ip = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: SubnetLifecycleState) -> Self {
        self.lifecycle_state = value;
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
    pub fn set_route_table_id(mut self, value: String) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set security_list_ids
    pub fn set_security_list_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.security_list_ids = value;
        self
    }

    /// Set subnet_domain_name
    pub fn set_subnet_domain_name(mut self, value: Option<String>) -> Self {
        self.subnet_domain_name = value;
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

    /// Set virtual_router_ip
    pub fn set_virtual_router_ip(mut self, value: String) -> Self {
        self.virtual_router_ip = value;
        self
    }

    /// Set virtual_router_mac
    pub fn set_virtual_router_mac(mut self, value: String) -> Self {
        self.virtual_router_mac = value;
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
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

    /// Set ipv6_virtual_router_ip (unwraps Option)
    pub fn with_ipv6_virtual_router_ip(mut self, value: impl Into<String>) -> Self {
        self.ipv6_virtual_router_ip = Some(value.into());
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

    /// Set security_list_ids (unwraps Option)
    pub fn with_security_list_ids(mut self, value: Vec<String>) -> Self {
        self.security_list_ids = Some(value);
        self
    }

    /// Set subnet_domain_name (unwraps Option)
    pub fn with_subnet_domain_name(mut self, value: impl Into<String>) -> Self {
        self.subnet_domain_name = Some(value.into());
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }
}
