use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The customer facing object includes host details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeHost {
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
    pub health: ComputeHostHealth,

    /// The lifecycle state of the host
    pub lifecycle_state: ComputeHostLifecycleState,

    /// The date and time that the compute host record was created, in the format defined by [RFC3339](https://tools .ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time that the compute host record was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_updated: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for Customer-unique HPC Island
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hpc_island_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Customer-unique host group associated with the Compute Bare Metal Host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_host_group_id: Option<String>,

    /// Configuration state of the Compute Bare Metal Host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_state: Option<ConfigurationState>,

    /// The date and time that the compute bare metal host configuration check was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_configuration_check: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_data: Option<ComputeHostConfigurationData>,

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

    /// Additional data that can be exposed to the customer.  Will include raw fault codes for strategic customers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_data: Option<HashMap<String, serde_json::Value>>,

    /// A free-form description detailing why the host is in its current state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_details: Option<HashMap<String, serde_json::Value>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Capacity Reserver that is currently on host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_reservation_id: Option<String>,

    /// A list that contains impacted components related to an unhealthy host. An impacted component will be a free-form structure of key values pairs that will provide more or less details based on data tiering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impacted_component_details: Option<HashMap<String, serde_json::Value>>,

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

/// Required fields for ComputeHost
pub struct ComputeHostRequired {
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
    pub health: ComputeHostHealth,

    /// The lifecycle state of the host
    pub lifecycle_state: ComputeHostLifecycleState,

    /// The date and time that the compute host record was created, in the format defined by [RFC3339](https://tools .ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time that the compute host record was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_updated: DateTime<Utc>,
}

impl ComputeHost {
    /// Create a new ComputeHost with required fields
    pub fn new(required: ComputeHostRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            id: required.id,

            fault_domain: required.fault_domain,

            shape: required.shape,

            health: required.health,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            time_updated: required.time_updated,

            hpc_island_id: None,

            compute_host_group_id: None,

            configuration_state: None,

            time_configuration_check: None,

            configuration_data: None,

            recycle_details: None,

            network_block_id: None,

            local_block_id: None,

            gpu_memory_fabric_id: None,

            instance_id: None,

            additional_data: None,

            lifecycle_details: None,

            capacity_reservation_id: None,

            impacted_component_details: None,

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

    /// Set configuration_state
    pub fn set_configuration_state(mut self, value: Option<ConfigurationState>) -> Self {
        self.configuration_state = value;
        self
    }

    /// Set time_configuration_check
    pub fn set_time_configuration_check(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_configuration_check = value;
        self
    }

    /// Set configuration_data
    pub fn set_configuration_data(mut self, value: Option<ComputeHostConfigurationData>) -> Self {
        self.configuration_data = value;
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
    pub fn set_health(mut self, value: ComputeHostHealth) -> Self {
        self.health = value;
        self
    }

    /// Set additional_data
    pub fn set_additional_data(
        mut self,
        value: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.additional_data = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ComputeHostLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set lifecycle_details
    pub fn set_lifecycle_details(
        mut self,
        value: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.lifecycle_details = value;
        self
    }

    /// Set capacity_reservation_id
    pub fn set_capacity_reservation_id(mut self, value: Option<String>) -> Self {
        self.capacity_reservation_id = value;
        self
    }

    /// Set impacted_component_details
    pub fn set_impacted_component_details(
        mut self,
        value: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.impacted_component_details = value;
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

    /// Set configuration_state (unwraps Option)
    pub fn with_configuration_state(mut self, value: ConfigurationState) -> Self {
        self.configuration_state = Some(value);
        self
    }

    /// Set time_configuration_check (unwraps Option)
    pub fn with_time_configuration_check(mut self, value: DateTime<Utc>) -> Self {
        self.time_configuration_check = Some(value);
        self
    }

    /// Set configuration_data (unwraps Option)
    pub fn with_configuration_data(mut self, value: ComputeHostConfigurationData) -> Self {
        self.configuration_data = Some(value);
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

    /// Set additional_data (unwraps Option)
    pub fn with_additional_data(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.additional_data = Some(value);
        self
    }

    /// Set lifecycle_details (unwraps Option)
    pub fn with_lifecycle_details(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.lifecycle_details = Some(value);
        self
    }

    /// Set capacity_reservation_id (unwraps Option)
    pub fn with_capacity_reservation_id(mut self, value: impl Into<String>) -> Self {
        self.capacity_reservation_id = Some(value.into());
        self
    }

    /// Set impacted_component_details (unwraps Option)
    pub fn with_impacted_component_details(
        mut self,
        value: HashMap<String, serde_json::Value>,
    ) -> Self {
        self.impacted_component_details = Some(value);
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
