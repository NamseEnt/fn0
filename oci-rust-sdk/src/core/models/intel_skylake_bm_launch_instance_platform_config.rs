use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The platform configuration used when launching a bare metal instance with an Intel X7-based processor (the Intel Skylake platform).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntelSkylakeBmLaunchInstancePlatformConfig {
    #[serde(rename = "type")]
    pub r#type: String,

    /// The number of NUMA nodes per socket (NPS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numa_nodes_per_socket: Option<IntelSkylakeBmLaunchInstancePlatformConfigNumaNodesPerSocket>,

    /// Whether symmetric multithreading is enabled on the instance. Symmetric multithreading is also called simultaneous multithreading (SMT) or Intel Hyper-Threading. <p> Intel and AMD processors have two hardware execution threads per core (OCPU). SMT permits multiple independent threads of execution, to better use the resources and increase the efficiency of the CPU. When multithreading is disabled, only one thread is permitted to run on each core, which can provide higher or more predictable performance for some workloads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symmetric_multi_threading_enabled: Option<bool>,

    /// Whether the input-output memory management unit is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_input_output_memory_management_unit_enabled: Option<bool>,

    /// The percentage of cores enabled. Value must be a multiple of 25%. If the requested percentage results in a fractional number of cores, the system rounds up the number of cores across processors and provisions an instance with a whole number of cores. <p> If the applications that you run on the instance use a core-based licensing model and need fewer cores than the full size of the shape, you can disable cores to reduce your licensing costs. The instance itself is billed for the full shape, regardless of whether all cores are enabled. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage_of_cores_enabled: Option<i64>,

    /// Instance Platform Configuration Configuration Map for flexible setting input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_map: Option<HashMap<String, String>>,
}

/// Required fields for IntelSkylakeBmLaunchInstancePlatformConfig
pub struct IntelSkylakeBmLaunchInstancePlatformConfigRequired {
    pub r#type: String,
}

impl IntelSkylakeBmLaunchInstancePlatformConfig {
    /// Create a new IntelSkylakeBmLaunchInstancePlatformConfig with required fields
    pub fn new(required: IntelSkylakeBmLaunchInstancePlatformConfigRequired) -> Self {
        Self {
            r#type: required.r#type,

            numa_nodes_per_socket: None,

            is_symmetric_multi_threading_enabled: None,

            is_input_output_memory_management_unit_enabled: None,

            percentage_of_cores_enabled: None,

            config_map: None,
        }
    }

    /// Set numa_nodes_per_socket
    pub fn set_numa_nodes_per_socket(
        mut self,
        value: Option<IntelSkylakeBmLaunchInstancePlatformConfigNumaNodesPerSocket>,
    ) -> Self {
        self.numa_nodes_per_socket = value;
        self
    }

    /// Set is_symmetric_multi_threading_enabled
    pub fn set_is_symmetric_multi_threading_enabled(mut self, value: Option<bool>) -> Self {
        self.is_symmetric_multi_threading_enabled = value;
        self
    }

    /// Set is_input_output_memory_management_unit_enabled
    pub fn set_is_input_output_memory_management_unit_enabled(
        mut self,
        value: Option<bool>,
    ) -> Self {
        self.is_input_output_memory_management_unit_enabled = value;
        self
    }

    /// Set percentage_of_cores_enabled
    pub fn set_percentage_of_cores_enabled(mut self, value: Option<i64>) -> Self {
        self.percentage_of_cores_enabled = value;
        self
    }

    /// Set config_map
    pub fn set_config_map(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.config_map = value;
        self
    }

    /// Set r#type
    pub fn set_type(mut self, value: String) -> Self {
        self.r#type = value;
        self
    }

    /// Set numa_nodes_per_socket (unwraps Option)
    pub fn with_numa_nodes_per_socket(
        mut self,
        value: IntelSkylakeBmLaunchInstancePlatformConfigNumaNodesPerSocket,
    ) -> Self {
        self.numa_nodes_per_socket = Some(value);
        self
    }

    /// Set is_symmetric_multi_threading_enabled (unwraps Option)
    pub fn with_is_symmetric_multi_threading_enabled(mut self, value: bool) -> Self {
        self.is_symmetric_multi_threading_enabled = Some(value);
        self
    }

    /// Set is_input_output_memory_management_unit_enabled (unwraps Option)
    pub fn with_is_input_output_memory_management_unit_enabled(mut self, value: bool) -> Self {
        self.is_input_output_memory_management_unit_enabled = Some(value);
        self
    }

    /// Set percentage_of_cores_enabled (unwraps Option)
    pub fn with_percentage_of_cores_enabled(mut self, value: i64) -> Self {
        self.percentage_of_cores_enabled = Some(value);
        self
    }

    /// Set config_map (unwraps Option)
    pub fn with_config_map(mut self, value: HashMap<String, String>) -> Self {
        self.config_map = Some(value);
        self
    }
}
