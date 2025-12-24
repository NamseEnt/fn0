use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Compute Host Configuration Data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeHostConfigurationData {
    /// The time that was last applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_last_apply: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_details: Option<ComputeHostConfigurationCheckDetails>,
}

impl ComputeHostConfigurationData {
    /// Create a new ComputeHostConfigurationData
    pub fn new() -> Self {
        Self {
            time_last_apply: None,

            check_details: None,
        }
    }

    /// Set time_last_apply
    pub fn set_time_last_apply(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_last_apply = value;
        self
    }

    /// Set check_details
    pub fn set_check_details(
        mut self,
        value: Option<ComputeHostConfigurationCheckDetails>,
    ) -> Self {
        self.check_details = value;
        self
    }

    /// Set time_last_apply (unwraps Option)
    pub fn with_time_last_apply(mut self, value: DateTime<Utc>) -> Self {
        self.time_last_apply = Some(value);
        self
    }

    /// Set check_details (unwraps Option)
    pub fn with_check_details(mut self, value: ComputeHostConfigurationCheckDetails) -> Self {
        self.check_details = Some(value);
        self
    }
}

impl Default for ComputeHostConfigurationData {
    fn default() -> Self {
        Self::new()
    }
}
