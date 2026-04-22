use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object that defines a software package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwarePackage {
    /// Package name.
    pub display_name: String,

    /// Unique identifier for the package. Note that this is not an OCID.
    pub name: String,

    /// Type of the package.
    #[serde(rename = "type")]
    pub r#type: String,

    /// Version of the package.
    pub version: String,

    /// The architecture for which this software was built
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<SoftwarePackageArchitecture>,

    /// The date and time the package was last modified (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<String>,

    /// Checksum of the package.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,

    /// Type of the checksum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_type: Option<String>,

    /// Description of the package.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Size of the package in bytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_bytes: Option<i64>,

    /// List of dependencies for the software package.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<SoftwarePackageDependency>>,

    /// List of files for the software package.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<SoftwarePackageFile>>,

    /// List of software sources that provide the software package. This property is deprecated and it will be removed in a future API release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_sources: Option<Vec<SoftwareSourceDetails>>,

    /// Indicates whether this package is the latest version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_latest: Option<bool>,

    /// The OS families the package belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_families: Option<Vec<OsFamily>>,
}

/// Required fields for SoftwarePackage
pub struct SoftwarePackageRequired {
    /// Package name.
    pub display_name: String,

    /// Unique identifier for the package. Note that this is not an OCID.
    pub name: String,

    /// Type of the package.
    pub r#type: String,

    /// Version of the package.
    pub version: String,
}

impl SoftwarePackage {
    /// Create a new SoftwarePackage with required fields
    pub fn new(required: SoftwarePackageRequired) -> Self {
        Self {
            display_name: required.display_name,

            name: required.name,

            r#type: required.r#type,

            version: required.version,

            architecture: None,

            last_modified_date: None,

            checksum: None,

            checksum_type: None,

            description: None,

            size_in_bytes: None,

            dependencies: None,

            files: None,

            software_sources: None,

            is_latest: None,

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

    /// Set last_modified_date
    pub fn set_last_modified_date(mut self, value: Option<String>) -> Self {
        self.last_modified_date = value;
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

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set size_in_bytes
    pub fn set_size_in_bytes(mut self, value: Option<i64>) -> Self {
        self.size_in_bytes = value;
        self
    }

    /// Set dependencies
    pub fn set_dependencies(mut self, value: Option<Vec<SoftwarePackageDependency>>) -> Self {
        self.dependencies = value;
        self
    }

    /// Set files
    pub fn set_files(mut self, value: Option<Vec<SoftwarePackageFile>>) -> Self {
        self.files = value;
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

    /// Set last_modified_date (unwraps Option)
    pub fn with_last_modified_date(mut self, value: impl Into<String>) -> Self {
        self.last_modified_date = Some(value.into());
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

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set size_in_bytes (unwraps Option)
    pub fn with_size_in_bytes(mut self, value: i64) -> Self {
        self.size_in_bytes = Some(value);
        self
    }

    /// Set dependencies (unwraps Option)
    pub fn with_dependencies(mut self, value: Vec<SoftwarePackageDependency>) -> Self {
        self.dependencies = Some(value);
        self
    }

    /// Set files (unwraps Option)
    pub fn with_files(mut self, value: Vec<SoftwarePackageFile>) -> Self {
        self.files = Some(value);
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

    /// Set os_families (unwraps Option)
    pub fn with_os_families(mut self, value: Vec<OsFamily>) -> Self {
        self.os_families = Some(value);
        self
    }
}
