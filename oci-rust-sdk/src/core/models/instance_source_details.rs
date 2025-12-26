use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSourceDetails {
    pub source_type: String,
}

/// Required fields for InstanceSourceDetails
pub struct InstanceSourceDetailsRequired {
    pub source_type: String,
}

impl InstanceSourceDetails {
    /// Create a new InstanceSourceDetails with required fields
    pub fn new(required: InstanceSourceDetailsRequired) -> Self {
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
