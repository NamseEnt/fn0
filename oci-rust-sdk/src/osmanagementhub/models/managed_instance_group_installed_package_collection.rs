use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of installed packages returned for the {@link #listManagedInstanceGroupInstalledPackages(ListManagedInstanceGroupInstalledPackagesRequest) listManagedInstanceGroupInstalledPackages} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceGroupInstalledPackageCollection {
    /// List of installed packages.
    pub items: Vec<ManagedInstanceGroupInstalledPackageSummary>,
}

/// Required fields for ManagedInstanceGroupInstalledPackageCollection
pub struct ManagedInstanceGroupInstalledPackageCollectionRequired {
    /// List of installed packages.
    pub items: Vec<ManagedInstanceGroupInstalledPackageSummary>,
}

impl ManagedInstanceGroupInstalledPackageCollection {
    /// Create a new ManagedInstanceGroupInstalledPackageCollection with required fields
    pub fn new(required: ManagedInstanceGroupInstalledPackageCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ManagedInstanceGroupInstalledPackageSummary>) -> Self {
        self.items = value;
        self
    }
}
