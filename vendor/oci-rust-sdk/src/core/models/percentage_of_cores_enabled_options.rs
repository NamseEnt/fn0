use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Configuration options for the percentage of cores enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PercentageOfCoresEnabledOptions {
    /// The minimum allowed percentage of cores enabled. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,

    /// The maximum allowed percentage of cores enabled. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,

    /// The default percentage of cores enabled. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<i64>,
}

impl PercentageOfCoresEnabledOptions {
    /// Create a new PercentageOfCoresEnabledOptions
    pub fn new() -> Self {
        Self {
            min: None,

            max: None,

            default_value: None,
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

    /// Set default_value
    pub fn set_default_value(mut self, value: Option<i64>) -> Self {
        self.default_value = value;
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

    /// Set default_value (unwraps Option)
    pub fn with_default_value(mut self, value: i64) -> Self {
        self.default_value = Some(value);
        self
    }
}

impl Default for PercentageOfCoresEnabledOptions {
    fn default() -> Self {
        Self::new()
    }
}
