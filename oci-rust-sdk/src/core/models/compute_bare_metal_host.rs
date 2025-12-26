use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A compute bare metal host.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeBareMetalHost {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute capacity topology.
    pub compute_capacity_topology_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute bare metal host.
    pub id: String,

    /// The shape of the compute instance that runs on the compute bare metal host.
    pub instance_shape: String,

    /// The current state of the compute bare metal host.
    pub lifecycle_state: ComputeBareMetalHostLifecycleState,

    /// The date and time that the compute bare metal host was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time that the compute bare metal host was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_updated: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute HPC island.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_hpc_island_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute local block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_local_block_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute network block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_network_block_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute instance that runs on the compute bare metal host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,

    /// The lifecycle state details of the compute bare metal host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_details: Option<ComputeBareMetalHostLifecycleDetails>,
}

/// Required fields for ComputeBareMetalHost
pub struct ComputeBareMetalHostRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute capacity topology.
    pub compute_capacity_topology_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute bare metal host.
    pub id: String,

    /// The shape of the compute instance that runs on the compute bare metal host.
    pub instance_shape: String,

    /// The current state of the compute bare metal host.
    pub lifecycle_state: ComputeBareMetalHostLifecycleState,

    /// The date and time that the compute bare metal host was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time that the compute bare metal host was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_updated: DateTime<Utc>,
}

impl ComputeBareMetalHost {
    /// Create a new ComputeBareMetalHost with required fields
    pub fn new(required: ComputeBareMetalHostRequired) -> Self {
        Self {
            compute_capacity_topology_id: required.compute_capacity_topology_id,

            id: required.id,

            instance_shape: required.instance_shape,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            time_updated: required.time_updated,

            compute_hpc_island_id: None,

            compute_local_block_id: None,

            compute_network_block_id: None,

            instance_id: None,

            lifecycle_details: None,
        }
    }

    /// Set compute_capacity_topology_id
    pub fn set_compute_capacity_topology_id(mut self, value: String) -> Self {
        self.compute_capacity_topology_id = value;
        self
    }

    /// Set compute_hpc_island_id
    pub fn set_compute_hpc_island_id(mut self, value: Option<String>) -> Self {
        self.compute_hpc_island_id = value;
        self
    }

    /// Set compute_local_block_id
    pub fn set_compute_local_block_id(mut self, value: Option<String>) -> Self {
        self.compute_local_block_id = value;
        self
    }

    /// Set compute_network_block_id
    pub fn set_compute_network_block_id(mut self, value: Option<String>) -> Self {
        self.compute_network_block_id = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: Option<String>) -> Self {
        self.instance_id = value;
        self
    }

    /// Set instance_shape
    pub fn set_instance_shape(mut self, value: String) -> Self {
        self.instance_shape = value;
        self
    }

    /// Set lifecycle_details
    pub fn set_lifecycle_details(
        mut self,
        value: Option<ComputeBareMetalHostLifecycleDetails>,
    ) -> Self {
        self.lifecycle_details = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: ComputeBareMetalHostLifecycleState) -> Self {
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

    /// Set compute_hpc_island_id (unwraps Option)
    pub fn with_compute_hpc_island_id(mut self, value: impl Into<String>) -> Self {
        self.compute_hpc_island_id = Some(value.into());
        self
    }

    /// Set compute_local_block_id (unwraps Option)
    pub fn with_compute_local_block_id(mut self, value: impl Into<String>) -> Self {
        self.compute_local_block_id = Some(value.into());
        self
    }

    /// Set compute_network_block_id (unwraps Option)
    pub fn with_compute_network_block_id(mut self, value: impl Into<String>) -> Self {
        self.compute_network_block_id = Some(value.into());
        self
    }

    /// Set instance_id (unwraps Option)
    pub fn with_instance_id(mut self, value: impl Into<String>) -> Self {
        self.instance_id = Some(value.into());
        self
    }

    /// Set lifecycle_details (unwraps Option)
    pub fn with_lifecycle_details(mut self, value: ComputeBareMetalHostLifecycleDetails) -> Self {
        self.lifecycle_details = Some(value);
        self
    }
}
