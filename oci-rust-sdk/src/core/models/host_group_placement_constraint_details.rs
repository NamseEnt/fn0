use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The details for providing placement constraints using the compute host group OCID.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostGroupPlacementConstraintDetails {
    /// The OCID of the compute host group. This is only available for dedicated capacity customers.
    pub compute_host_group_id: String,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for HostGroupPlacementConstraintDetails
pub struct HostGroupPlacementConstraintDetailsRequired {
    /// The OCID of the compute host group. This is only available for dedicated capacity customers.
    pub compute_host_group_id: String,

    pub r#type: String,
}

impl HostGroupPlacementConstraintDetails {
    /// Create a new HostGroupPlacementConstraintDetails with required fields
    pub fn new(required: HostGroupPlacementConstraintDetailsRequired) -> Self {
        Self {
            compute_host_group_id: required.compute_host_group_id,

            r#type: required.r#type,
        }
    }

    /// Set compute_host_group_id
    pub fn set_compute_host_group_id(mut self, value: String) -> Self {
        self.compute_host_group_id = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
