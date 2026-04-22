use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The retention period setting, specified in days. For more information, see [Setting Audit Log Retention Period](https://docs.oracle.com/iaas/Content/Audit/Tasks/settingretentionperiod.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    /// The retention period setting, specified in days. The minimum is 90, the maximum 365. <p> Example: {@code 90} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention_period_days: Option<i64>,
}

impl Configuration {
    /// Create a new Configuration
    pub fn new() -> Self {
        Self {
            retention_period_days: None,
        }
    }

    /// Set retention_period_days
    pub fn set_retention_period_days(mut self, value: Option<i64>) -> Self {
        self.retention_period_days = value;
        self
    }

    /// Set retention_period_days (unwraps Option)
    pub fn with_retention_period_days(mut self, value: i64) -> Self {
        self.retention_period_days = Some(value);
        self
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}
