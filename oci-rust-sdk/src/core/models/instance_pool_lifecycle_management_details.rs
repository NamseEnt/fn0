use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The lifecycle management options for the instance pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancePoolLifecycleManagementDetails {
    pub lifecycle_actions: InstancePoolLifecycleActionsDetails,
}

/// Required fields for InstancePoolLifecycleManagementDetails
pub struct InstancePoolLifecycleManagementDetailsRequired {
    pub lifecycle_actions: InstancePoolLifecycleActionsDetails,
}

impl InstancePoolLifecycleManagementDetails {
    /// Create a new InstancePoolLifecycleManagementDetails with required fields
    pub fn new(required: InstancePoolLifecycleManagementDetailsRequired) -> Self {
        Self {
            lifecycle_actions: required.lifecycle_actions,
        }
    }

    /// Set lifecycle_actions
    pub fn set_lifecycle_actions(mut self, value: InstancePoolLifecycleActionsDetails) -> Self {
        self.lifecycle_actions = value;
        self
    }
}
