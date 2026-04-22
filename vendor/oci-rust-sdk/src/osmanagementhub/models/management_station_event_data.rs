use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides additional information for a management station event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementStationEventData {
    /// Type of management station operation.
    pub operation_type: ManagementStationEventDataOperationType,

    /// Status of the management station operation.
    pub status: EventStatus,

    /// Health state of the management station
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_state: Option<ManagementStationEventDataHealthState>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_details: Option<WorkRequestEventDataAdditionalDetails>,
}

/// Required fields for ManagementStationEventData
pub struct ManagementStationEventDataRequired {
    /// Type of management station operation.
    pub operation_type: ManagementStationEventDataOperationType,

    /// Status of the management station operation.
    pub status: EventStatus,
}

impl ManagementStationEventData {
    /// Create a new ManagementStationEventData with required fields
    pub fn new(required: ManagementStationEventDataRequired) -> Self {
        Self {
            operation_type: required.operation_type,

            status: required.status,

            health_state: None,

            additional_details: None,
        }
    }

    /// Set operation_type
    pub fn set_operation_type(mut self, value: ManagementStationEventDataOperationType) -> Self {
        self.operation_type = value;
        self
    }

    /// Set health_state
    pub fn set_health_state(
        mut self,
        value: Option<ManagementStationEventDataHealthState>,
    ) -> Self {
        self.health_state = value;
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

    /// Set health_state (unwraps Option)
    pub fn with_health_state(mut self, value: ManagementStationEventDataHealthState) -> Self {
        self.health_state = Some(value);
        self
    }

    /// Set additional_details (unwraps Option)
    pub fn with_additional_details(mut self, value: WorkRequestEventDataAdditionalDetails) -> Self {
        self.additional_details = Some(value);
        self
    }
}
