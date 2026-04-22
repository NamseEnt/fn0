use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A set of available packages returned for the {@link #listManagedInstanceGroupAvailablePackages(ListManagedInstanceGroupAvailablePackagesRequest) listManagedInstanceGroupAvailablePackages} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceGroupAvailablePackageCollection {
    /// List of available packages.
    pub items: Vec<ManagedInstanceGroupAvailablePackageSummary>,
}

/// Required fields for ManagedInstanceGroupAvailablePackageCollection
pub struct ManagedInstanceGroupAvailablePackageCollectionRequired {
    /// List of available packages.
    pub items: Vec<ManagedInstanceGroupAvailablePackageSummary>,
}

impl ManagedInstanceGroupAvailablePackageCollection {
    /// Create a new ManagedInstanceGroupAvailablePackageCollection with required fields
    pub fn new(required: ManagedInstanceGroupAvailablePackageCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<ManagedInstanceGroupAvailablePackageSummary>) -> Self {
        self.items = value;
        self
    }
}
