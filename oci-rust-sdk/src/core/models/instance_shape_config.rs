use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The shape configuration for an instance. The shape configuration determines the resources allocated to an instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceShapeConfig {
    /// The total number of OCPUs available to the instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocpus: Option<i64>,

    /// The total amount of memory available to the instance, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_in_g_bs: Option<i64>,

    /// The baseline OCPU utilization for a subcore burstable VM instance. Leave this attribute blank for a non-burstable instance, or explicitly specify non-burstable with {@code BASELINE_1_1}. <p> The following values are supported: - {@code BASELINE_1_8} - baseline usage is 1/8 of an OCPU. - {@code BASELINE_1_2} - baseline usage is 1/2 of an OCPU. - {@code BASELINE_1_1} - baseline usage is the entire OCPU. This represents a non-burstable instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_ocpu_utilization: Option<InstanceShapeConfigBaselineOcpuUtilization>,

    /// A short description of the instance's processor (CPU).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processor_description: Option<String>,

    /// The networking bandwidth available to the instance, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networking_bandwidth_in_gbps: Option<i64>,

    /// The maximum number of VNIC attachments for the instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_vnic_attachments: Option<i64>,

    /// The number of GPUs available to the instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpus: Option<i64>,

    /// A short description of the instance's graphics processing unit (GPU). <p> If the instance does not have any GPUs, this field is {@code null}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_description: Option<String>,

    /// The number of local disks available to the instance. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_disks: Option<i64>,

    /// The aggregate size of all local disks, in gigabytes. <p> If the instance does not have any local disks, this field is {@code null}. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_disks_total_size_in_g_bs: Option<i64>,

    /// A short description of the local disks available to this instance. <p> If the instance does not have any local disks, this field is {@code null}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_disk_description: Option<String>,

    /// The total number of VCPUs available to the instance. This can be used instead of OCPUs, in which case the actual number of OCPUs will be calculated based on this value and the actual hardware. This must be a multiple of 2. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcpus: Option<i64>,

    /// This field is reserved for internal use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_management: Option<InstanceShapeConfigResourceManagement>,
}

impl InstanceShapeConfig {
    /// Create a new InstanceShapeConfig
    pub fn new() -> Self {
        Self {
            ocpus: None,

            memory_in_g_bs: None,

            baseline_ocpu_utilization: None,

            processor_description: None,

            networking_bandwidth_in_gbps: None,

            max_vnic_attachments: None,

            gpus: None,

            gpu_description: None,

            local_disks: None,

            local_disks_total_size_in_g_bs: None,

            local_disk_description: None,

            vcpus: None,

            resource_management: None,
        }
    }

    /// Set ocpus
    pub fn set_ocpus(mut self, value: Option<i64>) -> Self {
        self.ocpus = value;
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
        value: Option<InstanceShapeConfigBaselineOcpuUtilization>,
    ) -> Self {
        self.baseline_ocpu_utilization = value;
        self
    }

    /// Set processor_description
    pub fn set_processor_description(mut self, value: Option<String>) -> Self {
        self.processor_description = value;
        self
    }

    /// Set networking_bandwidth_in_gbps
    pub fn set_networking_bandwidth_in_gbps(mut self, value: Option<i64>) -> Self {
        self.networking_bandwidth_in_gbps = value;
        self
    }

    /// Set max_vnic_attachments
    pub fn set_max_vnic_attachments(mut self, value: Option<i64>) -> Self {
        self.max_vnic_attachments = value;
        self
    }

    /// Set gpus
    pub fn set_gpus(mut self, value: Option<i64>) -> Self {
        self.gpus = value;
        self
    }

    /// Set gpu_description
    pub fn set_gpu_description(mut self, value: Option<String>) -> Self {
        self.gpu_description = value;
        self
    }

    /// Set local_disks
    pub fn set_local_disks(mut self, value: Option<i64>) -> Self {
        self.local_disks = value;
        self
    }

    /// Set local_disks_total_size_in_g_bs
    pub fn set_local_disks_total_size_in_g_bs(mut self, value: Option<i64>) -> Self {
        self.local_disks_total_size_in_g_bs = value;
        self
    }

    /// Set local_disk_description
    pub fn set_local_disk_description(mut self, value: Option<String>) -> Self {
        self.local_disk_description = value;
        self
    }

    /// Set vcpus
    pub fn set_vcpus(mut self, value: Option<i64>) -> Self {
        self.vcpus = value;
        self
    }

    /// Set resource_management
    pub fn set_resource_management(
        mut self,
        value: Option<InstanceShapeConfigResourceManagement>,
    ) -> Self {
        self.resource_management = value;
        self
    }

    /// Set ocpus (unwraps Option)
    pub fn with_ocpus(mut self, value: i64) -> Self {
        self.ocpus = Some(value);
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
        value: InstanceShapeConfigBaselineOcpuUtilization,
    ) -> Self {
        self.baseline_ocpu_utilization = Some(value);
        self
    }

    /// Set processor_description (unwraps Option)
    pub fn with_processor_description(mut self, value: impl Into<String>) -> Self {
        self.processor_description = Some(value.into());
        self
    }

    /// Set networking_bandwidth_in_gbps (unwraps Option)
    pub fn with_networking_bandwidth_in_gbps(mut self, value: i64) -> Self {
        self.networking_bandwidth_in_gbps = Some(value);
        self
    }

    /// Set max_vnic_attachments (unwraps Option)
    pub fn with_max_vnic_attachments(mut self, value: i64) -> Self {
        self.max_vnic_attachments = Some(value);
        self
    }

    /// Set gpus (unwraps Option)
    pub fn with_gpus(mut self, value: i64) -> Self {
        self.gpus = Some(value);
        self
    }

    /// Set gpu_description (unwraps Option)
    pub fn with_gpu_description(mut self, value: impl Into<String>) -> Self {
        self.gpu_description = Some(value.into());
        self
    }

    /// Set local_disks (unwraps Option)
    pub fn with_local_disks(mut self, value: i64) -> Self {
        self.local_disks = Some(value);
        self
    }

    /// Set local_disks_total_size_in_g_bs (unwraps Option)
    pub fn with_local_disks_total_size_in_g_bs(mut self, value: i64) -> Self {
        self.local_disks_total_size_in_g_bs = Some(value);
        self
    }

    /// Set local_disk_description (unwraps Option)
    pub fn with_local_disk_description(mut self, value: impl Into<String>) -> Self {
        self.local_disk_description = Some(value.into());
        self
    }

    /// Set vcpus (unwraps Option)
    pub fn with_vcpus(mut self, value: i64) -> Self {
        self.vcpus = Some(value);
        self
    }

    /// Set resource_management (unwraps Option)
    pub fn with_resource_management(
        mut self,
        value: InstanceShapeConfigResourceManagement,
    ) -> Self {
        self.resource_management = Some(value);
        self
    }
}

impl Default for InstanceShapeConfig {
    fn default() -> Self {
        Self::new()
    }
}
