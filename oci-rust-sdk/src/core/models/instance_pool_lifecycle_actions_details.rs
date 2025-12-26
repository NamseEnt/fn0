use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The lifecycle actions for the instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancePoolLifecycleActionsDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_termination: Option<InstancePoolPreTerminationActionDetails>,
}

impl InstancePoolLifecycleActionsDetails {
    /// Create a new InstancePoolLifecycleActionsDetails
    pub fn new() -> Self {
        Self {
            pre_termination: None,
        }
    }

    /// Set pre_termination
    pub fn set_pre_termination(
        mut self,
        value: Option<InstancePoolPreTerminationActionDetails>,
    ) -> Self {
        self.pre_termination = value;
        self
    }

    /// Set pre_termination (unwraps Option)
    pub fn with_pre_termination(mut self, value: InstancePoolPreTerminationActionDetails) -> Self {
        self.pre_termination = Some(value);
        self
    }
}

impl Default for InstancePoolLifecycleActionsDetails {
    fn default() -> Self {
        Self::new()
    }
}
