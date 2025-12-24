use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationInstanceSourceDetails {
    pub source_type: String,
}

/// Required fields for InstanceConfigurationInstanceSourceDetails
pub struct InstanceConfigurationInstanceSourceDetailsRequired {
    pub source_type: String,
}

impl InstanceConfigurationInstanceSourceDetails {
    /// Create a new InstanceConfigurationInstanceSourceDetails with required fields
    pub fn new(required: InstanceConfigurationInstanceSourceDetailsRequired) -> Self {
        Self {
            source_type: required.source_type,
        }
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: String) -> Self {
        self.source_type = value;
        self
    }
}
