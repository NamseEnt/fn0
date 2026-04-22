use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An error encountered while executing a work request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestError {
    /// A machine-usable code for the error that occured. See [API Errors](https://docs.oracle.com/iaas/Content/API/References/apierrors.htm) for a list of error codes.
    pub code: String,

    /// A description of the issue encountered.
    pub message: String,

    /// The time the error occured, in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    pub timestamp: DateTime<Utc>,
}

/// Required fields for WorkRequestError
pub struct WorkRequestErrorRequired {
    /// A machine-usable code for the error that occured. See [API Errors](https://docs.oracle.com/iaas/Content/API/References/apierrors.htm) for a list of error codes.
    pub code: String,

    /// A description of the issue encountered.
    pub message: String,

    /// The time the error occured, in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    pub timestamp: DateTime<Utc>,
}

impl WorkRequestError {
    /// Create a new WorkRequestError with required fields
    pub fn new(required: WorkRequestErrorRequired) -> Self {
        Self {
            code: required.code,

            message: required.message,

            timestamp: required.timestamp,
        }
    }

    /// Set code
    pub fn set_code(mut self, value: String) -> Self {
        self.code = value;
        self
    }

    /// Set message
    pub fn set_message(mut self, value: String) -> Self {
        self.message = value;
        self
    }

    /// Set timestamp
    pub fn set_timestamp(mut self, value: DateTime<Utc>) -> Self {
        self.timestamp = value;
        self
    }
}
