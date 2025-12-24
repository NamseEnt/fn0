use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to update a work request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkRequestDetails {
    /// Status of current work request.
    pub status: OperationStatus,

    /// The percentage complete of the operation tracked by this work request. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent_complete: Option<i64>,

    /// The date and time the object was finished, as described in [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_finished: Option<DateTime<Utc>>,

    /// A short description about the work request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A short display for about the work request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Required fields for UpdateWorkRequestDetails
pub struct UpdateWorkRequestDetailsRequired {
    /// Status of current work request.
    pub status: OperationStatus,
}

impl UpdateWorkRequestDetails {
    /// Create a new UpdateWorkRequestDetails with required fields
    pub fn new(required: UpdateWorkRequestDetailsRequired) -> Self {
        Self {
            status: required.status,

            percent_complete: None,

            time_finished: None,

            description: None,

            display_name: None,
        }
    }

    /// Set status
    pub fn set_status(mut self, value: OperationStatus) -> Self {
        self.status = value;
        self
    }

    /// Set percent_complete
    pub fn set_percent_complete(mut self, value: Option<i64>) -> Self {
        self.percent_complete = value;
        self
    }

    /// Set time_finished
    pub fn set_time_finished(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_finished = value;
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

    /// Set percent_complete (unwraps Option)
    pub fn with_percent_complete(mut self, value: i64) -> Self {
        self.percent_complete = Some(value);
        self
    }

    /// Set time_finished (unwraps Option)
    pub fn with_time_finished(mut self, value: DateTime<Utc>) -> Self {
        self.time_finished = Some(value);
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
}
