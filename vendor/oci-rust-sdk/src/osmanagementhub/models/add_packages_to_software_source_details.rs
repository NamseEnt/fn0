use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the list of packages to add to a software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddPackagesToSoftwareSourceDetails {
    /// List of packages specified by the name of the package (N) or the full package name (NVRA or NEVRA).
    pub packages: Vec<String>,

    /// Indicates whether the service should generate a custom software source when the package list contains invalid values. When set to true, the service ignores any invalid packages and generates the custom software source with using the valid packages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_continue_on_missing_packages: Option<bool>,
}

/// Required fields for AddPackagesToSoftwareSourceDetails
pub struct AddPackagesToSoftwareSourceDetailsRequired {
    /// List of packages specified by the name of the package (N) or the full package name (NVRA or NEVRA).
    pub packages: Vec<String>,
}

impl AddPackagesToSoftwareSourceDetails {
    /// Create a new AddPackagesToSoftwareSourceDetails with required fields
    pub fn new(required: AddPackagesToSoftwareSourceDetailsRequired) -> Self {
        Self {
            packages: required.packages,

            is_continue_on_missing_packages: None,
        }
    }

    /// Set packages
    pub fn set_packages(mut self, value: Vec<String>) -> Self {
        self.packages = value;
        self
    }

    /// Set is_continue_on_missing_packages
    pub fn set_is_continue_on_missing_packages(mut self, value: Option<bool>) -> Self {
        self.is_continue_on_missing_packages = value;
        self
    }

    /// Set is_continue_on_missing_packages (unwraps Option)
    pub fn with_is_continue_on_missing_packages(mut self, value: bool) -> Self {
        self.is_continue_on_missing_packages = Some(value);
        self
    }
}
