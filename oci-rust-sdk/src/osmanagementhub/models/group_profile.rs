use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information for a group registration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupProfile {
    pub managed_instance_group: ManagedInstanceGroupDetails,

    pub profile_type: String,
}

/// Required fields for GroupProfile
pub struct GroupProfileRequired {
    pub managed_instance_group: ManagedInstanceGroupDetails,

    pub profile_type: String,
}

impl GroupProfile {
    /// Create a new GroupProfile with required fields
    pub fn new(required: GroupProfileRequired) -> Self {
        Self {
            managed_instance_group: required.managed_instance_group,

            profile_type: required.profile_type,
        }
    }

    /// Set managed_instance_group
    pub fn set_managed_instance_group(mut self, value: ManagedInstanceGroupDetails) -> Self {
        self.managed_instance_group = value;
        self
    }

    /// Set profile_type
    pub fn set_profile_type(mut self, value: String) -> Self {
        self.profile_type = value;
        self
    }
}
