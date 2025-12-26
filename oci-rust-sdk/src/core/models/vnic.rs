use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A virtual network interface card. Each VNIC resides in a subnet in a VCN. An instance attaches to a VNIC to obtain a network connection into the VCN through that subnet. Each instance has a *primary VNIC* that is automatically created and attached during launch. You can add *secondary VNICs* to an instance after it's launched. For more information, see [Virtual Network Interface Cards (VNICs)](https://docs.oracle.com/iaas/Content/Network/Tasks/managingVNICs.htm). <p> Each VNIC has a *primary private IP* that is automatically assigned during launch. You can add *secondary private IPs* to a VNIC after it's created. For more information, see {@link #createPrivateIp(CreatePrivateIpRequest) createPrivateIp} and [IP Addresses](https://docs.oracle.com/iaas/Content/Network/Tasks/managingIPaddresses.htm). <p> If you are an Oracle Cloud VMware Solution customer, you will have secondary VNICs that reside in a VLAN instead of a subnet. These VNICs have other differences, which are called out in the descriptions of the relevant attributes in the {@code Vnic} object. Also see {@link Vlan}. <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vnic {
    /// The VNIC's availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the VNIC.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC.
    pub id: String,

    /// The current state of the VNIC.
    pub lifecycle_state: VnicLifecycleState,

    /// The date and time the VNIC was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// [Security attributes](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/zpr-artifacts.htm#security-attributes) are labels for a resource that can be referenced in a [Zero Trust Packet Routing](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/overview.htm) (ZPR) policy to control access to ZPR-supported resources. <p> Example: {@code {\"Oracle-DataSecurity-ZPR\": {\"MaxEgressCount\": {\"value\":\"42\",\"mode\":\"audit\"}}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_attributes: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The hostname for the VNIC's primary private IP. Used for DNS. The value is the hostname portion of the primary private IP's fully qualified domain name (FQDN) (for example, {@code bminstance1} in FQDN {@code bminstance1.subnet123.vcn1.oraclevcn.com}). Must be unique across all VNICs in the subnet and comply with [RFC 952](https://tools.ietf.org/html/rfc952) and [RFC 1123](https://tools.ietf.org/html/rfc1123). <p> For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm). <p> Example: {@code bminstance1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname_label: Option<String>,

    /// Whether the VNIC is the primary VNIC (the VNIC that is automatically created and attached during instance launch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_primary: Option<bool>,

    /// The MAC address of the VNIC. <p> If the VNIC belongs to a VLAN as part of the Oracle Cloud VMware Solution, the MAC address is learned. If the VNIC belongs to a subnet, the MAC address is a static, Oracle-provided value. <p> Example: {@code 00:00:00:00:00:01}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,

    /// A list of the OCIDs of the network security groups that the VNIC belongs to. <p> If the VNIC belongs to a VLAN as part of the Oracle Cloud VMware Solution (instead of belonging to a subnet), the value of the {@code nsgIds} attribute is ignored. Instead, the VNIC belongs to the NSGs that are associated with the VLAN itself. See {@link Vlan}. <p> For more information about NSGs, see {@link NetworkSecurityGroup}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsg_ids: Option<Vec<String>>,

    /// If the VNIC belongs to a VLAN as part of the Oracle Cloud VMware Solution (instead of belonging to a subnet), the {@code vlanId} is the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VLAN the VNIC is in. See {@link Vlan}. If the VNIC is instead in a subnet, {@code subnetId} has a value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_id: Option<String>,

    /// The private IP address of the primary {@code privateIp} object on the VNIC. The address is within the CIDR of the VNIC's subnet. <p> Example: {@code 10.0.3.3}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_ip: Option<String>,

    /// The public IP address of the VNIC, if one is assigned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ip: Option<String>,

    /// Whether the source/destination check is disabled on the VNIC. Defaults to {@code false}, which means the check is performed. For information about why you would skip the source/destination check, see [Using a Private IP as a Route Target](https://docs.oracle.com/iaas/Content/Network/Tasks/managingroutetables.htm#privateip). <p> If the VNIC belongs to a VLAN as part of the Oracle Cloud VMware Solution (instead of belonging to a subnet), the {@code skipSourceDestCheck} attribute is {@code true}. This is because the source/destination check is always disabled for VNICs in a VLAN. <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_source_dest_check: Option<bool>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet the VNIC is in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,

    /// List of IPv6 addresses assigned to the VNIC. <p> Example: {@code 2001:DB8::}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_addresses: Option<Vec<String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the IP address or VNIC will use. For more information, see [Per-resource Routing](https://docs.oracle.com/iaas/Content/Network/Tasks/managingroutetables.htm#Overview_of_Routing_for_Your_VCN__source_routing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,
}

/// Required fields for Vnic
pub struct VnicRequired {
    /// The VNIC's availability domain. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the VNIC.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC.
    pub id: String,

    /// The current state of the VNIC.
    pub lifecycle_state: VnicLifecycleState,

    /// The date and time the VNIC was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl Vnic {
    /// Create a new Vnic with required fields
    pub fn new(required: VnicRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            defined_tags: None,

            display_name: None,

            security_attributes: None,

            freeform_tags: None,

            hostname_label: None,

            is_primary: None,

            mac_address: None,

            nsg_ids: None,

            vlan_id: None,

            private_ip: None,

            public_ip: None,

            skip_source_dest_check: None,

            subnet_id: None,

            ipv6_addresses: None,

            route_table_id: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
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

    /// Set security_attributes
    pub fn set_security_attributes(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.security_attributes = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set hostname_label
    pub fn set_hostname_label(mut self, value: Option<String>) -> Self {
        self.hostname_label = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set is_primary
    pub fn set_is_primary(mut self, value: Option<bool>) -> Self {
        self.is_primary = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: VnicLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set mac_address
    pub fn set_mac_address(mut self, value: Option<String>) -> Self {
        self.mac_address = value;
        self
    }

    /// Set nsg_ids
    pub fn set_nsg_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.nsg_ids = value;
        self
    }

    /// Set vlan_id
    pub fn set_vlan_id(mut self, value: Option<String>) -> Self {
        self.vlan_id = value;
        self
    }

    /// Set private_ip
    pub fn set_private_ip(mut self, value: Option<String>) -> Self {
        self.private_ip = value;
        self
    }

    /// Set public_ip
    pub fn set_public_ip(mut self, value: Option<String>) -> Self {
        self.public_ip = value;
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

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set ipv6_addresses
    pub fn set_ipv6_addresses(mut self, value: Option<Vec<String>>) -> Self {
        self.ipv6_addresses = value;
        self
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
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

    /// Set security_attributes (unwraps Option)
    pub fn with_security_attributes(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.security_attributes = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set hostname_label (unwraps Option)
    pub fn with_hostname_label(mut self, value: impl Into<String>) -> Self {
        self.hostname_label = Some(value.into());
        self
    }

    /// Set is_primary (unwraps Option)
    pub fn with_is_primary(mut self, value: bool) -> Self {
        self.is_primary = Some(value);
        self
    }

    /// Set mac_address (unwraps Option)
    pub fn with_mac_address(mut self, value: impl Into<String>) -> Self {
        self.mac_address = Some(value.into());
        self
    }

    /// Set nsg_ids (unwraps Option)
    pub fn with_nsg_ids(mut self, value: Vec<String>) -> Self {
        self.nsg_ids = Some(value);
        self
    }

    /// Set vlan_id (unwraps Option)
    pub fn with_vlan_id(mut self, value: impl Into<String>) -> Self {
        self.vlan_id = Some(value.into());
        self
    }

    /// Set private_ip (unwraps Option)
    pub fn with_private_ip(mut self, value: impl Into<String>) -> Self {
        self.private_ip = Some(value.into());
        self
    }

    /// Set public_ip (unwraps Option)
    pub fn with_public_ip(mut self, value: impl Into<String>) -> Self {
        self.public_ip = Some(value.into());
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

    /// Set ipv6_addresses (unwraps Option)
    pub fn with_ipv6_addresses(mut self, value: Vec<String>) -> Self {
        self.ipv6_addresses = Some(value);
        self
    }

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }
}
