use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the software sources and search parameters to get a list of associated package groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSoftwareSourcePackageGroupsDetails {
    /// List of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub software_source_ids: Vec<String>,

    /// The sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SearchSoftwareSourcePackageGroupsDetailsSortOrder>,

    /// The field to sort by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SearchSoftwareSourcePackageGroupsDetailsSortBy>,

    /// A filter that returns package groups with a name that contains the given string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_contains: Option<String>,

    /// Indicates if this is a group, category or environment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_type: Option<String>,
}

/// Required fields for SearchSoftwareSourcePackageGroupsDetails
pub struct SearchSoftwareSourcePackageGroupsDetailsRequired {
    /// List of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub software_source_ids: Vec<String>,
}

impl SearchSoftwareSourcePackageGroupsDetails {
    /// Create a new SearchSoftwareSourcePackageGroupsDetails with required fields
    pub fn new(required: SearchSoftwareSourcePackageGroupsDetailsRequired) -> Self {
        Self {
            software_source_ids: required.software_source_ids,

            sort_order: None,

            sort_by: None,

            name_contains: None,

            group_type: None,
        }
    }

    /// Set software_source_ids
    pub fn set_software_source_ids(mut self, value: Vec<String>) -> Self {
        self.software_source_ids = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(
        mut self,
        value: Option<SearchSoftwareSourcePackageGroupsDetailsSortOrder>,
    ) -> Self {
        self.sort_order = value;
        self
    }

    /// Set sort_by
    pub fn set_sort_by(
        mut self,
        value: Option<SearchSoftwareSourcePackageGroupsDetailsSortBy>,
    ) -> Self {
        self.sort_by = value;
        self
    }

    /// Set name_contains
    pub fn set_name_contains(mut self, value: Option<String>) -> Self {
        self.name_contains = value;
        self
    }

    /// Set group_type
    pub fn set_group_type(mut self, value: Option<String>) -> Self {
        self.group_type = value;
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(
        mut self,
        value: SearchSoftwareSourcePackageGroupsDetailsSortOrder,
    ) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: SearchSoftwareSourcePackageGroupsDetailsSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set name_contains (unwraps Option)
    pub fn with_name_contains(mut self, value: impl Into<String>) -> Self {
        self.name_contains = Some(value.into());
        self
    }

    /// Set group_type (unwraps Option)
    pub fn with_group_type(mut self, value: impl Into<String>) -> Self {
        self.group_type = Some(value.into());
        self
    }
}
