use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of packages returned for the {@link #listManagedInstanceUpdatablePackages(ListManagedInstanceUpdatablePackagesRequest) listManagedInstanceUpdatablePackages} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatablePackageCollection {
    /// List of updatable packages.
    pub items: Vec<UpdatablePackageSummary>,
}

/// Required fields for UpdatablePackageCollection
pub struct UpdatablePackageCollectionRequired {
    /// List of updatable packages.
    pub items: Vec<UpdatablePackageSummary>,
}

impl UpdatablePackageCollection {
    /// Create a new UpdatablePackageCollection with required fields
    pub fn new(required: UpdatablePackageCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<UpdatablePackageSummary>) -> Self {
        self.items = value;
        self
    }
}
