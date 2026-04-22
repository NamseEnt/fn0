use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information for a management station registration profile. A management station profile can only be used by a single management station.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationProfile {
    pub profile_type: String,
}

/// Required fields for StationProfile
pub struct StationProfileRequired {
    pub profile_type: String,
}

impl StationProfile {
    /// Create a new StationProfile with required fields
    pub fn new(required: StationProfileRequired) -> Self {
        Self {
            profile_type: required.profile_type,
        }
    }

    /// Set profile_type
    pub fn set_profile_type(mut self, value: String) -> Self {
        self.profile_type = value;
        self
    }
}
