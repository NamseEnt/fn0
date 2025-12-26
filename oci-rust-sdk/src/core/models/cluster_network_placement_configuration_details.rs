use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The location for where the instance pools in a cluster network will place instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterNetworkPlacementConfigurationDetails {
    /// The availability domain to place instances. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The placement constraint when reserving hosts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_constraint:
        Option<ClusterNetworkPlacementConfigurationDetailsPlacementConstraint>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the primary subnet to place instances. This field is deprecated. Use {@code primaryVnicSubnets} instead to set VNIC data for instances in the pool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_subnet_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_vnic_subnets: Option<InstancePoolPlacementPrimarySubnet>,

    /// The set of secondary VNIC data for instances in the pool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_vnic_subnets: Option<Vec<InstancePoolPlacementSecondaryVnicSubnet>>,
}

/// Required fields for ClusterNetworkPlacementConfigurationDetails
pub struct ClusterNetworkPlacementConfigurationDetailsRequired {
    /// The availability domain to place instances. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,
}

impl ClusterNetworkPlacementConfigurationDetails {
    /// Create a new ClusterNetworkPlacementConfigurationDetails with required fields
    pub fn new(required: ClusterNetworkPlacementConfigurationDetailsRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            placement_constraint: None,

            primary_subnet_id: None,

            primary_vnic_subnets: None,

            secondary_vnic_subnets: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set placement_constraint
    pub fn set_placement_constraint(
        mut self,
        value: Option<ClusterNetworkPlacementConfigurationDetailsPlacementConstraint>,
    ) -> Self {
        self.placement_constraint = value;
        self
    }

    /// Set primary_subnet_id
    pub fn set_primary_subnet_id(mut self, value: Option<String>) -> Self {
        self.primary_subnet_id = value;
        self
    }

    /// Set primary_vnic_subnets
    pub fn set_primary_vnic_subnets(
        mut self,
        value: Option<InstancePoolPlacementPrimarySubnet>,
    ) -> Self {
        self.primary_vnic_subnets = value;
        self
    }

    /// Set secondary_vnic_subnets
    pub fn set_secondary_vnic_subnets(
        mut self,
        value: Option<Vec<InstancePoolPlacementSecondaryVnicSubnet>>,
    ) -> Self {
        self.secondary_vnic_subnets = value;
        self
    }

    /// Set placement_constraint (unwraps Option)
    pub fn with_placement_constraint(
        mut self,
        value: ClusterNetworkPlacementConfigurationDetailsPlacementConstraint,
    ) -> Self {
        self.placement_constraint = Some(value);
        self
    }

    /// Set primary_subnet_id (unwraps Option)
    pub fn with_primary_subnet_id(mut self, value: impl Into<String>) -> Self {
        self.primary_subnet_id = Some(value.into());
        self
    }

    /// Set primary_vnic_subnets (unwraps Option)
    pub fn with_primary_vnic_subnets(mut self, value: InstancePoolPlacementPrimarySubnet) -> Self {
        self.primary_vnic_subnets = Some(value);
        self
    }

    /// Set secondary_vnic_subnets (unwraps Option)
    pub fn with_secondary_vnic_subnets(
        mut self,
        value: Vec<InstancePoolPlacementSecondaryVnicSubnet>,
    ) -> Self {
        self.secondary_vnic_subnets = Some(value);
        self
    }
}
