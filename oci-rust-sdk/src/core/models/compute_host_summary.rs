use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Summary information for a compute host.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeHostSummary {
    /// The availability domain of the compute host. <p> Example: {@code Uocm:US-CHICAGO-1-AD-2}
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the compartment. This should always be the root compartment.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Customer-unique host
    pub id: String,

    /// A fault domain is a grouping of hardware and infrastructure within an availability domain. Each availability domain contains three fault domains. Fault domains let you distribute your instances so that they are not on the same physical hardware within a single availability domain. A hardware failure or Compute hardware maintenance that affects one fault domain does not affect instances in other fault domains. <p> This field is the Fault domain of the host
    pub fault_domain: String,

    /// The shape of host
    pub shape: String,

    /// The heathy state of the host
    pub health: String,

    /// The lifecycle state of the host
    pub lifecycle_state: String,

    /// While listing a host the user will know if they have an impacted component or not. The user will have to issue a get host to see details.
    pub has_impacted_components: bool,

    /// The date and time that the compute host record was created, in the format defined by [RFC3339](https://tools .ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time that the compute host record was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_updated: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for Customer-unique HPC Island
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hpc_island_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Customer-unique host group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_host_group_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub recycle_details: Option<RecycleDetails>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for Customer-unique Network Block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_block_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for Customer-unique Local Block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_block_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for Customer-unique GPU Memory Fabric
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_memory_fabric_id: Option<String>,

    /// The public [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Virtual Machine or Bare Metal instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Capacity Reserver that is currently on host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_reservation_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

/// Required fields for ComputeHostSummary
pub struct ComputeHostSummaryRequired {
    /// The availability domain of the compute host. <p> Example: {@code Uocm:US-CHICAGO-1-AD-2}
    pub availability_domain: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the compartment. This should always be the root compartment.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Customer-unique host
    pub id: String,

    /// A fault domain is a grouping of hardware and infrastructure within an availability domain. Each availability domain contains three fault domains. Fault domains let you distribute your instances so that they are not on the same physical hardware within a single availability domain. A hardware failure or Compute hardware maintenance that affects one fault domain does not affect instances in other fault domains. <p> This field is the Fault domain of the host
    pub fault_domain: String,

    /// The shape of host
    pub shape: String,

    /// The heathy state of the host
    pub health: String,

    /// The lifecycle state of the host
    pub lifecycle_state: String,

    /// While listing a host the user will know if they have an impacted component or not. The user will have to issue a get host to see details.
    pub has_impacted_components: bool,

    /// The date and time that the compute host record was created, in the format defined by [RFC3339](https://tools .ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time that the compute host record was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_updated: DateTime<Utc>,
}

impl ComputeHostSummary {
    /// Create a new ComputeHostSummary with required fields
    pub fn new(required: ComputeHostSummaryRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            id: required.id,

            fault_domain: required.fault_domain,

            shape: required.shape,

            health: required.health,

            lifecycle_state: required.lifecycle_state,

            has_impacted_components: required.has_impacted_components,

            time_created: required.time_created,

            time_updated: required.time_updated,

            hpc_island_id: None,

            compute_host_group_id: None,

            recycle_details: None,

            network_block_id: None,

            local_block_id: None,

            gpu_memory_fabric_id: None,

            instance_id: None,

            capacity_reservation_id: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set fault_domain
    pub fn set_fault_domain(mut self, value: String) -> Self {
        self.fault_domain = value;
        self
    }

    /// Set hpc_island_id
    pub fn set_hpc_island_id(mut self, value: Option<String>) -> Self {
        self.hpc_island_id = value;
        self
    }

    /// Set compute_host_group_id
    pub fn set_compute_host_group_id(mut self, value: Option<String>) -> Self {
        self.compute_host_group_id = value;
        self
    }

    /// Set recycle_details
    pub fn set_recycle_details(mut self, value: Option<RecycleDetails>) -> Self {
        self.recycle_details = value;
        self
    }

    /// Set network_block_id
    pub fn set_network_block_id(mut self, value: Option<String>) -> Self {
        self.network_block_id = value;
        self
    }

    /// Set local_block_id
    pub fn set_local_block_id(mut self, value: Option<String>) -> Self {
        self.local_block_id = value;
        self
    }

    /// Set gpu_memory_fabric_id
    pub fn set_gpu_memory_fabric_id(mut self, value: Option<String>) -> Self {
        self.gpu_memory_fabric_id = value;
        self
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: Option<String>) -> Self {
        self.instance_id = value;
        self
    }

    /// Set shape
    pub fn set_shape(mut self, value: String) -> Self {
        self.shape = value;
        self
    }

    /// Set health
    pub fn set_health(mut self, value: String) -> Self {
        self.health = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: String) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set capacity_reservation_id
    pub fn set_capacity_reservation_id(mut self, value: Option<String>) -> Self {
        self.capacity_reservation_id = value;
        self
    }

    /// Set has_impacted_components
    pub fn set_has_impacted_components(mut self, value: bool) -> Self {
        self.has_impacted_components = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set time_updated
    pub fn set_time_updated(mut self, value: DateTime<Utc>) -> Self {
        self.time_updated = value;
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

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set hpc_island_id (unwraps Option)
    pub fn with_hpc_island_id(mut self, value: impl Into<String>) -> Self {
        self.hpc_island_id = Some(value.into());
        self
    }

    /// Set compute_host_group_id (unwraps Option)
    pub fn with_compute_host_group_id(mut self, value: impl Into<String>) -> Self {
        self.compute_host_group_id = Some(value.into());
        self
    }

    /// Set recycle_details (unwraps Option)
    pub fn with_recycle_details(mut self, value: RecycleDetails) -> Self {
        self.recycle_details = Some(value);
        self
    }

    /// Set network_block_id (unwraps Option)
    pub fn with_network_block_id(mut self, value: impl Into<String>) -> Self {
        self.network_block_id = Some(value.into());
        self
    }

    /// Set local_block_id (unwraps Option)
    pub fn with_local_block_id(mut self, value: impl Into<String>) -> Self {
        self.local_block_id = Some(value.into());
        self
    }

    /// Set gpu_memory_fabric_id (unwraps Option)
    pub fn with_gpu_memory_fabric_id(mut self, value: impl Into<String>) -> Self {
        self.gpu_memory_fabric_id = Some(value.into());
        self
    }

    /// Set instance_id (unwraps Option)
    pub fn with_instance_id(mut self, value: impl Into<String>) -> Self {
        self.instance_id = Some(value.into());
        self
    }

    /// Set capacity_reservation_id (unwraps Option)
    pub fn with_capacity_reservation_id(mut self, value: impl Into<String>) -> Self {
        self.capacity_reservation_id = Some(value.into());
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

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }
}
