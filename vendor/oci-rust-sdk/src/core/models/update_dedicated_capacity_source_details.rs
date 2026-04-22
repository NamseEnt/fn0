use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A capacity source of bare metal hosts that is dedicated to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDedicatedCapacitySourceDetails {
    pub capacity_type: String,
}

/// Required fields for UpdateDedicatedCapacitySourceDetails
pub struct UpdateDedicatedCapacitySourceDetailsRequired {
    pub capacity_type: String,
}

impl UpdateDedicatedCapacitySourceDetails {
    /// Create a new UpdateDedicatedCapacitySourceDetails with required fields
    pub fn new(required: UpdateDedicatedCapacitySourceDetailsRequired) -> Self {
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
