use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a filter for packages from a vendor software source to create or update a custom software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageFilter {
    /// The type of the filter.
    pub filter_type: FilterType,

    /// The package name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,

    /// The package name pattern.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_name_pattern: Option<String>,

    /// The package version, which is denoted by 'version-release', or 'epoch:version-release'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_version: Option<String>,
}

/// Required fields for PackageFilter
pub struct PackageFilterRequired {
    /// The type of the filter.
    pub filter_type: FilterType,
}

impl PackageFilter {
    /// Create a new PackageFilter with required fields
    pub fn new(required: PackageFilterRequired) -> Self {
        Self {
            filter_type: required.filter_type,

            package_name: None,

            package_name_pattern: None,

            package_version: None,
        }
    }

    /// Set package_name
    pub fn set_package_name(mut self, value: Option<String>) -> Self {
        self.package_name = value;
        self
    }

    /// Set package_name_pattern
    pub fn set_package_name_pattern(mut self, value: Option<String>) -> Self {
        self.package_name_pattern = value;
        self
    }

    /// Set package_version
    pub fn set_package_version(mut self, value: Option<String>) -> Self {
        self.package_version = value;
        self
    }

    /// Set filter_type
    pub fn set_filter_type(mut self, value: FilterType) -> Self {
        self.filter_type = value;
        self
    }

    /// Set package_name (unwraps Option)
    pub fn with_package_name(mut self, value: impl Into<String>) -> Self {
        self.package_name = Some(value.into());
        self
    }

    /// Set package_name_pattern (unwraps Option)
    pub fn with_package_name_pattern(mut self, value: impl Into<String>) -> Self {
        self.package_name_pattern = Some(value.into());
        self
    }

    /// Set package_version (unwraps Option)
    pub fn with_package_version(mut self, value: impl Into<String>) -> Self {
        self.package_version = Some(value.into());
        self
    }
}
