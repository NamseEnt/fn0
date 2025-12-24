use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// OCPU options for an image and shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageOcpuConstraints {
    /// The minimum number of OCPUs supported for this image and shape. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,

    /// The maximum number of OCPUs supported for this image and shape. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,
}

impl ImageOcpuConstraints {
    /// Create a new ImageOcpuConstraints
    pub fn new() -> Self {
        Self {
            min: None,

            max: None,
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
}

impl Default for ImageOcpuConstraints {
    fn default() -> Self {
        Self::new()
    }
}
