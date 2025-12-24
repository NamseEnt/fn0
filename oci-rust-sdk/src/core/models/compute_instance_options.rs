use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Multiple Compute Instance Configuration instance details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeInstanceOptions {
    pub instance_type: String,

    /// The Compute Instance Configuration parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ComputeInstanceDetails>>,
}

/// Required fields for ComputeInstanceOptions
pub struct ComputeInstanceOptionsRequired {
    pub instance_type: String,
}

impl ComputeInstanceOptions {
    /// Create a new ComputeInstanceOptions with required fields
    pub fn new(required: ComputeInstanceOptionsRequired) -> Self {
        Self {
            instance_type: required.instance_type,

            options: None,
        }
    }

    /// Set options
    pub fn set_options(mut self, value: Option<Vec<ComputeInstanceDetails>>) -> Self {
        self.options = value;
        self
    }

    /// Set instance_type
    pub fn set_instance_type(mut self, value: String) -> Self {
        self.instance_type = value;
        self
    }

    /// Set options (unwraps Option)
    pub fn with_options(mut self, value: Vec<ComputeInstanceDetails>) -> Self {
        self.options = Some(value);
        self
    }
}
