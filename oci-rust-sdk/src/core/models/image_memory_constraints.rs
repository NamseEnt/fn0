use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// For a flexible image and shape, the amount of memory supported for instances that use this image.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageMemoryConstraints {
    /// The minimum amount of memory, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_in_g_bs: Option<i64>,

    /// The maximum amount of memory, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_in_g_bs: Option<i64>,
}

impl ImageMemoryConstraints {
    /// Create a new ImageMemoryConstraints
    pub fn new() -> Self {
        Self {
            min_in_g_bs: None,

            max_in_g_bs: None,
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
}

impl Default for ImageMemoryConstraints {
    fn default() -> Self {
        Self::new()
    }
}
