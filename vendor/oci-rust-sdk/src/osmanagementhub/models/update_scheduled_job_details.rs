use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides the information used to update a scheduled job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateScheduledJobDetails {
    /// User-friendly name for the scheduled job. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// User-specified description for the scheduled job. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The type of scheduling frequency for the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_type: Option<ScheduleTypes>,

    /// The time of the next execution of this scheduled job (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_next_execution: Option<DateTime<Utc>>,

    /// The frequency schedule for a recurring scheduled job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurring_rule: Option<String>,

    /// The list of operations this scheduled job needs to perform. A scheduled job supports only one operation type, unless it is one of the following: * UPDATE_PACKAGES * UPDATE_ALL * UPDATE_SECURITY * UPDATE_BUGFIX * UPDATE_ENHANCEMENT * UPDATE_OTHER * UPDATE_KSPLICE_USERSPACE * UPDATE_KSPLICE_KERNEL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<Vec<ScheduledJobOperation>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The amount of time in minutes to wait until retrying the scheduled job. If set, the service will automatically retry a failed scheduled job after the interval. For example, you could set the interval to [2,5,10]. If the initial execution of the job fails, the service waits 2 minutes and then retries. If that fails, the service waits 5 minutes and then retries. If that fails, the service waits 10 minutes and then retries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_intervals: Option<Vec<i64>>,
}

impl UpdateScheduledJobDetails {
    /// Create a new UpdateScheduledJobDetails
    pub fn new() -> Self {
        Self {
            display_name: None,

            description: None,

            schedule_type: None,

            time_next_execution: None,

            recurring_rule: None,

            operations: None,

            freeform_tags: None,

            defined_tags: None,

            retry_intervals: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set schedule_type
    pub fn set_schedule_type(mut self, value: Option<ScheduleTypes>) -> Self {
        self.schedule_type = value;
        self
    }

    /// Set time_next_execution
    pub fn set_time_next_execution(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_next_execution = value;
        self
    }

    /// Set recurring_rule
    pub fn set_recurring_rule(mut self, value: Option<String>) -> Self {
        self.recurring_rule = value;
        self
    }

    /// Set operations
    pub fn set_operations(mut self, value: Option<Vec<ScheduledJobOperation>>) -> Self {
        self.operations = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set retry_intervals
    pub fn set_retry_intervals(mut self, value: Option<Vec<i64>>) -> Self {
        self.retry_intervals = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set schedule_type (unwraps Option)
    pub fn with_schedule_type(mut self, value: ScheduleTypes) -> Self {
        self.schedule_type = Some(value);
        self
    }

    /// Set time_next_execution (unwraps Option)
    pub fn with_time_next_execution(mut self, value: DateTime<Utc>) -> Self {
        self.time_next_execution = Some(value);
        self
    }

    /// Set recurring_rule (unwraps Option)
    pub fn with_recurring_rule(mut self, value: impl Into<String>) -> Self {
        self.recurring_rule = Some(value.into());
        self
    }

    /// Set operations (unwraps Option)
    pub fn with_operations(mut self, value: Vec<ScheduledJobOperation>) -> Self {
        self.operations = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set retry_intervals (unwraps Option)
    pub fn with_retry_intervals(mut self, value: Vec<i64>) -> Self {
        self.retry_intervals = Some(value);
        self
    }
}

impl Default for UpdateScheduledJobDetails {
    fn default() -> Self {
        Self::new()
    }
}
