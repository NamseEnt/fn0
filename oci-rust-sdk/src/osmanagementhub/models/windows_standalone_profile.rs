use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information for a Windows standalone registration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsStandaloneProfile {
    pub profile_type: String,
}

/// Required fields for WindowsStandaloneProfile
pub struct WindowsStandaloneProfileRequired {
    pub profile_type: String,
}

impl WindowsStandaloneProfile {
    /// Create a new WindowsStandaloneProfile with required fields
    pub fn new(required: WindowsStandaloneProfileRequired) -> Self {
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
