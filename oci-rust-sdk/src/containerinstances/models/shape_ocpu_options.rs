use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// For a flexible shape, the number of OCPUs available for container instances that use this shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeOcpuOptions {
    /// The minimum number of OCPUs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub min: i64,

    /// The maximum number of OCPUs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max: i64,
}

/// Required fields for ShapeOcpuOptions
pub struct ShapeOcpuOptionsRequired {
    /// The minimum number of OCPUs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub min: i64,

    /// The maximum number of OCPUs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max: i64,
}

impl ShapeOcpuOptions {
    /// Create a new ShapeOcpuOptions with required fields
    pub fn new(required: ShapeOcpuOptionsRequired) -> Self {
        Self {
            min: required.min,

            max: required.max,
        }
    }

    /// Set min
    pub fn set_min(mut self, value: i64) -> Self {
        self.min = value;
        self
    }

    /// Set max
    pub fn set_max(mut self, value: i64) -> Self {
        self.max = value;
        self
    }
}
