use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines the {@code AssociatedWith} relationship between virtual network topology entities. An {@code AssociatedWith} relationship is defined when there is no obvious {@code contains} relationship but entities are still related. For example, a DRG is associated with a VCN because a DRG is not managed by VCN but can be attached to a VCN.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyAssociatedWithEntityRelationship {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_with_details: Option<TopologyAssociatedWithRelationshipDetails>,
}

/// Required fields for TopologyAssociatedWithEntityRelationship
pub struct TopologyAssociatedWithEntityRelationshipRequired {
    pub r#type: String,
}

impl TopologyAssociatedWithEntityRelationship {
    /// Create a new TopologyAssociatedWithEntityRelationship with required fields
    pub fn new(required: TopologyAssociatedWithEntityRelationshipRequired) -> Self {
        Self {
            r#type: required.r#type,

            associated_with_details: None,
        }
    }

    /// Set associated_with_details
    pub fn set_associated_with_details(
        mut self,
        value: Option<TopologyAssociatedWithRelationshipDetails>,
    ) -> Self {
        self.associated_with_details = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set associated_with_details (unwraps Option)
    pub fn with_associated_with_details(
        mut self,
        value: TopologyAssociatedWithRelationshipDetails,
    ) -> Self {
        self.associated_with_details = Some(value);
        self
    }
}
