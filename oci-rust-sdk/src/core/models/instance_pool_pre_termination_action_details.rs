use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The data for pre-termination action for an instance pool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstancePoolPreTerminationActionDetails {
    /// Whether pre-termination action is enabled or not.
    pub is_enabled: bool,

    /// The timeout in seconds for pre-termination action for an instance pool. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub timeout: i64,

    pub on_timeout: InstancePoolPreTerminationActionHandleTimeoutDetails,
}

/// Required fields for InstancePoolPreTerminationActionDetails
pub struct InstancePoolPreTerminationActionDetailsRequired {
    /// Whether pre-termination action is enabled or not.
    pub is_enabled: bool,

    /// The timeout in seconds for pre-termination action for an instance pool. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub timeout: i64,

    pub on_timeout: InstancePoolPreTerminationActionHandleTimeoutDetails,
}

impl InstancePoolPreTerminationActionDetails {
    /// Create a new InstancePoolPreTerminationActionDetails with required fields
    pub fn new(required: InstancePoolPreTerminationActionDetailsRequired) -> Self {
        Self {
            is_enabled: required.is_enabled,

            timeout: required.timeout,

            on_timeout: required.on_timeout,
        }
    }

    /// Set is_enabled
    pub fn set_is_enabled(mut self, value: bool) -> Self {
        self.is_enabled = value;
        self
    }

    /// Set timeout
    pub fn set_timeout(mut self, value: i64) -> Self {
        self.timeout = value;
        self
    }

    /// Set on_timeout
    pub fn set_on_timeout(
        mut self,
        value: InstancePoolPreTerminationActionHandleTimeoutDetails,
    ) -> Self {
        self.on_timeout = value;
        self
    }
}
