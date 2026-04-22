use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A base object for all types of instance power action requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancePowerActionDetails {
    pub action_type: String,
}

/// Required fields for InstancePowerActionDetails
pub struct InstancePowerActionDetailsRequired {
    pub action_type: String,
}

impl InstancePowerActionDetails {
    /// Create a new InstancePowerActionDetails with required fields
    pub fn new(required: InstancePowerActionDetailsRequired) -> Self {
        Self {
            action_type: required.action_type,
        }
    }

    /// Set action_type
    pub fn set_action_type(mut self, value: String) -> Self {
        self.action_type = value;
        self
    }
}
