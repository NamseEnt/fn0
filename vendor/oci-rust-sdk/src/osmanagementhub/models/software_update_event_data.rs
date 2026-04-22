use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides additional information for a software update event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareUpdateEventData {
    /// Type of software update operation.
    pub operation_type: SoftwareUpdateEventDataOperationType,

    /// Status of the software update.
    pub status: EventStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_details: Option<WorkRequestEventDataAdditionalDetails>,
}

/// Required fields for SoftwareUpdateEventData
pub struct SoftwareUpdateEventDataRequired {
    /// Type of software update operation.
    pub operation_type: SoftwareUpdateEventDataOperationType,

    /// Status of the software update.
    pub status: EventStatus,
}

impl SoftwareUpdateEventData {
    /// Create a new SoftwareUpdateEventData with required fields
    pub fn new(required: SoftwareUpdateEventDataRequired) -> Self {
        Self {
            operation_type: required.operation_type,

            status: required.status,

            additional_details: None,
        }
    }

    /// Set operation_type
    pub fn set_operation_type(mut self, value: SoftwareUpdateEventDataOperationType) -> Self {
        self.operation_type = value;
        self
    }

    /// Set status
    pub fn set_status(mut self, value: EventStatus) -> Self {
        self.status = value;
        self
    }

    /// Set additional_details
    pub fn set_additional_details(
        mut self,
        value: Option<WorkRequestEventDataAdditionalDetails>,
    ) -> Self {
        self.additional_details = value;
        self
    }

    /// Set additional_details (unwraps Option)
    pub fn with_additional_details(mut self, value: WorkRequestEventDataAdditionalDetails) -> Self {
        self.additional_details = Some(value);
        self
    }
}
