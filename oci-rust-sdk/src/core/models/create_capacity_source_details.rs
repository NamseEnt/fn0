use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A capacity source of bare metal hosts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCapacitySourceDetails {
    pub capacity_type: String,
}

/// Required fields for CreateCapacitySourceDetails
pub struct CreateCapacitySourceDetailsRequired {
    pub capacity_type: String,
}

impl CreateCapacitySourceDetails {
    /// Create a new CreateCapacitySourceDetails with required fields
    pub fn new(required: CreateCapacitySourceDetailsRequired) -> Self {
        Self {
            capacity_type: required.capacity_type,
        }
    }

    /// Set capacity_type
    pub fn set_capacity_type(mut self, value: String) -> Self {
        self.capacity_type = value;
        self
    }
}
