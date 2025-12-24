use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a software package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwarePackageSummary {
    /// Package name.
    pub display_name: String,

    /// Unique identifier for the package. Note that this is not an OCID.
    pub name: String,

    /// Type of the package.
    #[serde(rename = "type")]
    pub r#type: String,

    /// Version of the package.
    pub version: String,

    /// The architecture for which this software was built.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<SoftwarePackageArchitecture>,

    /// Checksum of the package.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,

    /// Type of the checksum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_type: Option<String>,

    /// Indicates whether this package is the latest version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_latest: Option<bool>,

    /// List of software sources that provide the software package. This property is deprecated and it will be removed in a future API release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_sources: Option<Vec<SoftwareSourceDetails>>,

    /// The OS families the package belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_families: Option<Vec<OsFamily>>,
}

/// Required fields for SoftwarePackageSummary
pub struct SoftwarePackageSummaryRequired {
    /// Package name.
    pub display_name: String,

    /// Unique identifier for the package. Note that this is not an OCID.
    pub name: String,

    /// Type of the package.
    pub r#type: String,

    /// Version of the package.
    pub version: String,
}

impl SoftwarePackageSummary {
    /// Create a new SoftwarePackageSummary with required fields
    pub fn new(required: SoftwarePackageSummaryRequired) -> Self {
        Self {
            display_name: required.display_name,

            name: required.name,

            r#type: required.r#type,

            version: required.version,

            architecture: None,

            checksum: None,

            checksum_type: None,

            is_latest: None,

            software_sources: None,

            os_families: None,
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
    pub fn set_architecture(mut self, value: Option<SoftwarePackageArchitecture>) -> Self {
        self.architecture = value;
        self
    }

    /// Set checksum
    pub fn set_checksum(mut self, value: Option<String>) -> Self {
        self.checksum = value;
        self
    }

    /// Set checksum_type
    pub fn set_checksum_type(mut self, value: Option<String>) -> Self {
        self.checksum_type = value;
        self
    }

    /// Set is_latest
    pub fn set_is_latest(mut self, value: Option<bool>) -> Self {
        self.is_latest = value;
        self
    }

    /// Set software_sources
    pub fn set_software_sources(mut self, value: Option<Vec<SoftwareSourceDetails>>) -> Self {
        self.software_sources = value;
        self
    }

    /// Set os_families
    pub fn set_os_families(mut self, value: Option<Vec<OsFamily>>) -> Self {
        self.os_families = value;
        self
    }

    /// Set architecture (unwraps Option)
    pub fn with_architecture(mut self, value: SoftwarePackageArchitecture) -> Self {
        self.architecture = Some(value);
        self
    }

    /// Set checksum (unwraps Option)
    pub fn with_checksum(mut self, value: impl Into<String>) -> Self {
        self.checksum = Some(value.into());
        self
    }

    /// Set checksum_type (unwraps Option)
    pub fn with_checksum_type(mut self, value: impl Into<String>) -> Self {
        self.checksum_type = Some(value.into());
        self
    }

    /// Set is_latest (unwraps Option)
    pub fn with_is_latest(mut self, value: bool) -> Self {
        self.is_latest = Some(value);
        self
    }

    /// Set software_sources (unwraps Option)
    pub fn with_software_sources(mut self, value: Vec<SoftwareSourceDetails>) -> Self {
        self.software_sources = Some(value);
        self
    }

    /// Set os_families (unwraps Option)
    pub fn with_os_families(mut self, value: Vec<OsFamily>) -> Self {
        self.os_families = Some(value);
        self
    }
}
