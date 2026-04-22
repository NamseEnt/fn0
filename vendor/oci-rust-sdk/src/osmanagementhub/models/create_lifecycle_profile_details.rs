use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a lifecycle environment registration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLifecycleProfileDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the lifecycle stage that the instance will be associated with.
    pub lifecycle_stage_id: String,

    pub profile_type: String,
}

/// Required fields for CreateLifecycleProfileDetails
pub struct CreateLifecycleProfileDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the lifecycle stage that the instance will be associated with.
    pub lifecycle_stage_id: String,

    pub profile_type: String,
}

impl CreateLifecycleProfileDetails {
    /// Create a new CreateLifecycleProfileDetails with required fields
    pub fn new(required: CreateLifecycleProfileDetailsRequired) -> Self {
        Self {
            lifecycle_stage_id: required.lifecycle_stage_id,

            profile_type: required.profile_type,
        }
    }

    /// Set lifecycle_stage_id
    pub fn set_lifecycle_stage_id(mut self, value: String) -> Self {
        self.lifecycle_stage_id = value;
        self
    }

    /// Set profile_type
    pub fn set_profile_type(mut self, value: String) -> Self {
        self.profile_type = value;
        self
    }
}
