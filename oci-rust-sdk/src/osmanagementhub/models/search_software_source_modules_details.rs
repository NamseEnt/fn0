use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to search for a set of modules from a list software sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSoftwareSourceModulesDetails {
    /// List of sofware source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub software_source_ids: Vec<String>,

    /// The sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SearchSoftwareSourceModulesDetailsSortOrder>,

    /// The name of a module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// A filter to return modules with a name that contains the given string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_contains: Option<String>,

    /// The field to sort by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SearchSoftwareSourceModulesDetailsSortBy>,
}

/// Required fields for SearchSoftwareSourceModulesDetails
pub struct SearchSoftwareSourceModulesDetailsRequired {
    /// List of sofware source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub software_source_ids: Vec<String>,
}

impl SearchSoftwareSourceModulesDetails {
    /// Create a new SearchSoftwareSourceModulesDetails with required fields
    pub fn new(required: SearchSoftwareSourceModulesDetailsRequired) -> Self {
        Self {
            software_source_ids: required.software_source_ids,

            sort_order: None,

            name: None,

            name_contains: None,

            sort_by: None,
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
        value: Option<SearchSoftwareSourceModulesDetailsSortOrder>,
    ) -> Self {
        self.sort_order = value;
        self
    }

    /// Set name
    pub fn set_name(mut self, value: Option<String>) -> Self {
        self.name = value;
        self
    }

    /// Set name_contains
    pub fn set_name_contains(mut self, value: Option<String>) -> Self {
        self.name_contains = value;
        self
    }

    /// Set sort_by
    pub fn set_sort_by(mut self, value: Option<SearchSoftwareSourceModulesDetailsSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: SearchSoftwareSourceModulesDetailsSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set name (unwraps Option)
    pub fn with_name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    /// Set name_contains (unwraps Option)
    pub fn with_name_contains(mut self, value: impl Into<String>) -> Self {
        self.name_contains = Some(value.into());
        self
    }

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: SearchSoftwareSourceModulesDetailsSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }
}
