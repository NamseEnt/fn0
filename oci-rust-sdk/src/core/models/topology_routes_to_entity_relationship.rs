use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines the {@code routesTo} relationship between virtual network topology entities. A {@code RoutesTo} relationship is defined when a routing table and a routing rule  are used to govern how to route traffic from one entity to another. For example, a DRG might have a routing rule to send certain traffic to an LPG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyRoutesToEntityRelationship {
    pub route_rule_details: TopologyRoutesToRelationshipDetails,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for TopologyRoutesToEntityRelationship
pub struct TopologyRoutesToEntityRelationshipRequired {
    pub route_rule_details: TopologyRoutesToRelationshipDetails,

    pub r#type: String,
}

impl TopologyRoutesToEntityRelationship {
    /// Create a new TopologyRoutesToEntityRelationship with required fields
    pub fn new(required: TopologyRoutesToEntityRelationshipRequired) -> Self {
        Self {
            route_rule_details: required.route_rule_details,

            r#type: required.r#type,
        }
    }

    /// Set route_rule_details
    pub fn set_route_rule_details(mut self, value: TopologyRoutesToRelationshipDetails) -> Self {
        self.route_rule_details = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
