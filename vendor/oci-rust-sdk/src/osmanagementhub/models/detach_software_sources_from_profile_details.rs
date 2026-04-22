use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to detach software sources from a profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetachSoftwareSourcesFromProfileDetails {
    /// List of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to detach from the profile.
    pub software_sources: Vec<String>,
}

/// Required fields for DetachSoftwareSourcesFromProfileDetails
pub struct DetachSoftwareSourcesFromProfileDetailsRequired {
    /// List of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to detach from the profile.
    pub software_sources: Vec<String>,
}

impl DetachSoftwareSourcesFromProfileDetails {
    /// Create a new DetachSoftwareSourcesFromProfileDetails with required fields
    pub fn new(required: DetachSoftwareSourcesFromProfileDetailsRequired) -> Self {
        Self {
            software_sources: required.software_sources,
        }
    }

    /// Set software_sources
    pub fn set_software_sources(mut self, value: Vec<String>) -> Self {
        self.software_sources = value;
        self
    }
}
