use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object that defines a work request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequest {
    /// Type of the work request.
    pub operation_type: WorkRequestOperationType,

    /// Status of the work request.
    pub status: OperationStatus,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the work request. Work requests should be scoped to the same compartment as the resource it affects. If the work request affects multiple resources the different compartments, the services selects the compartment of the primary resource.
    pub compartment_id: String,

    /// The list of [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the resources affected by the work request.
    pub resources: Vec<WorkRequestResource>,

    /// The percentage complete of the operation tracked by this work request. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub percent_complete: i64,

    /// The date and time the work request was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_created: DateTime<Utc>,

    /// A short description about the work request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A short display name for the work request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// A progress or error message, if there is any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the parent work request, if there is any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// The list of [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the child work requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children_id: Option<Vec<String>>,

    /// A list of package names to be installed, updated, or removed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_names: Option<Vec<String>>,

    /// The UUIDs of the target Windows update (only used when operation type is INSTALL_WINDOWS_UPDATES).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows_update_names: Option<Vec<String>>,

    /// The list of appstream modules being operated on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_specs: Option<Vec<ModuleSpecDetails>>,

    /// The date and time the work request started (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_updated: Option<DateTime<Utc>>,

    /// The date and time the work request started (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_started: Option<DateTime<Utc>>,

    /// The date and time the work request completed (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_finished: Option<DateTime<Utc>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the resource that initiated the work request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiator_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub management_station: Option<WorkRequestManagementStationDetails>,

    /// The scheduled date and time to retry the work request (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_scheduled: Option<DateTime<Utc>>,

    /// The location of the bundle in the filesystem of the resource associated to this work request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_location: Option<String>,

    /// The event id of the content. This property is required when the work request type is IMPORT_CONTENT or REMOVE_CONTENT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,

    /// The EventFingerprint associated with the content. This property is required when the work request type is IMPORT_CONTENT or REMOVE_CONTENT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_checksum: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the original work request that is being retried.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_of_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the original work request that is being rerun.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rerun_of_id: Option<String>,

    /// The amount of time in minutes to wait until retrying the work request. If set, the service will automatically retry a failed work request after the interval. For example, An interval set to [2,5,10]. If the initial execution of the work request fails, the service waits 2 minutes and then retries. If that fails, the service waits 5 minutes and then retries. If that fails, the service waits 10 minutes and then retries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_intervals: Option<Vec<i64>>,

    /// Indicates whether this work request is managed by the Autonomous Linux service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_managed_by_autonomous_linux: Option<bool>,

    /// The number of minutes the service waits for the reboot to complete. If the managed instance doesn't reboot within the timeout, the service marks the reboot job as failed. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reboot_timeout_in_mins: Option<i64>,
}

/// Required fields for WorkRequest
pub struct WorkRequestRequired {
    /// Type of the work request.
    pub operation_type: WorkRequestOperationType,

    /// Status of the work request.
    pub status: OperationStatus,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the work request. Work requests should be scoped to the same compartment as the resource it affects. If the work request affects multiple resources the different compartments, the services selects the compartment of the primary resource.
    pub compartment_id: String,

    /// The list of [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the resources affected by the work request.
    pub resources: Vec<WorkRequestResource>,

    /// The percentage complete of the operation tracked by this work request. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub percent_complete: i64,

    /// The date and time the work request was created (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    pub time_created: DateTime<Utc>,
}

impl WorkRequest {
    /// Create a new WorkRequest with required fields
    pub fn new(required: WorkRequestRequired) -> Self {
        Self {
            operation_type: required.operation_type,

            status: required.status,

            id: required.id,

            compartment_id: required.compartment_id,

            resources: required.resources,

            percent_complete: required.percent_complete,

            time_created: required.time_created,

            description: None,

            display_name: None,

            message: None,

            parent_id: None,

            children_id: None,

            package_names: None,

            windows_update_names: None,

            module_specs: None,

            time_updated: None,

            time_started: None,

            time_finished: None,

            initiator_id: None,

            management_station: None,

            time_scheduled: None,

            content_location: None,

            event_id: None,

            content_checksum: None,

            retry_of_id: None,

            rerun_of_id: None,

            retry_intervals: None,

            is_managed_by_autonomous_linux: None,

            reboot_timeout_in_mins: None,
        }
    }

    /// Set operation_type
    pub fn set_operation_type(mut self, value: WorkRequestOperationType) -> Self {
        self.operation_type = value;
        self
    }

    /// Set status
    pub fn set_status(mut self, value: OperationStatus) -> Self {
        self.status = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set message
    pub fn set_message(mut self, value: Option<String>) -> Self {
        self.message = value;
        self
    }

    /// Set parent_id
    pub fn set_parent_id(mut self, value: Option<String>) -> Self {
        self.parent_id = value;
        self
    }

    /// Set children_id
    pub fn set_children_id(mut self, value: Option<Vec<String>>) -> Self {
        self.children_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set resources
    pub fn set_resources(mut self, value: Vec<WorkRequestResource>) -> Self {
        self.resources = value;
        self
    }

    /// Set package_names
    pub fn set_package_names(mut self, value: Option<Vec<String>>) -> Self {
        self.package_names = value;
        self
    }

    /// Set windows_update_names
    pub fn set_windows_update_names(mut self, value: Option<Vec<String>>) -> Self {
        self.windows_update_names = value;
        self
    }

    /// Set module_specs
    pub fn set_module_specs(mut self, value: Option<Vec<ModuleSpecDetails>>) -> Self {
        self.module_specs = value;
        self
    }

    /// Set percent_complete
    pub fn set_percent_complete(mut self, value: i64) -> Self {
        self.percent_complete = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_updated
    pub fn set_time_updated(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_updated = value;
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

    /// Set initiator_id
    pub fn set_initiator_id(mut self, value: Option<String>) -> Self {
        self.initiator_id = value;
        self
    }

    /// Set management_station
    pub fn set_management_station(
        mut self,
        value: Option<WorkRequestManagementStationDetails>,
    ) -> Self {
        self.management_station = value;
        self
    }

    /// Set time_scheduled
    pub fn set_time_scheduled(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_scheduled = value;
        self
    }

    /// Set content_location
    pub fn set_content_location(mut self, value: Option<String>) -> Self {
        self.content_location = value;
        self
    }

    /// Set event_id
    pub fn set_event_id(mut self, value: Option<String>) -> Self {
        self.event_id = value;
        self
    }

    /// Set content_checksum
    pub fn set_content_checksum(mut self, value: Option<String>) -> Self {
        self.content_checksum = value;
        self
    }

    /// Set retry_of_id
    pub fn set_retry_of_id(mut self, value: Option<String>) -> Self {
        self.retry_of_id = value;
        self
    }

    /// Set rerun_of_id
    pub fn set_rerun_of_id(mut self, value: Option<String>) -> Self {
        self.rerun_of_id = value;
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

    /// Set reboot_timeout_in_mins
    pub fn set_reboot_timeout_in_mins(mut self, value: Option<i64>) -> Self {
        self.reboot_timeout_in_mins = value;
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set message (unwraps Option)
    pub fn with_message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    /// Set parent_id (unwraps Option)
    pub fn with_parent_id(mut self, value: impl Into<String>) -> Self {
        self.parent_id = Some(value.into());
        self
    }

    /// Set children_id (unwraps Option)
    pub fn with_children_id(mut self, value: Vec<String>) -> Self {
        self.children_id = Some(value);
        self
    }

    /// Set package_names (unwraps Option)
    pub fn with_package_names(mut self, value: Vec<String>) -> Self {
        self.package_names = Some(value);
        self
    }

    /// Set windows_update_names (unwraps Option)
    pub fn with_windows_update_names(mut self, value: Vec<String>) -> Self {
        self.windows_update_names = Some(value);
        self
    }

    /// Set module_specs (unwraps Option)
    pub fn with_module_specs(mut self, value: Vec<ModuleSpecDetails>) -> Self {
        self.module_specs = Some(value);
        self
    }

    /// Set time_updated (unwraps Option)
    pub fn with_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = Some(value);
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

    /// Set initiator_id (unwraps Option)
    pub fn with_initiator_id(mut self, value: impl Into<String>) -> Self {
        self.initiator_id = Some(value.into());
        self
    }

    /// Set management_station (unwraps Option)
    pub fn with_management_station(mut self, value: WorkRequestManagementStationDetails) -> Self {
        self.management_station = Some(value);
        self
    }

    /// Set time_scheduled (unwraps Option)
    pub fn with_time_scheduled(mut self, value: DateTime<Utc>) -> Self {
        self.time_scheduled = Some(value);
        self
    }

    /// Set content_location (unwraps Option)
    pub fn with_content_location(mut self, value: impl Into<String>) -> Self {
        self.content_location = Some(value.into());
        self
    }

    /// Set event_id (unwraps Option)
    pub fn with_event_id(mut self, value: impl Into<String>) -> Self {
        self.event_id = Some(value.into());
        self
    }

    /// Set content_checksum (unwraps Option)
    pub fn with_content_checksum(mut self, value: impl Into<String>) -> Self {
        self.content_checksum = Some(value.into());
        self
    }

    /// Set retry_of_id (unwraps Option)
    pub fn with_retry_of_id(mut self, value: impl Into<String>) -> Self {
        self.retry_of_id = Some(value.into());
        self
    }

    /// Set rerun_of_id (unwraps Option)
    pub fn with_rerun_of_id(mut self, value: impl Into<String>) -> Self {
        self.rerun_of_id = Some(value.into());
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

    /// Set reboot_timeout_in_mins (unwraps Option)
    pub fn with_reboot_timeout_in_mins(mut self, value: i64) -> Self {
        self.reboot_timeout_in_mins = Some(value);
        self
    }
}
