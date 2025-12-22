use serde::{Deserialize, Serialize};

/// Configuration options for preemptible instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreemptibleInstanceConfigDetails {
    /// The action to take when the instance is preempted.
    pub preemption_action: super::PreemptionAction,
}

/// Required fields for PreemptibleInstanceConfigDetails
pub struct PreemptibleInstanceConfigDetailsRequired {
    pub preemption_action: super::PreemptionAction,
}

impl PreemptibleInstanceConfigDetails {
    /// Create new instance with required fields
    pub fn new(required: PreemptibleInstanceConfigDetailsRequired) -> Self {
        Self {
            preemption_action: required.preemption_action,
        }
    }

    /// Set the preemption action
    pub fn set_preemption_action(mut self, preemption_action: super::PreemptionAction) -> Self {
        self.preemption_action = preemption_action;
        self
    }
}
