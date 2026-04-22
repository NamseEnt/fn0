use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides additional information for an agent event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEventData {
    /// Type of agent operation.
    pub operation_type: AgentEventDataOperationType,

    /// Status of the agent operation.
    pub status: EventStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_details: Option<WorkRequestEventDataAdditionalDetails>,
}

/// Required fields for AgentEventData
pub struct AgentEventDataRequired {
    /// Type of agent operation.
    pub operation_type: AgentEventDataOperationType,

    /// Status of the agent operation.
    pub status: EventStatus,
}

impl AgentEventData {
    /// Create a new AgentEventData with required fields
    pub fn new(required: AgentEventDataRequired) -> Self {
        Self {
            operation_type: required.operation_type,

            status: required.status,

            additional_details: None,
        }
    }

    /// Set operation_type
    pub fn set_operation_type(mut self, value: AgentEventDataOperationType) -> Self {
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
