use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The summary of a work request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestSummary {
    /// Type of the work request.
    pub operation_type: WorkRequestOperationType,

    /// Status of the work request.
    pub status: OperationStatus,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the work request. Work requests should be scoped to the same compartment as the resource it affects. If the work request affects multiple resources the different compartments, the services selects the compartment of the primary resource.
    pub compartment_id: String,

    /// The date and time the request was created - as described in [RFC 3339](https://tools.ietf.org/rfc/rfc3339), section 14.29.
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

    /// The list of [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for child work requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children_id: Option<Vec<String>>,

    /// The percentage complete of the operation tracked by this work request. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent_complete: Option<i64>,

    /// The scheduled date and time to retry the work request (in [RFC 3339](https://tools.ietf.org/rfc/rfc3339) format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_scheduled: Option<DateTime<Utc>>,

    /// Indicates whether this work request is managed by Autonomous Linux
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_managed_by_autonomous_linux: Option<bool>,

    /// The number of minutes the service waits for the reboot to complete. If the managed instance doesn't reboot within the timeout, the service marks the reboot job as failed. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reboot_timeout_in_mins: Option<i64>,
}

/// Required fields for WorkRequestSummary
pub struct WorkRequestSummaryRequired {
    /// Type of the work request.
    pub operation_type: WorkRequestOperationType,

    /// Status of the work request.
    pub status: OperationStatus,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the work request. Work requests should be scoped to the same compartment as the resource it affects. If the work request affects multiple resources the different compartments, the services selects the compartment of the primary resource.
    pub compartment_id: String,

    /// The date and time the request was created - as described in [RFC 3339](https://tools.ietf.org/rfc/rfc3339), section 14.29.
    pub time_created: DateTime<Utc>,
}

impl WorkRequestSummary {
    /// Create a new WorkRequestSummary with required fields
    pub fn new(required: WorkRequestSummaryRequired) -> Self {
        Self {
            operation_type: required.operation_type,

            status: required.status,

            id: required.id,

            compartment_id: required.compartment_id,

            time_created: required.time_created,

            description: None,

            display_name: None,

            message: None,

            parent_id: None,

            children_id: None,

            percent_complete: None,

            time_scheduled: None,

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

    /// Set percent_complete
    pub fn set_percent_complete(mut self, value: Option<i64>) -> Self {
        self.percent_complete = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_scheduled
    pub fn set_time_scheduled(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_scheduled = value;
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

    /// Set percent_complete (unwraps Option)
    pub fn with_percent_complete(mut self, value: i64) -> Self {
        self.percent_complete = Some(value);
        self
    }

    /// Set time_scheduled (unwraps Option)
    pub fn with_time_scheduled(mut self, value: DateTime<Utc>) -> Self {
        self.time_scheduled = Some(value);
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
