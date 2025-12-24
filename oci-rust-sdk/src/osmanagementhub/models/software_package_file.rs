use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A file associated with a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwarePackageFile {
    /// File path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Type of the file.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// The date and time the file was last modified (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_modified: Option<DateTime<Utc>>,

    /// Checksum of the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,

    /// Type of the checksum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_type: Option<String>,

    /// Size of the file in bytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_in_bytes: Option<i64>,
}

impl SoftwarePackageFile {
    /// Create a new SoftwarePackageFile
    pub fn new() -> Self {
        Self {
            path: None,

            r#type: None,

            time_modified: None,

            checksum: None,

            checksum_type: None,

            size_in_bytes: None,
        }
    }

    /// Set path
    pub fn set_path(mut self, value: Option<String>) -> Self {
        self.path = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: Option<String>) -> Self {
        self.r#type = value;
        self
    }

    /// Set time_modified
    pub fn set_time_modified(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_modified = value;
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

    /// Set size_in_bytes
    pub fn set_size_in_bytes(mut self, value: Option<i64>) -> Self {
        self.size_in_bytes = value;
        self
    }

    /// Set path (unwraps Option)
    pub fn with_path(mut self, value: impl Into<String>) -> Self {
        self.path = Some(value.into());
        self
    }

    /// Set r#type (unwraps Option)
    pub fn with_type(mut self, value: impl Into<String>) -> Self {
        self.r#type = Some(value.into());
        self
    }

    /// Set time_modified (unwraps Option)
    pub fn with_time_modified(mut self, value: DateTime<Utc>) -> Self {
        self.time_modified = Some(value);
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

    /// Set size_in_bytes (unwraps Option)
    pub fn with_size_in_bytes(mut self, value: i64) -> Self {
        self.size_in_bytes = Some(value);
        self
    }
}

impl Default for SoftwarePackageFile {
    fn default() -> Self {
        Self::new()
    }
}
