use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A cluster network is a group of high performance computing (HPC), GPU, or optimized bare metal instances that are connected with an ultra low-latency remote direct memory access (RDMA) network. [Cluster networks with instance pools](https://docs.oracle.com/iaas/Content/Compute/Tasks/managingclusternetworks.htm) use instance pools to manage groups of identical instances. <p> Use cluster networks with instance pools when you want predictable capacity for a specific number of identical instances that are managed as a group. <p> If you want to manage instances in the RDMA network independently of each other or use different types of instances in the network group, use compute clusters instead. For details, see {@link ComputeCluster}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterNetwork {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cluster network.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the cluster network.
    pub compartment_id: String,

    /// The current state of the cluster network.
    pub lifecycle_state: ClusterNetworkLifecycleState,

    /// The date and time the resource was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time the resource was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_updated: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the HPC island used by the cluster network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hpc_island_id: Option<String>,

    /// The list of network block OCIDs of the HPC island.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_block_ids: Option<Vec<String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The instance pools in the cluster network. <p> Each cluster network can have one instance pool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_pools: Option<Vec<InstancePool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_configuration: Option<ClusterNetworkPlacementConfigurationDetails>,
}

/// Required fields for ClusterNetwork
pub struct ClusterNetworkRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cluster network.
    pub id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the cluster network.
    pub compartment_id: String,

    /// The current state of the cluster network.
    pub lifecycle_state: ClusterNetworkLifecycleState,

    /// The date and time the resource was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time the resource was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_updated: DateTime<Utc>,
}

impl ClusterNetwork {
    /// Create a new ClusterNetwork with required fields
    pub fn new(required: ClusterNetworkRequired) -> Self {
        Self {
            id: required.id,

            compartment_id: required.compartment_id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            time_updated: required.time_updated,

            hpc_island_id: None,

            network_block_ids: None,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            instance_pools: None,

            placement_configuration: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set hpc_island_id
    pub fn set_hpc_island_id(mut self, value: Option<String>) -> Self {
        self.hpc_island_id = value;
        self
    }

    /// Set network_block_ids
    pub fn set_network_block_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.network_block_ids = value;
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

    /// Set instance_pools
    pub fn set_instance_pools(mut self, value: Option<Vec<InstancePool>>) -> Self {
        self.instance_pools = value;
        self
    }

    /// Set placement_configuration
    pub fn set_placement_configuration(
        mut self,
        value: Option<ClusterNetworkPlacementConfigurationDetails>,
    ) -> Self {
        self.placement_configuration = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ClusterNetworkLifecycleState) -> Self {
        self.lifecycle_state = value;
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

    /// Set hpc_island_id (unwraps Option)
    pub fn with_hpc_island_id(mut self, value: impl Into<String>) -> Self {
        self.hpc_island_id = Some(value.into());
        self
    }

    /// Set network_block_ids (unwraps Option)
    pub fn with_network_block_ids(mut self, value: Vec<String>) -> Self {
        self.network_block_ids = Some(value);
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

    /// Set instance_pools (unwraps Option)
    pub fn with_instance_pools(mut self, value: Vec<InstancePool>) -> Self {
        self.instance_pools = Some(value);
        self
    }

    /// Set placement_configuration (unwraps Option)
    pub fn with_placement_configuration(
        mut self,
        value: ClusterNetworkPlacementConfigurationDetails,
    ) -> Self {
        self.placement_configuration = Some(value);
        self
    }
}
