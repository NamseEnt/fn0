use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The customer facing object includes GPU memory cluster details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeGpuMemoryCluster {
    /// The availability domain of the GPU memory cluster.
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Customer-unique GPU memory cluster
    pub id: String,

    /// The OCID of the Instance Configuration used to source launch details for this instance.
    pub instance_configuration_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the compute GPU memory cluster.
    pub compartment_id: String,

    /// The lifecycle state of the GPU memory cluster
    pub lifecycle_state: ComputeGpuMemoryClusterLifecycleState,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute cluster.
    pub compute_cluster_id: String,

    /// The number of instances currently running in the GpuMemoryCluster Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size: i64,

    /// The date and time the GPU memory cluster was created. <p> Example: {@code 2016-09-15T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the GPU memory fabric.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_memory_fabric_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Usage of system tag keys. These predefined keys are scoped to namespaces. Example: {@code { \"orcl-cloud\": { \"free-tier-retained\": \"true\" } }}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Required fields for ComputeGpuMemoryCluster
pub struct ComputeGpuMemoryClusterRequired {
    /// The availability domain of the GPU memory cluster.
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Customer-unique GPU memory cluster
    pub id: String,

    /// The OCID of the Instance Configuration used to source launch details for this instance.
    pub instance_configuration_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the compute GPU memory cluster.
    pub compartment_id: String,

    /// The lifecycle state of the GPU memory cluster
    pub lifecycle_state: ComputeGpuMemoryClusterLifecycleState,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute cluster.
    pub compute_cluster_id: String,

    /// The number of instances currently running in the GpuMemoryCluster Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub size: i64,

    /// The date and time the GPU memory cluster was created. <p> Example: {@code 2016-09-15T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl ComputeGpuMemoryCluster {
    /// Create a new ComputeGpuMemoryCluster with required fields
    pub fn new(required: ComputeGpuMemoryClusterRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            id: required.id,

            instance_configuration_id: required.instance_configuration_id,

            compartment_id: required.compartment_id,

            lifecycle_state: required.lifecycle_state,

            compute_cluster_id: required.compute_cluster_id,

            size: required.size,

            time_created: required.time_created,

            gpu_memory_fabric_id: None,

            defined_tags: None,

            freeform_tags: None,

            system_tags: None,

            display_name: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set instance_configuration_id
    pub fn set_instance_configuration_id(mut self, value: String) -> Self {
        self.instance_configuration_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ComputeGpuMemoryClusterLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set gpu_memory_fabric_id
    pub fn set_gpu_memory_fabric_id(mut self, value: Option<String>) -> Self {
        self.gpu_memory_fabric_id = value;
        self
    }

    /// Set compute_cluster_id
    pub fn set_compute_cluster_id(mut self, value: String) -> Self {
        self.compute_cluster_id = value;
        self
    }

    /// Set size
    pub fn set_size(mut self, value: i64) -> Self {
        self.size = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set system_tags
    pub fn set_system_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.system_tags = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set gpu_memory_fabric_id (unwraps Option)
    pub fn with_gpu_memory_fabric_id(mut self, value: impl Into<String>) -> Self {
        self.gpu_memory_fabric_id = Some(value.into());
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set system_tags (unwraps Option)
    pub fn with_system_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.system_tags = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }
}
