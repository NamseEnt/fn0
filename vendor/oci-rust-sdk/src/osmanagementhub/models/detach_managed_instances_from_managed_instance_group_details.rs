use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to detach managed instances from a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetachManagedInstancesFromManagedInstanceGroupDetails {
    /// List of managed instance [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to detach from the group.
    pub managed_instances: Vec<String>,
}

/// Required fields for DetachManagedInstancesFromManagedInstanceGroupDetails
pub struct DetachManagedInstancesFromManagedInstanceGroupDetailsRequired {
    /// List of managed instance [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to detach from the group.
    pub managed_instances: Vec<String>,
}

impl DetachManagedInstancesFromManagedInstanceGroupDetails {
    /// Create a new DetachManagedInstancesFromManagedInstanceGroupDetails with required fields
    pub fn new(required: DetachManagedInstancesFromManagedInstanceGroupDetailsRequired) -> Self {
        Self {
            managed_instances: required.managed_instances,
        }
    }

    /// Set managed_instances
    pub fn set_managed_instances(mut self, value: Vec<String>) -> Self {
        self.managed_instances = value;
        self
    }
}
