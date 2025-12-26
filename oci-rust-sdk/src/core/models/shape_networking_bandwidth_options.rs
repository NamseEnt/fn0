use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// For a flexible shape, the amount of networking bandwidth available for instances that use this shape. <p> If this field is null, then this shape has a fixed amount of bandwidth equivalent to {@code networkingBandwidthInGbps}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeNetworkingBandwidthOptions {
    /// The minimum amount of networking bandwidth, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_in_gbps: Option<i64>,

    /// The maximum amount of networking bandwidth, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_in_gbps: Option<i64>,

    /// The default amount of networking bandwidth per OCPU, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_per_ocpu_in_gbps: Option<i64>,
}

impl ShapeNetworkingBandwidthOptions {
    /// Create a new ShapeNetworkingBandwidthOptions
    pub fn new() -> Self {
        Self {
            min_in_gbps: None,

            max_in_gbps: None,

            default_per_ocpu_in_gbps: None,
        }
    }

    /// Set min_in_gbps
    pub fn set_min_in_gbps(mut self, value: Option<i64>) -> Self {
        self.min_in_gbps = value;
        self
    }

    /// Set max_in_gbps
    pub fn set_max_in_gbps(mut self, value: Option<i64>) -> Self {
        self.max_in_gbps = value;
        self
    }

    /// Set default_per_ocpu_in_gbps
    pub fn set_default_per_ocpu_in_gbps(mut self, value: Option<i64>) -> Self {
        self.default_per_ocpu_in_gbps = value;
        self
    }

    /// Set min_in_gbps (unwraps Option)
    pub fn with_min_in_gbps(mut self, value: i64) -> Self {
        self.min_in_gbps = Some(value);
        self
    }

    /// Set max_in_gbps (unwraps Option)
    pub fn with_max_in_gbps(mut self, value: i64) -> Self {
        self.max_in_gbps = Some(value);
        self
    }

    /// Set default_per_ocpu_in_gbps (unwraps Option)
    pub fn with_default_per_ocpu_in_gbps(mut self, value: i64) -> Self {
        self.default_per_ocpu_in_gbps = Some(value);
        self
    }
}

impl Default for ShapeNetworkingBandwidthOptions {
    fn default() -> Self {
        Self::new()
    }
}
