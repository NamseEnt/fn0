use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines the relationship between Virtual Network topology entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyEntityRelationship {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the first entity in the relationship.
    pub id1: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the second entity in the relationship.
    pub id2: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for TopologyEntityRelationship
pub struct TopologyEntityRelationshipRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the first entity in the relationship.
    pub id1: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the second entity in the relationship.
    pub id2: String,

    pub r#type: String,
}

impl TopologyEntityRelationship {
    /// Create a new TopologyEntityRelationship with required fields
    pub fn new(required: TopologyEntityRelationshipRequired) -> Self {
        Self {
            id1: required.id1,

            id2: required.id2,

            r#type: required.r#type,
        }
    }

    /// Set id1
    pub fn set_id1(mut self, value: String) -> Self {
        self.id1 = value;
        self
    }

    /// Set id2
    pub fn set_id2(mut self, value: String) -> Self {
        self.id2 = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
