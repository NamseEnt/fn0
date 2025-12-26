use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Contains properties for a VNIC. You use this object when creating the primary VNIC during instance launch or when creating a secondary VNIC. For more information about VNICs, see [Virtual Network Interface Cards (VNICs)](https://docs.oracle.com/iaas/Content/Network/Tasks/managingVNICs.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVnicDetails {
    /// Whether to allocate an IPv6 address at instance and VNIC creation from an IPv6 enabled subnet. Default: False. When provided you may optionally provide an IPv6 prefix ({@code ipv6SubnetCidr}) of your choice to assign the IPv6 address from. If {@code ipv6SubnetCidr} is not provided then an IPv6 prefix is chosen for you.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_ipv6_ip: Option<bool>,

    /// Whether the VNIC should be assigned a public IP address. Defaults to whether the subnet is public or private. If not set and the VNIC is being created in a private subnet (that is, where {@code prohibitPublicIpOnVnic} = true in the {@link Subnet}), then no public IP address is assigned. If not set and the subnet is public ({@code prohibitPublicIpOnVnic} = false), then a public IP address is assigned. If set to true and {@code prohibitPublicIpOnVnic} = true, an error is returned. <p> *Note:** This public IP address is associated with the primary private IP on the VNIC. For more information, see [IP Addresses](https://docs.oracle.com/iaas/Content/Network/Tasks/managingIPaddresses.htm). <p> *Note:** There's a limit to the number of {@link PublicIp} a VNIC or instance can have. If you try to create a secondary VNIC with an assigned public IP for an instance that has already reached its public IP limit, an error is returned. For information about the public IP limits, see [Public IP Addresses](https://docs.oracle.com/iaas/Content/Network/Tasks/managingpublicIPs.htm). <p> Example: {@code false} <p> If you specify a {@code vlanId}, then {@code assignPublicIp} must be set to false. See {@link Vlan}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_public_ip: Option<bool>,

    /// Whether the VNIC should be assigned a DNS record. If set to false, there will be no DNS record registration for the VNIC. If set to true, the DNS record will be registered. The default value is true. <p> If you specify a {@code hostnameLabel}, then {@code assignPrivateDnsRecord} must be set to true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_private_dns_record: Option<bool>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// [Security attributes](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/zpr-artifacts.htm#security-attributes) are labels for a resource that can be referenced in a [Zero Trust Packet Routing](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/overview.htm) (ZPR) policy to control access to ZPR-supported resources. <p> Example: {@code {\"Oracle-DataSecurity-ZPR\": {\"MaxEgressCount\": {\"value\":\"42\",\"mode\":\"audit\"}}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_attributes: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The hostname for the VNIC's primary private IP. Used for DNS. The value is the hostname portion of the primary private IP's fully qualified domain name (FQDN) (for example, {@code bminstance1} in FQDN {@code bminstance1.subnet123.vcn1.oraclevcn.com}). Must be unique across all VNICs in the subnet and comply with [RFC 952](https://tools.ietf.org/html/rfc952) and [RFC 1123](https://tools.ietf.org/html/rfc1123). The value appears in the {@link Vnic} object and also the {@link PrivateIp} object returned by {@link #listPrivateIps(ListPrivateIpsRequest) listPrivateIps} and {@link #getPrivateIp(GetPrivateIpRequest) getPrivateIp}. <p> For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm). <p> When launching an instance, use this {@code hostnameLabel} instead of the deprecated {@code hostnameLabel} in {@link #launchInstanceDetails(LaunchInstanceDetailsRequest) launchInstanceDetails}. If you provide both, the values must match. <p> Example: {@code bminstance1} <p> If you specify a {@code vlanId}, the {@code hostnameLabel} cannot be specified. VNICs on a VLAN can not be assigned a hostname. See {@link Vlan}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname_label: Option<String>,

    /// A list of IPv6 prefix ranges from which the VNIC is assigned an IPv6 address. You can provide only the prefix ranges from which OCI selects an available address from the range. You can optionally choose to leave the prefix range empty and instead provide the specific IPv6 address within that range to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_address_ipv6_subnet_cidr_pair_details:
        Option<Vec<Ipv6AddressIpv6SubnetCidrPairDetails>>,

    /// One of the IPv4 CIDR blocks allocated to the subnet. Represents the IP range from which the VNIC's private IP address will be assigned if {@code privateIp} or {@code privateIpId} is not specified. Either this field or the {@code privateIp} (or {@code privateIpId}, if applicable) field must be provided, but not both simultaneously. Example: {@code 192.168.1.0/28}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_cidr: Option<String>,

    /// A list of the OCIDs of the network security groups (NSGs) to add the VNIC to. For more information about NSGs, see {@link NetworkSecurityGroup}. <p> If a {@code vlanId} is specified, the {@code nsgIds} cannot be specified. The {@code vlanId} indicates that the VNIC will belong to a VLAN instead of a subnet. With VLANs, all VNICs in the VLAN belong to the NSGs that are associated with the VLAN. See {@link Vlan}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsg_ids: Option<Vec<String>>,

    /// A private IP address of your choice to assign to the VNIC. Must be an available IP address within the subnet's CIDR. If you don't specify a value, Oracle automatically assigns a private IP address from the subnet. This is the VNIC's *primary* private IP address. The value appears in the {@link Vnic} object and also the {@link PrivateIp} object returned by {@link #listPrivateIps(ListPrivateIpsRequest) listPrivateIps} and {@link #getPrivateIp(GetPrivateIpRequest) getPrivateIp}. <p> If you specify a {@code vlanId}, the {@code privateIp} cannot be specified. See {@link Vlan}. <p> Example: {@code 10.0.3.3}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_ip: Option<String>,

    /// Whether the source/destination check is disabled on the VNIC. Defaults to {@code false}, which means the check is performed. For information about why you would skip the source/destination check, see [Using a Private IP as a Route Target](https://docs.oracle.com/iaas/Content/Network/Tasks/managingroutetables.htm#privateip). <p> If you specify a {@code vlanId}, the {@code skipSourceDestCheck} cannot be specified because the source/destination check is always disabled for VNICs in a VLAN. See {@link Vlan}. <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_source_dest_check: Option<bool>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet to create the VNIC in. When launching an instance, use this {@code subnetId} instead of the deprecated {@code subnetId} in {@link #launchInstanceDetails(LaunchInstanceDetailsRequest) launchInstanceDetails}. At least one of them is required; if you provide both, the values must match. <p> If you are an Oracle Cloud VMware Solution customer and creating a secondary VNIC in a VLAN instead of a subnet, provide a {@code vlanId} instead of a {@code subnetId}. If you provide both a {@code vlanId} and {@code subnetId}, the request fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,

    /// Provide this attribute only if you are an Oracle Cloud VMware Solution customer and creating a secondary VNIC in a VLAN. The value is the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VLAN. See {@link Vlan}. <p> Provide a {@code vlanId} instead of a {@code subnetId}. If you provide both a {@code vlanId} and {@code subnetId}, the request fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_id: Option<String>,
}

impl CreateVnicDetails {
    /// Create a new CreateVnicDetails
    pub fn new() -> Self {
        Self {
            assign_ipv6_ip: None,

            assign_public_ip: None,

            assign_private_dns_record: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            security_attributes: None,

            hostname_label: None,

            ipv6_address_ipv6_subnet_cidr_pair_details: None,

            subnet_cidr: None,

            nsg_ids: None,

            private_ip: None,

            skip_source_dest_check: None,

            subnet_id: None,

            vlan_id: None,
        }
    }

    /// Set assign_ipv6_ip
    pub fn set_assign_ipv6_ip(mut self, value: Option<bool>) -> Self {
        self.assign_ipv6_ip = value;
        self
    }

    /// Set assign_public_ip
    pub fn set_assign_public_ip(mut self, value: Option<bool>) -> Self {
        self.assign_public_ip = value;
        self
    }

    /// Set assign_private_dns_record
    pub fn set_assign_private_dns_record(mut self, value: Option<bool>) -> Self {
        self.assign_private_dns_record = value;
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

    /// Set security_attributes
    pub fn set_security_attributes(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.security_attributes = value;
        self
    }

    /// Set hostname_label
    pub fn set_hostname_label(mut self, value: Option<String>) -> Self {
        self.hostname_label = value;
        self
    }

    /// Set ipv6_address_ipv6_subnet_cidr_pair_details
    pub fn set_ipv6_address_ipv6_subnet_cidr_pair_details(
        mut self,
        value: Option<Vec<Ipv6AddressIpv6SubnetCidrPairDetails>>,
    ) -> Self {
        self.ipv6_address_ipv6_subnet_cidr_pair_details = value;
        self
    }

    /// Set subnet_cidr
    pub fn set_subnet_cidr(mut self, value: Option<String>) -> Self {
        self.subnet_cidr = value;
        self
    }

    /// Set nsg_ids
    pub fn set_nsg_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.nsg_ids = value;
        self
    }

    /// Set private_ip
    pub fn set_private_ip(mut self, value: Option<String>) -> Self {
        self.private_ip = value;
        self
    }

    /// Set skip_source_dest_check
    pub fn set_skip_source_dest_check(mut self, value: Option<bool>) -> Self {
        self.skip_source_dest_check = value;
        self
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: Option<String>) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set vlan_id
    pub fn set_vlan_id(mut self, value: Option<String>) -> Self {
        self.vlan_id = value;
        self
    }

    /// Set assign_ipv6_ip (unwraps Option)
    pub fn with_assign_ipv6_ip(mut self, value: bool) -> Self {
        self.assign_ipv6_ip = Some(value);
        self
    }

    /// Set assign_public_ip (unwraps Option)
    pub fn with_assign_public_ip(mut self, value: bool) -> Self {
        self.assign_public_ip = Some(value);
        self
    }

    /// Set assign_private_dns_record (unwraps Option)
    pub fn with_assign_private_dns_record(mut self, value: bool) -> Self {
        self.assign_private_dns_record = Some(value);
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

    /// Set security_attributes (unwraps Option)
    pub fn with_security_attributes(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.security_attributes = Some(value);
        self
    }

    /// Set hostname_label (unwraps Option)
    pub fn with_hostname_label(mut self, value: impl Into<String>) -> Self {
        self.hostname_label = Some(value.into());
        self
    }

    /// Set ipv6_address_ipv6_subnet_cidr_pair_details (unwraps Option)
    pub fn with_ipv6_address_ipv6_subnet_cidr_pair_details(
        mut self,
        value: Vec<Ipv6AddressIpv6SubnetCidrPairDetails>,
    ) -> Self {
        self.ipv6_address_ipv6_subnet_cidr_pair_details = Some(value);
        self
    }

    /// Set subnet_cidr (unwraps Option)
    pub fn with_subnet_cidr(mut self, value: impl Into<String>) -> Self {
        self.subnet_cidr = Some(value.into());
        self
    }

    /// Set nsg_ids (unwraps Option)
    pub fn with_nsg_ids(mut self, value: Vec<String>) -> Self {
        self.nsg_ids = Some(value);
        self
    }

    /// Set private_ip (unwraps Option)
    pub fn with_private_ip(mut self, value: impl Into<String>) -> Self {
        self.private_ip = Some(value.into());
        self
    }

    /// Set skip_source_dest_check (unwraps Option)
    pub fn with_skip_source_dest_check(mut self, value: bool) -> Self {
        self.skip_source_dest_check = Some(value);
        self
    }

    /// Set subnet_id (unwraps Option)
    pub fn with_subnet_id(mut self, value: impl Into<String>) -> Self {
        self.subnet_id = Some(value.into());
        self
    }

    /// Set vlan_id (unwraps Option)
    pub fn with_vlan_id(mut self, value: impl Into<String>) -> Self {
        self.vlan_id = Some(value.into());
        self
    }
}

impl Default for CreateVnicDetails {
    fn default() -> Self {
        Self::new()
    }
}
