use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides summary information for a scheduled job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledJobSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the scheduled job.
    pub id: String,

    /// User-friendly name for the scheduled job.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the scheduled job.
    pub compartment_id: String,

    /// The type of scheduling this scheduled job follows.
    pub schedule_type: ScheduleTypes,

    /// The time this scheduled job was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_created: DateTime<Utc>,

    /// The time this scheduled job was updated (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_updated: DateTime<Utc>,

    /// The time of the next execution of this scheduled job (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_next_execution: DateTime<Utc>,

    /// The list of operations this scheduled job needs to perform. A scheduled job supports only one operation type, unless it is one of the following: * UPDATE_PACKAGES * UPDATE_ALL * UPDATE_SECURITY * UPDATE_BUGFIX * UPDATE_ENHANCEMENT * UPDATE_OTHER * UPDATE_KSPLICE_USERSPACE * UPDATE_KSPLICE_KERNEL
    pub operations: Vec<ScheduledJobOperation>,

    /// The current state of the scheduled job.
    pub lifecycle_state: String,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    pub freeform_tags: HashMap<String, String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    pub defined_tags: HashMap<String, HashMap<String, serde_json::Value>>,

    /// The list of locations this scheduled job should operate on for a job targeting on compartments. (Empty list means apply to all locations). This can only be set when managedCompartmentIds is not empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<ManagedInstanceLocation>>,

    /// The time of the last execution of this scheduled job (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).b.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_last_execution: Option<DateTime<Utc>>,

    /// The managed instance [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) that this scheduled job operates on. A scheduled job can only operate on one type of target, therefore this parameter is mutually exclusive with managedInstanceGroupIds, managedCompartmentIds, and lifecycleStageIds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_ids: Option<Vec<String>>,

    /// The managed instance group [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) that this scheduled job operates on. A scheduled job can only operate on one type of target, therefore this parameter is mutually exclusive with managedInstanceIds, managedCompartmentIds, and lifecycleStageIds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_group_ids: Option<Vec<String>>,

    /// The compartment [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) that this scheduled job operates on. A scheduled job can only operate on one type of target, therefore this parameter is mutually exclusive with managedInstanceIds, managedInstanceGroupIds, and lifecycleStageIds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_compartment_ids: Option<Vec<String>>,

    /// The lifecycle stage [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) that this scheduled job operates on. A scheduled job can only operate on one type of target, therefore this parameter is mutually exclusive with managedInstanceIds, managedInstanceGroupIds, and managedCompartmentIds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_stage_ids: Option<Vec<String>>,

    /// Indicates whether this scheduled job is managed by the Autonomous Linux service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_managed_by_autonomous_linux: Option<bool>,

    /// System tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"orcl-cloud\": {\"free-tier-retained\": \"true\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Indicates if the schedule job has restricted update and deletion capabilities. For restricted scheduled jobs, you can update only the timeNextExecution, recurringRule, and tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_restricted: Option<bool>,

    /// The amount of time in minutes to wait until retrying the scheduled job. If set, the service will automatically retry a failed scheduled job after the interval. For example, you could set the interval to [2,5,10]. If the initial execution of the job fails, the service waits 2 minutes and then retries. If that fails, the service waits 5 minutes and then retries. If that fails, the service waits 10 minutes and then retries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_intervals: Option<Vec<i64>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the work request that will be rerun.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_id: Option<String>,
}

/// Required fields for ScheduledJobSummary
pub struct ScheduledJobSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the scheduled job.
    pub id: String,

    /// User-friendly name for the scheduled job.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the scheduled job.
    pub compartment_id: String,

    /// The type of scheduling this scheduled job follows.
    pub schedule_type: ScheduleTypes,

    /// The time this scheduled job was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_created: DateTime<Utc>,

    /// The time this scheduled job was updated (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_updated: DateTime<Utc>,

    /// The time of the next execution of this scheduled job (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_next_execution: DateTime<Utc>,

    /// The list of operations this scheduled job needs to perform. A scheduled job supports only one operation type, unless it is one of the following: * UPDATE_PACKAGES * UPDATE_ALL * UPDATE_SECURITY * UPDATE_BUGFIX * UPDATE_ENHANCEMENT * UPDATE_OTHER * UPDATE_KSPLICE_USERSPACE * UPDATE_KSPLICE_KERNEL
    pub operations: Vec<ScheduledJobOperation>,

    /// The current state of the scheduled job.
    pub lifecycle_state: String,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    pub freeform_tags: HashMap<String, String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    pub defined_tags: HashMap<String, HashMap<String, serde_json::Value>>,
}

impl ScheduledJobSummary {
    /// Create a new ScheduledJobSummary with required fields
    pub fn new(required: ScheduledJobSummaryRequired) -> Self {
        Self {
            id: required.id,

            display_name: required.display_name,

            compartment_id: required.compartment_id,

            schedule_type: required.schedule_type,

            time_created: required.time_created,

            time_updated: required.time_updated,

            time_next_execution: required.time_next_execution,

            operations: required.operations,

            lifecycle_state: required.lifecycle_state,

            freeform_tags: required.freeform_tags,

            defined_tags: required.defined_tags,

            locations: None,

            time_last_execution: None,

            managed_instance_ids: None,

            managed_instance_group_ids: None,

            managed_compartment_ids: None,

            lifecycle_stage_ids: None,

            is_managed_by_autonomous_linux: None,

            system_tags: None,

            is_restricted: None,

            retry_intervals: None,

            work_request_id: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set schedule_type
    pub fn set_schedule_type(mut self, value: ScheduleTypes) -> Self {
        self.schedule_type = value;
        self
    }

    /// Set locations
    pub fn set_locations(mut self, value: Option<Vec<ManagedInstanceLocation>>) -> Self {
        self.locations = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_updated
    pub fn set_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = value;
        self
    }

    /// Set time_next_execution
    pub fn set_time_next_execution(mut self, value: DateTime<Utc>) -> Self {
        self.time_next_execution = value;
        self
    }

    /// Set time_last_execution
    pub fn set_time_last_execution(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_last_execution = value;
        self
    }

    /// Set managed_instance_ids
    pub fn set_managed_instance_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.managed_instance_ids = value;
        self
    }

    /// Set managed_instance_group_ids
    pub fn set_managed_instance_group_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.managed_instance_group_ids = value;
        self
    }

    /// Set managed_compartment_ids
    pub fn set_managed_compartment_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.managed_compartment_ids = value;
        self
    }

    /// Set lifecycle_stage_ids
    pub fn set_lifecycle_stage_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.lifecycle_stage_ids = value;
        self
    }

    /// Set operations
    pub fn set_operations(mut self, value: Vec<ScheduledJobOperation>) -> Self {
        self.operations = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: String) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set is_managed_by_autonomous_linux
    pub fn set_is_managed_by_autonomous_linux(mut self, value: Option<bool>) -> Self {
        self.is_managed_by_autonomous_linux = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set system_tags
    pub fn set_system_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.system_tags = value;
        self
    }

    /// Set is_restricted
    pub fn set_is_restricted(mut self, value: Option<bool>) -> Self {
        self.is_restricted = value;
        self
    }

    /// Set retry_intervals
    pub fn set_retry_intervals(mut self, value: Option<Vec<i64>>) -> Self {
        self.retry_intervals = value;
        self
    }

    /// Set work_request_id
    pub fn set_work_request_id(mut self, value: Option<String>) -> Self {
        self.work_request_id = value;
        self
    }

    /// Set locations (unwraps Option)
    pub fn with_locations(mut self, value: Vec<ManagedInstanceLocation>) -> Self {
        self.locations = Some(value);
        self
    }

    /// Set time_last_execution (unwraps Option)
    pub fn with_time_last_execution(mut self, value: DateTime<Utc>) -> Self {
        self.time_last_execution = Some(value);
        self
    }

    /// Set managed_instance_ids (unwraps Option)
    pub fn with_managed_instance_ids(mut self, value: Vec<String>) -> Self {
        self.managed_instance_ids = Some(value);
        self
    }

    /// Set managed_instance_group_ids (unwraps Option)
    pub fn with_managed_instance_group_ids(mut self, value: Vec<String>) -> Self {
        self.managed_instance_group_ids = Some(value);
        self
    }

    /// Set managed_compartment_ids (unwraps Option)
    pub fn with_managed_compartment_ids(mut self, value: Vec<String>) -> Self {
        self.managed_compartment_ids = Some(value);
        self
    }

    /// Set lifecycle_stage_ids (unwraps Option)
    pub fn with_lifecycle_stage_ids(mut self, value: Vec<String>) -> Self {
        self.lifecycle_stage_ids = Some(value);
        self
    }

    /// Set is_managed_by_autonomous_linux (unwraps Option)
    pub fn with_is_managed_by_autonomous_linux(mut self, value: bool) -> Self {
        self.is_managed_by_autonomous_linux = Some(value);
        self
    }

    /// Set system_tags (unwraps Option)
    pub fn with_system_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.system_tags = Some(value);
        self
    }

    /// Set is_restricted (unwraps Option)
    pub fn with_is_restricted(mut self, value: bool) -> Self {
        self.is_restricted = Some(value);
        self
    }

    /// Set retry_intervals (unwraps Option)
    pub fn with_retry_intervals(mut self, value: Vec<i64>) -> Self {
        self.retry_intervals = Some(value);
        self
    }

    /// Set work_request_id (unwraps Option)
    pub fn with_work_request_id(mut self, value: impl Into<String>) -> Self {
        self.work_request_id = Some(value.into());
        self
    }
}
