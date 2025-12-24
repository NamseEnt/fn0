use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines association details for an {@code associatedWith} relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyAssociatedWithRelationshipDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the entities via which the relationship is created. For example an instance is associated with a network security group via the VNIC attachment and the VNIC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<Vec<String>>,
}

impl TopologyAssociatedWithRelationshipDetails {
    /// Create a new TopologyAssociatedWithRelationshipDetails
    pub fn new() -> Self {
        Self { via: None }
    }

    /// Set via
    pub fn set_via(mut self, value: Option<Vec<String>>) -> Self {
        self.via = value;
        self
    }

    /// Set via (unwraps Option)
    pub fn with_via(mut self, value: Vec<String>) -> Self {
        self.via = Some(value);
        self
    }
}

impl Default for TopologyAssociatedWithRelationshipDetails {
    fn default() -> Self {
        Self::new()
    }
}
