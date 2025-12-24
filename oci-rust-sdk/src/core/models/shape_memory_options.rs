use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// For a flexible shape, the amount of memory available for instances that use this shape. <p> If this field is null, then this shape has a fixed amount of memory equivalent to {@code memoryInGBs}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeMemoryOptions {
    /// The minimum amount of memory, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_in_g_bs: Option<i64>,

    /// The maximum amount of memory, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_in_g_bs: Option<i64>,

    /// The default amount of memory per OCPU available for this shape, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_per_ocpu_in_g_bs: Option<i64>,

    /// The minimum amount of memory per OCPU available for this shape, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_per_ocpu_in_g_bs: Option<i64>,

    /// The maximum amount of memory per OCPU available for this shape, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_per_ocpu_in_g_bs: Option<i64>,

    /// The maximum amount of memory per NUMA node, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_per_numa_node_in_g_bs: Option<i64>,
}

impl ShapeMemoryOptions {
    /// Create a new ShapeMemoryOptions
    pub fn new() -> Self {
        Self {
            min_in_g_bs: None,

            max_in_g_bs: None,

            default_per_ocpu_in_g_bs: None,

            min_per_ocpu_in_g_bs: None,

            max_per_ocpu_in_g_bs: None,

            max_per_numa_node_in_g_bs: None,
        }
    }

    /// Set min_in_g_bs
    pub fn set_min_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.min_in_g_bs = value;
        self
    }

    /// Set max_in_g_bs
    pub fn set_max_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.max_in_g_bs = value;
        self
    }

    /// Set default_per_ocpu_in_g_bs
    pub fn set_default_per_ocpu_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.default_per_ocpu_in_g_bs = value;
        self
    }

    /// Set min_per_ocpu_in_g_bs
    pub fn set_min_per_ocpu_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.min_per_ocpu_in_g_bs = value;
        self
    }

    /// Set max_per_ocpu_in_g_bs
    pub fn set_max_per_ocpu_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.max_per_ocpu_in_g_bs = value;
        self
    }

    /// Set max_per_numa_node_in_g_bs
    pub fn set_max_per_numa_node_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.max_per_numa_node_in_g_bs = value;
        self
    }

    /// Set min_in_g_bs (unwraps Option)
    pub fn with_min_in_g_bs(mut self, value: i64) -> Self {
        self.min_in_g_bs = Some(value);
        self
    }

    /// Set max_in_g_bs (unwraps Option)
    pub fn with_max_in_g_bs(mut self, value: i64) -> Self {
        self.max_in_g_bs = Some(value);
        self
    }

    /// Set default_per_ocpu_in_g_bs (unwraps Option)
    pub fn with_default_per_ocpu_in_g_bs(mut self, value: i64) -> Self {
        self.default_per_ocpu_in_g_bs = Some(value);
        self
    }

    /// Set min_per_ocpu_in_g_bs (unwraps Option)
    pub fn with_min_per_ocpu_in_g_bs(mut self, value: i64) -> Self {
        self.min_per_ocpu_in_g_bs = Some(value);
        self
    }

    /// Set max_per_ocpu_in_g_bs (unwraps Option)
    pub fn with_max_per_ocpu_in_g_bs(mut self, value: i64) -> Self {
        self.max_per_ocpu_in_g_bs = Some(value);
        self
    }

    /// Set max_per_numa_node_in_g_bs (unwraps Option)
    pub fn with_max_per_numa_node_in_g_bs(mut self, value: i64) -> Self {
        self.max_per_numa_node_in_g_bs = Some(value);
        self
    }
}

impl Default for ShapeMemoryOptions {
    fn default() -> Self {
        Self::new()
    }
}
