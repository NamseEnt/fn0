use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information about a software package available for installation on a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailablePackageSummary {
    pub package_classification: String,
}

/// Required fields for AvailablePackageSummary
pub struct AvailablePackageSummaryRequired {
    pub package_classification: String,
}

impl AvailablePackageSummary {
    /// Create a new AvailablePackageSummary with required fields
    pub fn new(required: AvailablePackageSummaryRequired) -> Self {
        Self {
            package_classification: required.package_classification,
        }
    }

    /// Set package_classification
    pub fn set_package_classification(mut self, value: String) -> Self {
        self.package_classification = value;
        self
    }
}
