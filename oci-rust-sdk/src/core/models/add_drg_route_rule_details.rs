use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details needed when adding a DRG route rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDrgRouteRuleDetails {
    /// Type of destination for the rule. Allowed values: * {@code CIDR_BLOCK}: If the rule's {@code destination} is an IP address range in CIDR notation.
    pub destination_type: AddDrgRouteRuleDetailsDestinationType,

    /// This is the range of IP addresses used for matching when routing traffic. Only CIDR_BLOCK values are allowed. <p> Potential values: * IP address range in CIDR notation. This can be an IPv4 CIDR block or IPv6 prefix. For example: {@code 192.168.1.0/24} or {@code 2001:0db8:0123:45::/56}.
    pub destination: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the next hop DRG attachment. The next hop DRG attachment is responsible for reaching the network destination.
    pub next_hop_drg_attachment_id: String,
}

/// Required fields for AddDrgRouteRuleDetails
pub struct AddDrgRouteRuleDetailsRequired {
    /// Type of destination for the rule. Allowed values: * {@code CIDR_BLOCK}: If the rule's {@code destination} is an IP address range in CIDR notation.
    pub destination_type: AddDrgRouteRuleDetailsDestinationType,

    /// This is the range of IP addresses used for matching when routing traffic. Only CIDR_BLOCK values are allowed. <p> Potential values: * IP address range in CIDR notation. This can be an IPv4 CIDR block or IPv6 prefix. For example: {@code 192.168.1.0/24} or {@code 2001:0db8:0123:45::/56}.
    pub destination: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the next hop DRG attachment. The next hop DRG attachment is responsible for reaching the network destination.
    pub next_hop_drg_attachment_id: String,
}

impl AddDrgRouteRuleDetails {
    /// Create a new AddDrgRouteRuleDetails with required fields
    pub fn new(required: AddDrgRouteRuleDetailsRequired) -> Self {
        Self {
            destination_type: required.destination_type,

            destination: required.destination,

            next_hop_drg_attachment_id: required.next_hop_drg_attachment_id,
        }
    }

    /// Set destination_type
    pub fn set_destination_type(mut self, value: AddDrgRouteRuleDetailsDestinationType) -> Self {
        self.destination_type = value;
        self
    }

    /// Set destination
    pub fn set_destination(mut self, value: String) -> Self {
        self.destination = value;
        self
    }

    /// Set next_hop_drg_attachment_id
    pub fn set_next_hop_drg_attachment_id(mut self, value: String) -> Self {
        self.next_hop_drg_attachment_id = value;
        self
    }
}
