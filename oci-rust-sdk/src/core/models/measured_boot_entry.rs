use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// One Trusted Platform Module (TPM) Platform Configuration Register (PCR) entry. The entry might be measured during boot, or specified in a policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasuredBootEntry {
    /// The index of the policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pcr_index: Option<String>,

    /// The hashed PCR value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The type of algorithm used to calculate the hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_algorithm: Option<String>,
}

impl MeasuredBootEntry {
    /// Create a new MeasuredBootEntry
    pub fn new() -> Self {
        Self {
            pcr_index: None,

            value: None,

            hash_algorithm: None,
        }
    }

    /// Set pcr_index
    pub fn set_pcr_index(mut self, value: Option<String>) -> Self {
        self.pcr_index = value;
        self
    }

    /// Set value
    pub fn set_value(mut self, value: Option<String>) -> Self {
        self.value = value;
        self
    }

    /// Set hash_algorithm
    pub fn set_hash_algorithm(mut self, value: Option<String>) -> Self {
        self.hash_algorithm = value;
        self
    }

    /// Set pcr_index (unwraps Option)
    pub fn with_pcr_index(mut self, value: impl Into<String>) -> Self {
        self.pcr_index = Some(value.into());
        self
    }

    /// Set value (unwraps Option)
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set hash_algorithm (unwraps Option)
    pub fn with_hash_algorithm(mut self, value: impl Into<String>) -> Self {
        self.hash_algorithm = Some(value.into());
        self
    }
}

impl Default for MeasuredBootEntry {
    fn default() -> Self {
        Self::new()
    }
}
