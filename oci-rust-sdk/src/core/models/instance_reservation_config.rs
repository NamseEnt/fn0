use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Data that defines the capacity configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceReservationConfig {
    /// The shape to use when launching instances using compute capacity reservations. The shape determines the number of CPUs, the amount of memory, and other resources allocated to the instance. You can list all available shapes by calling {@link ListComputeCapacityReservationInstanceShapes}.
    pub instance_shape: String,

    /// The total number of instances that can be launched from the capacity configuration. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub reserved_count: i64,

    /// The amount of capacity in use out of the total capacity reserved in this capacity configuration. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub used_count: i64,

    /// The fault domain of this capacity configuration. If a value is not supplied, this capacity configuration is applicable to all fault domains in the specified availability domain. For more information, see [Capacity Reservations](https://docs.oracle.com/iaas/Content/Compute/Tasks/reserve-capacity.htm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_config: Option<ClusterConfigDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_shape_config: Option<InstanceReservationShapeConfigDetails>,

    /// The OCID of the cluster placement group for this instance reservation capacity configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_placement_group_id: Option<String>,
}

/// Required fields for InstanceReservationConfig
pub struct InstanceReservationConfigRequired {
    /// The shape to use when launching instances using compute capacity reservations. The shape determines the number of CPUs, the amount of memory, and other resources allocated to the instance. You can list all available shapes by calling {@link ListComputeCapacityReservationInstanceShapes}.
    pub instance_shape: String,

    /// The total number of instances that can be launched from the capacity configuration. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub reserved_count: i64,

    /// The amount of capacity in use out of the total capacity reserved in this capacity configuration. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub used_count: i64,
}

impl InstanceReservationConfig {
    /// Create a new InstanceReservationConfig with required fields
    pub fn new(required: InstanceReservationConfigRequired) -> Self {
        Self {
            instance_shape: required.instance_shape,

            reserved_count: required.reserved_count,

            used_count: required.used_count,

            fault_domain: None,

            cluster_config: None,

            instance_shape_config: None,

            cluster_placement_group_id: None,
        }
    }

    /// Set fault_domain
    pub fn set_fault_domain(mut self, value: Option<String>) -> Self {
        self.fault_domain = value;
        self
    }

    /// Set cluster_config
    pub fn set_cluster_config(mut self, value: Option<ClusterConfigDetails>) -> Self {
        self.cluster_config = value;
        self
    }

    /// Set instance_shape
    pub fn set_instance_shape(mut self, value: String) -> Self {
        self.instance_shape = value;
        self
    }

    /// Set instance_shape_config
    pub fn set_instance_shape_config(
        mut self,
        value: Option<InstanceReservationShapeConfigDetails>,
    ) -> Self {
        self.instance_shape_config = value;
        self
    }

    /// Set reserved_count
    pub fn set_reserved_count(mut self, value: i64) -> Self {
        self.reserved_count = value;
        self
    }

    /// Set used_count
    pub fn set_used_count(mut self, value: i64) -> Self {
        self.used_count = value;
        self
    }

    /// Set cluster_placement_group_id
    pub fn set_cluster_placement_group_id(mut self, value: Option<String>) -> Self {
        self.cluster_placement_group_id = value;
        self
    }

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
        self
    }

    /// Set cluster_config (unwraps Option)
    pub fn with_cluster_config(mut self, value: ClusterConfigDetails) -> Self {
        self.cluster_config = Some(value);
        self
    }

    /// Set instance_shape_config (unwraps Option)
    pub fn with_instance_shape_config(
        mut self,
        value: InstanceReservationShapeConfigDetails,
    ) -> Self {
        self.instance_shape_config = Some(value);
        self
    }

    /// Set cluster_placement_group_id (unwraps Option)
    pub fn with_cluster_placement_group_id(mut self, value: impl Into<String>) -> Self {
        self.cluster_placement_group_id = Some(value.into());
        self
    }
}
