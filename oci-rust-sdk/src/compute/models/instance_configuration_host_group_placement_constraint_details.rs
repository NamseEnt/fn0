use serde::{Deserialize, Serialize};

/// The details for providing placement constraints using the compute host group OCID.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationHostGroupPlacementConstraintDetails {
    /// The OCID of the compute host group.
    pub compute_host_group_id: String,
}

/// Required fields for InstanceConfigurationHostGroupPlacementConstraintDetails
pub struct InstanceConfigurationHostGroupPlacementConstraintDetailsRequired {
    pub compute_host_group_id: String,
}

impl InstanceConfigurationHostGroupPlacementConstraintDetails {
    /// Create new instance with required fields
    pub fn new(required: InstanceConfigurationHostGroupPlacementConstraintDetailsRequired) -> Self {
        Self {
            compute_host_group_id: required.compute_host_group_id,
        }
    }

    /// Set the compute host group ID
    pub fn set_compute_host_group_id(mut self, compute_host_group_id: String) -> Self {
        self.compute_host_group_id = compute_host_group_id;
        self
    }
}
