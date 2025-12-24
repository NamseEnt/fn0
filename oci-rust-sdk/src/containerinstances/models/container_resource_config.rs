use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The resource configuration for a container. The resource configuration determines the amount of resources allocated to the container and the maximum allowed resources for a container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerResourceConfig {
    /// The maximum amount of CPUs that can be consumed by the container's process. <p> If you do not set a value, then the process may use all available CPU resources on the container instance. <p> CPU usage is defined in terms of logical CPUs. This means that the maximum possible value on an E3 ContainerInstance with 1 OCPU is 2.0. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcpus_limit: Option<i64>,

    /// The maximum amount of memory that can be consumed by the container's process. If you do not set a value, then the process may use all available memory on the instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit_in_g_bs: Option<i64>,
}

impl ContainerResourceConfig {
    /// Create a new ContainerResourceConfig
    pub fn new() -> Self {
        Self {
            vcpus_limit: None,

            memory_limit_in_g_bs: None,
        }
    }

    /// Set vcpus_limit
    pub fn set_vcpus_limit(mut self, value: Option<i64>) -> Self {
        self.vcpus_limit = value;
        self
    }

    /// Set memory_limit_in_g_bs
    pub fn set_memory_limit_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.memory_limit_in_g_bs = value;
        self
    }

    /// Set vcpus_limit (unwraps Option)
    pub fn with_vcpus_limit(mut self, value: i64) -> Self {
        self.vcpus_limit = Some(value);
        self
    }

    /// Set memory_limit_in_g_bs (unwraps Option)
    pub fn with_memory_limit_in_g_bs(mut self, value: i64) -> Self {
        self.memory_limit_in_g_bs = Some(value);
        self
    }
}

impl Default for ContainerResourceConfig {
    fn default() -> Self {
        Self::new()
    }
}
