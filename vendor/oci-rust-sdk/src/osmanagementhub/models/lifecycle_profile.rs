use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information for a lifecycle environment registration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleProfile {
    pub lifecycle_stage: LifecycleStageDetails,

    pub profile_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_environment: Option<LifecycleEnvironmentDetails>,
}

/// Required fields for LifecycleProfile
pub struct LifecycleProfileRequired {
    pub lifecycle_stage: LifecycleStageDetails,

    pub profile_type: String,
}

impl LifecycleProfile {
    /// Create a new LifecycleProfile with required fields
    pub fn new(required: LifecycleProfileRequired) -> Self {
        Self {
            lifecycle_stage: required.lifecycle_stage,

            profile_type: required.profile_type,

            lifecycle_environment: None,
        }
    }

    /// Set lifecycle_environment
    pub fn set_lifecycle_environment(mut self, value: Option<LifecycleEnvironmentDetails>) -> Self {
        self.lifecycle_environment = value;
        self
    }

    /// Set lifecycle_stage
    pub fn set_lifecycle_stage(mut self, value: LifecycleStageDetails) -> Self {
        self.lifecycle_stage = value;
        self
    }

    /// Set profile_type
    pub fn set_profile_type(mut self, value: String) -> Self {
        self.profile_type = value;
        self
    }

    /// Set lifecycle_environment (unwraps Option)
    pub fn with_lifecycle_environment(mut self, value: LifecycleEnvironmentDetails) -> Self {
        self.lifecycle_environment = Some(value);
        self
    }
}
