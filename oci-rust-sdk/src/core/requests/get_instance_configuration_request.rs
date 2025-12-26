use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInstanceConfigurationRequest {
    /// The OCID of the instance configuration.
    pub instance_configuration_id: String,
}


/// Required fields for GetInstanceConfigurationRequest
pub struct GetInstanceConfigurationRequestRequired {
    /// The OCID of the instance configuration.
    pub instance_configuration_id: String,
}

impl GetInstanceConfigurationRequest {
    /// Create a new GetInstanceConfigurationRequest with required fields
    pub fn new(required: GetInstanceConfigurationRequestRequired) -> Self {
        Self {
            instance_configuration_id: required.instance_configuration_id,
}
    }

    /// Set instance_configuration_id
    pub fn set_instance_configuration_id(mut self, value: String) -> Self {
        self.instance_configuration_id = value;
        self
    }
}


