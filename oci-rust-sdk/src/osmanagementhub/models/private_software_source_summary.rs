use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a private software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateSoftwareSourceSummary {
    pub software_source_type: String,

    /// Indicates if this software source can be mirrored to a management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mirror_sync_allowed: Option<bool>,
}

/// Required fields for PrivateSoftwareSourceSummary
pub struct PrivateSoftwareSourceSummaryRequired {
    pub software_source_type: String,
}

impl PrivateSoftwareSourceSummary {
    /// Create a new PrivateSoftwareSourceSummary with required fields
    pub fn new(required: PrivateSoftwareSourceSummaryRequired) -> Self {
        Self {
            software_source_type: required.software_source_type,

            is_mirror_sync_allowed: None,
        }
    }

    /// Set is_mirror_sync_allowed
    pub fn set_is_mirror_sync_allowed(mut self, value: Option<bool>) -> Self {
        self.is_mirror_sync_allowed = value;
        self
    }

    /// Set software_source_type
    pub fn set_software_source_type(mut self, value: String) -> Self {
        self.software_source_type = value;
        self
    }

    /// Set is_mirror_sync_allowed (unwraps Option)
    pub fn with_is_mirror_sync_allowed(mut self, value: bool) -> Self {
        self.is_mirror_sync_allowed = Some(value);
        self
    }
}
