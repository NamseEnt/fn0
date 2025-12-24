use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The shape configuration for a container instance. The shape configuration determines the resources thats are available to the container instance and its containers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInstanceShapeConfig {
    /// The total number of OCPUs available to the container instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub ocpus: i64,

    /// The total amount of memory available to the container instance, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub memory_in_g_bs: i64,

    /// A short description of the container instance's processor (CPU).
    pub processor_description: String,

    /// The networking bandwidth available to the container instance, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub networking_bandwidth_in_gbps: i64,
}

/// Required fields for ContainerInstanceShapeConfig
pub struct ContainerInstanceShapeConfigRequired {
    /// The total number of OCPUs available to the container instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub ocpus: i64,

    /// The total amount of memory available to the container instance, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub memory_in_g_bs: i64,

    /// A short description of the container instance's processor (CPU).
    pub processor_description: String,

    /// The networking bandwidth available to the container instance, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub networking_bandwidth_in_gbps: i64,
}

impl ContainerInstanceShapeConfig {
    /// Create a new ContainerInstanceShapeConfig with required fields
    pub fn new(required: ContainerInstanceShapeConfigRequired) -> Self {
        Self {
            ocpus: required.ocpus,

            memory_in_g_bs: required.memory_in_g_bs,

            processor_description: required.processor_description,

            networking_bandwidth_in_gbps: required.networking_bandwidth_in_gbps,
        }
    }

    /// Set ocpus
    pub fn set_ocpus(mut self, value: i64) -> Self {
        self.ocpus = value;
        self
    }

    /// Set memory_in_g_bs
    pub fn set_memory_in_g_bs(mut self, value: i64) -> Self {
        self.memory_in_g_bs = value;
        self
    }

    /// Set processor_description
    pub fn set_processor_description(mut self, value: String) -> Self {
        self.processor_description = value;
        self
    }

    /// Set networking_bandwidth_in_gbps
    pub fn set_networking_bandwidth_in_gbps(mut self, value: i64) -> Self {
        self.networking_bandwidth_in_gbps = value;
        self
    }
}
