use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Contains the properties of the VNIC for an instance configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationCreateVnicDetails {
    /// Whether to allocate an IPv6 address at instance and VNIC creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_ipv6_ip: Option<bool>,

    /// Whether the VNIC should be assigned a public IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_public_ip: Option<bool>,

    /// Whether the VNIC should be assigned a private DNS record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_private_dns_record: Option<bool>,

    /// Defined tags for this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Security attributes for this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_attributes: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A list of IPv6 prefixes from which the VNIC should be assigned an IPv6 address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_address_ipv6_subnet_cidr_pair_details: Option<Vec<super::InstanceConfigurationIpv6AddressIpv6SubnetCidrPairDetails>>,

    /// The hostname for the VNIC's primary private IP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname_label: Option<String>,

    /// A list of the OCIDs of the network security groups (NSGs) to add the VNIC to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsg_ids: Option<Vec<String>>,

    /// One of the IPv4 CIDR blocks allocated to the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_cidr: Option<String>,

    /// A private IP address of your choice to assign to the VNIC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_ip: Option<String>,

    /// Whether the source/destination check is disabled on the VNIC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_source_dest_check: Option<bool>,

    /// The OCID of the subnet to create the VNIC in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,
}

impl InstanceConfigurationCreateVnicDetails {
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

    pub fn with_subnet_id(mut self, subnet_id: impl Into<String>) -> Self {
        self.subnet_id = Some(subnet_id.into());
        self
    }

    pub fn with_assign_public_ip(mut self, assign: bool) -> Self {
        self.assign_public_ip = Some(assign);
        self
    }

    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn with_hostname_label(mut self, label: impl Into<String>) -> Self {
        self.hostname_label = Some(label.into());
        self
    }

    pub fn with_private_ip(mut self, ip: impl Into<String>) -> Self {
        self.private_ip = Some(ip.into());
        self
    }

    pub fn with_nsg_ids(mut self, ids: Vec<String>) -> Self {
        self.nsg_ids = Some(ids);
        self
    }

    pub fn with_skip_source_dest_check(mut self, skip: bool) -> Self {
        self.skip_source_dest_check = Some(skip);
        self
    }
}

impl Default for InstanceConfigurationCreateVnicDetails {
    fn default() -> Self {
        Self::new()
    }
}
