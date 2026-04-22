use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to apply filters to a vendor software source to create or update a custom software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomSoftwareSourceFilter {
    /// The list of package filters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_filters: Option<Vec<PackageFilter>>,

    /// The list of module stream/profile filters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_stream_profile_filters: Option<Vec<ModuleStreamProfileFilter>>,

    /// The list of group filters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_group_filters: Option<Vec<PackageGroupFilter>>,
}

impl CustomSoftwareSourceFilter {
    /// Create a new CustomSoftwareSourceFilter
    pub fn new() -> Self {
        Self {
            package_filters: None,

            module_stream_profile_filters: None,

            package_group_filters: None,
        }
    }

    /// Set package_filters
    pub fn set_package_filters(mut self, value: Option<Vec<PackageFilter>>) -> Self {
        self.package_filters = value;
        self
    }

    /// Set module_stream_profile_filters
    pub fn set_module_stream_profile_filters(
        mut self,
        value: Option<Vec<ModuleStreamProfileFilter>>,
    ) -> Self {
        self.module_stream_profile_filters = value;
        self
    }

    /// Set package_group_filters
    pub fn set_package_group_filters(mut self, value: Option<Vec<PackageGroupFilter>>) -> Self {
        self.package_group_filters = value;
        self
    }

    /// Set package_filters (unwraps Option)
    pub fn with_package_filters(mut self, value: Vec<PackageFilter>) -> Self {
        self.package_filters = Some(value);
        self
    }

    /// Set module_stream_profile_filters (unwraps Option)
    pub fn with_module_stream_profile_filters(
        mut self,
        value: Vec<ModuleStreamProfileFilter>,
    ) -> Self {
        self.module_stream_profile_filters = Some(value);
        self
    }

    /// Set package_group_filters (unwraps Option)
    pub fn with_package_group_filters(mut self, value: Vec<PackageGroupFilter>) -> Self {
        self.package_group_filters = Some(value);
        self
    }
}

impl Default for CustomSoftwareSourceFilter {
    fn default() -> Self {
        Self::new()
    }
}
