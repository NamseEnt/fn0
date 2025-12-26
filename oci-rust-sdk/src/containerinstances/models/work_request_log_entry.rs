use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A log message from a work request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestLogEntry {
    /// Human-readable log message.
    pub message: String,

    /// The time the log message was written, in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    pub timestamp: DateTime<Utc>,
}

/// Required fields for WorkRequestLogEntry
pub struct WorkRequestLogEntryRequired {
    /// Human-readable log message.
    pub message: String,

    /// The time the log message was written, in the format defined by [RFC 3339](https://tools.ietf.org/rfc/rfc3339).
    pub timestamp: DateTime<Utc>,
}

impl WorkRequestLogEntry {
    /// Create a new WorkRequestLogEntry with required fields
    pub fn new(required: WorkRequestLogEntryRequired) -> Self {
        Self {
            message: required.message,

            timestamp: required.timestamp,
        }
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
