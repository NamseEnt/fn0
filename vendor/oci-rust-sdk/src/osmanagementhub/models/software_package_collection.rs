use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of software packages returned for the {@link #listSoftwarePackages(ListSoftwarePackagesRequest) listSoftwarePackages} operation or {@link #listAllSoftwarePackages(ListAllSoftwarePackagesRequest) listAllSoftwarePackages} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwarePackageCollection {
    /// List of software packages.
    pub items: Vec<SoftwarePackageSummary>,
}

/// Required fields for SoftwarePackageCollection
pub struct SoftwarePackageCollectionRequired {
    /// List of software packages.
    pub items: Vec<SoftwarePackageSummary>,
}

impl SoftwarePackageCollection {
    /// Create a new SoftwarePackageCollection with required fields
    pub fn new(required: SoftwarePackageCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<SoftwarePackageSummary>) -> Self {
        self.items = value;
        self
    }
}
