use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The platform configuration requested for the instance.
///
/// If you provide the parameter, the instance is created with the platform configuration that you specify.
/// For any values that you omit, the instance uses the default configuration values for the shape that you specify.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstanceConfigurationLaunchInstancePlatformConfig {
    AmdMilanBm(AmdMilanBmPlatformConfig),
    IntelVm(IntelVmPlatformConfig),
    AmdMilanBmGpu(AmdMilanBmGpuPlatformConfig),
    IntelIcelakeBm(IntelIcelakeBmPlatformConfig),
    GenericBm(GenericBmPlatformConfig),
    AmdRomeBm(AmdRomeBmPlatformConfig),
    IntelSkylakeBm(IntelSkylakeBmPlatformConfig),
    AmdRomeBmGpu(AmdRomeBmGpuPlatformConfig),
    AmdVm(AmdVmPlatformConfig),
}

/// Common platform configuration fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonPlatformConfig {
    /// Whether Secure Boot is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_secure_boot_enabled: Option<bool>,

    /// Whether the Trusted Platform Module (TPM) is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_trusted_platform_module_enabled: Option<bool>,

    /// Whether the Measured Boot feature is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_measured_boot_enabled: Option<bool>,

    /// Whether the instance is a confidential instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_memory_encryption_enabled: Option<bool>,
}

/// AMD Milan BM platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmdMilanBmPlatformConfig {
    #[serde(flatten)]
    pub common: CommonPlatformConfig,

    /// The number of NUMA nodes per socket (NPS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numa_nodes_per_socket: Option<NumaNodesPerSocket>,

    /// Whether symmetric multithreading is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symmetric_multi_threading_enabled: Option<bool>,

    /// Whether the Access Control Service is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_access_control_service_enabled: Option<bool>,

    /// Whether virtualization instructions are available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub are_virtual_instructions_enabled: Option<bool>,

    /// Whether the input-output memory management unit is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_input_output_memory_management_unit_enabled: Option<bool>,

    /// The percentage of cores enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage_of_cores_enabled: Option<i32>,

    /// Instance Platform Configuration Configuration Map for flexible setting input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_map: Option<HashMap<String, String>>,
}

/// NUMA nodes per socket configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumaNodesPerSocket {
    #[serde(rename = "NPS0")]
    Nps0,
    #[serde(rename = "NPS1")]
    Nps1,
    #[serde(rename = "NPS2")]
    Nps2,
    #[serde(rename = "NPS4")]
    Nps4,
    #[serde(rename = "NPS6")]
    Nps6,
}

/// Intel VM platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntelVmPlatformConfig {
    #[serde(flatten)]
    pub common: CommonPlatformConfig,

    /// Whether symmetric multithreading is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symmetric_multi_threading_enabled: Option<bool>,
}

/// AMD Milan BM GPU platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmdMilanBmGpuPlatformConfig {
    #[serde(flatten)]
    pub common: CommonPlatformConfig,

    /// The number of NUMA nodes per socket (NPS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numa_nodes_per_socket: Option<NumaNodesPerSocket>,

    /// Whether symmetric multithreading is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symmetric_multi_threading_enabled: Option<bool>,

    /// Whether the Access Control Service is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_access_control_service_enabled: Option<bool>,

    /// Whether virtualization instructions are available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub are_virtual_instructions_enabled: Option<bool>,

    /// Whether the input-output memory management unit is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_input_output_memory_management_unit_enabled: Option<bool>,
}

/// Intel Icelake BM platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntelIcelakeBmPlatformConfig {
    #[serde(flatten)]
    pub common: CommonPlatformConfig,

    /// Whether symmetric multithreading is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symmetric_multi_threading_enabled: Option<bool>,

    /// The percentage of cores enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage_of_cores_enabled: Option<i32>,

    /// Instance Platform Configuration Configuration Map for flexible setting input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_map: Option<HashMap<String, String>>,
}

/// Generic BM platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericBmPlatformConfig {
    #[serde(flatten)]
    pub common: CommonPlatformConfig,
}

/// AMD Rome BM platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmdRomeBmPlatformConfig {
    #[serde(flatten)]
    pub common: CommonPlatformConfig,

    /// The number of NUMA nodes per socket (NPS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numa_nodes_per_socket: Option<NumaNodesPerSocket>,

    /// Whether symmetric multithreading is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symmetric_multi_threading_enabled: Option<bool>,

    /// The percentage of cores enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage_of_cores_enabled: Option<i32>,
}

/// Intel Skylake BM platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntelSkylakeBmPlatformConfig {
    #[serde(flatten)]
    pub common: CommonPlatformConfig,

    /// Whether symmetric multithreading is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symmetric_multi_threading_enabled: Option<bool>,

    /// The percentage of cores enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage_of_cores_enabled: Option<i32>,
}

/// AMD Rome BM GPU platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmdRomeBmGpuPlatformConfig {
    #[serde(flatten)]
    pub common: CommonPlatformConfig,

    /// The number of NUMA nodes per socket (NPS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numa_nodes_per_socket: Option<NumaNodesPerSocket>,

    /// Whether symmetric multithreading is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symmetric_multi_threading_enabled: Option<bool>,
}

/// AMD VM platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmdVmPlatformConfig {
    #[serde(flatten)]
    pub common: CommonPlatformConfig,

    /// Whether symmetric multithreading is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symmetric_multi_threading_enabled: Option<bool>,
}
