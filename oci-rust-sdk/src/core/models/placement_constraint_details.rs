use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The details for providing placement constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacementConstraintDetails {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for PlacementConstraintDetails
pub struct PlacementConstraintDetailsRequired {
    pub r#type: String,
}

impl PlacementConstraintDetails {
    /// Create a new PlacementConstraintDetails with required fields
    pub fn new(required: PlacementConstraintDetailsRequired) -> Self {
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
