use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A mapping between a destination IP address range and a virtual device to route matching packets to (a target).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteRule {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the route rule's target. For information about the type of targets you can specify, see [Route Tables](https://docs.oracle.com/iaas/Content/Network/Tasks/managingroutetables.htm).
    pub network_entity_id: String,

    /// Deprecated. Instead use {@code destination} and {@code destinationType}. Requests that include both {@code cidrBlock} and {@code destination} will be rejected. <p> A destination IP address range in CIDR notation. Matching packets will be routed to the indicated network entity (the target). <p> Cannot be an IPv6 prefix. <p> Example: {@code 0.0.0.0/0}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cidr_block: Option<String>,

    /// Conceptually, this is the range of IP addresses used for matching when routing traffic. Required if you provide a {@code destinationType}. <p> Allowed values: <p> IP address range in CIDR notation. Can be an IPv4 CIDR block or IPv6 prefix. For example: {@code 192.168.1.0/24} or {@code 2001:0db8:0123:45::/56}. If you set this to an IPv6 prefix, the route rule's target can only be a DRG or internet gateway. IPv6 addressing is supported for all commercial and government regions. See [IPv6 Addresses](https://docs.oracle.com/iaas/Content/Network/Concepts/ipv6.htm). <p> The {@code cidrBlock} value for a {@link Service}, if you're setting up a route rule for traffic destined for a particular {@code Service} through a service gateway. For example: {@code oci-phx-objectstorage}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,

    /// Type of destination for the rule. Required if you provide a {@code destination}. <p> {@code CIDR_BLOCK}: If the rule's {@code destination} is an IP address range in CIDR notation. <p> {@code SERVICE_CIDR_BLOCK}: If the rule's {@code destination} is the {@code cidrBlock} value for a {@link Service} (the rule is for traffic destined for a particular {@code Service} through a service gateway).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_type: Option<RouteRuleDestinationType>,

    /// An optional description of your choice for the rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A route rule can be STATIC if manually added to the route table, LOCAL if added by OCI to the route table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_type: Option<RouteRuleRouteType>,
}

/// Required fields for RouteRule
pub struct RouteRuleRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the route rule's target. For information about the type of targets you can specify, see [Route Tables](https://docs.oracle.com/iaas/Content/Network/Tasks/managingroutetables.htm).
    pub network_entity_id: String,
}

impl RouteRule {
    /// Create a new RouteRule with required fields
    pub fn new(required: RouteRuleRequired) -> Self {
        Self {
            network_entity_id: required.network_entity_id,

            cidr_block: None,

            destination: None,

            destination_type: None,

            description: None,

            route_type: None,
        }
    }

    /// Set cidr_block
    pub fn set_cidr_block(mut self, value: Option<String>) -> Self {
        self.cidr_block = value;
        self
    }

    /// Set destination
    pub fn set_destination(mut self, value: Option<String>) -> Self {
        self.destination = value;
        self
    }

    /// Set destination_type
    pub fn set_destination_type(mut self, value: Option<RouteRuleDestinationType>) -> Self {
        self.destination_type = value;
        self
    }

    /// Set network_entity_id
    pub fn set_network_entity_id(mut self, value: String) -> Self {
        self.network_entity_id = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set route_type
    pub fn set_route_type(mut self, value: Option<RouteRuleRouteType>) -> Self {
        self.route_type = value;
        self
    }

    /// Set cidr_block (unwraps Option)
    pub fn with_cidr_block(mut self, value: impl Into<String>) -> Self {
        self.cidr_block = Some(value.into());
        self
    }

    /// Set destination (unwraps Option)
    pub fn with_destination(mut self, value: impl Into<String>) -> Self {
        self.destination = Some(value.into());
        self
    }

    /// Set destination_type (unwraps Option)
    pub fn with_destination_type(mut self, value: RouteRuleDestinationType) -> Self {
        self.destination_type = Some(value);
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set route_type (unwraps Option)
    pub fn with_route_type(mut self, value: RouteRuleRouteType) -> Self {
        self.route_type = Some(value);
        self
    }
}
