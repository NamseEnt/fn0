use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a package installed on a managed instance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceGroupInstalledPackageSummary {
    /// The name of the package that is installed on the managed instance group.
    pub name: String,

    /// The architecture of the package that is installed on the managed instance group.
    pub architecture: String,
}

/// Required fields for ManagedInstanceGroupInstalledPackageSummary
pub struct ManagedInstanceGroupInstalledPackageSummaryRequired {
    /// The name of the package that is installed on the managed instance group.
    pub name: String,

    /// The architecture of the package that is installed on the managed instance group.
    pub architecture: String,
}

impl ManagedInstanceGroupInstalledPackageSummary {
    /// Create a new ManagedInstanceGroupInstalledPackageSummary with required fields
    pub fn new(required: ManagedInstanceGroupInstalledPackageSummaryRequired) -> Self {
        Self {
            name: required.name,

            architecture: required.architecture,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set architecture
    pub fn set_architecture(mut self, value: String) -> Self {
        self.architecture = value;
        self
    }
}
