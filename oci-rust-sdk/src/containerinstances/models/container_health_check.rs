use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Type of container health check which could be either HTTP, TCP, or Command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerHealthCheck {
    pub health_check_type: String,

    /// Health check name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The initial delay in seconds before start checking container health status. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_delay_in_seconds: Option<i64>,

    /// Number of seconds between two consecutive runs for checking container health. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval_in_seconds: Option<i64>,

    /// Number of consecutive failures at which we consider the check failed. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_threshold: Option<i64>,

    /// Number of consecutive successes at which we consider the check succeeded again after it was in failure state. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_threshold: Option<i64>,

    /// Length of waiting time in seconds before marking health check failed. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_in_seconds: Option<i64>,

    /// Status of container
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ContainerHealthCheckStatus>,

    /// A message describing the current status in more details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_details: Option<String>,

    /// The action will be triggered when the container health check fails. There are two types of action: KILL or NONE. The default action is KILL. If failure action is KILL, the container will be subject to the container restart policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_action: Option<ContainerHealthCheckFailureAction>,
}

/// Required fields for ContainerHealthCheck
pub struct ContainerHealthCheckRequired {
    pub health_check_type: String,
}

impl ContainerHealthCheck {
    /// Create a new ContainerHealthCheck with required fields
    pub fn new(required: ContainerHealthCheckRequired) -> Self {
        Self {
            health_check_type: required.health_check_type,

            name: None,

            initial_delay_in_seconds: None,

            interval_in_seconds: None,

            failure_threshold: None,

            success_threshold: None,

            timeout_in_seconds: None,

            status: None,

            status_details: None,

            failure_action: None,
        }
    }

    /// Set name
    pub fn set_name(mut self, value: Option<String>) -> Self {
        self.name = value;
        self
    }

    /// Set initial_delay_in_seconds
    pub fn set_initial_delay_in_seconds(mut self, value: Option<i64>) -> Self {
        self.initial_delay_in_seconds = value;
        self
    }

    /// Set interval_in_seconds
    pub fn set_interval_in_seconds(mut self, value: Option<i64>) -> Self {
        self.interval_in_seconds = value;
        self
    }

    /// Set failure_threshold
    pub fn set_failure_threshold(mut self, value: Option<i64>) -> Self {
        self.failure_threshold = value;
        self
    }

    /// Set success_threshold
    pub fn set_success_threshold(mut self, value: Option<i64>) -> Self {
        self.success_threshold = value;
        self
    }

    /// Set timeout_in_seconds
    pub fn set_timeout_in_seconds(mut self, value: Option<i64>) -> Self {
        self.timeout_in_seconds = value;
        self
    }

    /// Set status
    pub fn set_status(mut self, value: Option<ContainerHealthCheckStatus>) -> Self {
        self.status = value;
        self
    }

    /// Set status_details
    pub fn set_status_details(mut self, value: Option<String>) -> Self {
        self.status_details = value;
        self
    }

    /// Set failure_action
    pub fn set_failure_action(mut self, value: Option<ContainerHealthCheckFailureAction>) -> Self {
        self.failure_action = value;
        self
    }

    /// Set health_check_type
    pub fn set_health_check_type(mut self, value: String) -> Self {
        self.health_check_type = value;
        self
    }

    /// Set name (unwraps Option)
    pub fn with_name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    /// Set initial_delay_in_seconds (unwraps Option)
    pub fn with_initial_delay_in_seconds(mut self, value: i64) -> Self {
        self.initial_delay_in_seconds = Some(value);
        self
    }

    /// Set interval_in_seconds (unwraps Option)
    pub fn with_interval_in_seconds(mut self, value: i64) -> Self {
        self.interval_in_seconds = Some(value);
        self
    }

    /// Set failure_threshold (unwraps Option)
    pub fn with_failure_threshold(mut self, value: i64) -> Self {
        self.failure_threshold = Some(value);
        self
    }

    /// Set success_threshold (unwraps Option)
    pub fn with_success_threshold(mut self, value: i64) -> Self {
        self.success_threshold = Some(value);
        self
    }

    /// Set timeout_in_seconds (unwraps Option)
    pub fn with_timeout_in_seconds(mut self, value: i64) -> Self {
        self.timeout_in_seconds = Some(value);
        self
    }

    /// Set status (unwraps Option)
    pub fn with_status(mut self, value: ContainerHealthCheckStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set status_details (unwraps Option)
    pub fn with_status_details(mut self, value: impl Into<String>) -> Self {
        self.status_details = Some(value.into());
        self
    }

    /// Set failure_action (unwraps Option)
    pub fn with_failure_action(mut self, value: ContainerHealthCheckFailureAction) -> Self {
        self.failure_action = Some(value);
        self
    }
}
