use serde::{Deserialize, Serialize};

/// The action to run when the preemptible instance is interrupted for eviction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PreemptionAction {
    Terminate(super::TerminatePreemptionAction),
}

impl PreemptionAction {
    pub fn terminate(action: super::TerminatePreemptionAction) -> Self {
        Self::Terminate(action)
    }
}
