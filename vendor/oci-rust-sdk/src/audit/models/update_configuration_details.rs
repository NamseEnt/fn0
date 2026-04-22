use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The configuration details for the retention period setting, specified in days. For more information, see [Setting Audit Log Retention Period](https://docs.oracle.com/iaas/Content/Audit/Tasks/settingretentionperiod.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigurationDetails {
    /// The retention period setting, specified in days. The minimum is 90, the maximum 365. <p> Example: {@code 90} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub retention_period_days: i64,
}

/// Required fields for UpdateConfigurationDetails
pub struct UpdateConfigurationDetailsRequired {
    /// The retention period setting, specified in days. The minimum is 90, the maximum 365. <p> Example: {@code 90} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub retention_period_days: i64,
}

impl UpdateConfigurationDetails {
    /// Create a new UpdateConfigurationDetails with required fields
    pub fn new(required: UpdateConfigurationDetailsRequired) -> Self {
        Self {
            retention_period_days: required.retention_period_days,
        }
    }

    /// Set retention_period_days
    pub fn set_retention_period_days(mut self, value: i64) -> Self {
        self.retention_period_days = value;
        self
    }
}
