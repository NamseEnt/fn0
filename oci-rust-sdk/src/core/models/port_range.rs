use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortRange {
    /// The maximum port number, which must not be less than the minimum port number. To specify a single port number, set both the min and max to the same value. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max: i64,

    /// The minimum port number, which must not be greater than the maximum port number. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub min: i64,
}

/// Required fields for PortRange
pub struct PortRangeRequired {
    /// The maximum port number, which must not be less than the minimum port number. To specify a single port number, set both the min and max to the same value. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max: i64,

    /// The minimum port number, which must not be greater than the maximum port number. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub min: i64,
}

impl PortRange {
    /// Create a new PortRange with required fields
    pub fn new(required: PortRangeRequired) -> Self {
        Self {
            max: required.max,

            min: required.min,
        }
    }

    /// Set max
    pub fn set_max(mut self, value: i64) -> Self {
        self.max = value;
        self
    }

    /// Set min
    pub fn set_min(mut self, value: i64) -> Self {
        self.min = value;
        self
    }
}
