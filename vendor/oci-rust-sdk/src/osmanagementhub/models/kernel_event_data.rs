use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information about the kernel event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KernelEventData {
    pub content: KernelEventContent,

    /// Number of times the event has occurred. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub count: i64,

    /// Fingerprint of the event.
    pub event_fingerprint: String,

    /// Reason for the event.
    pub reason: String,

    /// The date and time that the event first occurred.
    pub time_first_occurred: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_details: Option<KernelEventAdditionalDetails>,
}

/// Required fields for KernelEventData
pub struct KernelEventDataRequired {
    pub content: KernelEventContent,

    /// Number of times the event has occurred. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub count: i64,

    /// Fingerprint of the event.
    pub event_fingerprint: String,

    /// Reason for the event.
    pub reason: String,

    /// The date and time that the event first occurred.
    pub time_first_occurred: DateTime<Utc>,
}

impl KernelEventData {
    /// Create a new KernelEventData with required fields
    pub fn new(required: KernelEventDataRequired) -> Self {
        Self {
            content: required.content,

            count: required.count,

            event_fingerprint: required.event_fingerprint,

            reason: required.reason,

            time_first_occurred: required.time_first_occurred,

            additional_details: None,
        }
    }

    /// Set content
    pub fn set_content(mut self, value: KernelEventContent) -> Self {
        self.content = value;
        self
    }

    /// Set count
    pub fn set_count(mut self, value: i64) -> Self {
        self.count = value;
        self
    }

    /// Set event_fingerprint
    pub fn set_event_fingerprint(mut self, value: String) -> Self {
        self.event_fingerprint = value;
        self
    }

    /// Set reason
    pub fn set_reason(mut self, value: String) -> Self {
        self.reason = value;
        self
    }

    /// Set time_first_occurred
    pub fn set_time_first_occurred(mut self, value: DateTime<Utc>) -> Self {
        self.time_first_occurred = value;
        self
    }

    /// Set additional_details
    pub fn set_additional_details(mut self, value: Option<KernelEventAdditionalDetails>) -> Self {
        self.additional_details = value;
        self
    }

    /// Set additional_details (unwraps Option)
    pub fn with_additional_details(mut self, value: KernelEventAdditionalDetails) -> Self {
        self.additional_details = Some(value);
        self
    }
}
