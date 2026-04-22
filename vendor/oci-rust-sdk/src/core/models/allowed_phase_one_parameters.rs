use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Allowed phase one parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowedPhaseOneParameters {
    /// Allowed phase one encryption algorithms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_algorithms: Option<Vec<String>>,

    /// Allowed phase one authentication algorithms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_algorithms: Option<Vec<String>>,

    /// Allowed phase one Diffie-Hellman groups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dh_groups: Option<Vec<String>>,
}

impl AllowedPhaseOneParameters {
    /// Create a new AllowedPhaseOneParameters
    pub fn new() -> Self {
        Self {
            encryption_algorithms: None,

            authentication_algorithms: None,

            dh_groups: None,
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

    /// Set dh_groups
    pub fn set_dh_groups(mut self, value: Option<Vec<String>>) -> Self {
        self.dh_groups = value;
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

    /// Set dh_groups (unwraps Option)
    pub fn with_dh_groups(mut self, value: Vec<String>) -> Self {
        self.dh_groups = Some(value);
        self
    }
}

impl Default for AllowedPhaseOneParameters {
    fn default() -> Self {
        Self::new()
    }
}
