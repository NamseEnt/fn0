use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The object that defines a third-party software source. A software source is a collection of packages. For more information, see [Managing Software Sources](https://docs.oracle.com/iaas/osmh/doc/software-sources.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThirdPartySoftwareSource {
    pub software_source_type: String,

    /// Whether signature verification should be done for the software source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_gpg_check_enabled: Option<bool>,

    /// Whether SSL validation needs to be turned on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_ssl_verify_enabled: Option<bool>,

    /// Advanced repository options for the software source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advanced_repo_options: Option<String>,

    /// Whether this software source can be synced to a management station
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mirror_sync_allowed: Option<bool>,
}

/// Required fields for ThirdPartySoftwareSource
pub struct ThirdPartySoftwareSourceRequired {
    pub software_source_type: String,
}

impl ThirdPartySoftwareSource {
    /// Create a new ThirdPartySoftwareSource with required fields
    pub fn new(required: ThirdPartySoftwareSourceRequired) -> Self {
        Self {
            software_source_type: required.software_source_type,

            is_gpg_check_enabled: None,

            is_ssl_verify_enabled: None,

            advanced_repo_options: None,

            is_mirror_sync_allowed: None,
        }
    }

    /// Set is_gpg_check_enabled
    pub fn set_is_gpg_check_enabled(mut self, value: Option<bool>) -> Self {
        self.is_gpg_check_enabled = value;
        self
    }

    /// Set is_ssl_verify_enabled
    pub fn set_is_ssl_verify_enabled(mut self, value: Option<bool>) -> Self {
        self.is_ssl_verify_enabled = value;
        self
    }

    /// Set advanced_repo_options
    pub fn set_advanced_repo_options(mut self, value: Option<String>) -> Self {
        self.advanced_repo_options = value;
        self
    }

    /// Set is_mirror_sync_allowed
    pub fn set_is_mirror_sync_allowed(mut self, value: Option<bool>) -> Self {
        self.is_mirror_sync_allowed = value;
        self
    }

    /// Set software_source_type
    pub fn set_software_source_type(mut self, value: String) -> Self {
        self.software_source_type = value;
        self
    }

    /// Set is_gpg_check_enabled (unwraps Option)
    pub fn with_is_gpg_check_enabled(mut self, value: bool) -> Self {
        self.is_gpg_check_enabled = Some(value);
        self
    }

    /// Set is_ssl_verify_enabled (unwraps Option)
    pub fn with_is_ssl_verify_enabled(mut self, value: bool) -> Self {
        self.is_ssl_verify_enabled = Some(value);
        self
    }

    /// Set advanced_repo_options (unwraps Option)
    pub fn with_advanced_repo_options(mut self, value: impl Into<String>) -> Self {
        self.advanced_repo_options = Some(value.into());
        self
    }

    /// Set is_mirror_sync_allowed (unwraps Option)
    pub fn with_is_mirror_sync_allowed(mut self, value: bool) -> Self {
        self.is_mirror_sync_allowed = Some(value);
        self
    }
}
