use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// For a flexible shape, the amount of networking bandwidth available for container instances that use this shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeNetworkingBandwidthOptions {
    /// The minimum amount of networking bandwidth, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub min_in_gbps: i64,

    /// The maximum amount of networking bandwidth, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max_in_gbps: i64,

    /// The default amount of networking bandwidth per OCPU, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub default_per_ocpu_in_gbps: i64,
}

/// Required fields for ShapeNetworkingBandwidthOptions
pub struct ShapeNetworkingBandwidthOptionsRequired {
    /// The minimum amount of networking bandwidth, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub min_in_gbps: i64,

    /// The maximum amount of networking bandwidth, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max_in_gbps: i64,

    /// The default amount of networking bandwidth per OCPU, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub default_per_ocpu_in_gbps: i64,
}

impl ShapeNetworkingBandwidthOptions {
    /// Create a new ShapeNetworkingBandwidthOptions with required fields
    pub fn new(required: ShapeNetworkingBandwidthOptionsRequired) -> Self {
        Self {
            min_in_gbps: required.min_in_gbps,

            max_in_gbps: required.max_in_gbps,

            default_per_ocpu_in_gbps: required.default_per_ocpu_in_gbps,
        }
    }

    /// Set min_in_gbps
    pub fn set_min_in_gbps(mut self, value: i64) -> Self {
        self.min_in_gbps = value;
        self
    }

    /// Set max_in_gbps
    pub fn set_max_in_gbps(mut self, value: i64) -> Self {
        self.max_in_gbps = value;
        self
    }

    /// Set default_per_ocpu_in_gbps
    pub fn set_default_per_ocpu_in_gbps(mut self, value: i64) -> Self {
        self.default_per_ocpu_in_gbps = value;
        self
    }
}
