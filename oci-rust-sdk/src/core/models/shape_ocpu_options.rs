use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// For a flexible shape, the number of OCPUs available for instances that use this shape. <p> If this field is null, then this shape has a fixed number of OCPUs equal to {@code ocpus}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeOcpuOptions {
    /// The minimum number of OCPUs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,

    /// The maximum number of OCPUs. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,

    /// The maximum number of cores available per NUMA node. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_per_numa_node: Option<i64>,
}

impl ShapeOcpuOptions {
    /// Create a new ShapeOcpuOptions
    pub fn new() -> Self {
        Self {
            min: None,

            max: None,

            max_per_numa_node: None,
        }
    }

    /// Set min
    pub fn set_min(mut self, value: Option<i64>) -> Self {
        self.min = value;
        self
    }

    /// Set max
    pub fn set_max(mut self, value: Option<i64>) -> Self {
        self.max = value;
        self
    }

    /// Set max_per_numa_node
    pub fn set_max_per_numa_node(mut self, value: Option<i64>) -> Self {
        self.max_per_numa_node = value;
        self
    }

    /// Set min (unwraps Option)
    pub fn with_min(mut self, value: i64) -> Self {
        self.min = Some(value);
        self
    }

    /// Set max (unwraps Option)
    pub fn with_max(mut self, value: i64) -> Self {
        self.max = Some(value);
        self
    }

    /// Set max_per_numa_node (unwraps Option)
    pub fn with_max_per_numa_node(mut self, value: i64) -> Self {
        self.max_per_numa_node = Some(value);
        self
    }
}

impl Default for ShapeOcpuOptions {
    fn default() -> Self {
        Self::new()
    }
}
