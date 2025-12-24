use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to attach software sources to a profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachSoftwareSourcesToProfileDetails {
    /// List of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to attach to the profile.
    pub software_sources: Vec<String>,
}

/// Required fields for AttachSoftwareSourcesToProfileDetails
pub struct AttachSoftwareSourcesToProfileDetailsRequired {
    /// List of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to attach to the profile.
    pub software_sources: Vec<String>,
}

impl AttachSoftwareSourcesToProfileDetails {
    /// Create a new AttachSoftwareSourcesToProfileDetails with required fields
    pub fn new(required: AttachSoftwareSourcesToProfileDetailsRequired) -> Self {
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
