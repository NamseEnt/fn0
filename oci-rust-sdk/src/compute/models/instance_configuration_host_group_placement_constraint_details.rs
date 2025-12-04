use serde::{Deserialize, Serialize};

/// The details for providing placement constraints using the compute host group OCID.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationHostGroupPlacementConstraintDetails {
    /// The OCID of the compute host group.
    pub compute_host_group_id: String,
}

impl InstanceConfigurationHostGroupPlacementConstraintDetails {
    pub fn new(compute_host_group_id: impl Into<String>) -> Self {
        Self {
            compute_host_group_id: compute_host_group_id.into(),
        }
    }
}
