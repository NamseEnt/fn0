use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Volume's performace will be tuned to the lower cost settings once detached.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationDetachedVolumeAutotunePolicy {
    pub autotune_type: String,
}

/// Required fields for InstanceConfigurationDetachedVolumeAutotunePolicy
pub struct InstanceConfigurationDetachedVolumeAutotunePolicyRequired {
    pub autotune_type: String,
}

impl InstanceConfigurationDetachedVolumeAutotunePolicy {
    /// Create a new InstanceConfigurationDetachedVolumeAutotunePolicy with required fields
    pub fn new(required: InstanceConfigurationDetachedVolumeAutotunePolicyRequired) -> Self {
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
