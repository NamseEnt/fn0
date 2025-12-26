use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Default phase one parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultPhaseOneParameters {
    /// Default phase one encryption algorithms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_encryption_algorithms: Option<Vec<String>>,

    /// Default phase one authentication algorithms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_authentication_algorithms: Option<Vec<String>>,

    /// Default phase one Diffie-Hellman groups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_dh_groups: Option<Vec<String>>,
}

impl DefaultPhaseOneParameters {
    /// Create a new DefaultPhaseOneParameters
    pub fn new() -> Self {
        Self {
            default_encryption_algorithms: None,

            default_authentication_algorithms: None,

            default_dh_groups: None,
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

    /// Set default_dh_groups
    pub fn set_default_dh_groups(mut self, value: Option<Vec<String>>) -> Self {
        self.default_dh_groups = value;
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

    /// Set default_dh_groups (unwraps Option)
    pub fn with_default_dh_groups(mut self, value: Vec<String>) -> Self {
        self.default_dh_groups = Some(value);
        self
    }
}

impl Default for DefaultPhaseOneParameters {
    fn default() -> Self {
        Self::new()
    }
}
