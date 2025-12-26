use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines route rule details for a {@code routesTo} relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyRoutesToRelationshipDetails {
    /// The destinationType can be set to one of two values: <p> Use {@code CIDR_BLOCK} if the rule's {@code destination} is an IP address range in CIDR notation. <p> Use {@code SERVICE_CIDR_BLOCK} if the rule's {@code destination} is the {@code cidrBlock} value for a {@link Service}.
    pub destination_type: String,

    /// An IP address range in CIDR notation or the {@code cidrBlock} value for a {@link Service}.
    pub destination: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the routing table that contains the route rule.
    pub route_table_id: String,

    /// A route rule can be {@code STATIC} if manually added to the route table or {@code DYNAMIC} if imported from another route table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_type: Option<TopologyRoutesToRelationshipDetailsRouteType>,
}

/// Required fields for TopologyRoutesToRelationshipDetails
pub struct TopologyRoutesToRelationshipDetailsRequired {
    /// The destinationType can be set to one of two values: <p> Use {@code CIDR_BLOCK} if the rule's {@code destination} is an IP address range in CIDR notation. <p> Use {@code SERVICE_CIDR_BLOCK} if the rule's {@code destination} is the {@code cidrBlock} value for a {@link Service}.
    pub destination_type: String,

    /// An IP address range in CIDR notation or the {@code cidrBlock} value for a {@link Service}.
    pub destination: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the routing table that contains the route rule.
    pub route_table_id: String,
}

impl TopologyRoutesToRelationshipDetails {
    /// Create a new TopologyRoutesToRelationshipDetails with required fields
    pub fn new(required: TopologyRoutesToRelationshipDetailsRequired) -> Self {
        Self {
            destination_type: required.destination_type,

            destination: required.destination,

            route_table_id: required.route_table_id,

            route_type: None,
        }
    }

    /// Set destination_type
    pub fn set_destination_type(mut self, value: String) -> Self {
        self.destination_type = value;
        self
    }

    /// Set destination
    pub fn set_destination(mut self, value: String) -> Self {
        self.destination = value;
        self
    }

    /// Set route_table_id
    pub fn set_route_table_id(mut self, value: String) -> Self {
        self.route_table_id = value;
        self
    }

    /// Set route_type
    pub fn set_route_type(
        mut self,
        value: Option<TopologyRoutesToRelationshipDetailsRouteType>,
    ) -> Self {
        self.route_type = value;
        self
    }

    /// Set route_type (unwraps Option)
    pub fn with_route_type(mut self, value: TopologyRoutesToRelationshipDetailsRouteType) -> Self {
        self.route_type = Some(value);
        self
    }
}
