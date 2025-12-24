use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIpv6Details {
    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// An IPv6 address of your choice. Must be an available IP address within the subnet's CIDR. If you don't specify a value, Oracle automatically assigns an IPv6 address from the subnet. The subnet is the one that contains the VNIC you specify in {@code vnicId}. <p> Example: {@code 2001:DB8::}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// Length of cidr range. Optional field to specify flexible cidr. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_prefix_length: Option<i64>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VNIC to assign the IPv6 to. The IPv6 will be in the VNIC's subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnic_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet from which the IPv6 is to be drawn. The IP address, *if supplied*, must be valid for the given subnet, only valid for reserved IPs currently.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_id: Option<String>,

    /// Lifetime of the IP address. There are two types of IPs: - Ephemeral - Reserved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime: Option<CreateIpv6DetailsLifetime>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the IP address or VNIC will use. For more information, see [Per-resource Routing](https://docs.oracle.com/iaas/Content/Network/Tasks/managingroutetables.htm#Overview_of_Routing_for_Your_VCN__source_routing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,

    /// The IPv6 prefix allocated to the subnet. This is required if more than one IPv6 prefix exists on the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_subnet_cidr: Option<String>,
}

impl CreateIpv6Details {
    /// Create a new CreateIpv6Details
    pub fn new() -> Self {
        Self {
            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            ip_address: None,

            cidr_prefix_length: None,

            vnic_id: None,

            subnet_id: None,

            lifetime: None,

            route_table_id: None,

            ipv6_subnet_cidr: None,
        }
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

    /// Set vnic_id
    pub fn set_vnic_id(mut self, value: Option<String>) -> Self {
        self.vnic_id = value;
        self
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: Option<String>) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set lifetime
    pub fn set_lifetime(mut self, value: Option<CreateIpv6DetailsLifetime>) -> Self {
        self.lifetime = value;
        self
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: Option<String>) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set ipv6_subnet_cidr
    pub fn set_ipv6_subnet_cidr(mut self, value: Option<String>) -> Self {
        self.ipv6_subnet_cidr = value;
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

    /// Set vnic_id (unwraps Option)
    pub fn with_vnic_id(mut self, value: impl Into<String>) -> Self {
        self.vnic_id = Some(value.into());
        self
    }

    /// Set subnet_id (unwraps Option)
    pub fn with_subnet_id(mut self, value: impl Into<String>) -> Self {
        self.subnet_id = Some(value.into());
        self
    }

    /// Set lifetime (unwraps Option)
    pub fn with_lifetime(mut self, value: CreateIpv6DetailsLifetime) -> Self {
        self.lifetime = Some(value);
        self
    }

    /// Set route_table_id (unwraps Option)
    pub fn with_route_table_id(mut self, value: impl Into<String>) -> Self {
        self.route_table_id = Some(value.into());
        self
    }

    /// Set ipv6_subnet_cidr (unwraps Option)
    pub fn with_ipv6_subnet_cidr(mut self, value: impl Into<String>) -> Self {
        self.ipv6_subnet_cidr = Some(value.into());
        self
    }
}

impl Default for CreateIpv6Details {
    fn default() -> Self {
        Self::new()
    }
}
