use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to update a versioned custom software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVersionedCustomSoftwareSourceDetails {
    pub software_source_type: String,
}

/// Required fields for UpdateVersionedCustomSoftwareSourceDetails
pub struct UpdateVersionedCustomSoftwareSourceDetailsRequired {
    pub software_source_type: String,
}

impl UpdateVersionedCustomSoftwareSourceDetails {
    /// Create a new UpdateVersionedCustomSoftwareSourceDetails with required fields
    pub fn new(required: UpdateVersionedCustomSoftwareSourceDetailsRequired) -> Self {
        Self {
            software_source_type: required.software_source_type,
        }
    }

    /// Set software_source_type
    pub fn set_software_source_type(mut self, value: String) -> Self {
        self.software_source_type = value;
        self
    }
}
