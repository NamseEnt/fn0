use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details for creating an instance configuration by providing a list of configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstanceConfigurationDetails {
    pub instance_details: ComputeInstanceOptions,

    pub source: String,
}

/// Required fields for CreateInstanceConfigurationDetails
pub struct CreateInstanceConfigurationDetailsRequired {
    pub instance_details: ComputeInstanceOptions,

    pub source: String,
}

impl CreateInstanceConfigurationDetails {
    /// Create a new CreateInstanceConfigurationDetails with required fields
    pub fn new(required: CreateInstanceConfigurationDetailsRequired) -> Self {
        Self {
            instance_details: required.instance_details,

            source: required.source,
        }
    }

    /// Set instance_details
    pub fn set_instance_details(mut self, value: ComputeInstanceOptions) -> Self {
        self.instance_details = value;
        self
    }

    /// Set source
    pub fn set_source(mut self, value: String) -> Self {
        self.source = value;
        self
    }
}
