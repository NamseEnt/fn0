use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to update the availability of a list of software sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeAvailabilityOfSoftwareSourcesDetails {
    /// List of vendor software sources and their availability statuses.
    pub software_source_availabilities: Vec<SoftwareSourceAvailability>,
}

/// Required fields for ChangeAvailabilityOfSoftwareSourcesDetails
pub struct ChangeAvailabilityOfSoftwareSourcesDetailsRequired {
    /// List of vendor software sources and their availability statuses.
    pub software_source_availabilities: Vec<SoftwareSourceAvailability>,
}

impl ChangeAvailabilityOfSoftwareSourcesDetails {
    /// Create a new ChangeAvailabilityOfSoftwareSourcesDetails with required fields
    pub fn new(required: ChangeAvailabilityOfSoftwareSourcesDetailsRequired) -> Self {
        Self {
            software_source_availabilities: required.software_source_availabilities,
        }
    }

    /// Set software_source_availabilities
    pub fn set_software_source_availabilities(
        mut self,
        value: Vec<SoftwareSourceAvailability>,
    ) -> Self {
        self.software_source_availabilities = value;
        self
    }
}
