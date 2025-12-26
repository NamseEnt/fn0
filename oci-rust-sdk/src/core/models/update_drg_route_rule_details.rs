use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details used to update a route rule in the DRG route table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDrgRouteRuleDetails {
    /// The Oracle-assigned ID of each DRG route rule to update.
    pub id: String,

    /// The range of IP addresses used for matching when routing traffic. <p> Potential values: * IP address range in CIDR notation. Can be an IPv4 CIDR block or IPv6 prefix. For example: {@code 192.168.1.0/24} or {@code 2001:0db8:0123:45::/56}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,

    /// Type of destination for the rule. Allowed values: * {@code CIDR_BLOCK}: If the rule's {@code destination} is an IP address range in CIDR notation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_type: Option<UpdateDrgRouteRuleDetailsDestinationType>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the next hop DRG attachment. The next hop DRG attachment is responsible for reaching the network destination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_hop_drg_attachment_id: Option<String>,
}

/// Required fields for UpdateDrgRouteRuleDetails
pub struct UpdateDrgRouteRuleDetailsRequired {
    /// The Oracle-assigned ID of each DRG route rule to update.
    pub id: String,
}

impl UpdateDrgRouteRuleDetails {
    /// Create a new UpdateDrgRouteRuleDetails with required fields
    pub fn new(required: UpdateDrgRouteRuleDetailsRequired) -> Self {
        Self {
            id: required.id,

            destination: None,

            destination_type: None,

            next_hop_drg_attachment_id: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set destination
    pub fn set_destination(mut self, value: Option<String>) -> Self {
        self.destination = value;
        self
    }

    /// Set destination_type
    pub fn set_destination_type(
        mut self,
        value: Option<UpdateDrgRouteRuleDetailsDestinationType>,
    ) -> Self {
        self.destination_type = value;
        self
    }

    /// Set next_hop_drg_attachment_id
    pub fn set_next_hop_drg_attachment_id(mut self, value: Option<String>) -> Self {
        self.next_hop_drg_attachment_id = value;
        self
    }

    /// Set destination (unwraps Option)
    pub fn with_destination(mut self, value: impl Into<String>) -> Self {
        self.destination = Some(value.into());
        self
    }

    /// Set destination_type (unwraps Option)
    pub fn with_destination_type(
        mut self,
        value: UpdateDrgRouteRuleDetailsDestinationType,
    ) -> Self {
        self.destination_type = Some(value);
        self
    }

    /// Set next_hop_drg_attachment_id (unwraps Option)
    pub fn with_next_hop_drg_attachment_id(mut self, value: impl Into<String>) -> Self {
        self.next_hop_drg_attachment_id = Some(value.into());
        self
    }
}
