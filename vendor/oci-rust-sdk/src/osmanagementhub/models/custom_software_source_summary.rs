use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a custom software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomSoftwareSourceSummary {
    /// List of vendor software sources that are used for the basis of the custom software source..
    pub vendor_software_sources: Vec<Id>,

    pub software_source_type: String,

    /// Identifies how the custom software source was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_sub_type: Option<SoftwareSourceSubType>,
}

/// Required fields for CustomSoftwareSourceSummary
pub struct CustomSoftwareSourceSummaryRequired {
    /// List of vendor software sources that are used for the basis of the custom software source..
    pub vendor_software_sources: Vec<Id>,

    pub software_source_type: String,
}

impl CustomSoftwareSourceSummary {
    /// Create a new CustomSoftwareSourceSummary with required fields
    pub fn new(required: CustomSoftwareSourceSummaryRequired) -> Self {
        Self {
            vendor_software_sources: required.vendor_software_sources,

            software_source_type: required.software_source_type,

            software_source_sub_type: None,
        }
    }

    /// Set vendor_software_sources
    pub fn set_vendor_software_sources(mut self, value: Vec<Id>) -> Self {
        self.vendor_software_sources = value;
        self
    }

    /// Set software_source_sub_type
    pub fn set_software_source_sub_type(mut self, value: Option<SoftwareSourceSubType>) -> Self {
        self.software_source_sub_type = value;
        self
    }

    /// Set software_source_type
    pub fn set_software_source_type(mut self, value: String) -> Self {
        self.software_source_type = value;
        self
    }

    /// Set software_source_sub_type (unwraps Option)
    pub fn with_software_source_sub_type(mut self, value: SoftwareSourceSubType) -> Self {
        self.software_source_sub_type = Some(value);
        self
    }
}
