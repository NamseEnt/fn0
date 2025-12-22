use serde::{Deserialize, Serialize};

/// The shape configuration requested for the instance.
///
/// If the parameter is provided, the instance is created with the resources that you specify.
/// If some properties are missing or the entire parameter is not provided, the instance is created
/// with the default configuration values for the shape that you specify.
///
/// Each shape only supports certain configurable values. If the values that you provide are not
/// valid for the specified shape, an error is returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInstanceShapeConfigDetails {
    /// The total number of OCPUs available to the instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocpus: Option<f64>,

    /// The total number of VCPUs available to the instance. This can be used instead of OCPUs,
    /// in which case the actual number of OCPUs will be calculated based on this value
    /// and the actual hardware. This must be a multiple of 2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcpus: Option<i32>,

    /// The total amount of memory available to the instance, in gigabytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_in_g_bs: Option<f64>,

    /// The baseline OCPU utilization for a subcore burstable VM instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_ocpu_utilization: Option<BaselineOcpuUtilization>,

    /// The number of NVMe drives to be used for storage. A single drive has 6.8 TB available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvmes: Option<i32>,

    /// This field is reserved for internal use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_management: Option<ResourceManagement>,
}

/// The baseline OCPU utilization for a subcore burstable VM instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BaselineOcpuUtilization {
    /// Baseline usage is 1/8 of an OCPU
    #[serde(rename = "BASELINE_1_8")]
    Baseline18,
    /// Baseline usage is 1/2 of an OCPU
    #[serde(rename = "BASELINE_1_2")]
    Baseline12,
    /// Baseline usage is an entire OCPU (non-burstable instance)
    #[serde(rename = "BASELINE_1_1")]
    Baseline11,
}

/// Resource management mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceManagement {
    /// Dynamic resource management
    #[serde(rename = "DYNAMIC")]
    Dynamic,
    /// Static resource management
    #[serde(rename = "STATIC")]
    Static,
}

impl LaunchInstanceShapeConfigDetails {
    /// Create a new empty shape config
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

    /// Set the number of OCPUs
    pub fn with_ocpus(mut self, ocpus: f64) -> Self {
        self.ocpus = Some(ocpus);
        self
    }

    /// Set the number of VCPUs
    pub fn with_vcpus(mut self, vcpus: i32) -> Self {
        self.vcpus = Some(vcpus);
        self
    }

    /// Set the memory in GBs
    pub fn with_memory_in_gbs(mut self, memory: f64) -> Self {
        self.memory_in_g_bs = Some(memory);
        self
    }

    /// Set the baseline OCPU utilization
    pub fn with_baseline_ocpu_utilization(mut self, utilization: BaselineOcpuUtilization) -> Self {
        self.baseline_ocpu_utilization = Some(utilization);
        self
    }

    /// Set the number of NVMe drives
    pub fn with_nvmes(mut self, nvmes: i32) -> Self {
        self.nvmes = Some(nvmes);
        self
    }

    /// Set the resource management mode
    pub fn with_resource_management(mut self, mode: ResourceManagement) -> Self {
        self.resource_management = Some(mode);
        self
    }

    /// Set ocpus
    pub fn set_ocpus(mut self, ocpus: Option<f64>) -> Self {
        self.ocpus = ocpus;
        self
    }

    /// Set vcpus
    pub fn set_vcpus(mut self, vcpus: Option<i32>) -> Self {
        self.vcpus = vcpus;
        self
    }

    /// Set memory_in_g_bs
    pub fn set_memory_in_g_bs(mut self, memory_in_g_bs: Option<f64>) -> Self {
        self.memory_in_g_bs = memory_in_g_bs;
        self
    }

    /// Set baseline_ocpu_utilization
    pub fn set_baseline_ocpu_utilization(mut self, baseline_ocpu_utilization: Option<BaselineOcpuUtilization>) -> Self {
        self.baseline_ocpu_utilization = baseline_ocpu_utilization;
        self
    }

    /// Set nvmes
    pub fn set_nvmes(mut self, nvmes: Option<i32>) -> Self {
        self.nvmes = nvmes;
        self
    }

    /// Set resource_management
    pub fn set_resource_management(mut self, resource_management: Option<ResourceManagement>) -> Self {
        self.resource_management = resource_management;
        self
    }
}

impl Default for LaunchInstanceShapeConfigDetails {
    fn default() -> Self {
        Self::new()
    }
}
