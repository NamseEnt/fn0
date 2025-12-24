use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for an available package for a managed instance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceGroupAvailablePackageSummary {
    /// Package name.
    pub display_name: String,

    /// Unique identifier for the package. Note that this is not an OCID.
    pub name: String,

    /// Type of the package.
    #[serde(rename = "type")]
    pub r#type: String,

    /// Version of the available package.
    pub version: String,

    /// The architecture for which this package was built.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<ArchType>,

    /// List of software sources that provide the software package.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_sources: Option<Vec<SoftwareSourceDetails>>,

    /// Indicates whether this is the latest package version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_latest: Option<bool>,
}

/// Required fields for ManagedInstanceGroupAvailablePackageSummary
pub struct ManagedInstanceGroupAvailablePackageSummaryRequired {
    /// Package name.
    pub display_name: String,

    /// Unique identifier for the package. Note that this is not an OCID.
    pub name: String,

    /// Type of the package.
    pub r#type: String,

    /// Version of the available package.
    pub version: String,
}

impl ManagedInstanceGroupAvailablePackageSummary {
    /// Create a new ManagedInstanceGroupAvailablePackageSummary with required fields
    pub fn new(required: ManagedInstanceGroupAvailablePackageSummaryRequired) -> Self {
        Self {
            display_name: required.display_name,

            name: required.name,

            r#type: required.r#type,

            version: required.version,

            architecture: None,

            software_sources: None,

            is_latest: None,
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

    /// Set is_latest
    pub fn set_is_latest(mut self, value: Option<bool>) -> Self {
        self.is_latest = value;
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

    /// Set is_latest (unwraps Option)
    pub fn with_is_latest(mut self, value: bool) -> Self {
        self.is_latest = Some(value);
        self
    }
}
