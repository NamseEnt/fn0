use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Information collected during the event, such as logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventContent {
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Required fields for EventContent
pub struct EventContentRequired {
    pub r#type: String,
}

impl EventContent {
    /// Create a new EventContent with required fields
    pub fn new(required: EventContentRequired) -> Self {
        Self {
            r#type: required.r#type,
        }
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }
}
