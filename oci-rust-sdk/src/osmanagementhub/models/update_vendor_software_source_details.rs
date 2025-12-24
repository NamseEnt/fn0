use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information for updating the tags of a vendor software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVendorSoftwareSourceDetails {
    pub software_source_type: String,
}

/// Required fields for UpdateVendorSoftwareSourceDetails
pub struct UpdateVendorSoftwareSourceDetailsRequired {
    pub software_source_type: String,
}

impl UpdateVendorSoftwareSourceDetails {
    /// Create a new UpdateVendorSoftwareSourceDetails with required fields
    pub fn new(required: UpdateVendorSoftwareSourceDetailsRequired) -> Self {
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
