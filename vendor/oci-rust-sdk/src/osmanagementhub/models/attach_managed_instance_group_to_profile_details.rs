use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to attach a managed instance group to a profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachManagedInstanceGroupToProfileDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the managed instance group that the instance will be associated with.
    pub managed_instance_group_id: String,
}

/// Required fields for AttachManagedInstanceGroupToProfileDetails
pub struct AttachManagedInstanceGroupToProfileDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the managed instance group that the instance will be associated with.
    pub managed_instance_group_id: String,
}

impl AttachManagedInstanceGroupToProfileDetails {
    /// Create a new AttachManagedInstanceGroupToProfileDetails with required fields
    pub fn new(required: AttachManagedInstanceGroupToProfileDetailsRequired) -> Self {
        Self {
            managed_instance_group_id: required.managed_instance_group_id,
        }
    }

    /// Set managed_instance_group_id
    pub fn set_managed_instance_group_id(mut self, value: String) -> Self {
        self.managed_instance_group_id = value;
        self
    }
}
