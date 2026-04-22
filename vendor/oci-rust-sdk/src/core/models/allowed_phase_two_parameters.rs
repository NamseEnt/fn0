use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Allowed phase two parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowedPhaseTwoParameters {
    /// Allowed phase two encryption algorithms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_algorithms: Option<Vec<String>>,

    /// Allowed phase two authentication algorithms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_algorithms: Option<Vec<String>>,

    /// Allowed perfect forward secrecy Diffie-Hellman groups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfs_dh_groups: Option<Vec<String>>,
}

impl AllowedPhaseTwoParameters {
    /// Create a new AllowedPhaseTwoParameters
    pub fn new() -> Self {
        Self {
            encryption_algorithms: None,

            authentication_algorithms: None,

            pfs_dh_groups: None,
        }
    }

    /// Set encryption_algorithms
    pub fn set_encryption_algorithms(mut self, value: Option<Vec<String>>) -> Self {
        self.encryption_algorithms = value;
        self
    }

    /// Set authentication_algorithms
    pub fn set_authentication_algorithms(mut self, value: Option<Vec<String>>) -> Self {
        self.authentication_algorithms = value;
        self
    }

    /// Set pfs_dh_groups
    pub fn set_pfs_dh_groups(mut self, value: Option<Vec<String>>) -> Self {
        self.pfs_dh_groups = value;
        self
    }

    /// Set encryption_algorithms (unwraps Option)
    pub fn with_encryption_algorithms(mut self, value: Vec<String>) -> Self {
        self.encryption_algorithms = Some(value);
        self
    }

    /// Set authentication_algorithms (unwraps Option)
    pub fn with_authentication_algorithms(mut self, value: Vec<String>) -> Self {
        self.authentication_algorithms = Some(value);
        self
    }

    /// Set pfs_dh_groups (unwraps Option)
    pub fn with_pfs_dh_groups(mut self, value: Vec<String>) -> Self {
        self.pfs_dh_groups = Some(value);
        self
    }
}

impl Default for AllowedPhaseTwoParameters {
    fn default() -> Self {
        Self::new()
    }
}
