use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Condensed instance data when listing instances in a compute capacity reservation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapacityReservationInstanceSummary {
    /// The OCID of the instance.
    pub id: String,

    /// The availability domain the instance is running in.
    pub availability_domain: String,

    /// The OCID of the compartment that contains the instance.
    pub compartment_id: String,

    /// The shape of the instance. The shape determines the number of CPUs, amount of memory, and other resources allocated to the instance. <p> You can enumerate all available shapes by calling {@link #listComputeCapacityReservationInstanceShapes(ListComputeCapacityReservationInstanceShapesRequest) listComputeCapacityReservationInstanceShapes}.
    pub shape: String,

    /// The fault domain the instance is running in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// The OCID of the cluster placement group of the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_placement_group_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape_config: Option<InstanceReservationShapeConfigDetails>,
}

/// Required fields for CapacityReservationInstanceSummary
pub struct CapacityReservationInstanceSummaryRequired {
    /// The OCID of the instance.
    pub id: String,

    /// The availability domain the instance is running in.
    pub availability_domain: String,

    /// The OCID of the compartment that contains the instance.
    pub compartment_id: String,

    /// The shape of the instance. The shape determines the number of CPUs, amount of memory, and other resources allocated to the instance. <p> You can enumerate all available shapes by calling {@link #listComputeCapacityReservationInstanceShapes(ListComputeCapacityReservationInstanceShapesRequest) listComputeCapacityReservationInstanceShapes}.
    pub shape: String,
}

impl CapacityReservationInstanceSummary {
    /// Create a new CapacityReservationInstanceSummary with required fields
    pub fn new(required: CapacityReservationInstanceSummaryRequired) -> Self {
        Self {
            id: required.id,

            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            shape: required.shape,

            fault_domain: None,

            cluster_placement_group_id: None,

            shape_config: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
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

    /// Set fault_domain
    pub fn set_fault_domain(mut self, value: Option<String>) -> Self {
        self.fault_domain = value;
        self
    }

    /// Set cluster_placement_group_id
    pub fn set_cluster_placement_group_id(mut self, value: Option<String>) -> Self {
        self.cluster_placement_group_id = value;
        self
    }

    /// Set shape_config
    pub fn set_shape_config(
        mut self,
        value: Option<InstanceReservationShapeConfigDetails>,
    ) -> Self {
        self.shape_config = value;
        self
    }

    /// Set shape
    pub fn set_shape(mut self, value: String) -> Self {
        self.shape = value;
        self
    }

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
        self
    }

    /// Set cluster_placement_group_id (unwraps Option)
    pub fn with_cluster_placement_group_id(mut self, value: impl Into<String>) -> Self {
        self.cluster_placement_group_id = Some(value.into());
        self
    }

    /// Set shape_config (unwraps Option)
    pub fn with_shape_config(mut self, value: InstanceReservationShapeConfigDetails) -> Self {
        self.shape_config = Some(value);
        self
    }
}
