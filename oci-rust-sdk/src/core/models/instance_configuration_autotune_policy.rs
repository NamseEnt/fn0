use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An autotune policy automatically tunes the volume's performace based on the type of the policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationAutotunePolicy {
    pub autotune_type: String,
}

/// Required fields for InstanceConfigurationAutotunePolicy
pub struct InstanceConfigurationAutotunePolicyRequired {
    pub autotune_type: String,
}

impl InstanceConfigurationAutotunePolicy {
    /// Create a new InstanceConfigurationAutotunePolicy with required fields
    pub fn new(required: InstanceConfigurationAutotunePolicyRequired) -> Self {
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
