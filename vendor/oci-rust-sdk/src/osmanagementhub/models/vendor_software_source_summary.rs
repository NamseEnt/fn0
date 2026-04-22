use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a vendor software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorSoftwareSourceSummary {
    /// Name of the vendor providing the software source.
    pub vendor_name: VendorName,

    pub software_source_type: String,

    /// Indicates whether the software source is required for the Autonomous Linux service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mandatory_for_autonomous_linux: Option<bool>,
}

/// Required fields for VendorSoftwareSourceSummary
pub struct VendorSoftwareSourceSummaryRequired {
    /// Name of the vendor providing the software source.
    pub vendor_name: VendorName,

    pub software_source_type: String,
}

impl VendorSoftwareSourceSummary {
    /// Create a new VendorSoftwareSourceSummary with required fields
    pub fn new(required: VendorSoftwareSourceSummaryRequired) -> Self {
        Self {
            vendor_name: required.vendor_name,

            software_source_type: required.software_source_type,

            is_mandatory_for_autonomous_linux: None,
        }
    }

    /// Set vendor_name
    pub fn set_vendor_name(mut self, value: VendorName) -> Self {
        self.vendor_name = value;
        self
    }

    /// Set is_mandatory_for_autonomous_linux
    pub fn set_is_mandatory_for_autonomous_linux(mut self, value: Option<bool>) -> Self {
        self.is_mandatory_for_autonomous_linux = value;
        self
    }

    /// Set software_source_type
    pub fn set_software_source_type(mut self, value: String) -> Self {
        self.software_source_type = value;
        self
    }

    /// Set is_mandatory_for_autonomous_linux (unwraps Option)
    pub fn with_is_mandatory_for_autonomous_linux(mut self, value: bool) -> Self {
        self.is_mandatory_for_autonomous_linux = Some(value);
        self
    }
}
