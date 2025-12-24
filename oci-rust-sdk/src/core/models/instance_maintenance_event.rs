use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// It is the event in which the maintenance action will be be performed on the customer instance on the scheduled date and time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceMaintenanceEvent {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the maintenance event.
    pub id: String,

    /// The OCID of the instance.
    pub instance_id: String,

    /// The OCID of the compartment that contains the instance.
    pub compartment_id: String,

    /// This indicates the priority and allowed actions for this Maintenance. Higher priority forms of Maintenance have tighter restrictions and may not be rescheduled, while lower priority/severity Maintenance can be rescheduled, deferred, or even cancelled. Please see the [Instance Maintenance](https://docs.oracle.com/iaas/Content/Compute/Tasks/placeholder.htm) documentation for details.
    pub maintenance_category: InstanceMaintenanceEventMaintenanceCategory,

    /// This is the reason that Maintenance is being performed. See [Instance Maintenance](https://docs.oracle.com/iaas/Content/Compute/Tasks/placeholder.htm) documentation for details.
    pub maintenance_reason: InstanceMaintenanceEventMaintenanceReason,

    /// This is the action that will be performed on the Instance by OCI when the Maintenance begins.
    pub instance_action: InstanceMaintenanceEventInstanceAction,

    /// These are alternative actions to the requested instanceAction that can be taken to resolve the Maintenance.
    pub alternative_resolution_actions: Vec<InstanceMaintenanceEventAlternativeResolutionActions>,

    /// The beginning of the time window when Maintenance is scheduled to begin. The Maintenance will not begin before this time.
    pub time_window_start: DateTime<Utc>,

    /// Indicates if this MaintenanceEvent is capable of being rescheduled up to the timeHardDueDate.
    pub can_reschedule: bool,

    /// The date and time the maintenance event was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The current state of the maintenance event.
    pub lifecycle_state: InstanceMaintenanceEventLifecycleState,

    /// The creator of the maintenance event.
    pub created_by: InstanceMaintenanceEventCreatedBy,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The time at which the Maintenance actually started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_started: Option<DateTime<Utc>>,

    /// The time at which the Maintenance actually finished.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_finished: Option<DateTime<Utc>>,

    /// The duration of the time window Maintenance is scheduled to begin within.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_window_duration: Option<String>,

    /// This is the estimated duration of the Maintenance, once the Maintenance has entered the STARTED state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration: Option<String>,

    /// It is the scheduled hard due date and time of the maintenance event. The maintenance event will happen at this time and the due date will not be extended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_hard_due_date: Option<DateTime<Utc>>,

    /// Provides more details about the state of the maintenance event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_details: Option<String>,

    /// It is the descriptive information about the maintenance taking place on the customer instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A unique identifier that will group Instances that have a relationship with one another and must be scheduled together for the Maintenance to proceed. Any Instances that have a relationship with one another from a Maintenance perspective will have a matching correlationToken.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_token: Option<String>,

    /// For Instances that have local storage, this field is set to true when local storage will be deleted as a result of the Maintenance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_delete_local_storage: Option<bool>,

    /// Additional details of the maintenance in the form of json.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_details: Option<HashMap<String, String>>,
}

/// Required fields for InstanceMaintenanceEvent
pub struct InstanceMaintenanceEventRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the maintenance event.
    pub id: String,

    /// The OCID of the instance.
    pub instance_id: String,

    /// The OCID of the compartment that contains the instance.
    pub compartment_id: String,

    /// This indicates the priority and allowed actions for this Maintenance. Higher priority forms of Maintenance have tighter restrictions and may not be rescheduled, while lower priority/severity Maintenance can be rescheduled, deferred, or even cancelled. Please see the [Instance Maintenance](https://docs.oracle.com/iaas/Content/Compute/Tasks/placeholder.htm) documentation for details.
    pub maintenance_category: InstanceMaintenanceEventMaintenanceCategory,

    /// This is the reason that Maintenance is being performed. See [Instance Maintenance](https://docs.oracle.com/iaas/Content/Compute/Tasks/placeholder.htm) documentation for details.
    pub maintenance_reason: InstanceMaintenanceEventMaintenanceReason,

    /// This is the action that will be performed on the Instance by OCI when the Maintenance begins.
    pub instance_action: InstanceMaintenanceEventInstanceAction,

    /// These are alternative actions to the requested instanceAction that can be taken to resolve the Maintenance.
    pub alternative_resolution_actions: Vec<InstanceMaintenanceEventAlternativeResolutionActions>,

    /// The beginning of the time window when Maintenance is scheduled to begin. The Maintenance will not begin before this time.
    pub time_window_start: DateTime<Utc>,

    /// Indicates if this MaintenanceEvent is capable of being rescheduled up to the timeHardDueDate.
    pub can_reschedule: bool,

    /// The date and time the maintenance event was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The current state of the maintenance event.
    pub lifecycle_state: InstanceMaintenanceEventLifecycleState,

    /// The creator of the maintenance event.
    pub created_by: InstanceMaintenanceEventCreatedBy,
}

impl InstanceMaintenanceEvent {
    /// Create a new InstanceMaintenanceEvent with required fields
    pub fn new(required: InstanceMaintenanceEventRequired) -> Self {
        Self {
            id: required.id,

            instance_id: required.instance_id,

            compartment_id: required.compartment_id,

            maintenance_category: required.maintenance_category,

            maintenance_reason: required.maintenance_reason,

            instance_action: required.instance_action,

            alternative_resolution_actions: required.alternative_resolution_actions,

            time_window_start: required.time_window_start,

            can_reschedule: required.can_reschedule,

            time_created: required.time_created,

            lifecycle_state: required.lifecycle_state,

            created_by: required.created_by,

            display_name: None,

            defined_tags: None,

            freeform_tags: None,

            time_started: None,

            time_finished: None,

            start_window_duration: None,

            estimated_duration: None,

            time_hard_due_date: None,

            lifecycle_details: None,

            description: None,

            correlation_token: None,

            can_delete_local_storage: None,

            additional_details: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
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

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set maintenance_category
    pub fn set_maintenance_category(
        mut self,
        value: InstanceMaintenanceEventMaintenanceCategory,
    ) -> Self {
        self.maintenance_category = value;
        self
    }

    /// Set maintenance_reason
    pub fn set_maintenance_reason(
        mut self,
        value: InstanceMaintenanceEventMaintenanceReason,
    ) -> Self {
        self.maintenance_reason = value;
        self
    }

    /// Set instance_action
    pub fn set_instance_action(mut self, value: InstanceMaintenanceEventInstanceAction) -> Self {
        self.instance_action = value;
        self
    }

    /// Set alternative_resolution_actions
    pub fn set_alternative_resolution_actions(
        mut self,
        value: Vec<InstanceMaintenanceEventAlternativeResolutionActions>,
    ) -> Self {
        self.alternative_resolution_actions = value;
        self
    }

    /// Set time_started
    pub fn set_time_started(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_started = value;
        self
    }

    /// Set time_finished
    pub fn set_time_finished(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_finished = value;
        self
    }

    /// Set time_window_start
    pub fn set_time_window_start(mut self, value: DateTime<Utc>) -> Self {
        self.time_window_start = value;
        self
    }

    /// Set start_window_duration
    pub fn set_start_window_duration(mut self, value: Option<String>) -> Self {
        self.start_window_duration = value;
        self
    }

    /// Set estimated_duration
    pub fn set_estimated_duration(mut self, value: Option<String>) -> Self {
        self.estimated_duration = value;
        self
    }

    /// Set time_hard_due_date
    pub fn set_time_hard_due_date(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_hard_due_date = value;
        self
    }

    /// Set can_reschedule
    pub fn set_can_reschedule(mut self, value: bool) -> Self {
        self.can_reschedule = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: InstanceMaintenanceEventLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set lifecycle_details
    pub fn set_lifecycle_details(mut self, value: Option<String>) -> Self {
        self.lifecycle_details = value;
        self
    }

    /// Set created_by
    pub fn set_created_by(mut self, value: InstanceMaintenanceEventCreatedBy) -> Self {
        self.created_by = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set correlation_token
    pub fn set_correlation_token(mut self, value: Option<String>) -> Self {
        self.correlation_token = value;
        self
    }

    /// Set can_delete_local_storage
    pub fn set_can_delete_local_storage(mut self, value: Option<bool>) -> Self {
        self.can_delete_local_storage = value;
        self
    }

    /// Set additional_details
    pub fn set_additional_details(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.additional_details = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
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

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set time_started (unwraps Option)
    pub fn with_time_started(mut self, value: DateTime<Utc>) -> Self {
        self.time_started = Some(value);
        self
    }

    /// Set time_finished (unwraps Option)
    pub fn with_time_finished(mut self, value: DateTime<Utc>) -> Self {
        self.time_finished = Some(value);
        self
    }

    /// Set start_window_duration (unwraps Option)
    pub fn with_start_window_duration(mut self, value: impl Into<String>) -> Self {
        self.start_window_duration = Some(value.into());
        self
    }

    /// Set estimated_duration (unwraps Option)
    pub fn with_estimated_duration(mut self, value: impl Into<String>) -> Self {
        self.estimated_duration = Some(value.into());
        self
    }

    /// Set time_hard_due_date (unwraps Option)
    pub fn with_time_hard_due_date(mut self, value: DateTime<Utc>) -> Self {
        self.time_hard_due_date = Some(value);
        self
    }

    /// Set lifecycle_details (unwraps Option)
    pub fn with_lifecycle_details(mut self, value: impl Into<String>) -> Self {
        self.lifecycle_details = Some(value.into());
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set correlation_token (unwraps Option)
    pub fn with_correlation_token(mut self, value: impl Into<String>) -> Self {
        self.correlation_token = Some(value.into());
        self
    }

    /// Set can_delete_local_storage (unwraps Option)
    pub fn with_can_delete_local_storage(mut self, value: bool) -> Self {
        self.can_delete_local_storage = Some(value);
        self
    }

    /// Set additional_details (unwraps Option)
    pub fn with_additional_details(mut self, value: HashMap<String, String>) -> Self {
        self.additional_details = Some(value);
        self
    }
}
