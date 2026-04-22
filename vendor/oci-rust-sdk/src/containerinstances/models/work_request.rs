use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A description of the work request status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequest {
    /// Type of work request.
    pub operation_type: OperationType,

    /// Status of current work request.
    pub status: OperationStatus,

    /// The ID of the work request.
    pub id: String,

    /// The OCID of the compartment that contains the work request. Work requests should be scoped to the same compartment as the resource the work request affects.
    pub compartment_id: String,

    /// The resources affected by this work request.
    pub resources: Vec<WorkRequestResource>,

    /// Percentage of the request completed. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub percent_complete: i64,

    /// The date and time the request was created, as described in [RFC 3339](https://tools.ietf.org/rfc/rfc3339), section 14.29.
    pub time_accepted: DateTime<Utc>,

    /// The date and time the request was started, as described in [RFC 3339](https://tools.ietf.org/rfc/rfc3339), section 14.29.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_started: Option<DateTime<Utc>>,

    /// The date and time the object was finished, as described in [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_finished: Option<DateTime<Utc>>,
}

/// Required fields for WorkRequest
pub struct WorkRequestRequired {
    /// Type of work request.
    pub operation_type: OperationType,

    /// Status of current work request.
    pub status: OperationStatus,

    /// The ID of the work request.
    pub id: String,

    /// The OCID of the compartment that contains the work request. Work requests should be scoped to the same compartment as the resource the work request affects.
    pub compartment_id: String,

    /// The resources affected by this work request.
    pub resources: Vec<WorkRequestResource>,

    /// Percentage of the request completed. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub percent_complete: i64,

    /// The date and time the request was created, as described in [RFC 3339](https://tools.ietf.org/rfc/rfc3339), section 14.29.
    pub time_accepted: DateTime<Utc>,
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

            time_accepted: required.time_accepted,

            time_started: None,

            time_finished: None,
        }
    }

    /// Set operation_type
    pub fn set_operation_type(mut self, value: OperationType) -> Self {
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

    /// Set percent_complete
    pub fn set_percent_complete(mut self, value: i64) -> Self {
        self.percent_complete = value;
        self
    }

    /// Set time_accepted
    pub fn set_time_accepted(mut self, value: DateTime<Utc>) -> Self {
        self.time_accepted = value;
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
}
