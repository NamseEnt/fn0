use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The platform configuration used when updating a virtual machine instance with the AMD platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmdVmUpdateInstancePlatformConfig {
    #[serde(rename = "type")]
    pub r#type: String,

    /// Whether symmetric multithreading is enabled on the instance. Symmetric multithreading is also called simultaneous multithreading (SMT) or Intel Hyper-Threading. <p> Intel and AMD processors have two hardware execution threads per core (OCPU). SMT permits multiple independent threads of execution, to better use the resources and increase the efficiency of the CPU. When multithreading is disabled, only one thread is permitted to run on each core, which can provide higher or more predictable performance for some workloads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symmetric_multi_threading_enabled: Option<bool>,
}

/// Required fields for AmdVmUpdateInstancePlatformConfig
pub struct AmdVmUpdateInstancePlatformConfigRequired {
    pub r#type: String,
}

impl AmdVmUpdateInstancePlatformConfig {
    /// Create a new AmdVmUpdateInstancePlatformConfig with required fields
    pub fn new(required: AmdVmUpdateInstancePlatformConfigRequired) -> Self {
        Self {
            r#type: required.r#type,

            is_symmetric_multi_threading_enabled: None,
        }
    }

    /// Set is_symmetric_multi_threading_enabled
    pub fn set_is_symmetric_multi_threading_enabled(mut self, value: Option<bool>) -> Self {
        self.is_symmetric_multi_threading_enabled = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set is_symmetric_multi_threading_enabled (unwraps Option)
    pub fn with_is_symmetric_multi_threading_enabled(mut self, value: bool) -> Self {
        self.is_symmetric_multi_threading_enabled = Some(value);
        self
    }
}
