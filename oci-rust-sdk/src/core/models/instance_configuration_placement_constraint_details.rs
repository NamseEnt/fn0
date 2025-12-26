use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The details for providing placement constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationPlacementConstraintDetails {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for InstanceConfigurationPlacementConstraintDetails
pub struct InstanceConfigurationPlacementConstraintDetailsRequired {
    pub r#type: String,
}

impl InstanceConfigurationPlacementConstraintDetails {
    /// Create a new InstanceConfigurationPlacementConstraintDetails with required fields
    pub fn new(required: InstanceConfigurationPlacementConstraintDetailsRequired) -> Self {
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
