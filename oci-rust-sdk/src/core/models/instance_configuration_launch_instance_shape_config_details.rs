use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The shape configuration requested for the instance. <p> If the parameter is provided, the instance is created with the resources that you specify. If some properties are missing or the entire parameter is not provided, the instance is created with the default configuration values for the {@code shape} that you specify. <p> Each shape only supports certain configurable values. If the values that you provide are not valid for the specified {@code shape}, an error is returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationLaunchInstanceShapeConfigDetails {
    /// The total number of OCPUs available to the instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocpus: Option<i64>,

    /// The total number of VCPUs available to the instance. This can be used instead of OCPUs, in which case the actual number of OCPUs will be calculated based on this value and the actual hardware. This must be a multiple of 2. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcpus: Option<i64>,

    /// The total amount of memory available to the instance, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_in_g_bs: Option<i64>,

    /// The baseline OCPU utilization for a subcore burstable VM instance. Leave this attribute blank for a non-burstable instance, or explicitly specify non-burstable with {@code BASELINE_1_1}. <p> The following values are supported: - {@code BASELINE_1_8} - baseline usage is 1/8 of an OCPU. - {@code BASELINE_1_2} - baseline usage is 1/2 of an OCPU. - {@code BASELINE_1_1} - baseline usage is an entire OCPU. This represents a non-burstable instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_ocpu_utilization:
        Option<InstanceConfigurationLaunchInstanceShapeConfigDetailsBaselineOcpuUtilization>,

    /// The number of NVMe drives to be used for storage. A single drive has 6.8 TB available. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvmes: Option<i64>,

    /// This field is reserved for internal use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_management:
        Option<InstanceConfigurationLaunchInstanceShapeConfigDetailsResourceManagement>,
}

impl InstanceConfigurationLaunchInstanceShapeConfigDetails {
    /// Create a new InstanceConfigurationLaunchInstanceShapeConfigDetails
    pub fn new() -> Self {
        Self {
            ocpus: None,

            vcpus: None,

            memory_in_g_bs: None,

            baseline_ocpu_utilization: None,

            nvmes: None,

            resource_management: None,
        }
    }

    /// Set ocpus
    pub fn set_ocpus(mut self, value: Option<i64>) -> Self {
        self.ocpus = value;
        self
    }

    /// Set vcpus
    pub fn set_vcpus(mut self, value: Option<i64>) -> Self {
        self.vcpus = value;
        self
    }

    /// Set memory_in_g_bs
    pub fn set_memory_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.memory_in_g_bs = value;
        self
    }

    /// Set baseline_ocpu_utilization
    pub fn set_baseline_ocpu_utilization(
        mut self,
        value: Option<InstanceConfigurationLaunchInstanceShapeConfigDetailsBaselineOcpuUtilization>,
    ) -> Self {
        self.baseline_ocpu_utilization = value;
        self
    }

    /// Set nvmes
    pub fn set_nvmes(mut self, value: Option<i64>) -> Self {
        self.nvmes = value;
        self
    }

    /// Set resource_management
    pub fn set_resource_management(
        mut self,
        value: Option<InstanceConfigurationLaunchInstanceShapeConfigDetailsResourceManagement>,
    ) -> Self {
        self.resource_management = value;
        self
    }

    /// Set ocpus (unwraps Option)
    pub fn with_ocpus(mut self, value: i64) -> Self {
        self.ocpus = Some(value);
        self
    }

    /// Set vcpus (unwraps Option)
    pub fn with_vcpus(mut self, value: i64) -> Self {
        self.vcpus = Some(value);
        self
    }

    /// Set memory_in_g_bs (unwraps Option)
    pub fn with_memory_in_g_bs(mut self, value: i64) -> Self {
        self.memory_in_g_bs = Some(value);
        self
    }

    /// Set baseline_ocpu_utilization (unwraps Option)
    pub fn with_baseline_ocpu_utilization(
        mut self,
        value: InstanceConfigurationLaunchInstanceShapeConfigDetailsBaselineOcpuUtilization,
    ) -> Self {
        self.baseline_ocpu_utilization = Some(value);
        self
    }

    /// Set nvmes (unwraps Option)
    pub fn with_nvmes(mut self, value: i64) -> Self {
        self.nvmes = Some(value);
        self
    }

    /// Set resource_management (unwraps Option)
    pub fn with_resource_management(
        mut self,
        value: InstanceConfigurationLaunchInstanceShapeConfigDetailsResourceManagement,
    ) -> Self {
        self.resource_management = Some(value);
        self
    }
}

impl Default for InstanceConfigurationLaunchInstanceShapeConfigDetails {
    fn default() -> Self {
        Self::new()
    }
}
