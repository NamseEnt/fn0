use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An autotune policy automatically tunes the volume's performace based on the type of the policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutotunePolicy {
    pub autotune_type: String,
}

/// Required fields for AutotunePolicy
pub struct AutotunePolicyRequired {
    pub autotune_type: String,
}

impl AutotunePolicy {
    /// Create a new AutotunePolicy with required fields
    pub fn new(required: AutotunePolicyRequired) -> Self {
        Self {
            autotune_type: required.autotune_type,
        }
    }

    /// Set autotune_type
    pub fn set_autotune_type(mut self, value: String) -> Self {
        self.autotune_type = value;
        self
    }
}
