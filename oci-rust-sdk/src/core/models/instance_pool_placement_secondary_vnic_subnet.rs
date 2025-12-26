use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The secondary VNIC object for the placement configuration for an instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancePoolPlacementSecondaryVnicSubnet {
    /// The subnet [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the secondary VNIC.
    pub subnet_id: String,

    /// The display name of the VNIC. This is also used to match against the instance configuration defined secondary VNIC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Whether to allocate an IPv6 address at instance and VNIC creation from an IPv6 enabled subnet. Default: False. When provided you may optionally provide an IPv6 prefix ({@code ipv6SubnetCidr}) of your choice to assign the IPv6 address from. If {@code ipv6SubnetCidr} is not provided then an IPv6 prefix is chosen for you.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_assign_ipv6_ip: Option<bool>,

    /// A list of IPv6 prefix ranges from which the VNIC should be assigned an IPv6 address. You can provide only the prefix ranges and OCI will select an available address from the range. You can optionally choose to leave the prefix range empty and instead provide the specific IPv6 address that should be used from within that range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_address_ipv6_subnet_cidr_pair_details:
        Option<Vec<InstancePoolPlacementIpv6AddressIpv6SubnetCidrDetails>>,
}

/// Required fields for InstancePoolPlacementSecondaryVnicSubnet
pub struct InstancePoolPlacementSecondaryVnicSubnetRequired {
    /// The subnet [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the secondary VNIC.
    pub subnet_id: String,
}

impl InstancePoolPlacementSecondaryVnicSubnet {
    /// Create a new InstancePoolPlacementSecondaryVnicSubnet with required fields
    pub fn new(required: InstancePoolPlacementSecondaryVnicSubnetRequired) -> Self {
        Self {
            subnet_id: required.subnet_id,

            display_name: None,

            is_assign_ipv6_ip: None,

            ipv6_address_ipv6_subnet_cidr_pair_details: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set is_assign_ipv6_ip
    pub fn set_is_assign_ipv6_ip(mut self, value: Option<bool>) -> Self {
        self.is_assign_ipv6_ip = value;
        self
    }

    /// Set ipv6_address_ipv6_subnet_cidr_pair_details
    pub fn set_ipv6_address_ipv6_subnet_cidr_pair_details(
        mut self,
        value: Option<Vec<InstancePoolPlacementIpv6AddressIpv6SubnetCidrDetails>>,
    ) -> Self {
        self.ipv6_address_ipv6_subnet_cidr_pair_details = value;
        self
    }

    /// Set subnet_id
    pub fn set_subnet_id(mut self, value: String) -> Self {
        self.subnet_id = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set is_assign_ipv6_ip (unwraps Option)
    pub fn with_is_assign_ipv6_ip(mut self, value: bool) -> Self {
        self.is_assign_ipv6_ip = Some(value);
        self
    }

    /// Set ipv6_address_ipv6_subnet_cidr_pair_details (unwraps Option)
    pub fn with_ipv6_address_ipv6_subnet_cidr_pair_details(
        mut self,
        value: Vec<InstancePoolPlacementIpv6AddressIpv6SubnetCidrDetails>,
    ) -> Self {
        self.ipv6_address_ipv6_subnet_cidr_pair_details = Some(value);
        self
    }
}
