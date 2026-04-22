use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the IP Inventory details of a subnet and its associated resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpInventorySubnetResourceSummary {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_id: Option<String>,

    /// Lists the allocated private IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// Lifetime of the allocated private IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address_lifetime: Option<IpInventorySubnetResourceSummaryIpAddressLifetime>,

    /// The address range the IP address is assigned from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_cidr: Option<String>,

    /// Associated public IP address for the private IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_public_ip: Option<String>,

    /// Lifetime of the assigned public IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ip_lifetime: Option<IpInventorySubnetResourceSummaryPublicIpLifetime>,

    /// Public IP address Pool the IP address is allocated from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_public_ip_pool: Option<IpInventorySubnetResourceSummaryAssociatedPublicIpPool>,

    /// DNS hostname of the IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_host_name: Option<String>,

    /// Name of the created resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_resource_name: Option<String>,

    /// Primary flag for IP Resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_primary: Option<bool>,

    /// Type of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_resource_type: Option<IpInventorySubnetResourceSummaryAssignedResourceType>,

    /// Address type of the allocated private IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_type: Option<String>,

    /// Assigned time of the private IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_time: Option<DateTime<Utc>>,
}

impl IpInventorySubnetResourceSummary {
    /// Create a new IpInventorySubnetResourceSummary
    pub fn new() -> Self {
        Self {
            ip_id: None,

            ip_address: None,

            ip_address_lifetime: None,

            parent_cidr: None,

            associated_public_ip: None,

            public_ip_lifetime: None,

            associated_public_ip_pool: None,

            dns_host_name: None,

            assigned_resource_name: None,

            is_primary: None,

            assigned_resource_type: None,

            address_type: None,

            assigned_time: None,
        }
    }

    /// Set ip_id
    pub fn set_ip_id(mut self, value: Option<String>) -> Self {
        self.ip_id = value;
        self
    }

    /// Set ip_address
    pub fn set_ip_address(mut self, value: Option<String>) -> Self {
        self.ip_address = value;
        self
    }

    /// Set ip_address_lifetime
    pub fn set_ip_address_lifetime(
        mut self,
        value: Option<IpInventorySubnetResourceSummaryIpAddressLifetime>,
    ) -> Self {
        self.ip_address_lifetime = value;
        self
    }

    /// Set parent_cidr
    pub fn set_parent_cidr(mut self, value: Option<String>) -> Self {
        self.parent_cidr = value;
        self
    }

    /// Set associated_public_ip
    pub fn set_associated_public_ip(mut self, value: Option<String>) -> Self {
        self.associated_public_ip = value;
        self
    }

    /// Set public_ip_lifetime
    pub fn set_public_ip_lifetime(
        mut self,
        value: Option<IpInventorySubnetResourceSummaryPublicIpLifetime>,
    ) -> Self {
        self.public_ip_lifetime = value;
        self
    }

    /// Set associated_public_ip_pool
    pub fn set_associated_public_ip_pool(
        mut self,
        value: Option<IpInventorySubnetResourceSummaryAssociatedPublicIpPool>,
    ) -> Self {
        self.associated_public_ip_pool = value;
        self
    }

    /// Set dns_host_name
    pub fn set_dns_host_name(mut self, value: Option<String>) -> Self {
        self.dns_host_name = value;
        self
    }

    /// Set assigned_resource_name
    pub fn set_assigned_resource_name(mut self, value: Option<String>) -> Self {
        self.assigned_resource_name = value;
        self
    }

    /// Set is_primary
    pub fn set_is_primary(mut self, value: Option<bool>) -> Self {
        self.is_primary = value;
        self
    }

    /// Set assigned_resource_type
    pub fn set_assigned_resource_type(
        mut self,
        value: Option<IpInventorySubnetResourceSummaryAssignedResourceType>,
    ) -> Self {
        self.assigned_resource_type = value;
        self
    }

    /// Set address_type
    pub fn set_address_type(mut self, value: Option<String>) -> Self {
        self.address_type = value;
        self
    }

    /// Set assigned_time
    pub fn set_assigned_time(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.assigned_time = value;
        self
    }

    /// Set ip_id (unwraps Option)
    pub fn with_ip_id(mut self, value: impl Into<String>) -> Self {
        self.ip_id = Some(value.into());
        self
    }

    /// Set ip_address (unwraps Option)
    pub fn with_ip_address(mut self, value: impl Into<String>) -> Self {
        self.ip_address = Some(value.into());
        self
    }

    /// Set ip_address_lifetime (unwraps Option)
    pub fn with_ip_address_lifetime(
        mut self,
        value: IpInventorySubnetResourceSummaryIpAddressLifetime,
    ) -> Self {
        self.ip_address_lifetime = Some(value);
        self
    }

    /// Set parent_cidr (unwraps Option)
    pub fn with_parent_cidr(mut self, value: impl Into<String>) -> Self {
        self.parent_cidr = Some(value.into());
        self
    }

    /// Set associated_public_ip (unwraps Option)
    pub fn with_associated_public_ip(mut self, value: impl Into<String>) -> Self {
        self.associated_public_ip = Some(value.into());
        self
    }

    /// Set public_ip_lifetime (unwraps Option)
    pub fn with_public_ip_lifetime(
        mut self,
        value: IpInventorySubnetResourceSummaryPublicIpLifetime,
    ) -> Self {
        self.public_ip_lifetime = Some(value);
        self
    }

    /// Set associated_public_ip_pool (unwraps Option)
    pub fn with_associated_public_ip_pool(
        mut self,
        value: IpInventorySubnetResourceSummaryAssociatedPublicIpPool,
    ) -> Self {
        self.associated_public_ip_pool = Some(value);
        self
    }

    /// Set dns_host_name (unwraps Option)
    pub fn with_dns_host_name(mut self, value: impl Into<String>) -> Self {
        self.dns_host_name = Some(value.into());
        self
    }

    /// Set assigned_resource_name (unwraps Option)
    pub fn with_assigned_resource_name(mut self, value: impl Into<String>) -> Self {
        self.assigned_resource_name = Some(value.into());
        self
    }

    /// Set is_primary (unwraps Option)
    pub fn with_is_primary(mut self, value: bool) -> Self {
        self.is_primary = Some(value);
        self
    }

    /// Set assigned_resource_type (unwraps Option)
    pub fn with_assigned_resource_type(
        mut self,
        value: IpInventorySubnetResourceSummaryAssignedResourceType,
    ) -> Self {
        self.assigned_resource_type = Some(value);
        self
    }

    /// Set address_type (unwraps Option)
    pub fn with_address_type(mut self, value: impl Into<String>) -> Self {
        self.address_type = Some(value.into());
        self
    }

    /// Set assigned_time (unwraps Option)
    pub fn with_assigned_time(mut self, value: DateTime<Utc>) -> Self {
        self.assigned_time = Some(value);
        self
    }
}

impl Default for IpInventorySubnetResourceSummary {
    fn default() -> Self {
        Self::new()
    }
}
