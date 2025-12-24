use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of installed packages on a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPackageCollection {
    /// List of installed packages.
    pub items: Vec<InstalledPackageSummary>,
}

/// Required fields for InstalledPackageCollection
pub struct InstalledPackageCollectionRequired {
    /// List of installed packages.
    pub items: Vec<InstalledPackageSummary>,
}

impl InstalledPackageCollection {
    /// Create a new InstalledPackageCollection with required fields
    pub fn new(required: InstalledPackageCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<InstalledPackageSummary>) -> Self {
        self.items = value;
        self
    }
}
