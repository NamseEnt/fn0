use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines the {@code contains} relationship between virtual network topology entities. A {@code Contains} relationship is defined when an entity fully owns, contains or manages another entity. For example, a subnet is contained and managed in the scope of a VCN, therefore a VCN has a {@code contains} relationship to a subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyContainsEntityRelationship {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for TopologyContainsEntityRelationship
pub struct TopologyContainsEntityRelationshipRequired {
    pub r#type: String,
}

impl TopologyContainsEntityRelationship {
    /// Create a new TopologyContainsEntityRelationship with required fields
    pub fn new(required: TopologyContainsEntityRelationshipRequired) -> Self {
        Self {
            r#type: required.r#type,
        }
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
