use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Summary information for a compute network block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeNetworkBlockSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute capacity topology.
    pub compute_capacity_topology_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute HPC island.
    pub compute_hpc_island_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute network block.
    pub id: String,

    /// The current state of the compute network block.
    pub lifecycle_state: String,

    /// The date and time that the compute network block was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time that the compute network block was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_updated: DateTime<Utc>,

    /// The total number of compute bare metal hosts located in the compute network block. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_compute_bare_metal_host_count: i64,
}

/// Required fields for ComputeNetworkBlockSummary
pub struct ComputeNetworkBlockSummaryRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute capacity topology.
    pub compute_capacity_topology_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute HPC island.
    pub compute_hpc_island_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compute network block.
    pub id: String,

    /// The current state of the compute network block.
    pub lifecycle_state: String,

    /// The date and time that the compute network block was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The date and time that the compute network block was updated, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_updated: DateTime<Utc>,

    /// The total number of compute bare metal hosts located in the compute network block. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub total_compute_bare_metal_host_count: i64,
}

impl ComputeNetworkBlockSummary {
    /// Create a new ComputeNetworkBlockSummary with required fields
    pub fn new(required: ComputeNetworkBlockSummaryRequired) -> Self {
        Self {
            compute_capacity_topology_id: required.compute_capacity_topology_id,

            compute_hpc_island_id: required.compute_hpc_island_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            time_updated: required.time_updated,

            total_compute_bare_metal_host_count: required.total_compute_bare_metal_host_count,
        }
    }

    /// Set compute_capacity_topology_id
    pub fn set_compute_capacity_topology_id(mut self, value: String) -> Self {
        self.compute_capacity_topology_id = value;
        self
    }

    /// Set compute_hpc_island_id
    pub fn set_compute_hpc_island_id(mut self, value: String) -> Self {
        self.compute_hpc_island_id = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: String) -> Self {
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

    /// Set total_compute_bare_metal_host_count
    pub fn set_total_compute_bare_metal_host_count(mut self, value: i64) -> Self {
        self.total_compute_bare_metal_host_count = value;
        self
    }
}
