use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The managed instances to detach from the lifecycle stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetachManagedInstancesFromLifecycleStageDetails {
    pub managed_instance_details: ManagedInstancesDetails,
}

/// Required fields for DetachManagedInstancesFromLifecycleStageDetails
pub struct DetachManagedInstancesFromLifecycleStageDetailsRequired {
    pub managed_instance_details: ManagedInstancesDetails,
}

impl DetachManagedInstancesFromLifecycleStageDetails {
    /// Create a new DetachManagedInstancesFromLifecycleStageDetails with required fields
    pub fn new(required: DetachManagedInstancesFromLifecycleStageDetailsRequired) -> Self {
        Self {
            managed_instance_details: required.managed_instance_details,
        }
    }

    /// Set managed_instance_details
    pub fn set_managed_instance_details(mut self, value: ManagedInstancesDetails) -> Self {
        self.managed_instance_details = value;
        self
    }
}
