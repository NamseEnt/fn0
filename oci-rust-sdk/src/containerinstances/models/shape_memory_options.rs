use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// For a flexible shape, the amount of memory available for container instances that use this shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeMemoryOptions {
    /// The minimum amount of memory (GB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub min_in_g_bs: i64,

    /// The maximum amount of memory (GB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max_in_g_bs: i64,

    /// The default amount of memory per OCPU available for this shape (GB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub default_per_ocpu_in_g_bs: i64,

    /// The minimum amount of memory per OCPU available for this shape (GB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub min_per_ocpu_in_g_bs: i64,

    /// The maximum amount of memory per OCPU available for this shape (GB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max_per_ocpu_in_g_bs: i64,
}

/// Required fields for ShapeMemoryOptions
pub struct ShapeMemoryOptionsRequired {
    /// The minimum amount of memory (GB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub min_in_g_bs: i64,

    /// The maximum amount of memory (GB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max_in_g_bs: i64,

    /// The default amount of memory per OCPU available for this shape (GB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub default_per_ocpu_in_g_bs: i64,

    /// The minimum amount of memory per OCPU available for this shape (GB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub min_per_ocpu_in_g_bs: i64,

    /// The maximum amount of memory per OCPU available for this shape (GB). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub max_per_ocpu_in_g_bs: i64,
}

impl ShapeMemoryOptions {
    /// Create a new ShapeMemoryOptions with required fields
    pub fn new(required: ShapeMemoryOptionsRequired) -> Self {
        Self {
            min_in_g_bs: required.min_in_g_bs,

            max_in_g_bs: required.max_in_g_bs,

            default_per_ocpu_in_g_bs: required.default_per_ocpu_in_g_bs,

            min_per_ocpu_in_g_bs: required.min_per_ocpu_in_g_bs,

            max_per_ocpu_in_g_bs: required.max_per_ocpu_in_g_bs,
        }
    }

    /// Set min_in_g_bs
    pub fn set_min_in_g_bs(mut self, value: i64) -> Self {
        self.min_in_g_bs = value;
        self
    }

    /// Set max_in_g_bs
    pub fn set_max_in_g_bs(mut self, value: i64) -> Self {
        self.max_in_g_bs = value;
        self
    }

    /// Set default_per_ocpu_in_g_bs
    pub fn set_default_per_ocpu_in_g_bs(mut self, value: i64) -> Self {
        self.default_per_ocpu_in_g_bs = value;
        self
    }

    /// Set min_per_ocpu_in_g_bs
    pub fn set_min_per_ocpu_in_g_bs(mut self, value: i64) -> Self {
        self.min_per_ocpu_in_g_bs = value;
        self
    }

    /// Set max_per_ocpu_in_g_bs
    pub fn set_max_per_ocpu_in_g_bs(mut self, value: i64) -> Self {
        self.max_per_ocpu_in_g_bs = value;
        self
    }
}
