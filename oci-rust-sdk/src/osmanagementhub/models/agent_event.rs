use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides information about the agent event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEvent {
    pub data: AgentEventData,

    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for AgentEvent
pub struct AgentEventRequired {
    pub data: AgentEventData,

    pub r#type: String,
}

impl AgentEvent {
    /// Create a new AgentEvent with required fields
    pub fn new(required: AgentEventRequired) -> Self {
        Self {
            data: required.data,

            r#type: required.r#type,
        }
    }

    /// Set data
    pub fn set_data(mut self, value: AgentEventData) -> Self {
        self.data = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
