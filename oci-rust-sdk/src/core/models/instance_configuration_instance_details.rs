use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationInstanceDetails {
    pub instance_type: String,
}

/// Required fields for InstanceConfigurationInstanceDetails
pub struct InstanceConfigurationInstanceDetailsRequired {
    pub instance_type: String,
}

impl InstanceConfigurationInstanceDetails {
    /// Create a new InstanceConfigurationInstanceDetails with required fields
    pub fn new(required: InstanceConfigurationInstanceDetailsRequired) -> Self {
        Self {
            instance_type: required.instance_type,
        }
    }

    /// Set instance_type
    pub fn set_instance_type(mut self, value: String) -> Self {
        self.instance_type = value;
        self
    }
}
