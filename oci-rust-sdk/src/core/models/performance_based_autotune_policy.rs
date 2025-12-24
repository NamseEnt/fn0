use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// If a volume is being throttled at the current setting for a certain period of time, auto-tune will gradually increase the volume\u2019s performance limited up to Maximum VPUs/GB. After the volume has been idle at the current setting for a certain period of time, auto-tune will gradually decrease the volume\u2019s performance limited down to Default/Minimum VPUs/GB.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceBasedAutotunePolicy {
    /// This will be the maximum VPUs/GB performance level that the volume will be auto-tuned temporarily based on performance monitoring. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max_vpus_per_g_b: i64,

    pub autotune_type: String,
}

/// Required fields for PerformanceBasedAutotunePolicy
pub struct PerformanceBasedAutotunePolicyRequired {
    /// This will be the maximum VPUs/GB performance level that the volume will be auto-tuned temporarily based on performance monitoring. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max_vpus_per_g_b: i64,

    pub autotune_type: String,
}

impl PerformanceBasedAutotunePolicy {
    /// Create a new PerformanceBasedAutotunePolicy with required fields
    pub fn new(required: PerformanceBasedAutotunePolicyRequired) -> Self {
        Self {
            max_vpus_per_g_b: required.max_vpus_per_g_b,

            autotune_type: required.autotune_type,
        }
    }

    /// Set max_vpus_per_g_b
    pub fn set_max_vpus_per_g_b(mut self, value: i64) -> Self {
        self.max_vpus_per_g_b = value;
        self
    }

    /// Set autotune_type
    pub fn set_autotune_type(mut self, value: String) -> Self {
        self.autotune_type = value;
        self
    }
}
