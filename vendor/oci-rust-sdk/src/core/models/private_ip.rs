use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A *private IP* is a conceptual term that refers to an IPv4 private IP address and related properties. The {@code privateIp} object is the API representation of a private IP. <p> *Note:** For information about IPv6 addresses, see {@link Ipv6}. <p> Each instance has a *primary private IP* that is automatically created and assigned to the primary VNIC during instance launch. If you add a secondary VNIC to the instance, it also automatically gets a primary private IP. You can't remove a primary private IP from its VNIC. The primary private IP is automatically deleted when the VNIC is terminated. <p> You can add *secondary private IPs* to a VNIC after it's created. For more information, see the {@code privateIp} operations and also [IP Addresses](https://docs.oracle.com/iaas/Content/Network/Tasks/managingIPaddresses.htm). <p> *Note:** Only {@link #listPrivateIps(ListPrivateIpsRequest) listPrivateIps} and {@link #getPrivateIp(GetPrivateIpRequest) getPrivateIp} work with *primary* private IPs. To create and update primary private IPs, you instead work with instance and VNIC operations. For example, a primary private IP's properties come from the values you specify in {@link CreateVnicDetails} when calling either {@link #launchInstance(LaunchInstanceRequest) launchInstance} or {@link #attachVnic(AttachVnicRequest) attachVnic}. To update the hostname for a primary private IP, you use {@link #updateVnic(UpdateVnicRequest) updateVnic}. <p> {@code PrivateIp} objects that are created for use with the Oracle Cloud VMware Solution are assigned to a VLAN and not a VNIC in a subnet. See the descriptions of the relevant attributes in the {@code PrivateIp} object. Also see {@link Vlan}. <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateIp {
    /// The private IP's availability domain. This attribute will be null if this is a *secondary* private IP assigned to a VNIC that is in a *regional* subnet. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the private IP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The hostname for the private IP. Used for DNS. The value is the hostname portion of the private IP's fully qualified domain name (FQDN) (for example, {@code bminstance1} in FQDN {@code bminstance1.subnet123.vcn1.oraclevcn.com}). Must be unique across all VNICs in the subnet and comply with [RFC 952](https://tools.ietf.org/html/rfc952) and [RFC 1123](https://tools.ietf.org/html/rfc1123). <p> For more information, see [DNS in Your Virtual Cloud Network](https://docs.oracle.com/iaas/Content/Network/Concepts/dns.htm). <p> Example: {@code bminstance1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname_label: Option<String>,

    /// The private IP's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The private IP address of the {@code privateIp} object. The address is within the CIDR of the VNIC's subnet. <p> However, if the {@code PrivateIp} object is being used with a VLAN as part of the Oracle Cloud VMware Solution, the address is from the range specified by the {@code cidrBlock} attribute for the VLAN. See {@link Vlan}. <p> Example: {@code 10.0.3.3}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// The secondary IPv4 CIDR prefix length. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_prefix_length: Option<i64>,

    /// Whether this private IP is the primary one on the VNIC. Primary private IPs are unassigned and deleted automatically when the VNIC is terminated. <p> Example: {@code true}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_primary: Option<bool>,

    /// Applicable only if the {@code PrivateIp} object is being used with a VLAN as part of the Oracle Cloud VMware Solution. The {@code vlanId} is the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VLAN. See {@link Vlan}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet the VNIC is in. <p> However, if the {@code PrivateIp} object is being used with a VLAN as part of the Oracle Cloud VMware Solution, the {@code subnetId} is null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,

    /// The date and time the private IP was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<DateTime<Utc>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC the private IP is assigned to. The VNIC and private IP must be in the same subnet. However, if the {@code PrivateIp} object is being used with a VLAN as part of the Oracle Cloud VMware Solution, the {@code vnicId} is null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnic_id: Option<String>,

    /// State of the IP address. If an IP address is assigned to a VNIC it is ASSIGNED, otherwise it is AVAILABLE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_state: Option<PrivateIpIpState>,

    /// Lifetime of the IP address. There are two types of IPs: - Ephemeral - Reserved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime: Option<PrivateIpLifetime>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the IP address or VNIC will use. For more information, see [Per-resource Routing](https://docs.oracle.com/iaas/Content/Network/Tasks/managingroutetables.htm#Overview_of_Routing_for_Your_VCN__source_routing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,

    /// Ipv4 Subnet CIDR specified whn creating the PrivateIP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_subnet_cidr_at_creation: Option<String>,
}

impl PrivateIp {
    /// Create a new PrivateIp
    pub fn new() -> Self {
        Self {
            availability_domain: None,

            compartment_id: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            hostname_label: None,

            id: None,

            ip_address: None,

            cidr_prefix_length: None,

            is_primary: None,

            vlan_id: None,

            subnet_id: None,

            time_created: None,

            vnic_id: None,

            ip_state: None,

            lifetime: None,

            route_table_id: None,

            ipv4_subnet_cidr_at_creation: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
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

    /// Set hostname_label
    pub fn set_hostname_label(mut self, value: Option<String>) -> Self {
        self.hostname_label = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
        self.id = value;
        self
    }

    /// Set ip_address
    pub fn set_ip_address(mut self, value: Option<String>) -> Self {
        self.ip_address = value;
        self
    }

    /// Set cidr_prefix_length
    pub fn set_cidr_prefix_length(mut self, value: Option<i64>) -> Self {
        self.cidr_prefix_length = value;
        self
    }

    /// Set is_primary
    pub fn set_is_primary(mut self, value: Option<bool>) -> Self {
        self.is_primary = value;
        self
    }

    /// Set vlan_id
    pub fn set_vlan_id(mut self, value: Option<String>) -> Self {
        self.vlan_id = value;
        self
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: Option<String>) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_created = value;
        self
    }

    /// Set vnic_id
    pub fn set_vnic_id(mut self, value: Option<String>) -> Self {
        self.vnic_id = value;
        self
    }

    /// Set ip_state
    pub fn set_ip_state(mut self, value: Option<PrivateIpIpState>) -> Self {
        self.ip_state = value;
        self
    }

    /// Set lifetime
    pub fn set_lifetime(mut self, value: Option<PrivateIpLifetime>) -> Self {
        self.lifetime = value;
        self
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set ipv4_subnet_cidr_at_creation
    pub fn set_ipv4_subnet_cidr_at_creation(mut self, value: Option<String>) -> Self {
        self.ipv4_subnet_cidr_at_creation = value;
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
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

    /// Set hostname_label (unwraps Option)
    pub fn with_hostname_label(mut self, value: impl Into<String>) -> Self {
        self.hostname_label = Some(value.into());
        self
    }

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    /// Set ip_address (unwraps Option)
    pub fn with_ip_address(mut self, value: impl Into<String>) -> Self {
        self.ip_address = Some(value.into());
        self
    }

    /// Set cidr_prefix_length (unwraps Option)
    pub fn with_cidr_prefix_length(mut self, value: i64) -> Self {
        self.cidr_prefix_length = Some(value);
        self
    }

    /// Set is_primary (unwraps Option)
    pub fn with_is_primary(mut self, value: bool) -> Self {
        self.is_primary = Some(value);
        self
    }

    /// Set vlan_id (unwraps Option)
    pub fn with_vlan_id(mut self, value: impl Into<String>) -> Self {
        self.vlan_id = Some(value.into());
        self
    }

    /// Set subnet_id (unwraps Option)
    pub fn with_subnet_id(mut self, value: impl Into<String>) -> Self {
        self.subnet_id = Some(value.into());
        self
    }

    /// Set time_created (unwraps Option)
    pub fn with_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = Some(value);
        self
    }

    /// Set vnic_id (unwraps Option)
    pub fn with_vnic_id(mut self, value: impl Into<String>) -> Self {
        self.vnic_id = Some(value.into());
        self
    }

    /// Set ip_state (unwraps Option)
    pub fn with_ip_state(mut self, value: PrivateIpIpState) -> Self {
        self.ip_state = Some(value);
        self
    }

    /// Set lifetime (unwraps Option)
    pub fn with_lifetime(mut self, value: PrivateIpLifetime) -> Self {
        self.lifetime = Some(value);
        self
    }

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }

    /// Set ipv4_subnet_cidr_at_creation (unwraps Option)
    pub fn with_ipv4_subnet_cidr_at_creation(mut self, value: impl Into<String>) -> Self {
        self.ipv4_subnet_cidr_at_creation = Some(value.into());
        self
    }
}

impl Default for PrivateIp {
    fn default() -> Self {
        Self::new()
    }
}
