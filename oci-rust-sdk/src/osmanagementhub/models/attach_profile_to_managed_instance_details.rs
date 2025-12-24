use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to set a profile for a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachProfileToManagedInstanceDetails {
    /// The profile [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to attach to the managed instance.
    pub profile_id: String,
}

/// Required fields for AttachProfileToManagedInstanceDetails
pub struct AttachProfileToManagedInstanceDetailsRequired {
    /// The profile [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to attach to the managed instance.
    pub profile_id: String,
}

impl AttachProfileToManagedInstanceDetails {
    /// Create a new AttachProfileToManagedInstanceDetails with required fields
    pub fn new(required: AttachProfileToManagedInstanceDetailsRequired) -> Self {
        Self {
            profile_id: required.profile_id,
        }
    }

    /// Set profile_id
    pub fn set_profile_id(mut self, value: String) -> Self {
        self.profile_id = value;
        self
    }
}
