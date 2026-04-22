use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An individual FastConnect location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossConnectLocation {
    /// A description of the location.
    pub description: String,

    /// The name of the location. <p> Example: {@code CyrusOne, Chandler, AZ}
    pub name: String,
}

/// Required fields for CrossConnectLocation
pub struct CrossConnectLocationRequired {
    /// A description of the location.
    pub description: String,

    /// The name of the location. <p> Example: {@code CyrusOne, Chandler, AZ}
    pub name: String,
}

impl CrossConnectLocation {
    /// Create a new CrossConnectLocation with required fields
    pub fn new(required: CrossConnectLocationRequired) -> Self {
        Self {
            description: required.description,

            name: required.name,
        }
    }

    /// Set description
    pub fn set_description(mut self, value: String) -> Self {
        self.description = value;
        self
    }

    /// Set name
    pub fn set_name(mut self, value: String) -> Self {
        self.name = value;
        self
    }
}
