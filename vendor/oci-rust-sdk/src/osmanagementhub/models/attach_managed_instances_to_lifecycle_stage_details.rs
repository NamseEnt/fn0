use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The managed instances to attach to the lifecycle stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachManagedInstancesToLifecycleStageDetails {
    pub managed_instance_details: ManagedInstancesDetails,
}

/// Required fields for AttachManagedInstancesToLifecycleStageDetails
pub struct AttachManagedInstancesToLifecycleStageDetailsRequired {
    pub managed_instance_details: ManagedInstancesDetails,
}

impl AttachManagedInstancesToLifecycleStageDetails {
    /// Create a new AttachManagedInstancesToLifecycleStageDetails with required fields
    pub fn new(required: AttachManagedInstancesToLifecycleStageDetailsRequired) -> Self {
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
