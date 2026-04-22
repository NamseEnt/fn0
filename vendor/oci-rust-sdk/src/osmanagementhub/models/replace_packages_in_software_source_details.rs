use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides a list of packages that will replace the existing packages in the software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplacePackagesInSoftwareSourceDetails {
    /// List of packages specified by the name of the package (N) or the full package name (NVRA or NEVRA).
    pub packages: Vec<String>,
}

/// Required fields for ReplacePackagesInSoftwareSourceDetails
pub struct ReplacePackagesInSoftwareSourceDetailsRequired {
    /// List of packages specified by the name of the package (N) or the full package name (NVRA or NEVRA).
    pub packages: Vec<String>,
}

impl ReplacePackagesInSoftwareSourceDetails {
    /// Create a new ReplacePackagesInSoftwareSourceDetails with required fields
    pub fn new(required: ReplacePackagesInSoftwareSourceDetailsRequired) -> Self {
        Self {
            packages: required.packages,
        }
    }

    /// Set packages
    pub fn set_packages(mut self, value: Vec<String>) -> Self {
        self.packages = value;
        self
    }
}
