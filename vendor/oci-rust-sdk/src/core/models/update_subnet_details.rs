use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubnetDetails {
    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the set of DHCP options the subnet will use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhcp_options_id: Option<String>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the route table the subnet will use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_table_id: Option<String>,

    /// The OCIDs of the security list or lists the subnet will use. This replaces the entire current set of security lists. Remember that security lists are associated *with the subnet*, but the rules are applied to the individual VNICs in the subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_list_ids: Option<Vec<String>>,

    /// The CIDR block of the subnet. The new CIDR block must meet the following criteria: <p> - Must be valid. - The CIDR block's IP range must be completely within one of the VCN's CIDR block ranges. - The old and new CIDR block ranges must use the same network address. Example: {@code 10.0.0.0/25} and {@code 10.0.0.0/24}. - Must contain all IP addresses in use in the old CIDR range. - The new CIDR range's broadcast address (last IP address of CIDR range) must not be an IP address in use in the old CIDR range. <p> *Note:** If you are changing the CIDR block, you cannot create VNICs or private IPs for this resource while the update is in progress. <p> Example: {@code 172.16.0.0/16}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_block: Option<String>,

    /// This is the IPv6 prefix for the subnet's IP address space. The subnet size is always /64. See [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). The provided prefix must maintain the following rules - <p> a. The IPv6 prefix is valid and correctly formatted. b. The IPv6 prefix is within the parent VCN IPv6 range. <p> Example: {@code 2001:0db8:0123:1111::/64}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_block: Option<String>,

    /// The list of all IPv6 prefixes (Oracle allocated IPv6 GUA, ULA or private IPv6 prefix, BYOIPv6 prefixes) for the subnet that meets the following criteria: - The prefixes must be valid. - Multiple prefixes must not overlap each other or the on-premises network prefix. - The number of prefixes must not exceed the limit of IPv6 prefixes allowed to a subnet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_cidr_blocks: Option<Vec<String>>,
}

impl UpdateSubnetDetails {
    /// Create a new UpdateSubnetDetails
    pub fn new() -> Self {
        Self {
            defined_tags: None,

            dhcp_options_id: None,

            display_name: None,

            freeform_tags: None,

            route_table_id: None,

            security_list_ids: None,

            cidr_block: None,

            ipv6_cidr_block: None,

            ipv6_cidr_blocks: None,
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

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
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

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: Option<String>) -> Self {
        self.cidr_block = value;
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

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
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

    /// Set cidr_block (unwraps Option)
    pub fn with_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.cidr_block = Some(value.into());
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
}

impl Default for UpdateSubnetDetails {
    fn default() -> Self {
        Self::new()
    }
}
