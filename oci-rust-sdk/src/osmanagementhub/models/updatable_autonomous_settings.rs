use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Updatable settings for the Autonomous Linux service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatableAutonomousSettings {
    /// Indicates whether Autonomous Linux will collect crash files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_data_collection_authorized: Option<bool>,
}

impl UpdatableAutonomousSettings {
    /// Create a new UpdatableAutonomousSettings
    pub fn new() -> Self {
        Self {
            is_data_collection_authorized: None,
        }
    }

    /// Set is_data_collection_authorized
    pub fn set_is_data_collection_authorized(mut self, value: Option<bool>) -> Self {
        self.is_data_collection_authorized = value;
        self
    }

    /// Set is_data_collection_authorized (unwraps Option)
    pub fn with_is_data_collection_authorized(mut self, value: bool) -> Self {
        self.is_data_collection_authorized = Some(value);
        self
    }
}

impl Default for UpdatableAutonomousSettings {
    fn default() -> Self {
        Self::new()
    }
}
