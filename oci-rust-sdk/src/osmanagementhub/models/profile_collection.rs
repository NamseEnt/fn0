use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of registration profiles returned for the {@link #listProfiles(ListProfilesRequest) listProfiles} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileCollection {
    /// List of registration profiles.
    pub items: Vec<ProfileSummary>,
}

/// Required fields for ProfileCollection
pub struct ProfileCollectionRequired {
    /// List of registration profiles.
    pub items: Vec<ProfileSummary>,
}

impl ProfileCollection {
    /// Create a new ProfileCollection with required fields
    pub fn new(required: ProfileCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ProfileSummary>) -> Self {
        self.items = value;
        self
    }
}
