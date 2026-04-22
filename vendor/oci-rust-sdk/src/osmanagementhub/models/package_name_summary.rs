use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information about a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageNameSummary {
    /// Full package name in NERVA format. This value should be unique.
    pub display_name: String,

    /// The name of the software package.
    pub name: String,

    /// Type of the package.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// The version of the software package.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// The CPU architecture type for which this package was built.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<ArchType>,
}

/// Required fields for PackageNameSummary
pub struct PackageNameSummaryRequired {
    /// Full package name in NERVA format. This value should be unique.
    pub display_name: String,

    /// The name of the software package.
    pub name: String,
}

impl PackageNameSummary {
    /// Create a new PackageNameSummary with required fields
    pub fn new(required: PackageNameSummaryRequired) -> Self {
        Self {
            display_name: required.display_name,

            name: required.name,

            r#type: None,

            version: None,

            architecture: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: Option<String>) -> Self {
        self.r#type = value;
        self
    }

    /// Set version
    pub fn set_version(mut self, value: Option<String>) -> Self {
        self.version = value;
        self
    }

    /// Set architecture
    pub fn set_architecture(mut self, value: Option<ArchType>) -> Self {
        self.architecture = value;
        self
    }

    /// Set r#type (unwraps Option)
    pub fn with_type(mut self, value: impl Into<String>) -> Self {
        self.r#type = Some(value.into());
        self
    }

    /// Set version (unwraps Option)
    pub fn with_version(mut self, value: impl Into<String>) -> Self {
        self.version = Some(value.into());
        self
    }

    /// Set architecture (unwraps Option)
    pub fn with_architecture(mut self, value: ArchType) -> Self {
        self.architecture = Some(value);
        self
    }
}
