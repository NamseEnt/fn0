use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of packages returned for the {@link #listManagedInstanceAvailablePackages(ListManagedInstanceAvailablePackagesRequest) listManagedInstanceAvailablePackages} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailablePackageCollection {
    /// List of available packages.
    pub items: Vec<AvailablePackageSummary>,
}

/// Required fields for AvailablePackageCollection
pub struct AvailablePackageCollectionRequired {
    /// List of available packages.
    pub items: Vec<AvailablePackageSummary>,
}

impl AvailablePackageCollection {
    /// Create a new AvailablePackageCollection with required fields
    pub fn new(required: AvailablePackageCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<AvailablePackageSummary>) -> Self {
        self.items = value;
        self
    }
}
