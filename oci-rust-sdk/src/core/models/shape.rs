use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A compute instance shape that can be used in {@link #launchInstance(LaunchInstanceRequest) launchInstance}. For more information, see [Overview of the Compute Service](https://docs.oracle.com/iaas/Content/Compute/Concepts/computeoverview.htm) and [Compute Shapes](https://docs.oracle.com/iaas/Content/Compute/References/computeshapes.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shape {
    /// The name of the shape. You can enumerate all available shapes by calling {@link #listShapes(ListShapesRequest) listShapes}.
    pub shape: String,

    /// For a subcore burstable VM, the supported baseline OCPU utilization for instances that use this shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_ocpu_utilizations: Option<Vec<ShapeBaselineOcpuUtilizations>>,

    /// For a subcore burstable VM, the minimum total baseline OCPUs required. The total baseline OCPUs is equal to baselineOcpuUtilization chosen multiplied by the number of OCPUs chosen. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_total_baseline_ocpus_required: Option<i64>,

    /// A short description of the shape's processor (CPU).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processor_description: Option<String>,

    /// The default number of OCPUs available for this shape. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocpus: Option<i64>,

    /// The default amount of memory available for this shape, in gigabytes. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_in_g_bs: Option<i64>,

    /// The number of physical network interface card (NIC) ports available for this shape. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_ports: Option<i64>,

    /// The networking bandwidth available for this shape, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networking_bandwidth_in_gbps: Option<i64>,

    /// The maximum number of VNIC attachments available for this shape. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_vnic_attachments: Option<i64>,

    /// The number of GPUs available for this shape. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpus: Option<i64>,

    /// A short description of the graphics processing unit (GPU) available for this shape. <p> If the shape does not have any GPUs, this field is {@code null}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_description: Option<String>,

    /// The number of local disks available for this shape. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_disks: Option<i64>,

    /// The aggregate size of the local disks available for this shape, in gigabytes. <p> If the shape does not have any local disks, this field is {@code null}. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_disks_total_size_in_g_bs: Option<i64>,

    /// A short description of the local disks available for this shape. <p> If the shape does not have any local disks, this field is {@code null}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_disk_description: Option<String>,

    /// The number of networking ports available for the remote direct memory access (RDMA) network between nodes in a high performance computing (HPC) cluster network. If the shape does not support cluster networks, this value is {@code 0}. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rdma_ports: Option<i64>,

    /// The networking bandwidth available for the remote direct memory access (RDMA) network for this shape, in gigabits per second. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rdma_bandwidth_in_gbps: Option<i64>,

    /// Whether live migration is supported for this shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_live_migration_supported: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocpu_options: Option<ShapeOcpuOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_options: Option<ShapeMemoryOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub networking_bandwidth_options: Option<ShapeNetworkingBandwidthOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_vnic_attachment_options: Option<ShapeMaxVnicAttachmentOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_config_options: Option<ShapePlatformConfigOptions>,

    /// Whether billing continues when the instances that use this shape are in the stopped state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_billed_for_stopped_instance: Option<bool>,

    /// How instances that use this shape are charged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_type: Option<ShapeBillingType>,

    /// The list of of compartment quotas for the shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_names: Option<Vec<String>>,

    /// Whether the shape supports creating subcore or burstable instances. A [burstable instance](https://docs.oracle.com/iaas/Content/Compute/References/burstable-instances.htm) is a virtual machine (VM) instance that provides a baseline level of CPU performance with the ability to burst to a higher level to support occasional spikes in usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_subcore: Option<bool>,

    /// Whether the shape supports creating flexible instances. A [flexible shape](https://docs.oracle.com/iaas/Content/Compute/References/computeshapes.htm#flexible) is a shape that lets you customize the number of OCPUs and the amount of memory when launching or resizing your instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_flexible: Option<bool>,

    /// The list of compatible shapes that this shape can be changed to. For more information, see [Changing the Shape of an Instance](https://docs.oracle.com/iaas/Content/Compute/Tasks/resizinginstances.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resize_compatible_shapes: Option<Vec<String>>,

    /// The list of shapes and shape details (if applicable) that Oracle recommends that you use as an alternative to the current shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_alternatives: Option<Vec<ShapeAlternativeObject>>,
}

/// Required fields for Shape
pub struct ShapeRequired {
    /// The name of the shape. You can enumerate all available shapes by calling {@link #listShapes(ListShapesRequest) listShapes}.
    pub shape: String,
}

impl Shape {
    /// Create a new Shape with required fields
    pub fn new(required: ShapeRequired) -> Self {
        Self {
            shape: required.shape,

            baseline_ocpu_utilizations: None,

            min_total_baseline_ocpus_required: None,

            processor_description: None,

            ocpus: None,

            memory_in_g_bs: None,

            network_ports: None,

            networking_bandwidth_in_gbps: None,

            max_vnic_attachments: None,

            gpus: None,

            gpu_description: None,

            local_disks: None,

            local_disks_total_size_in_g_bs: None,

            local_disk_description: None,

            rdma_ports: None,

            rdma_bandwidth_in_gbps: None,

            is_live_migration_supported: None,

            ocpu_options: None,

            memory_options: None,

            networking_bandwidth_options: None,

            max_vnic_attachment_options: None,

            platform_config_options: None,

            is_billed_for_stopped_instance: None,

            billing_type: None,

            quota_names: None,

            is_subcore: None,

            is_flexible: None,

            resize_compatible_shapes: None,

            recommended_alternatives: None,
        }
    }

    /// Set baseline_ocpu_utilizations
    pub fn set_baseline_ocpu_utilizations(
        mut self,
        value: Option<Vec<ShapeBaselineOcpuUtilizations>>,
    ) -> Self {
        self.baseline_ocpu_utilizations = value;
        self
    }

    /// Set min_total_baseline_ocpus_required
    pub fn set_min_total_baseline_ocpus_required(mut self, value: Option<i64>) -> Self {
        self.min_total_baseline_ocpus_required = value;
        self
    }

    /// Set shape
    pub fn set_shape(mut self, value: String) -> Self {
        self.shape = value;
        self
    }

    /// Set processor_description
    pub fn set_processor_description(mut self, value: Option<String>) -> Self {
        self.processor_description = value;
        self
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

    /// Set network_ports
    pub fn set_network_ports(mut self, value: Option<i64>) -> Self {
        self.network_ports = value;
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

    /// Set rdma_ports
    pub fn set_rdma_ports(mut self, value: Option<i64>) -> Self {
        self.rdma_ports = value;
        self
    }

    /// Set rdma_bandwidth_in_gbps
    pub fn set_rdma_bandwidth_in_gbps(mut self, value: Option<i64>) -> Self {
        self.rdma_bandwidth_in_gbps = value;
        self
    }

    /// Set is_live_migration_supported
    pub fn set_is_live_migration_supported(mut self, value: Option<bool>) -> Self {
        self.is_live_migration_supported = value;
        self
    }

    /// Set ocpu_options
    pub fn set_ocpu_options(mut self, value: Option<ShapeOcpuOptions>) -> Self {
        self.ocpu_options = value;
        self
    }

    /// Set memory_options
    pub fn set_memory_options(mut self, value: Option<ShapeMemoryOptions>) -> Self {
        self.memory_options = value;
        self
    }

    /// Set networking_bandwidth_options
    pub fn set_networking_bandwidth_options(
        mut self,
        value: Option<ShapeNetworkingBandwidthOptions>,
    ) -> Self {
        self.networking_bandwidth_options = value;
        self
    }

    /// Set max_vnic_attachment_options
    pub fn set_max_vnic_attachment_options(
        mut self,
        value: Option<ShapeMaxVnicAttachmentOptions>,
    ) -> Self {
        self.max_vnic_attachment_options = value;
        self
    }

    /// Set platform_config_options
    pub fn set_platform_config_options(
        mut self,
        value: Option<ShapePlatformConfigOptions>,
    ) -> Self {
        self.platform_config_options = value;
        self
    }

    /// Set is_billed_for_stopped_instance
    pub fn set_is_billed_for_stopped_instance(mut self, value: Option<bool>) -> Self {
        self.is_billed_for_stopped_instance = value;
        self
    }

    /// Set billing_type
    pub fn set_billing_type(mut self, value: Option<ShapeBillingType>) -> Self {
        self.billing_type = value;
        self
    }

    /// Set quota_names
    pub fn set_quota_names(mut self, value: Option<Vec<String>>) -> Self {
        self.quota_names = value;
        self
    }

    /// Set is_subcore
    pub fn set_is_subcore(mut self, value: Option<bool>) -> Self {
        self.is_subcore = value;
        self
    }

    /// Set is_flexible
    pub fn set_is_flexible(mut self, value: Option<bool>) -> Self {
        self.is_flexible = value;
        self
    }

    /// Set resize_compatible_shapes
    pub fn set_resize_compatible_shapes(mut self, value: Option<Vec<String>>) -> Self {
        self.resize_compatible_shapes = value;
        self
    }

    /// Set recommended_alternatives
    pub fn set_recommended_alternatives(
        mut self,
        value: Option<Vec<ShapeAlternativeObject>>,
    ) -> Self {
        self.recommended_alternatives = value;
        self
    }

    /// Set baseline_ocpu_utilizations (unwraps Option)
    pub fn with_baseline_ocpu_utilizations(
        mut self,
        value: Vec<ShapeBaselineOcpuUtilizations>,
    ) -> Self {
        self.baseline_ocpu_utilizations = Some(value);
        self
    }

    /// Set min_total_baseline_ocpus_required (unwraps Option)
    pub fn with_min_total_baseline_ocpus_required(mut self, value: i64) -> Self {
        self.min_total_baseline_ocpus_required = Some(value);
        self
    }

    /// Set processor_description (unwraps Option)
    pub fn with_processor_description(mut self, value: impl Into<String>) -> Self {
        self.processor_description = Some(value.into());
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

    /// Set network_ports (unwraps Option)
    pub fn with_network_ports(mut self, value: i64) -> Self {
        self.network_ports = Some(value);
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

    /// Set rdma_ports (unwraps Option)
    pub fn with_rdma_ports(mut self, value: i64) -> Self {
        self.rdma_ports = Some(value);
        self
    }

    /// Set rdma_bandwidth_in_gbps (unwraps Option)
    pub fn with_rdma_bandwidth_in_gbps(mut self, value: i64) -> Self {
        self.rdma_bandwidth_in_gbps = Some(value);
        self
    }

    /// Set is_live_migration_supported (unwraps Option)
    pub fn with_is_live_migration_supported(mut self, value: bool) -> Self {
        self.is_live_migration_supported = Some(value);
        self
    }

    /// Set ocpu_options (unwraps Option)
    pub fn with_ocpu_options(mut self, value: ShapeOcpuOptions) -> Self {
        self.ocpu_options = Some(value);
        self
    }

    /// Set memory_options (unwraps Option)
    pub fn with_memory_options(mut self, value: ShapeMemoryOptions) -> Self {
        self.memory_options = Some(value);
        self
    }

    /// Set networking_bandwidth_options (unwraps Option)
    pub fn with_networking_bandwidth_options(
        mut self,
        value: ShapeNetworkingBandwidthOptions,
    ) -> Self {
        self.networking_bandwidth_options = Some(value);
        self
    }

    /// Set max_vnic_attachment_options (unwraps Option)
    pub fn with_max_vnic_attachment_options(
        mut self,
        value: ShapeMaxVnicAttachmentOptions,
    ) -> Self {
        self.max_vnic_attachment_options = Some(value);
        self
    }

    /// Set platform_config_options (unwraps Option)
    pub fn with_platform_config_options(mut self, value: ShapePlatformConfigOptions) -> Self {
        self.platform_config_options = Some(value);
        self
    }

    /// Set is_billed_for_stopped_instance (unwraps Option)
    pub fn with_is_billed_for_stopped_instance(mut self, value: bool) -> Self {
        self.is_billed_for_stopped_instance = Some(value);
        self
    }

    /// Set billing_type (unwraps Option)
    pub fn with_billing_type(mut self, value: ShapeBillingType) -> Self {
        self.billing_type = Some(value);
        self
    }

    /// Set quota_names (unwraps Option)
    pub fn with_quota_names(mut self, value: Vec<String>) -> Self {
        self.quota_names = Some(value);
        self
    }

    /// Set is_subcore (unwraps Option)
    pub fn with_is_subcore(mut self, value: bool) -> Self {
        self.is_subcore = Some(value);
        self
    }

    /// Set is_flexible (unwraps Option)
    pub fn with_is_flexible(mut self, value: bool) -> Self {
        self.is_flexible = Some(value);
        self
    }

    /// Set resize_compatible_shapes (unwraps Option)
    pub fn with_resize_compatible_shapes(mut self, value: Vec<String>) -> Self {
        self.resize_compatible_shapes = Some(value);
        self
    }

    /// Set recommended_alternatives (unwraps Option)
    pub fn with_recommended_alternatives(mut self, value: Vec<ShapeAlternativeObject>) -> Self {
        self.recommended_alternatives = Some(value);
        self
    }
}
