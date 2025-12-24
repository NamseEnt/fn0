use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a group registration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupProfileDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the managed instance group that the instance will join after registration.
    pub managed_instance_group_id: String,

    pub profile_type: String,
}

/// Required fields for CreateGroupProfileDetails
pub struct CreateGroupProfileDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the managed instance group that the instance will join after registration.
    pub managed_instance_group_id: String,

    pub profile_type: String,
}

impl CreateGroupProfileDetails {
    /// Create a new CreateGroupProfileDetails with required fields
    pub fn new(required: CreateGroupProfileDetailsRequired) -> Self {
        Self {
            managed_instance_group_id: required.managed_instance_group_id,

            profile_type: required.profile_type,
        }
    }

    /// Set managed_instance_group_id
    pub fn set_managed_instance_group_id(mut self, value: String) -> Self {
        self.managed_instance_group_id = value;
        self
    }

    /// Set profile_type
    pub fn set_profile_type(mut self, value: String) -> Self {
        self.profile_type = value;
        self
    }
}
