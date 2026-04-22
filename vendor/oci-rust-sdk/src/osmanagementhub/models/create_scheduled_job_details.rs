use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a scheduled job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateScheduledJobDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the scheduled job.
    pub compartment_id: String,

    /// The type of scheduling frequency for the scheduled job.
    pub schedule_type: ScheduleTypes,

    /// The desired time of the next execution of this scheduled job (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_next_execution: DateTime<Utc>,

    /// The list of operations this scheduled job needs to perform. A scheduled job supports only one operation type, unless it is one of the following: * UPDATE_PACKAGES * UPDATE_ALL * UPDATE_SECURITY * UPDATE_BUGFIX * UPDATE_ENHANCEMENT * UPDATE_OTHER * UPDATE_KSPLICE_USERSPACE * UPDATE_KSPLICE_KERNEL
    pub operations: Vec<ScheduledJobOperation>,

    /// User-friendly name for the scheduled job. Does not have to be unique and you can change the name later. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// User-specified description of the scheduled job. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The list of locations this scheduled job should operate on for a job targeting on compartments. (Empty list means apply to all locations). This can only be set when managedCompartmentIds is not empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<ManagedInstanceLocation>>,

    /// The frequency schedule for a recurring scheduled job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurring_rule: Option<String>,

    /// The managed instance [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) that this scheduled job operates on. A scheduled job can only operate on one type of target, therefore you must supply either this or managedInstanceGroupIds, or managedCompartmentIds, or lifecycleStageIds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_ids: Option<Vec<String>>,

    /// The managed instance group [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) that this scheduled job operates on. A scheduled job can only operate on one type of target, therefore you must supply either this or managedInstanceIds, or managedCompartmentIds, or lifecycleStageIds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_group_ids: Option<Vec<String>>,

    /// The compartment [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) that this scheduled job operates on. To apply the job to all compartments in the tenancy, set this to the tenancy OCID (root compartment) and set isSubcompartmentIncluded to true. A scheduled job can only operate on one type of target, therefore you must supply either this or managedInstanceIds, or managedInstanceGroupIds, or lifecycleStageIds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_compartment_ids: Option<Vec<String>>,

    /// The lifecycle stage [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) that this scheduled job operates on. A scheduled job can only operate on one type of target, therefore you must supply either this or managedInstanceIds, or managedInstanceGroupIds, or managedCompartmentIds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_stage_ids: Option<Vec<String>>,

    /// Indicates whether to apply the scheduled job to all compartments in the tenancy when managedCompartmentIds specifies the tenancy [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) (root compartment).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_subcompartment_included: Option<bool>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The amount of time in minutes to wait until retrying the scheduled job. If set, the service will automatically retry a failed scheduled job after the interval. For example, you could set the interval to [2,5,10]. If the initial execution of the job fails, the service waits 2 minutes and then retries. If that fails, the service waits 5 minutes and then retries. If that fails, the service waits 10 minutes and then retries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_intervals: Option<Vec<i64>>,

    /// Indicates whether this scheduled job is managed by the Autonomous Linux service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_managed_by_autonomous_linux: Option<bool>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the work request that will be rerun.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_id: Option<String>,
}

/// Required fields for CreateScheduledJobDetails
pub struct CreateScheduledJobDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the scheduled job.
    pub compartment_id: String,

    /// The type of scheduling frequency for the scheduled job.
    pub schedule_type: ScheduleTypes,

    /// The desired time of the next execution of this scheduled job (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_next_execution: DateTime<Utc>,

    /// The list of operations this scheduled job needs to perform. A scheduled job supports only one operation type, unless it is one of the following: * UPDATE_PACKAGES * UPDATE_ALL * UPDATE_SECURITY * UPDATE_BUGFIX * UPDATE_ENHANCEMENT * UPDATE_OTHER * UPDATE_KSPLICE_USERSPACE * UPDATE_KSPLICE_KERNEL
    pub operations: Vec<ScheduledJobOperation>,
}

impl CreateScheduledJobDetails {
    /// Create a new CreateScheduledJobDetails with required fields
    pub fn new(required: CreateScheduledJobDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            schedule_type: required.schedule_type,

            time_next_execution: required.time_next_execution,

            operations: required.operations,

            display_name: None,

            description: None,

            locations: None,

            recurring_rule: None,

            managed_instance_ids: None,

            managed_instance_group_ids: None,

            managed_compartment_ids: None,

            lifecycle_stage_ids: None,

            is_subcompartment_included: None,

            freeform_tags: None,

            defined_tags: None,

            retry_intervals: None,

            is_managed_by_autonomous_linux: None,

            work_request_id: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
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
    pub fn set_schedule_type(mut self, value: ScheduleTypes) -> Self {
        self.schedule_type = value;
        self
    }

    /// Set locations
    pub fn set_locations(mut self, value: Option<Vec<ManagedInstanceLocation>>) -> Self {
        self.locations = value;
        self
    }

    /// Set time_next_execution
    pub fn set_time_next_execution(mut self, value: DateTime<Utc>) -> Self {
        self.time_next_execution = value;
        self
    }

    /// Set recurring_rule
    pub fn set_recurring_rule(mut self, value: Option<String>) -> Self {
        self.recurring_rule = value;
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

    /// Set is_subcompartment_included
    pub fn set_is_subcompartment_included(mut self, value: Option<bool>) -> Self {
        self.is_subcompartment_included = value;
        self
    }

    /// Set operations
    pub fn set_operations(mut self, value: Vec<ScheduledJobOperation>) -> Self {
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

    /// Set is_managed_by_autonomous_linux
    pub fn set_is_managed_by_autonomous_linux(mut self, value: Option<bool>) -> Self {
        self.is_managed_by_autonomous_linux = value;
        self
    }

    /// Set work_request_id
    pub fn set_work_request_id(mut self, value: Option<String>) -> Self {
        self.work_request_id = value;
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

    /// Set locations (unwraps Option)
    pub fn with_locations(mut self, value: Vec<ManagedInstanceLocation>) -> Self {
        self.locations = Some(value);
        self
    }

    /// Set recurring_rule (unwraps Option)
    pub fn with_recurring_rule(mut self, value: impl Into<String>) -> Self {
        self.recurring_rule = Some(value.into());
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

    /// Set is_subcompartment_included (unwraps Option)
    pub fn with_is_subcompartment_included(mut self, value: bool) -> Self {
        self.is_subcompartment_included = Some(value);
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

    /// Set is_managed_by_autonomous_linux (unwraps Option)
    pub fn with_is_managed_by_autonomous_linux(mut self, value: bool) -> Self {
        self.is_managed_by_autonomous_linux = Some(value);
        self
    }

    /// Set work_request_id (unwraps Option)
    pub fn with_work_request_id(mut self, value: impl Into<String>) -> Self {
        self.work_request_id = Some(value.into());
        self
    }
}
