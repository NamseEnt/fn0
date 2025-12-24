use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to attach a lifecycle stage to a profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachLifecycleStageToProfileDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the lifecycle stage that the instance will be associated with.
    pub lifecycle_stage_id: String,
}

/// Required fields for AttachLifecycleStageToProfileDetails
pub struct AttachLifecycleStageToProfileDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the lifecycle stage that the instance will be associated with.
    pub lifecycle_stage_id: String,
}

impl AttachLifecycleStageToProfileDetails {
    /// Create a new AttachLifecycleStageToProfileDetails with required fields
    pub fn new(required: AttachLifecycleStageToProfileDetailsRequired) -> Self {
        Self {
            lifecycle_stage_id: required.lifecycle_stage_id,
        }
    }

    /// Set lifecycle_stage_id
    pub fn set_lifecycle_stage_id(mut self, value: String) -> Self {
        self.lifecycle_stage_id = value;
        self
    }
}
