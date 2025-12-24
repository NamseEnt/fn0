use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The set of package groups returned for the {@link #listPackageGroups(ListPackageGroupsRequest) listPackageGroups} {@link #searchSoftwareSourcePackageGroups(SearchSoftwareSourcePackageGroupsRequest) searchSoftwareSourcePackageGroups} operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageGroupCollection {
    /// List of package groups.
    pub items: Vec<PackageGroupSummary>,
}

/// Required fields for PackageGroupCollection
pub struct PackageGroupCollectionRequired {
    /// List of package groups.
    pub items: Vec<PackageGroupSummary>,
}

impl PackageGroupCollection {
    /// Create a new PackageGroupCollection with required fields
    pub fn new(required: PackageGroupCollectionRequired) -> Self {
        Self {
            items: required.items,
        }
    }

    /// Set items
    pub fn set_items(mut self, value: Vec<PackageGroupSummary>) -> Self {
        self.items = value;
        self
    }
}
