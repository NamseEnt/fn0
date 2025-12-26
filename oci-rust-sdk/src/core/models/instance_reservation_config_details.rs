use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A template that contains the settings to use when defining the instance capacity configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceReservationConfigDetails {
    /// The shape requested when launching instances using reserved capacity. The shape determines the number of CPUs, amount of memory, and other resources allocated to the instance. You can list all available shapes by calling {@link ListComputeCapacityReservationInstanceShapes}.
    pub instance_shape: String,

    /// The total number of instances that can be launched from the capacity configuration. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub reserved_count: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_shape_config: Option<InstanceReservationShapeConfigDetails>,

    /// The fault domain to use for instances created using this capacity configuration. For more information, see [Fault Domains](https://docs.oracle.com/iaas/Content/General/Concepts/regions.htm#fault). If you do not specify the fault domain, the capacity is available for an instance that does not specify a fault domain. To change the fault domain for a reservation, delete the reservation and create a new one in the preferred fault domain. <p> To retrieve a list of fault domains, use the {@code ListFaultDomains} operation in the [Identity and Access Management Service API](https://docs.oracle.com/iaas/api/#/en/identity/20160918/). <p> Example: {@code FAULT-DOMAIN-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_config: Option<ClusterConfigDetails>,

    /// The OCID of the cluster placement group for this instance reservation capacity configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_placement_group_id: Option<String>,
}

/// Required fields for InstanceReservationConfigDetails
pub struct InstanceReservationConfigDetailsRequired {
    /// The shape requested when launching instances using reserved capacity. The shape determines the number of CPUs, amount of memory, and other resources allocated to the instance. You can list all available shapes by calling {@link ListComputeCapacityReservationInstanceShapes}.
    pub instance_shape: String,

    /// The total number of instances that can be launched from the capacity configuration. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub reserved_count: i64,
}

impl InstanceReservationConfigDetails {
    /// Create a new InstanceReservationConfigDetails with required fields
    pub fn new(required: InstanceReservationConfigDetailsRequired) -> Self {
        Self {
            instance_shape: required.instance_shape,

            reserved_count: required.reserved_count,

            instance_shape_config: None,

            fault_domain: None,

            cluster_config: None,

            cluster_placement_group_id: None,
        }
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

    /// Set reserved_count
    pub fn set_reserved_count(mut self, value: i64) -> Self {
        self.reserved_count = value;
        self
    }

    /// Set cluster_placement_group_id
    pub fn set_cluster_placement_group_id(mut self, value: Option<String>) -> Self {
        self.cluster_placement_group_id = value;
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

    /// Set cluster_placement_group_id (unwraps Option)
    pub fn with_cluster_placement_group_id(mut self, value: impl Into<String>) -> Self {
        self.cluster_placement_group_id = Some(value.into());
        self
    }
}
