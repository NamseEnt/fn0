use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a filter for groups from a vendor software source to create or update a custom software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageGroupFilter {
    /// The type of the filter.
    pub filter_type: FilterType,

    /// List of package group names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_groups: Option<Vec<String>>,
}

/// Required fields for PackageGroupFilter
pub struct PackageGroupFilterRequired {
    /// The type of the filter.
    pub filter_type: FilterType,
}

impl PackageGroupFilter {
    /// Create a new PackageGroupFilter with required fields
    pub fn new(required: PackageGroupFilterRequired) -> Self {
        Self {
            filter_type: required.filter_type,

            package_groups: None,
        }
    }

    /// Set package_groups
    pub fn set_package_groups(mut self, value: Option<Vec<String>>) -> Self {
        self.package_groups = value;
        self
    }

    /// Set filter_type
    pub fn set_filter_type(mut self, value: FilterType) -> Self {
        self.filter_type = value;
        self
    }

    /// Set package_groups (unwraps Option)
    pub fn with_package_groups(mut self, value: Vec<String>) -> Self {
        self.package_groups = Some(value);
        self
    }
}
