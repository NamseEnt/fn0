use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The list of one or more BYOIPv6 prefixes for the VCN that meets the following criteria: - The prefix must be from a BYOIPv6 range. - The IPv6 prefixes must be valid. - Multiple prefix must not overlap each other or the on-premises network prefix. - The number of prefixes must not exceed the limit of IPv6 prefixes allowed to a VCN.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Byoipv6CidrDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource to which the CIDR block belongs.
    pub byoipv6_range_id: String,

    /// An IPv6 prefix required to create a VCN with a BYOIP prefix. It could be the whole prefix identified in {@code byoipv6RangeId}, or a subrange. Example: {@code 2001:0db8:0123::/48}
    pub ipv6_cidr_block: String,
}

/// Required fields for Byoipv6CidrDetails
pub struct Byoipv6CidrDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the {@code ByoipRange} resource to which the CIDR block belongs.
    pub byoipv6_range_id: String,

    /// An IPv6 prefix required to create a VCN with a BYOIP prefix. It could be the whole prefix identified in {@code byoipv6RangeId}, or a subrange. Example: {@code 2001:0db8:0123::/48}
    pub ipv6_cidr_block: String,
}

impl Byoipv6CidrDetails {
    /// Create a new Byoipv6CidrDetails with required fields
    pub fn new(required: Byoipv6CidrDetailsRequired) -> Self {
        Self {
            byoipv6_range_id: required.byoipv6_range_id,

            ipv6_cidr_block: required.ipv6_cidr_block,
        }
    }

    /// Set byoipv6_range_id
    pub fn set_byoipv6_range_id(mut self, value: String) -> Self {
        self.byoipv6_range_id = value;
        self
    }

    /// Set ipv6_cidr_block
    pub fn set_ipv6_cidr_block(mut self, value: String) -> Self {
        self.ipv6_cidr_block = value;
        self
    }
}
