use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Specifies the host group id
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetachComputeHostGroupHostDetails {
    /// 'The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute host group.'
    pub compute_host_group_id: String,
}

/// Required fields for DetachComputeHostGroupHostDetails
pub struct DetachComputeHostGroupHostDetailsRequired {
    /// 'The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute host group.'
    pub compute_host_group_id: String,
}

impl DetachComputeHostGroupHostDetails {
    /// Create a new DetachComputeHostGroupHostDetails with required fields
    pub fn new(required: DetachComputeHostGroupHostDetailsRequired) -> Self {
        Self {
            compute_host_group_id: required.compute_host_group_id,
        }
    }

    /// Set compute_host_group_id
    pub fn set_compute_host_group_id(mut self, value: String) -> Self {
        self.compute_host_group_id = value;
        self
    }
}
