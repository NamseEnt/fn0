use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Configuration options for preemptible instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreemptibleInstanceConfigDetails {
    pub preemption_action: TerminatePreemptionAction,
}

/// Required fields for PreemptibleInstanceConfigDetails
pub struct PreemptibleInstanceConfigDetailsRequired {
    pub preemption_action: TerminatePreemptionAction,
}

impl PreemptibleInstanceConfigDetails {
    /// Create a new PreemptibleInstanceConfigDetails with required fields
    pub fn new(required: PreemptibleInstanceConfigDetailsRequired) -> Self {
        Self {
            preemption_action: required.preemption_action,
        }
    }

    /// Set preemption_action
    pub fn set_preemption_action(mut self, value: TerminatePreemptionAction) -> Self {
        self.preemption_action = value;
        self
    }
}
