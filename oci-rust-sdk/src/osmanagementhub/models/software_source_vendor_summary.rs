use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information about a software source vendor, including name, operating system family, and architecture type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareSourceVendorSummary {
    /// Name of the vendor providing the software source.
    pub name: VendorName,

    /// List of corresponding operating system families.
    pub os_families: Vec<SoftwareSourceVendorSummaryOsFamilies>,

    /// List of corresponding architecture types.
    pub arch_types: Vec<SoftwareSourceVendorSummaryArchTypes>,
}

/// Required fields for SoftwareSourceVendorSummary
pub struct SoftwareSourceVendorSummaryRequired {
    /// Name of the vendor providing the software source.
    pub name: VendorName,

    /// List of corresponding operating system families.
    pub os_families: Vec<SoftwareSourceVendorSummaryOsFamilies>,

    /// List of corresponding architecture types.
    pub arch_types: Vec<SoftwareSourceVendorSummaryArchTypes>,
}

impl SoftwareSourceVendorSummary {
    /// Create a new SoftwareSourceVendorSummary with required fields
    pub fn new(required: SoftwareSourceVendorSummaryRequired) -> Self {
        Self {
            name: required.name,

            os_families: required.os_families,

            arch_types: required.arch_types,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: VendorName) -> Self {
        self.name = value;
        self
    }

    /// Set os_families
    pub fn set_os_families(mut self, value: Vec<SoftwareSourceVendorSummaryOsFamilies>) -> Self {
        self.os_families = value;
        self
    }

    /// Set arch_types
    pub fn set_arch_types(mut self, value: Vec<SoftwareSourceVendorSummaryArchTypes>) -> Self {
        self.arch_types = value;
        self
    }
}
