use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides additional information for a software source event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareSourceEventData {
    /// Type of software source operation.
    pub operation_type: SoftwareSourceEventDataOperationType,

    /// Status of the software source operation.
    pub status: EventStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_details: Option<WorkRequestEventDataAdditionalDetails>,
}

/// Required fields for SoftwareSourceEventData
pub struct SoftwareSourceEventDataRequired {
    /// Type of software source operation.
    pub operation_type: SoftwareSourceEventDataOperationType,

    /// Status of the software source operation.
    pub status: EventStatus,
}

impl SoftwareSourceEventData {
    /// Create a new SoftwareSourceEventData with required fields
    pub fn new(required: SoftwareSourceEventDataRequired) -> Self {
        Self {
            operation_type: required.operation_type,

            status: required.status,

            additional_details: None,
        }
    }

    /// Set operation_type
    pub fn set_operation_type(mut self, value: SoftwareSourceEventDataOperationType) -> Self {
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
