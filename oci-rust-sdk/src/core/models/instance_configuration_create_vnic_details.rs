use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Contains the properties of the VNIC for an instance configuration. See {@link CreateVnicDetails} and [Instance Configurations](https://docs.oracle.com/iaas/Content/Compute/Concepts/instancemanagement.htm#config) for more information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationCreateVnicDetails {
    /// Whether to allocate an IPv6 address at instance and VNIC creation from an IPv6 enabled subnet. Default: False. When provided you may optionally provide an IPv6 prefix ({@code ipv6SubnetCidr}) of your choice to assign the IPv6 address from. If {@code ipv6SubnetCidr} is not provided then an IPv6 prefix is chosen for you.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_ipv6_ip: Option<bool>,

    /// Whether the VNIC should be assigned a public IP address. See the {@code assignPublicIp} attribute of {@link CreateVnicDetails} for more information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_public_ip: Option<bool>,

    /// Whether the VNIC should be assigned a private DNS record. See the {@code assignPrivateDnsRecord} attribute of {@link CreateVnicDetails} for more information.
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

    /// A list of IPv6 prefixes from which the VNIC should be assigned an IPv6 address. You can provide only the prefix and OCI selects an available address from the range. You can optionally choose to leave the prefix range empty and instead provide the specific IPv6 address that should be used from within that range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_address_ipv6_subnet_cidr_pair_details:
        Option<Vec<InstanceConfigurationIpv6AddressIpv6SubnetCidrPairDetails>>,

    /// The hostname for the VNIC's primary private IP. See the {@code hostnameLabel} attribute of {@link CreateVnicDetails} for more information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname_label: Option<String>,

    /// A list of the OCIDs of the network security groups (NSGs) to add the VNIC to. For more information about NSGs, see {@link NetworkSecurityGroup}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsg_ids: Option<Vec<String>>,

    /// One of the IPv4 CIDR blocks allocated to the subnet. Represents the IP range from which the VNIC's private IP address will be assigned if {@code privateIp} or {@code privateIpId} is not specified. Either this field or the {@code privateIp} (or {@code privateIpId}, if applicable) field must be provided, but not both simultaneously. Example: {@code 192.168.1.0/28} See the {@code subnetCidr} attribute of {@link CreateVnicDetails} for more information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_cidr: Option<String>,

    /// A private IP address of your choice to assign to the VNIC. See the {@code privateIp} attribute of {@link CreateVnicDetails} for more information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_ip: Option<String>,

    /// Whether the source/destination check is disabled on the VNIC. See the {@code skipSourceDestCheck} attribute of {@link CreateVnicDetails} for more information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_source_dest_check: Option<bool>,

    /// The OCID of the subnet to create the VNIC in. See the {@code subnetId} attribute of {@link CreateVnicDetails} for more information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,
}

impl InstanceConfigurationCreateVnicDetails {
    /// Create a new InstanceConfigurationCreateVnicDetails
    pub fn new() -> Self {
        Self {
            assign_ipv6_ip: None,

            assign_public_ip: None,

            assign_private_dns_record: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            security_attributes: None,

            ipv6_address_ipv6_subnet_cidr_pair_details: None,

            hostname_label: None,

            nsg_ids: None,

            subnet_cidr: None,

            private_ip: None,

            skip_source_dest_check: None,

            subnet_id: None,
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

    /// Set ipv6_address_ipv6_subnet_cidr_pair_details
    pub fn set_ipv6_address_ipv6_subnet_cidr_pair_details(
        mut self,
        value: Option<Vec<InstanceConfigurationIpv6AddressIpv6SubnetCidrPairDetails>>,
    ) -> Self {
        self.ipv6_address_ipv6_subnet_cidr_pair_details = value;
        self
    }

    /// Set hostname_label
    pub fn set_hostname_label(mut self, value: Option<String>) -> Self {
        self.hostname_label = value;
        self
    }

    /// Set nsg_ids
    pub fn set_nsg_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.nsg_ids = value;
        self
    }

    /// Set subnet_cidr
    pub fn set_subnet_cidr(mut self, value: Option<String>) -> Self {
        self.subnet_cidr = value;
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

    /// Set ipv6_address_ipv6_subnet_cidr_pair_details (unwraps Option)
    pub fn with_ipv6_address_ipv6_subnet_cidr_pair_details(
        mut self,
        value: Vec<InstanceConfigurationIpv6AddressIpv6SubnetCidrPairDetails>,
    ) -> Self {
        self.ipv6_address_ipv6_subnet_cidr_pair_details = Some(value);
        self
    }

    /// Set hostname_label (unwraps Option)
    pub fn with_hostname_label(mut self, value: impl Into<String>) -> Self {
        self.hostname_label = Some(value.into());
        self
    }

    /// Set nsg_ids (unwraps Option)
    pub fn with_nsg_ids(mut self, value: Vec<String>) -> Self {
        self.nsg_ids = Some(value);
        self
    }

    /// Set subnet_cidr (unwraps Option)
    pub fn with_subnet_cidr(mut self, value: impl Into<String>) -> Self {
        self.subnet_cidr = Some(value.into());
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
}

impl Default for InstanceConfigurationCreateVnicDetails {
    fn default() -> Self {
        Self::new()
    }
}
