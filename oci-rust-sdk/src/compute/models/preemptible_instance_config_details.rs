use serde::{Deserialize, Serialize};

/// Configuration options for preemptible instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreemptibleInstanceConfigDetails {
    /// The action to take when the instance is preempted.
    pub preemption_action: super::PreemptionAction,
}

impl PreemptibleInstanceConfigDetails {
    pub fn new(preemption_action: super::PreemptionAction) -> Self {
        Self { preemption_action }
    }
}
