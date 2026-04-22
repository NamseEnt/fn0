use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to update a private software source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePrivateSoftwareSourceDetails {
    pub software_source_type: String,

    /// URL for the private software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// URI of the GPG key for this software source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpg_key_url: Option<String>,

    /// Whether signature verification should be done for the software source.
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

/// Required fields for UpdatePrivateSoftwareSourceDetails
pub struct UpdatePrivateSoftwareSourceDetailsRequired {
    pub software_source_type: String,
}

impl UpdatePrivateSoftwareSourceDetails {
    /// Create a new UpdatePrivateSoftwareSourceDetails with required fields
    pub fn new(required: UpdatePrivateSoftwareSourceDetailsRequired) -> Self {
        Self {
            software_source_type: required.software_source_type,

            url: None,

            gpg_key_url: None,

            is_gpg_check_enabled: None,

            is_ssl_verify_enabled: None,

            advanced_repo_options: None,

            is_mirror_sync_allowed: None,
        }
    }

    /// Set url
    pub fn set_url(mut self, value: Option<String>) -> Self {
        self.url = value;
        self
    }

    /// Set gpg_key_url
    pub fn set_gpg_key_url(mut self, value: Option<String>) -> Self {
        self.gpg_key_url = value;
        self
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

    /// Set url (unwraps Option)
    pub fn with_url(mut self, value: impl Into<String>) -> Self {
        self.url = Some(value.into());
        self
    }

    /// Set gpg_key_url (unwraps Option)
    pub fn with_gpg_key_url(mut self, value: impl Into<String>) -> Self {
        self.gpg_key_url = Some(value.into());
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
