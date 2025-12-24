use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The action to run when the preemptible instance is interrupted for eviction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreemptionAction {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for PreemptionAction
pub struct PreemptionActionRequired {
    pub r#type: String,
}

impl PreemptionAction {
    /// Create a new PreemptionAction with required fields
    pub fn new(required: PreemptionActionRequired) -> Self {
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
