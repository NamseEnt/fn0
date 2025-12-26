use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details for creating an instance configuration using an existing instance as a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstanceConfigurationFromInstanceDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance to use to create the instance configuration.
    pub instance_id: String,

    pub source: String,
}

/// Required fields for CreateInstanceConfigurationFromInstanceDetails
pub struct CreateInstanceConfigurationFromInstanceDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the instance to use to create the instance configuration.
    pub instance_id: String,

    pub source: String,
}

impl CreateInstanceConfigurationFromInstanceDetails {
    /// Create a new CreateInstanceConfigurationFromInstanceDetails with required fields
    pub fn new(required: CreateInstanceConfigurationFromInstanceDetailsRequired) -> Self {
        Self {
            instance_id: required.instance_id,

            source: required.source,
        }
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: String) -> Self {
        self.instance_id = value;
        self
    }

    /// Set source
    pub fn set_source(mut self, value: String) -> Self {
        self.source = value;
        self
    }
}
