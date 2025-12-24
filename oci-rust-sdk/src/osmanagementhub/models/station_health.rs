use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Overall health information of the management station.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationHealth {
    /// Overall health status of the management station.
    pub state: HealthState,

    /// Explanation of the health status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Required fields for StationHealth
pub struct StationHealthRequired {
    /// Overall health status of the management station.
    pub state: HealthState,
}

impl StationHealth {
    /// Create a new StationHealth with required fields
    pub fn new(required: StationHealthRequired) -> Self {
        Self {
            state: required.state,

            description: None,
        }
    }

    /// Set state
    pub fn set_state(mut self, value: HealthState) -> Self {
        self.state = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }
}
