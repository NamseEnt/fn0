use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information used to create the mirror configuration for a management station.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMirrorConfigurationDetails {
    /// Path to the data volume on the management station where software source mirrors are stored.
    pub directory: String,

    /// Default mirror listening port for http.
    pub port: String,

    /// Default mirror listening port for https.
    pub sslport: String,

    /// Path to the SSL cerfificate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sslcert: Option<String>,

    /// When enabled, the SSL certificate is verified whenever an instance installs or updates a package from a software source that is mirrored on the management station.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_sslverify_enabled: Option<bool>,
}

/// Required fields for CreateMirrorConfigurationDetails
pub struct CreateMirrorConfigurationDetailsRequired {
    /// Path to the data volume on the management station where software source mirrors are stored.
    pub directory: String,

    /// Default mirror listening port for http.
    pub port: String,

    /// Default mirror listening port for https.
    pub sslport: String,
}

impl CreateMirrorConfigurationDetails {
    /// Create a new CreateMirrorConfigurationDetails with required fields
    pub fn new(required: CreateMirrorConfigurationDetailsRequired) -> Self {
        Self {
            directory: required.directory,

            port: required.port,

            sslport: required.sslport,

            sslcert: None,

            is_sslverify_enabled: None,
        }
    }

    /// Set directory
    pub fn set_directory(mut self, value: String) -> Self {
        self.directory = value;
        self
    }

    /// Set port
    pub fn set_port(mut self, value: String) -> Self {
        self.port = value;
        self
    }

    /// Set sslport
    pub fn set_sslport(mut self, value: String) -> Self {
        self.sslport = value;
        self
    }

    /// Set sslcert
    pub fn set_sslcert(mut self, value: Option<String>) -> Self {
        self.sslcert = value;
        self
    }

    /// Set is_sslverify_enabled
    pub fn set_is_sslverify_enabled(mut self, value: Option<bool>) -> Self {
        self.is_sslverify_enabled = value;
        self
    }

    /// Set sslcert (unwraps Option)
    pub fn with_sslcert(mut self, value: impl Into<String>) -> Self {
        self.sslcert = Some(value.into());
        self
    }

    /// Set is_sslverify_enabled (unwraps Option)
    pub fn with_is_sslverify_enabled(mut self, value: bool) -> Self {
        self.is_sslverify_enabled = Some(value);
        self
    }
}
