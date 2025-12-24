use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information for a software source registration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareSourceProfile {
    /// The list of software sources that the registration profile will use.
    pub software_sources: Vec<SoftwareSourceDetails>,

    pub profile_type: String,
}

/// Required fields for SoftwareSourceProfile
pub struct SoftwareSourceProfileRequired {
    /// The list of software sources that the registration profile will use.
    pub software_sources: Vec<SoftwareSourceDetails>,

    pub profile_type: String,
}

impl SoftwareSourceProfile {
    /// Create a new SoftwareSourceProfile with required fields
    pub fn new(required: SoftwareSourceProfileRequired) -> Self {
        Self {
            software_sources: required.software_sources,

            profile_type: required.profile_type,
        }
    }

    /// Set software_sources
    pub fn set_software_sources(mut self, value: Vec<SoftwareSourceDetails>) -> Self {
        self.software_sources = value;
        self
    }

    /// Set profile_type
    pub fn set_profile_type(mut self, value: String) -> Self {
        self.profile_type = value;
        self
    }
}
