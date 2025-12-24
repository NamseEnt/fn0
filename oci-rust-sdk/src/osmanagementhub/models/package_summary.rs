use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a software package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageSummary {
    /// Package name.
    pub display_name: String,

    /// Unique identifier for the package.
    pub name: String,

    /// Type of the package.
    #[serde(rename = "type")]
    pub r#type: String,

    /// Version of the installed package.
    pub version: String,

    pub package_classification: String,

    /// The architecture for which this package was built.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<ArchType>,

    /// List of software sources that provide the software package.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_sources: Option<Vec<SoftwareSourceDetails>>,
}

/// Required fields for PackageSummary
pub struct PackageSummaryRequired {
    /// Package name.
    pub display_name: String,

    /// Unique identifier for the package.
    pub name: String,

    /// Type of the package.
    pub r#type: String,

    /// Version of the installed package.
    pub version: String,

    pub package_classification: String,
}

impl PackageSummary {
    /// Create a new PackageSummary with required fields
    pub fn new(required: PackageSummaryRequired) -> Self {
        Self {
            display_name: required.display_name,

            name: required.name,

            r#type: required.r#type,

            version: required.version,

            package_classification: required.package_classification,

            architecture: None,

            software_sources: None,
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
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set version
    pub fn set_version(mut self, value: String) -> Self {
        self.version = value;
        self
    }

    /// Set architecture
    pub fn set_architecture(mut self, value: Option<ArchType>) -> Self {
        self.architecture = value;
        self
    }

    /// Set software_sources
    pub fn set_software_sources(mut self, value: Option<Vec<SoftwareSourceDetails>>) -> Self {
        self.software_sources = value;
        self
    }

    /// Set package_classification
    pub fn set_package_classification(mut self, value: String) -> Self {
        self.package_classification = value;
        self
    }

    /// Set architecture (unwraps Option)
    pub fn with_architecture(mut self, value: ArchType) -> Self {
        self.architecture = Some(value);
        self
    }

    /// Set software_sources (unwraps Option)
    pub fn with_software_sources(mut self, value: Vec<SoftwareSourceDetails>) -> Self {
        self.software_sources = Some(value);
        self
    }
}
