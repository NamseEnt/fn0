use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// An *IPv6* is a conceptual term that refers to an IPv6 address and related properties. The {@code IPv6} object is the API representation of an IPv6. <p> You can create and assign an IPv6 to any VNIC that is in an IPv6-enabled subnet in an IPv6-enabled VCN. <p> *Note:** IPv6 addressing is supported for all commercial and government regions. For important details about IPv6 addressing in a VCN, see [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv6 {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the IPv6. This is the same as the VNIC's compartment.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPv6.
    pub id: String,

    /// The IPv6 address of the {@code IPv6} object. The address is within the IPv6 prefix of the VNIC's subnet (see the {@code ipv6CidrBlock} attribute for the {@link Subnet} object. <p> Example: {@code 2001:0db8:0123:1111:abcd:ef01:2345:6789}
    pub ip_address: String,

    /// The IPv6's current state.
    pub lifecycle_state: Ipv6LifecycleState,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet the VNIC is in.
    pub subnet_id: String,

    /// The date and time the IPv6 was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Length of cidr range. Optional field to specify flexible cidr. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_prefix_length: Option<i64>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC the IPv6 is assigned to. The VNIC and IPv6 must be in the same subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnic_id: Option<String>,

    /// State of the IP address. If an IP address is assigned to a VNIC it is ASSIGNED, otherwise it is AVAILABLE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_state: Option<Ipv6IpState>,

    /// Lifetime of the IP address. There are two types of IPs: - Ephemeral - Reserved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime: Option<Ipv6Lifetime>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the IP address or VNIC will use. For more information, see [Per-resource Routing](https://docs.oracle.com/iaas/Content/Network/Tasks/managingroutetables.htm#Overview_of_Routing_for_Your_VCN__source_routing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,
}

/// Required fields for Ipv6
pub struct Ipv6Required {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the IPv6. This is the same as the VNIC's compartment.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IPv6.
    pub id: String,

    /// The IPv6 address of the {@code IPv6} object. The address is within the IPv6 prefix of the VNIC's subnet (see the {@code ipv6CidrBlock} attribute for the {@link Subnet} object. <p> Example: {@code 2001:0db8:0123:1111:abcd:ef01:2345:6789}
    pub ip_address: String,

    /// The IPv6's current state.
    pub lifecycle_state: Ipv6LifecycleState,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet the VNIC is in.
    pub subnet_id: String,

    /// The date and time the IPv6 was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl Ipv6 {
    /// Create a new Ipv6 with required fields
    pub fn new(required: Ipv6Required) -> Self {
        Self {
            compartment_id: required.compartment_id,

            display_name: required.display_name,

            id: required.id,

            ip_address: required.ip_address,

            lifecycle_state: required.lifecycle_state,

            subnet_id: required.subnet_id,

            time_created: required.time_created,

            defined_tags: None,

            freeform_tags: None,

            cidr_prefix_length: None,

            vnic_id: None,

            ip_state: None,

            lifetime: None,

            route_table_id: None,
        }
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
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
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

    /// Set ip_address
    pub fn set_ip_address(mut self, value: String) -> Self {
        self.ip_address = value;
        self
    }

    /// Set cidr_prefix_length
    pub fn set_cidr_prefix_length(mut self, value: Option<i64>) -> Self {
        self.cidr_prefix_length = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: Ipv6LifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: String) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set vnic_id
    pub fn set_vnic_id(mut self, value: Option<String>) -> Self {
        self.vnic_id = value;
        self
    }

    /// Set ip_state
    pub fn set_ip_state(mut self, value: Option<Ipv6IpState>) -> Self {
        self.ip_state = value;
        self
    }

    /// Set lifetime
    pub fn set_lifetime(mut self, value: Option<Ipv6Lifetime>) -> Self {
        self.lifetime = value;
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

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set cidr_prefix_length (unwraps Option)
    pub fn with_cidr_prefix_length(mut self, value: i64) -> Self {
        self.cidr_prefix_length = Some(value);
        self
    }

    /// Set vnic_id (unwraps Option)
    pub fn with_vnic_id(mut self, value: impl Into<String>) -> Self {
        self.vnic_id = Some(value.into());
        self
    }

    /// Set ip_state (unwraps Option)
    pub fn with_ip_state(mut self, value: Ipv6IpState) -> Self {
        self.ip_state = Some(value);
        self
    }

    /// Set lifetime (unwraps Option)
    pub fn with_lifetime(mut self, value: Ipv6Lifetime) -> Self {
        self.lifetime = Some(value);
        self
    }

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }
}
