use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Default phase two parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultPhaseTwoParameters {
    /// Default phase two encryption algorithms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_encryption_algorithms: Option<Vec<String>>,

    /// Default phase two authentication algorithms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_authentication_algorithms: Option<Vec<String>>,

    /// Default perfect forward secrecy Diffie-Hellman groups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_pfs_dh_group: Option<String>,
}

impl DefaultPhaseTwoParameters {
    /// Create a new DefaultPhaseTwoParameters
    pub fn new() -> Self {
        Self {
            default_encryption_algorithms: None,

            default_authentication_algorithms: None,

            default_pfs_dh_group: None,
        }
    }

    /// Set default_encryption_algorithms
    pub fn set_default_encryption_algorithms(mut self, value: Option<Vec<String>>) -> Self {
        self.default_encryption_algorithms = value;
        self
    }

    /// Set default_authentication_algorithms
    pub fn set_default_authentication_algorithms(mut self, value: Option<Vec<String>>) -> Self {
        self.default_authentication_algorithms = value;
        self
    }

    /// Set default_pfs_dh_group
    pub fn set_default_pfs_dh_group(mut self, value: Option<String>) -> Self {
        self.default_pfs_dh_group = value;
        self
    }

    /// Set default_encryption_algorithms (unwraps Option)
    pub fn with_default_encryption_algorithms(mut self, value: Vec<String>) -> Self {
        self.default_encryption_algorithms = Some(value);
        self
    }

    /// Set default_authentication_algorithms (unwraps Option)
    pub fn with_default_authentication_algorithms(mut self, value: Vec<String>) -> Self {
        self.default_authentication_algorithms = Some(value);
        self
    }

    /// Set default_pfs_dh_group (unwraps Option)
    pub fn with_default_pfs_dh_group(mut self, value: impl Into<String>) -> Self {
        self.default_pfs_dh_group = Some(value.into());
        self
    }
}

impl Default for DefaultPhaseTwoParameters {
    fn default() -> Self {
        Self::new()
    }
}
