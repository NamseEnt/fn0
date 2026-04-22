use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides a list of packages to be removed from the software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovePackagesFromSoftwareSourceDetails {
    /// List of packages specified by the name of the package (N) or the full package name (NVRA or NEVRA).
    pub packages: Vec<String>,
}

/// Required fields for RemovePackagesFromSoftwareSourceDetails
pub struct RemovePackagesFromSoftwareSourceDetailsRequired {
    /// List of packages specified by the name of the package (N) or the full package name (NVRA or NEVRA).
    pub packages: Vec<String>,
}

impl RemovePackagesFromSoftwareSourceDetails {
    /// Create a new RemovePackagesFromSoftwareSourceDetails with required fields
    pub fn new(required: RemovePackagesFromSoftwareSourceDetailsRequired) -> Self {
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
