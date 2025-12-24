use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The location for where an instance pool will place instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstancePoolPlacementConfigurationDetails {
    /// The availability domain to place instances. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The fault domains to place instances. <p> If you don't provide any values, the system makes a best effort to distribute instances across all fault domains based on capacity. <p> To distribute the instances evenly across selected fault domains, provide a set of fault domains. For example, you might want instances to be evenly distributed if your applications require high availability. <p> To get a list of fault domains, use the {@link #listFaultDomains(ListFaultDomainsRequest) listFaultDomains} operation in the Identity and Access Management Service API. <p> Example: {@code [FAULT-DOMAIN-1, FAULT-DOMAIN-2, FAULT-DOMAIN-3]}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domains: Option<Vec<String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the primary subnet in which to place instances. This field is deprecated. Use {@code primaryVnicSubnets} instead to set VNIC data for instances in the pool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_subnet_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_vnic_subnets: Option<InstancePoolPlacementPrimarySubnet>,

    /// The set of secondary VNIC data for instances in the pool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_vnic_subnets: Option<Vec<InstancePoolPlacementSecondaryVnicSubnet>>,
}

/// Required fields for UpdateInstancePoolPlacementConfigurationDetails
pub struct UpdateInstancePoolPlacementConfigurationDetailsRequired {
    /// The availability domain to place instances. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,
}

impl UpdateInstancePoolPlacementConfigurationDetails {
    /// Create a new UpdateInstancePoolPlacementConfigurationDetails with required fields
    pub fn new(required: UpdateInstancePoolPlacementConfigurationDetailsRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            fault_domains: None,

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

    /// Set fault_domains
    pub fn set_fault_domains(mut self, value: Option<Vec<String>>) -> Self {
        self.fault_domains = value;
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

    /// Set fault_domains (unwraps Option)
    pub fn with_fault_domains(mut self, value: Vec<String>) -> Self {
        self.fault_domains = Some(value);
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
