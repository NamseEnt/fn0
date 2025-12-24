use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to search for a set of module streams from a list software sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSoftwareSourceModuleStreamsDetails {
    /// List of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub software_source_ids: Vec<String>,

    /// The sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SearchSoftwareSourceModuleStreamsDetailsSortOrder>,

    /// The name of a module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_name: Option<String>,

    /// The field to sort by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SearchSoftwareSourceModuleStreamsDetailsSortBy>,
}

/// Required fields for SearchSoftwareSourceModuleStreamsDetails
pub struct SearchSoftwareSourceModuleStreamsDetailsRequired {
    /// List of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm).
    pub software_source_ids: Vec<String>,
}

impl SearchSoftwareSourceModuleStreamsDetails {
    /// Create a new SearchSoftwareSourceModuleStreamsDetails with required fields
    pub fn new(required: SearchSoftwareSourceModuleStreamsDetailsRequired) -> Self {
        Self {
            software_source_ids: required.software_source_ids,

            sort_order: None,

            module_name: None,

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
        value: Option<SearchSoftwareSourceModuleStreamsDetailsSortOrder>,
    ) -> Self {
        self.sort_order = value;
        self
    }

    /// Set module_name
    pub fn set_module_name(mut self, value: Option<String>) -> Self {
        self.module_name = value;
        self
    }

    /// Set sort_by
    pub fn set_sort_by(
        mut self,
        value: Option<SearchSoftwareSourceModuleStreamsDetailsSortBy>,
    ) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(
        mut self,
        value: SearchSoftwareSourceModuleStreamsDetailsSortOrder,
    ) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set module_name (unwraps Option)
    pub fn with_module_name(mut self, value: impl Into<String>) -> Self {
        self.module_name = Some(value.into());
        self
    }

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: SearchSoftwareSourceModuleStreamsDetailsSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }
}
